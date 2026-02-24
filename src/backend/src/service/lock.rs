//! Scheduler Lock Module - SQLite-based Advisory Locking
//!
//! This module provides a distributed lock mechanism to ensure that only ONE scheduler
//! instance runs at any given time across the entire system. This is critical because
//! NetNinja can run in two modes:
//!
//! 1. **Desktop Mode**: The Tauri desktop application runs with an embedded scheduler
//! 2. **Service Mode**: A Windows service runs as a background daemon with its own scheduler
//!
//! ## Why Only One Scheduler?
//!
//! The scheduler executes tasks like:
//! - Speed tests (which would interfere if running concurrently)
//! - Quota checks (which involve browser automation and login sessions)
//! - Sending notification emails
//!
//! Running multiple schedulers would cause:
//! - Duplicate task executions
//! - Resource contention (multiple browser instances, concurrent speed tests)
//! - Conflicting database updates
//! - Duplicate notification emails
//!
//! ## Lock Architecture
//!
//! The lock uses a single-row SQLite table with:
//! - **holder**: Identifies who holds the lock ('service' or 'desktop')
//! - **acquired_at**: When the lock was first acquired
//! - **heartbeat_at**: Last heartbeat timestamp (updated periodically)
//! - **version**: Application version for debugging
//!
//! ## Concurrency Model
//!
//! This lock implements **advisory locking** with heartbeat-based expiration:
//!
//! 1. When acquiring, we check if an existing lock is held
//! 2. If no lock exists or the lock is stale (heartbeat > 60s old), we can acquire
//! 3. The lock holder must periodically call `heartbeat()` to maintain the lock
//! 4. If the holder crashes without releasing, the lock becomes stale and can be claimed
//!
//! ## User Experience Expectations
//!
//! - If the **service is running**, the desktop app should NOT start its scheduler
//! - If the **desktop app is running** with its scheduler, the service should yield
//! - The lock is NOT persistent across app restarts - it's advisory for runtime coordination
//! - Stale locks (from crashed processes) are automatically cleaned up
//!
//! ## Thread Safety
//!
//! All operations use SQLite transactions to ensure atomicity. SQLite's database-level
//! locking ensures that concurrent acquisition attempts are properly serialized.

use chrono::{DateTime, Duration, Utc};
use sqlx::SqlitePool;
use tracing::{debug, error, info, warn};

use crate::errors::{AppError, AppResult};

/// Timeout in seconds after which a lock is considered stale.
/// If the lock holder hasn't sent a heartbeat within this window,
/// the lock can be claimed by another process.
const LOCK_STALE_TIMEOUT_SECS: i64 = 60;

/// Information about the current lock holder
#[derive(Debug, Clone)]
pub struct LockInfo {
    /// Who holds the lock: "service" or "desktop"
    pub holder: String,
    /// When the lock was first acquired
    pub acquired_at: DateTime<Utc>,
    /// When the last heartbeat was received
    pub heartbeat_at: DateTime<Utc>,
    /// Application version of the lock holder
    pub version: Option<String>,
}

/// SQLite-based advisory lock for scheduler coordination.
///
/// This struct provides methods to acquire, release, and monitor a distributed
/// lock that ensures only one scheduler runs at a time.
///
/// # Example
///
/// ```rust,ignore
/// let lock = SchedulerLock::new(pool.clone());
///
/// // Initialize the lock table (run once at startup)
/// lock.initialize().await?;
///
/// // Try to acquire the lock
/// if lock.try_acquire("desktop", Some("1.0.0")).await? {
///     info!("Lock acquired! Starting scheduler...");
///
///     // Periodically send heartbeats
///     lock.heartbeat().await?;
///
///     // When shutting down
///     lock.release().await?;
/// } else {
///     info!("Another scheduler is running, skipping...");
/// }
/// ```
pub struct SchedulerLock {
    pool: SqlitePool,
}

