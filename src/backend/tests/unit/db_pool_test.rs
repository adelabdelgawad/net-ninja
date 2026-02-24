use net_ninja::db::DbPool;
use net_ninja::config::DatabaseType;

#[tokio::test]
async fn test_connect_sqlite_memory() {
    let pool = DbPool::connect("sqlite::memory:").await.unwrap();
    assert_eq!(pool.db_type(), DatabaseType::SQLite);
}

#[tokio::test]
async fn test_connect_invalid_url() {
    let result = DbPool::connect("invalid://localhost/db").await;
    assert!(result.is_err());
}

#[test]
fn test_db_type_detection() {
    // This will be tested via connect tests above
    // Just verify the enum variants exist
    let pg_url = "postgresql://localhost/test";
    let mysql_url = "mysql://localhost/test";
    let sqlite_url = "sqlite::memory:";

    assert!(DatabaseType::from_url(pg_url).is_ok());
    assert!(DatabaseType::from_url(mysql_url).is_ok());
    assert!(DatabaseType::from_url(sqlite_url).is_ok());
}

#[tokio::test]
async fn test_close_connection() {
    let pool = DbPool::connect("sqlite::memory:").await.unwrap();
    pool.close().await;
    // If no panic, test passes
}

#[tokio::test]
async fn test_as_postgres() {
    let pool = DbPool::connect("sqlite::memory:").await.unwrap();

    // SQLite pool cannot be accessed as PostgreSQL
    assert!(pool.as_postgres().is_none());
}

#[tokio::test]
async fn test_as_sqlite() {
    let pool = DbPool::connect("sqlite::memory:").await.unwrap();

    // SQLite pool can be accessed as SQLite
    assert!(pool.as_sqlite().is_some());
}

#[tokio::test]
async fn test_as_mysql() {
    let pool = DbPool::connect("sqlite::memory:").await.unwrap();

    // SQLite pool cannot be accessed as MySQL
    assert!(pool.as_mysql().is_none());
}
