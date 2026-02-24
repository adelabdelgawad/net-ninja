#[cfg(test)]
mod mysql_tests {
    use net_ninja::models::types::ProcessId;
    use sqlx::MySqlPool;
    use std::env;

    async fn get_test_pool() -> MySqlPool {
        let url = env::var("TEST_DATABASE_URL_MYSQL")
            .unwrap_or_else(|_| "mysql://root:root@localhost:3306/netninja_test".to_string());
        MySqlPool::connect(&url).await.unwrap()
    }

    #[tokio::test]
    #[ignore] // Run with --ignored flag when MySQL is available
    async fn test_process_id_mysql_roundtrip() {
        let pool = get_test_pool().await;

        // Create temp table
        sqlx::query("CREATE TEMPORARY TABLE test_process (id BINARY(16))")
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
    #[ignore] // Run with --ignored flag when MySQL is available
    #[cfg(feature = "bigdecimal")] // DecimalValue not currently implemented
    async fn test_decimal_value_mysql_roundtrip() {
        use net_ninja::models::types::DecimalValue;
        use sqlx::types::BigDecimal;
        use std::str::FromStr;

        let pool = get_test_pool().await;

        // Create temp table with DECIMAL column
        sqlx::query("CREATE TEMPORARY TABLE test_decimal (value DECIMAL(10, 2))")
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
    #[ignore] // Run with --ignored flag when MySQL is available
    async fn test_timestamp_mysql_roundtrip() {
        use chrono::{DateTime, Utc};
        use net_ninja::models::types::Timestamp;

        let pool = get_test_pool().await;

        // Create temp table with DATETIME column
        sqlx::query("CREATE TEMPORARY TABLE test_timestamp (ts DATETIME)")
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

        // MySQL DATETIME has second precision, so truncate microseconds for comparison
        assert_eq!(
            original.inner().timestamp(),
            retrieved.inner().timestamp()
        );
    }
}
