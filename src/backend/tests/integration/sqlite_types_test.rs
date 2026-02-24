#[cfg(test)]
mod sqlite_tests {
    use net_ninja::models::types::ProcessId;
    use sqlx::SqlitePool;

    async fn get_test_pool() -> SqlitePool {
        SqlitePool::connect("sqlite::memory:").await.unwrap()
    }

    #[tokio::test]
    async fn test_process_id_sqlite_roundtrip() {
        let pool = get_test_pool().await;

        // Create table
        sqlx::query("CREATE TABLE test_process (id TEXT)")
            .execute(&pool)
            .await
            .unwrap();

        let original = ProcessId::new();

        // Insert
        sqlx::query("INSERT INTO test_process (id) VALUES (?)")
            .bind(original)
            .execute(&pool)
            .await
            .unwrap();

        // Retrieve
        let retrieved: ProcessId = sqlx::query_scalar("SELECT id FROM test_process LIMIT 1")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(original, retrieved);
    }

    #[tokio::test]
    #[cfg(feature = "bigdecimal")] // DecimalValue not currently implemented
    async fn test_decimal_value_sqlite_roundtrip() {
        use net_ninja::models::types::DecimalValue;
        use sqlx::types::BigDecimal;
        use std::str::FromStr;

        let pool = get_test_pool().await;

        // Create table with TEXT column (SQLite stores decimals as TEXT)
        sqlx::query("CREATE TABLE test_decimal (value TEXT)")
            .execute(&pool)
            .await
            .unwrap();

        // Test value: 123.45
        let original = DecimalValue::from(BigDecimal::from_str("123.45").unwrap());

        // Insert
        sqlx::query("INSERT INTO test_decimal (value) VALUES (?)")
            .bind(&original)
            .execute(&pool)
            .await
            .unwrap();

        // Retrieve
        let retrieved: DecimalValue = sqlx::query_scalar("SELECT value FROM test_decimal LIMIT 1")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(original, retrieved);
    }

    #[tokio::test]
    async fn test_timestamp_sqlite_roundtrip() {
        use chrono::{DateTime, Utc};
        use net_ninja::models::types::Timestamp;

        let pool = get_test_pool().await;

        // Create table with TEXT column (SQLite stores timestamps as TEXT in ISO-8601 format)
        sqlx::query("CREATE TABLE test_timestamp (ts TEXT)")
            .execute(&pool)
            .await
            .unwrap();

        // Test value: specific UTC timestamp
        let dt = DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let original = Timestamp::from(dt);

        // Insert
        sqlx::query("INSERT INTO test_timestamp (ts) VALUES (?)")
            .bind(&original)
            .execute(&pool)
            .await
            .unwrap();

        // Retrieve
        let retrieved: Timestamp = sqlx::query_scalar("SELECT ts FROM test_timestamp LIMIT 1")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(original, retrieved);
    }
}
