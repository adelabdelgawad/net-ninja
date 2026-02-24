#[cfg(test)]
mod postgres_tests {
    use net_ninja::models::types::ProcessId;
    use sqlx::PgPool;
    use std::env;

    async fn get_test_pool() -> PgPool {
        let url = env::var("TEST_DATABASE_URL_POSTGRES")
            .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/netninja_test".to_string());
        PgPool::connect(&url).await.unwrap()
    }

    #[tokio::test]
    #[ignore] // Run with --ignored flag when PG is available
    async fn test_process_id_postgres_roundtrip() {
        let pool = get_test_pool().await;

        // Create temp table
        sqlx::query("CREATE TEMP TABLE test_process (id UUID)")
            .execute(&pool)
            .await
            .unwrap();

        let original = ProcessId::new();

        // Insert
        sqlx::query("INSERT INTO test_process (id) VALUES ($1)")
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
    #[ignore] // Run with --ignored flag when PG is available
    #[cfg(feature = "bigdecimal")] // DecimalValue not currently implemented
    async fn test_decimal_value_postgres_roundtrip() {
        use net_ninja::models::types::DecimalValue;
        use sqlx::types::BigDecimal;
        use std::str::FromStr;

        let pool = get_test_pool().await;

        // Create temp table with NUMERIC column
        sqlx::query("CREATE TEMP TABLE test_decimal (value NUMERIC(10, 2))")
            .execute(&pool)
            .await
            .unwrap();

        // Test value: 123.45
        let original = DecimalValue::from(BigDecimal::from_str("123.45").unwrap());

        // Insert
        sqlx::query("INSERT INTO test_decimal (value) VALUES ($1)")
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
    #[ignore] // Run with --ignored flag when PG is available
    async fn test_timestamp_postgres_roundtrip() {
        use chrono::{DateTime, Utc};
        use net_ninja::models::types::Timestamp;

        let pool = get_test_pool().await;

        // Create temp table with TIMESTAMPTZ column
        sqlx::query("CREATE TEMP TABLE test_timestamp (ts TIMESTAMPTZ)")
            .execute(&pool)
            .await
            .unwrap();

        // Test value: specific UTC timestamp
        let dt = DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let original = Timestamp::from(dt);

        // Insert
        sqlx::query("INSERT INTO test_timestamp (ts) VALUES ($1)")
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
