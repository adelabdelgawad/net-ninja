//! Repository for the `service_info` key/value table.
//!
//! Used for lightweight operational instrumentation that survives restarts and
//! is shared across the desktop app, web admin, and Windows service (they all
//! open the same SQLite file): the running service version, last startup, and
//! "last successful run" markers per scheduled job.
//!
//! The table is created by migration `20260203000001_create_scheduler_lock_and_service_info`.

use sqlx::SqlitePool;

use crate::errors::AppResult;

/// Key prefix for per-job last-success markers, e.g. `last_success:quota_check`.
pub const LAST_SUCCESS_PREFIX: &str = "last_success:";

pub struct ServiceInfoRepository;

impl ServiceInfoRepository {
    /// Upsert a key/value, stamping `updated_at` with the current UTC time.
    pub async fn set(pool: &SqlitePool, key: &str, value: &str) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO service_info (key, value, updated_at)
            VALUES ($1, $2, datetime('now', 'utc'))
            ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at
            "#,
        )
        .bind(key)
        .bind(value)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Fetch a single (value, updated_at) pair for a key, if present.
    pub async fn get(pool: &SqlitePool, key: &str) -> AppResult<Option<(String, String)>> {
        let row: Option<(String, String)> =
            sqlx::query_as("SELECT value, updated_at FROM service_info WHERE key = $1")
                .bind(key)
                .fetch_optional(pool)
                .await?;
        Ok(row)
    }

    /// Fetch all (key, value, updated_at) rows.
    pub async fn get_all(pool: &SqlitePool) -> AppResult<Vec<(String, String, String)>> {
        let rows: Vec<(String, String, String)> =
            sqlx::query_as("SELECT key, value, updated_at FROM service_info ORDER BY key")
                .fetch_all(pool)
                .await?;
        Ok(rows)
    }

    /// Record that a scheduled job (`quota_check`, `speed_test`, `cleanup`, ...)
    /// just completed successfully. Stores `last_success:<job>` = RFC3339 UTC.
    ///
    /// Best-effort instrumentation: callers log-and-ignore failures so a metric
    /// write never fails the job itself.
    pub async fn record_job_success(pool: &SqlitePool, job: &str) -> AppResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        Self::set(pool, &format!("{LAST_SUCCESS_PREFIX}{job}"), &now).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn pool_with_table() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("in-memory pool");
        sqlx::query(
            "CREATE TABLE service_info (key TEXT PRIMARY KEY, value TEXT NOT NULL, updated_at TEXT NOT NULL)",
        )
        .execute(&pool)
        .await
        .expect("create table");
        pool
    }

    #[tokio::test]
    async fn test_set_is_upsert_and_get_roundtrips() {
        let pool = pool_with_table().await;

        ServiceInfoRepository::set(&pool, "k", "v1").await.unwrap();
        assert_eq!(
            ServiceInfoRepository::get(&pool, "k")
                .await
                .unwrap()
                .map(|(v, _)| v),
            Some("v1".to_string())
        );

        // Second set on the same key updates rather than inserting a duplicate.
        ServiceInfoRepository::set(&pool, "k", "v2").await.unwrap();
        assert_eq!(
            ServiceInfoRepository::get(&pool, "k")
                .await
                .unwrap()
                .map(|(v, _)| v),
            Some("v2".to_string())
        );
        assert_eq!(
            ServiceInfoRepository::get_all(&pool).await.unwrap().len(),
            1
        );
    }

    #[tokio::test]
    async fn test_record_job_success_uses_prefix() {
        let pool = pool_with_table().await;
        ServiceInfoRepository::record_job_success(&pool, "quota_check")
            .await
            .unwrap();

        let rows = ServiceInfoRepository::get_all(&pool).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, format!("{LAST_SUCCESS_PREFIX}quota_check"));
        // Stored value parses as an RFC3339 timestamp.
        assert!(chrono::DateTime::parse_from_rfc3339(&rows[0].1).is_ok());
    }
}
