use net_ninja::db::DbPool;

#[tokio::test]
async fn test_execute_query_sqlite() {
    let pool = DbPool::connect("sqlite::memory:").await.unwrap();

    // CREATE TABLE should succeed without error
    let _rows_affected = pool.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)").await.unwrap();
}

#[tokio::test]
async fn test_insert_query_sqlite() {
    let pool = DbPool::connect("sqlite::memory:").await.unwrap();

    pool.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, value TEXT)").await.unwrap();

    let rows = pool.execute("INSERT INTO test (value) VALUES ('test')").await.unwrap();
    assert_eq!(rows, 1);
}
