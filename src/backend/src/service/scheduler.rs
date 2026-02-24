//! Scheduler loop implementation for Windows Service mode
//!
//! This module provides the core scheduler loop that runs when NetNinja operates
//! as a Windows Service. It handles:
//!
//! - Acquiring an exclusive scheduler lock to prevent multiple instances
//! - Writing a version stamp to the database on startup
//! - Running the job scheduler loop
//! - Listening for shutdown signals
//! - Performing graceful cleanup
//!
//! # Lock Mechanism
//!
//! This module uses the `SchedulerLock` from `crate::service::lock` for coordination.
//! Only one scheduler instance should run at a time to prevent duplicate job
//! execution. The lock is:
//! - Acquired on startup
//! - Updated periodically via heartbeat to indicate the scheduler is still alive
//! - Released on graceful shutdown
//! - Considered stale after a timeout (allows recovery from crashes)

use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use sqlx::SqlitePool;
use tokio::sync::oneshot;
use tokio::time::interval;

use crate::config::{paths, Settings};
use crate::db::run_pending_migrations;
use crate::errors::{AppError, AppResult};
use crate::jobs::JobRunner;
use crate::service::lock::SchedulerLock;

/// Heartbeat interval in seconds.
///
/// How often the running scheduler updates its heartbeat timestamp.
/// This must be significantly less than the lock stale timeout (60s).
const HEARTBEAT_INTERVAL_SECS: u64 = 15;

/// Service version written to database on startup.
///
/// This helps identify which version of the service is running.
const SERVICE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Handle to a running scheduler that can be used to trigger shutdown.
///
/// This struct is returned by `run_scheduler_loop()` and provides a way
/// to signal the scheduler to stop and wait for graceful shutdown.
pub struct SchedulerHandle {
    /// Channel to send shutdown signal
    shutdown_tx: Option<oneshot::Sender<()>>,
    /// Join handle for the main scheduler task
    join_handle: Option<tokio::task::JoinHandle<AppResult<()>>>,
}

impl SchedulerHandle {
    /// Signal the scheduler to shut down and wait for it to complete.
    ///
    /// This performs a graceful shutdown:
    /// 1. Sends shutdown signal to the scheduler loop
    /// 2. Waits for in-progress jobs to complete
    /// 3. Releases the scheduler lock
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if shutdown was successful, or an error if the
    /// scheduler encountered problems during shutdown.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let handle = run_scheduler_loop().await?;
    /// // ... wait for shutdown signal ...
    /// handle.shutdown().await?;
    /// ```
    pub async fn shutdown(mut self) -> AppResult<()> {
        // Send shutdown signal
        if let Some(tx) = self.shutdown_tx.take() {
            // Ignore send error - receiver may have already dropped
            let _ = tx.send(());
        }

        // Wait for the scheduler task to complete
        if let Some(handle) = self.join_handle.take() {
            match handle.await {
                Ok(result) => result,
                Err(e) => Err(AppError::Scheduler(format!(
                    "Scheduler task panicked: {}",
                    e
                ))),
            }
        } else {
            Ok(())
        }
    }
}

