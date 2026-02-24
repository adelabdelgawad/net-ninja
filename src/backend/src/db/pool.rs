//! SQLite database connection pool
//!
//! Provides direct SQLite pool creation without multi-database abstraction.
//!
//! # Concurrency Model: SYSTEM Service + Desktop App
//!
//! This module is designed to support concurrent database access between two separate processes:
//!
//! 1. **SYSTEM Service Process**: A background Windows service running as SYSTEM user that performs
//!    scheduled tasks like quota checking and speed tests. This process has read-write access to
//!    insert new results and update task schedules.
//!
//! 2. **Desktop UI Process**: The Tauri application running in the current user's session. This
//!    process primarily reads data for display but may also write configuration changes.
//!
//! ## Why WAL Mode is Required
//!
//! SQLite's default rollback journal mode uses exclusive file locks during writes, which blocks
//! all other readers and writers. This causes problems when:
//! - The SYSTEM service is writing speed test results while the user views the dashboard
//! - The desktop app reads task schedules while the service updates job status
//! - Both processes attempt concurrent database operations
//!
//! **Write-Ahead Logging (WAL)** mode solves this by:
//! - Allowing readers to proceed without blocking during writes
//! - Enabling multiple concurrent readers (including from different processes)
//! - Only blocking when multiple processes attempt simultaneous writes
//! - Providing better crash recovery since the main database file is not modified during writes
//!
//! ## busy_timeout Configuration
//!
//! Even with WAL mode, write operations still require an exclusive lock. The `busy_timeout`
//! pragma tells SQLite how long to wait (in milliseconds) when the database is locked by
//! another process before returning SQLITE_BUSY. A 5000ms timeout accommodates:
//! - Network latency when writing large result sets
//! - Occasional slow disk I/O
//! - Brief contention during service startup/shutdown
//!
//! Without busy_timeout, concurrent writes would immediately fail with "database is locked".

use sqlx::SqlitePool;
use crate::config::get_sqlite_path;
use crate::errors::AppError;

/// Create a read-write SQLite connection pool with WAL mode enabled.
///
/// This pool is intended for processes that need full database access (read + write).
/// It enables WAL mode and sets appropriate timeouts for concurrent access scenarios.
///
/// The database file is stored at a platform-specific location determined by `get_sqlite_path()`.
/// Uses the `mode=rwc` flag to automatically create the database if it doesn't exist.
///
/// # WAL Mode Configuration
///
/// - `journal_mode=WAL`: Enables Write-Ahead Logging for concurrent read/write access
/// - `busy_timeout=5000`: Wait up to 5 seconds for locks before failing
/// - `foreign_keys=ON`: Enforce referential integrity constraints
///
/// # Usage
///
/// Use this pool for:
/// - The SYSTEM service process (needs write access for results)
/// - The desktop app when write access is required (settings changes)
pub async fn create_sqlite_pool() -> Result<SqlitePool, AppError> {
    let db_path = get_sqlite_path();
    let database_url = format!("sqlite:{}?mode=rwc", db_path.display());

    tracing::info!("Creating read-write SQLite pool at: {}", db_path.display());

    let pool = SqlitePool::connect(&database_url).await.map_err(|e| {
        tracing::error!("Failed to connect to SQLite database: {:?}", e);
        AppError::Database(e)
    })?;

    // Enable WAL mode for concurrent access between SYSTEM service and desktop processes.
    // WAL mode allows readers to proceed without blocking during writes, which is essential
    // when the background service is writing results while the UI reads data for display.
    // Note: WAL mode is persistent - once set, it remains until explicitly changed.
    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to enable WAL mode: {:?}", e);
            AppError::Database(e)
        })?;

    // Set busy_timeout to 5000ms (5 seconds) to handle lock contention gracefully.
    // When the SYSTEM service and desktop app both attempt writes, one will wait up to
    // 5 seconds for the lock rather than immediately failing with SQLITE_BUSY.
    // This is especially important during service startup when multiple initialization
    // queries may run concurrently.
    sqlx::query("PRAGMA busy_timeout=5000")
        .execute(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to set busy_timeout: {:?}", e);
            AppError::Database(e)
        })?;

    // Enable foreign keys for referential integrity.
    // SQLite has foreign keys disabled by default for backward compatibility.
    sqlx::query("PRAGMA foreign_keys=ON")
        .execute(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to enable foreign keys: {:?}", e);
            AppError::Database(e)
        })?;

    tracing::info!("SQLite connection pool created successfully with WAL mode enabled");
    Ok(pool)
}
