use sqlx::SqlitePool;
use uuid::Uuid;

use crate::errors::AppResult;
use crate::models::{CreateQuotaResultRequest, QuotaResult, QuotaResultRow};

pub struct QuotaResultRepository;

impl QuotaResultRepository {
    pub async fn get_by_id(pool: &SqlitePool, id: i32) -> AppResult<Option<QuotaResult>> {
        let row = sqlx::query_as::<_, QuotaResultRow>("SELECT * FROM quota_results WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;
        Ok(row.map(QuotaResult::from))
    }

    pub async fn get_paginated(pool: &SqlitePool, offset: i64, limit: i64) -> AppResult<(Vec<QuotaResult>, i64)> {
        let rows = sqlx::query_as::<_, QuotaResultRow>(
            "SELECT * FROM quota_results ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM quota_results")
            .fetch_one(pool)
            .await?;

        Ok((rows.into_iter().map(QuotaResult::from).collect(), count.0))
    }

    pub async fn get_by_line_id(pool: &SqlitePool, line_id: i32, limit: i64) -> AppResult<Vec<QuotaResult>> {
        let rows = sqlx::query_as::<_, QuotaResultRow>(
            "SELECT * FROM quota_results WHERE line_id = $1 ORDER BY created_at DESC LIMIT $2"
        )
        .bind(line_id)
        .bind(limit)
        .fetch_all(pool)
        .await?;
        Ok(rows.into_iter().map(QuotaResult::from).collect())
    }

    pub async fn get_latest_by_line_id(pool: &SqlitePool, line_id: i32) -> AppResult<Option<QuotaResult>> {
        let row = sqlx::query_as::<_, QuotaResultRow>(
            "SELECT * FROM quota_results WHERE line_id = $1 ORDER BY created_at DESC LIMIT 1"
        )
        .bind(line_id)
        .fetch_optional(pool)
        .await?;
        Ok(row.map(QuotaResult::from))
    }

    pub async fn get_by_process_id(pool: &SqlitePool, process_id: Uuid) -> AppResult<Vec<QuotaResult>> {
        let rows = sqlx::query_as::<_, QuotaResultRow>(
            "SELECT * FROM quota_results WHERE process_id = $1 ORDER BY created_at DESC"
        )
        .bind(process_id.to_string())
        .fetch_all(pool)
        .await?;
        Ok(rows.into_iter().map(QuotaResult::from).collect())
    }

    pub async fn get_latest_for_all_lines(pool: &SqlitePool) -> AppResult<Vec<QuotaResult>> {
        let rows = sqlx::query_as::<_, QuotaResultRow>(
            r#"
            SELECT * FROM quota_results WHERE id IN (
                SELECT MAX(id) FROM quota_results GROUP BY line_id
            )
            ORDER BY line_id
            "#
        )
        .fetch_all(pool)
        .await?;
        Ok(rows.into_iter().map(QuotaResult::from).collect())
    }

    pub async fn create(pool: &SqlitePool, req: &CreateQuotaResultRequest) -> AppResult<QuotaResult> {
        // Convert f64 values to strings for TEXT columns
        let balance_str = req.balance.map(|v| v.to_string());
        let quota_percentage_str = req.quota_percentage.map(|v| v.to_string());
        let used_quota_str = req.used_quota.map(|v| v.to_string());
        let remaining_quota_str = req.remaining_quota.map(|v| v.to_string());
        let total_quota_str = req.total_quota.map(|v| v.to_string());
        let renewal_cost_str = req.renewal_cost.map(|v| v.to_string());
        let extra_quota_str = req.extra_quota.map(|v| v.to_string());
        let renewal_date_str = req.renewal_date.map(|d| d.format("%Y-%m-%d").to_string());

        // Insert and get last inserted ID
        let result = sqlx::query(
            r#"
            INSERT INTO quota_results (line_id, process_id, balance, quota_percentage, used_quota, remaining_quota, total_quota, renewal_date, renewal_cost, extra_quota, status, message)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#
        )
        .bind(req.line_id)
        .bind(req.process_id.to_string())
        .bind(balance_str)
        .bind(quota_percentage_str)
        .bind(used_quota_str)
        .bind(remaining_quota_str)
        .bind(total_quota_str)
        .bind(renewal_date_str)
        .bind(renewal_cost_str)
        .bind(extra_quota_str)
        .bind(req.status.clone())
        .bind(req.message.clone())
        .execute(pool)
        .await?;

        let id = result.last_insert_rowid() as i32;

        // Fetch the created record
        Self::get_by_id(pool, id)
            .await?
            .ok_or_else(|| crate::errors::AppError::NotFound(format!("QuotaResult with id {} not found after insert", id)))
    }

    pub async fn delete_old(pool: &SqlitePool, days: i64) -> AppResult<u64> {
        let result: sqlx::sqlite::SqliteQueryResult = sqlx::query(
            "DELETE FROM quota_results WHERE created_at < datetime('now', '-' || $1 || ' days')"
        )
        .bind(days)
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }
}