/// Start the scheduler loop.
///
/// This function:
/// 1. Connects to the database at the service-specific path
/// 2. Runs any pending migrations
/// 3. Acquires the scheduler lock (fails immediately if held by another instance)
/// 4. Writes the service version stamp to the database
/// 5. Starts the job scheduler
/// 6. Returns a handle that can be used to trigger shutdown
///
/// # Lock Acquisition
///
/// The scheduler lock is acquired using `SchedulerLock::try_acquire()`.
/// If another instance holds the lock and the heartbeat is recent (within
/// 60 seconds), this function returns an error immediately rather than waiting.
///
/// # Database Path
///
/// On Windows, the database is stored at `%ProgramData%\NetNinja\netninja.db`.
/// This path is shared between the service and desktop application.
///
/// # Returns
///
/// Returns a `SchedulerHandle` that can be used to trigger graceful shutdown,
/// or an error if:
/// - The lock is already held by another instance
/// - Database connection fails
/// - Job registration fails
///
/// # Example
///
/// ```ignore
/// let handle = run_scheduler_loop().await?;
/// // Service is now running...
/// // When shutdown is requested:
/// handle.shutdown().await?;
/// ```
pub async fn run_scheduler_loop() -> AppResult<SchedulerHandle> {
    tracing::info!("Starting scheduler loop...");

    // Load .env from ProgramData so service picks up CHROME_PATH and other settings
    #[cfg(windows)]
    {
        let service_env = paths::get_service_config_path();
        if service_env.exists() {
            tracing::info!("Loading service .env from: {:?}", service_env);
            dotenvy::from_path(&service_env).ok();
        }
    }

    // Use service-specific database path on Windows
    let db_path = paths::get_service_sqlite_path();
    let database_url = format!("sqlite:{}?mode=rwc", db_path.display());

    tracing::info!("Connecting to database at: {}", db_path.display());

    // Create database connection pool
    let pool = SqlitePool::connect(&database_url).await.map_err(|e| {
        tracing::error!("Failed to connect to database: {:?}", e);
        AppError::Database(e)
    })?;

    // Configure database pragmas (WAL mode, busy_timeout, foreign_keys)
    configure_database(&pool).await?;

    // Run any pending migrations
    run_pending_migrations(&pool).await?;

    // Initialize and acquire the scheduler lock
    let scheduler_lock = SchedulerLock::new(pool.clone());
    scheduler_lock.initialize().await?;

    // Try to acquire the scheduler lock
    // This will fail immediately if another instance holds an active lock
    if !scheduler_lock.try_acquire("service", Some(SERVICE_VERSION)).await? {
        // Check who holds the lock for a more informative error message
        if let Some(lock_info) = scheduler_lock.get_lock_holder().await? {
            return Err(AppError::Scheduler(format!(
                "Scheduler lock is held by '{}' (version: {}, last heartbeat: {})",
                lock_info.holder,
                lock_info.version.unwrap_or_else(|| "unknown".to_string()),
                lock_info.heartbeat_at.format("%Y-%m-%d %H:%M:%S UTC")
            )));
        } else {
            return Err(AppError::Scheduler(
                "Scheduler lock could not be acquired (race condition?)".to_string()
            ));
        }
    }

    tracing::info!("Scheduler lock acquired");

    // Write service version stamp to database
    write_version_stamp(&pool).await?;

    // Load settings
    let settings = Arc::new(Settings::load()?);

    // Create and start the job runner
    let job_runner = JobRunner::with_pool(settings, pool.clone()).await?;
    job_runner.register_jobs().await?;
    job_runner.start().await?;

    // Create shutdown channel
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    // Spawn the main scheduler loop task
    let pool_clone = pool.clone();
    let join_handle = tokio::spawn(async move {
        run_loop_inner(pool_clone, scheduler_lock, shutdown_rx, job_runner).await
    });

    Ok(SchedulerHandle {
        shutdown_tx: Some(shutdown_tx),
        join_handle: Some(join_handle),
    })
}

/// Inner loop that runs until shutdown is signaled.
///
/// This function:
/// - Periodically updates the heartbeat timestamp via `SchedulerLock::heartbeat()`
/// - Waits for the shutdown signal
/// - Performs cleanup when shutdown is received
///
/// # Arguments
///
/// * `scheduler_lock` - Lock instance for heartbeat updates
/// * `shutdown_rx` - Channel receiver for shutdown signal
/// * `job_runner` - The job runner to shut down on exit
async fn run_loop_inner(
    _pool: SqlitePool,  // Kept for potential future use
    scheduler_lock: SchedulerLock,
    mut shutdown_rx: oneshot::Receiver<()>,
    mut job_runner: JobRunner,
) -> AppResult<()> {
    let mut heartbeat_interval = interval(Duration::from_secs(HEARTBEAT_INTERVAL_SECS));

    loop {
        tokio::select! {
            // Handle shutdown signal
            _ = &mut shutdown_rx => {
                tracing::info!("Shutdown signal received, cleaning up...");
                break;
            }

            // Update heartbeat periodically
            _ = heartbeat_interval.tick() => {
                match scheduler_lock.heartbeat().await {
                    Ok(true) => {
                        tracing::trace!("Heartbeat updated successfully");
                    }
                    Ok(false) => {
                        // Lock was lost - this is a serious error
                        tracing::error!("Scheduler lock was lost! Another instance may have taken over.");
                        // Continue running but log the warning
                        // The service should ideally be restarted in this case
                    }
                    Err(e) => {
                        tracing::warn!("Failed to update heartbeat: {}", e);
                        // Continue running - a single heartbeat failure isn't fatal
                    }
                }
            }
        }
    }

    // Graceful shutdown
    tracing::info!("Stopping job scheduler...");
    job_runner.shutdown().await?;

    tracing::info!("Releasing scheduler lock...");
    scheduler_lock.release().await?;

    tracing::info!("Scheduler loop stopped");
    Ok(())
}

