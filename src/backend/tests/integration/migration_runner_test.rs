use net_ninja::db::{DbPool, get_migration_status, run_pending_migrations, MigrationStatus};

#[tokio::test]
async fn test_migration_status_check() {
    let pool = DbPool::connect("sqlite::memory:").await.unwrap();

    // Initially no migrations applied
    let status = get_migration_status(&pool).await.unwrap();

    // All migrations should be pending (at least the migration tracking table)
    assert!(status.iter().any(|info| info.status == MigrationStatus::Pending));
}

#[tokio::test]
async fn test_run_migrations_sqlite() {
    let pool = DbPool::connect("sqlite::memory:").await.unwrap();

    // Run migrations
    let count = run_pending_migrations(&pool).await.unwrap();

    // Should have applied at least the migration tracking table
    assert!(count > 0);

    // Running again should apply no new migrations
    let count2 = run_pending_migrations(&pool).await.unwrap();
    assert_eq!(count2, 0);
}

#[tokio::test]
async fn test_migration_idempotency() {
    let pool = DbPool::connect("sqlite::memory:").await.unwrap();

    // Run migrations twice
    run_pending_migrations(&pool).await.unwrap();
    let count = run_pending_migrations(&pool).await.unwrap();

    // Second run should find no pending migrations
    assert_eq!(count, 0);
}
