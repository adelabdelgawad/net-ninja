use sqlx::SqlitePool;

mod pool;
pub use pool::create_sqlite_pool;

mod migrations;
pub use migrations::run_pending_migrations;

/// Create a SQLite connection pool
///
/// This is a convenience function that uses the platform-specific database path.
pub async fn create_pool() -> Result<SqlitePool, crate::errors::AppError> {
    create_sqlite_pool().await
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_compiles() {
        // Basic compilation test
        assert!(true);
    }
}