/// Configure database pragmas for service mode.
///
/// Enables WAL mode and sets appropriate timeouts for service operation.
/// This is essential for concurrent access between the service and desktop app.
///
/// # Configuration
///
/// - `journal_mode=WAL`: Enables Write-Ahead Logging for concurrent read/write access
/// - `busy_timeout=5000`: Wait up to 5 seconds for locks before failing
/// - `foreign_keys=ON`: Enforce referential integrity constraints
async fn configure_database(pool: &SqlitePool) -> AppResult<()> {
    // Enable WAL mode for concurrent access between service and desktop
    // WAL mode allows readers to proceed without blocking during writes
    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to enable WAL mode: {:?}", e);
            AppError::Database(e)
        })?;

    // Set busy timeout for lock contention
    // When both service and desktop app attempt writes, one will wait
    sqlx::query("PRAGMA busy_timeout=5000")
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to set busy_timeout: {:?}", e);
            AppError::Database(e)
        })?;

    // Enable foreign keys for referential integrity
    sqlx::query("PRAGMA foreign_keys=ON")
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to enable foreign keys: {:?}", e);
            AppError::Database(e)
        })?;

    tracing::debug!("Database pragmas configured (WAL mode, busy_timeout=5000ms)");
    Ok(())
}

/// Write the service version stamp to the database.
///
/// This creates or updates records in a `service_info` table that stores:
/// - Service version
/// - Startup timestamp
///
/// This information is useful for debugging and verifying which version
/// of the service is running, especially when troubleshooting issues.
///
/// # Table Schema
///
/// ```sql
/// CREATE TABLE IF NOT EXISTS service_info (
///     key TEXT PRIMARY KEY,
///     value TEXT NOT NULL,
///     updated_at TEXT NOT NULL
/// )
/// ```
async fn write_version_stamp(pool: &SqlitePool) -> AppResult<()> {
    // Ensure the table exists
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS service_info (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create service_info table: {:?}", e);
        AppError::Database(e)
    })?;

    let now = Utc::now().to_rfc3339();

    // Upsert service version
    sqlx::query(
        r#"
        INSERT INTO service_info (key, value, updated_at)
        VALUES ('service_version', ?, ?)
        ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at
        "#,
    )
    .bind(SERVICE_VERSION)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to write service version: {:?}", e);
        AppError::Database(e)
    })?;

    // Upsert last startup timestamp
    sqlx::query(
        r#"
        INSERT INTO service_info (key, value, updated_at)
        VALUES ('last_startup', ?, ?)
        ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at
        "#,
    )
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to write last_startup: {:?}", e);
        AppError::Database(e)
    })?;

    tracing::info!("Service version stamp written: v{}", SERVICE_VERSION);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heartbeat_interval_is_reasonable() {
        // Heartbeat must be frequent enough to prevent false stale detection
        // The lock stale timeout is 60 seconds
        assert!(HEARTBEAT_INTERVAL_SECS < 60);
        // Should be at least 2x more frequent for safety
        assert!(HEARTBEAT_INTERVAL_SECS * 2 < 60);
    }

    #[test]
    fn test_service_version_is_set() {
        // Ensure the version is available at compile time
        assert!(!SERVICE_VERSION.is_empty());
    }
}