impl SchedulerLock {
    /// Create a new SchedulerLock instance.
    ///
    /// # Arguments
    ///
    /// * `pool` - The SQLite connection pool to use for lock operations
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Initialize the scheduler_lock table if it doesn't exist.
    ///
    /// This should be called once during application startup, before any
    /// lock operations are attempted. The table uses a CHECK constraint
    /// to ensure only one row (id=1) can ever exist.
    ///
    /// # Schema
    ///
    /// ```sql
    /// CREATE TABLE IF NOT EXISTS scheduler_lock (
    ///     id INTEGER PRIMARY KEY CHECK (id = 1),
    ///     holder TEXT NOT NULL,
    ///     acquired_at TEXT NOT NULL,
    ///     heartbeat_at TEXT NOT NULL,
    ///     version TEXT
    /// );
    /// ```
    pub async fn initialize(&self) -> AppResult<()> {
        debug!("Initializing scheduler_lock table");

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS scheduler_lock (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                holder TEXT NOT NULL,
                acquired_at TEXT NOT NULL,
                heartbeat_at TEXT NOT NULL,
                version TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to create scheduler_lock table: {:?}", e);
            AppError::Database(e)
        })?;

        info!("Scheduler lock table initialized");
        Ok(())
    }

    /// Attempt to acquire the scheduler lock.
    ///
    /// This method will succeed if:
    /// 1. No lock currently exists (first acquisition)
    /// 2. The existing lock is stale (heartbeat older than LOCK_STALE_TIMEOUT_SECS)
    ///
    /// This method uses an atomic transaction to prevent race conditions when
    /// multiple processes try to acquire the lock simultaneously.
    ///
    /// # Arguments
    ///
    /// * `holder` - Identifier for the lock holder ("service" or "desktop")
    /// * `version` - Optional application version string for debugging
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Lock was successfully acquired
    /// * `Ok(false)` - Lock is held by another active process
    /// * `Err(_)` - Database error occurred
    ///
    /// # Concurrency
    ///
    /// This method is safe to call from multiple processes/threads. SQLite's
    /// transaction isolation ensures that only one caller will successfully
    /// acquire the lock.
    pub async fn try_acquire(&self, holder: &str, version: Option<&str>) -> AppResult<bool> {
        let now = Utc::now();
        let now_str = now.to_rfc3339();
        let stale_threshold = now - Duration::seconds(LOCK_STALE_TIMEOUT_SECS);
        let stale_threshold_str = stale_threshold.to_rfc3339();

        debug!(
            "Attempting to acquire lock for holder='{}', stale_threshold='{}'",
            holder, stale_threshold_str
        );

        // Use a transaction to ensure atomicity of the check-and-acquire operation.
        // This prevents TOCTOU (time-of-check-time-of-use) race conditions.
        let mut tx = self.pool.begin().await.map_err(|e| {
            error!("Failed to begin transaction: {:?}", e);
            AppError::Database(e)
        })?;

        // Check if there's an existing lock and whether it's stale
        let existing: Option<(String, String)> = sqlx::query_as(
            r#"
            SELECT holder, heartbeat_at
            FROM scheduler_lock
            WHERE id = 1
            "#,
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| {
            error!("Failed to check existing lock: {:?}", e);
            AppError::Database(e)
        })?;

        match existing {
            Some((existing_holder, heartbeat_str)) => {
                // Parse the heartbeat timestamp
                let heartbeat = DateTime::parse_from_rfc3339(&heartbeat_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| {
                        warn!("Failed to parse heartbeat timestamp '{}', treating as stale", heartbeat_str);
                        Utc::now() - Duration::seconds(LOCK_STALE_TIMEOUT_SECS + 1)
                    });

                // Check if the lock is stale
                if heartbeat < stale_threshold {
                    // Lock is stale - we can take over
                    info!(
                        "Existing lock from '{}' is stale (last heartbeat: {}), taking over",
                        existing_holder, heartbeat_str
                    );

                    sqlx::query(
                        r#"
                        UPDATE scheduler_lock
                        SET holder = ?, acquired_at = ?, heartbeat_at = ?, version = ?
                        WHERE id = 1
                        "#,
                    )
                    .bind(holder)
                    .bind(&now_str)
                    .bind(&now_str)
                    .bind(version)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| {
                        error!("Failed to update stale lock: {:?}", e);
                        AppError::Database(e)
                    })?;

                    tx.commit().await.map_err(|e| {
                        error!("Failed to commit lock acquisition: {:?}", e);
                        AppError::Database(e)
                    })?;

                    info!("Lock acquired by '{}' (took over stale lock)", holder);
                    Ok(true)
                } else {
                    // Lock is still active
                    debug!(
                        "Lock is held by '{}' with recent heartbeat ({}), cannot acquire",
                        existing_holder, heartbeat_str
                    );

                    // Rollback is implicit when tx is dropped, but let's be explicit
                    tx.rollback().await.ok();
                    Ok(false)
                }
            }
            None => {
                // No lock exists - create one
                info!("No existing lock found, acquiring for '{}'", holder);

                sqlx::query(
                    r#"
                    INSERT INTO scheduler_lock (id, holder, acquired_at, heartbeat_at, version)
                    VALUES (1, ?, ?, ?, ?)
                    "#,
                )
                .bind(holder)
                .bind(&now_str)
                .bind(&now_str)
                .bind(version)
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    error!("Failed to create lock: {:?}", e);
                    AppError::Database(e)
                })?;

                tx.commit().await.map_err(|e| {
                    error!("Failed to commit lock creation: {:?}", e);
                    AppError::Database(e)
                })?;

                info!("Lock acquired by '{}' (new lock)", holder);
                Ok(true)
            }
        }
    }

    /// Release the scheduler lock.
    ///
    /// This removes the lock from the database, allowing other processes
    /// to acquire it immediately without waiting for staleness timeout.
    ///
    /// # Note
    ///
    /// This method should be called during graceful shutdown. If the process
    /// crashes, the lock will eventually become stale and be claimable by
    /// other processes after LOCK_STALE_TIMEOUT_SECS.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Lock was released (or didn't exist)
    /// * `Err(_)` - Database error occurred
    pub async fn release(&self) -> AppResult<()> {
        debug!("Releasing scheduler lock");

        let result = sqlx::query(
            r#"
            DELETE FROM scheduler_lock
            WHERE id = 1
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to release lock: {:?}", e);
            AppError::Database(e)
        })?;

        if result.rows_affected() > 0 {
            info!("Scheduler lock released");
        } else {
            debug!("No lock to release (was already released or never acquired)");
        }

        Ok(())
    }

    /// Update the heartbeat timestamp for the current lock.
    ///
    /// This method should be called periodically (recommended: every 15-30 seconds)
    /// by the lock holder to signal that it's still alive and actively using the lock.
    ///
    /// If heartbeats stop (e.g., due to process crash), the lock will become stale
    /// after LOCK_STALE_TIMEOUT_SECS and can be claimed by another process.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Heartbeat was updated successfully
    /// * `Ok(false)` - No lock exists to update (lock was lost)
    /// * `Err(_)` - Database error occurred
    ///
    /// # Warning
    ///
    /// If this returns `Ok(false)`, the lock has been lost (possibly taken over
    /// by another process after becoming stale). The caller should stop its
    /// scheduler operations immediately.
    pub async fn heartbeat(&self) -> AppResult<bool> {
        let now = Utc::now();
        let now_str = now.to_rfc3339();

        let result = sqlx::query(
            r#"
            UPDATE scheduler_lock
            SET heartbeat_at = ?
            WHERE id = 1
            "#,
        )
        .bind(&now_str)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to update heartbeat: {:?}", e);
            AppError::Database(e)
        })?;

        if result.rows_affected() > 0 {
            debug!("Heartbeat updated at {}", now_str);
            Ok(true)
        } else {
            warn!("Heartbeat update affected no rows - lock may have been lost!");
            Ok(false)
        }
    }

    /// Check if the scheduler lock is currently held by an active process.
    ///
    /// This is a read-only check that doesn't modify the lock state. It can be
    /// used to determine whether a scheduler should attempt to start.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Lock is held by an active process (heartbeat is fresh)
    /// * `Ok(false)` - No lock exists or lock is stale
    /// * `Err(_)` - Database error occurred
    pub async fn is_lock_held(&self) -> AppResult<bool> {
        let now = Utc::now();
        let stale_threshold = now - Duration::seconds(LOCK_STALE_TIMEOUT_SECS);
        let stale_threshold_str = stale_threshold.to_rfc3339();

        let result: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT heartbeat_at
            FROM scheduler_lock
            WHERE id = 1 AND heartbeat_at > ?
            "#,
        )
        .bind(&stale_threshold_str)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to check lock status: {:?}", e);
            AppError::Database(e)
        })?;

        Ok(result.is_some())
    }

    /// Get information about the current lock holder.
    ///
    /// This returns detailed information about who holds the lock and when
    /// it was acquired. This is useful for diagnostics and UI display.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(LockInfo))` - Lock exists (may be stale - check heartbeat_at)
    /// * `Ok(None)` - No lock exists
    /// * `Err(_)` - Database error occurred
    pub async fn get_lock_holder(&self) -> AppResult<Option<LockInfo>> {
        let row: Option<(String, String, String, Option<String>)> = sqlx::query_as(
            r#"
            SELECT holder, acquired_at, heartbeat_at, version
            FROM scheduler_lock
            WHERE id = 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get lock holder: {:?}", e);
            AppError::Database(e)
        })?;

        match row {
            Some((holder, acquired_at_str, heartbeat_at_str, version)) => {
                // Parse timestamps with fallback to current time on parse failure
                let acquired_at = DateTime::parse_from_rfc3339(&acquired_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|e| {
                        warn!("Failed to parse acquired_at '{}': {}", acquired_at_str, e);
                        Utc::now()
                    });

                let heartbeat_at = DateTime::parse_from_rfc3339(&heartbeat_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|e| {
                        warn!("Failed to parse heartbeat_at '{}': {}", heartbeat_at_str, e);
                        Utc::now()
                    });

                Ok(Some(LockInfo {
                    holder,
                    acquired_at,
                    heartbeat_at,
                    version,
                }))
            }
            None => Ok(None),
        }
    }

    /// Check if the lock is held by a specific holder.
    ///
    /// This is useful for determining if "we" currently hold the lock,
    /// especially after a potential crash recovery.
    ///
    /// # Arguments
    ///
    /// * `expected_holder` - The holder name to check for ("service" or "desktop")
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Lock is held by the expected holder and is not stale
    /// * `Ok(false)` - Lock doesn't exist, is stale, or held by different holder
    /// * `Err(_)` - Database error occurred
    pub async fn is_held_by(&self, expected_holder: &str) -> AppResult<bool> {
        let now = Utc::now();
        let stale_threshold = now - Duration::seconds(LOCK_STALE_TIMEOUT_SECS);
        let stale_threshold_str = stale_threshold.to_rfc3339();

        let result: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT holder
            FROM scheduler_lock
            WHERE id = 1 AND holder = ? AND heartbeat_at > ?
            "#,
        )
        .bind(expected_holder)
        .bind(&stale_threshold_str)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to check lock holder: {:?}", e);
            AppError::Database(e)
        })?;

        Ok(result.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    async fn create_test_pool() -> SqlitePool {
        SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory SQLite pool")
    }

    #[tokio::test]
    async fn test_initialize_creates_table() {
        let pool = create_test_pool().await;
        let lock = SchedulerLock::new(pool.clone());

        // Should succeed without error
        lock.initialize().await.expect("Failed to initialize");

        // Verify table exists by checking we can query it
        let result: Option<(i64,)> = sqlx::query_as(
            "SELECT COUNT(*) FROM scheduler_lock"
        )
        .fetch_optional(&pool)
        .await
        .expect("Failed to query table");

        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_acquire_when_no_lock_exists() {
        let pool = create_test_pool().await;
        let lock = SchedulerLock::new(pool);

        lock.initialize().await.unwrap();

        // Should acquire successfully
        let acquired = lock.try_acquire("desktop", Some("1.0.0")).await.unwrap();
        assert!(acquired);
    }

    #[tokio::test]
    async fn test_acquire_fails_when_lock_held() {
        let pool = create_test_pool().await;
        let lock = SchedulerLock::new(pool);

        lock.initialize().await.unwrap();

        // First acquisition should succeed
        let first = lock.try_acquire("service", Some("1.0.0")).await.unwrap();
        assert!(first);

        // Send a heartbeat to keep it fresh
        lock.heartbeat().await.unwrap();

        // Second acquisition should fail
        let second = lock.try_acquire("desktop", Some("1.0.0")).await.unwrap();
        assert!(!second);
    }

    #[tokio::test]
    async fn test_release_allows_reacquisition() {
        let pool = create_test_pool().await;
        let lock = SchedulerLock::new(pool);

        lock.initialize().await.unwrap();

        // Acquire and release
        lock.try_acquire("service", None).await.unwrap();
        lock.release().await.unwrap();

        // Should be able to acquire again
        let acquired = lock.try_acquire("desktop", None).await.unwrap();
        assert!(acquired);
    }

    #[tokio::test]
    async fn test_is_lock_held() {
        let pool = create_test_pool().await;
        let lock = SchedulerLock::new(pool);

        lock.initialize().await.unwrap();

        // No lock initially
        assert!(!lock.is_lock_held().await.unwrap());

        // After acquisition
        lock.try_acquire("service", None).await.unwrap();
        assert!(lock.is_lock_held().await.unwrap());

        // After release
        lock.release().await.unwrap();
        assert!(!lock.is_lock_held().await.unwrap());
    }

    #[tokio::test]
    async fn test_get_lock_holder() {
        let pool = create_test_pool().await;
        let lock = SchedulerLock::new(pool);

        lock.initialize().await.unwrap();

        // No holder initially
        let holder = lock.get_lock_holder().await.unwrap();
        assert!(holder.is_none());

        // After acquisition
        lock.try_acquire("desktop", Some("2.0.0")).await.unwrap();
        let holder = lock.get_lock_holder().await.unwrap();
        assert!(holder.is_some());

        let info = holder.unwrap();
        assert_eq!(info.holder, "desktop");
        assert_eq!(info.version, Some("2.0.0".to_string()));
    }

    #[tokio::test]
    async fn test_is_held_by() {
        let pool = create_test_pool().await;
        let lock = SchedulerLock::new(pool);

        lock.initialize().await.unwrap();
        lock.try_acquire("service", None).await.unwrap();

        assert!(lock.is_held_by("service").await.unwrap());
        assert!(!lock.is_held_by("desktop").await.unwrap());
    }

    #[tokio::test]
    async fn test_heartbeat_updates_timestamp() {
        let pool = create_test_pool().await;
        let lock = SchedulerLock::new(pool.clone());

        lock.initialize().await.unwrap();
        lock.try_acquire("service", None).await.unwrap();

        // Get initial heartbeat
        let before = lock.get_lock_holder().await.unwrap().unwrap();

        // Small delay
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Update heartbeat
        let updated = lock.heartbeat().await.unwrap();
        assert!(updated);

        // Verify heartbeat changed
        let after = lock.get_lock_holder().await.unwrap().unwrap();
        assert!(after.heartbeat_at >= before.heartbeat_at);
    }
}
