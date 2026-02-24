use sqlx::SqlitePool;
use uuid::Uuid;

use crate::errors::AppResult;
use crate::models::{CreateSpeedTestResultRequest, SpeedTestResult, SpeedTestResultRow};

pub struct SpeedTestRepository;

impl SpeedTestRepository {
    pub async fn get_by_id(pool: &SqlitePool, id: i32) -> AppResult<Option<SpeedTestResult>> {
        let row = sqlx::query_as::<_, SpeedTestResultRow>("SELECT * FROM speed_tests WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;
        Ok(row.map(SpeedTestResult::from))
    }

    pub async fn get_paginated(pool: &SqlitePool, offset: i64, limit: i64) -> AppResult<(Vec<SpeedTestResult>, i64)> {
        let rows = sqlx::query_as::<_, SpeedTestResultRow>(
            "SELECT * FROM speed_tests ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM speed_tests")
            .fetch_one(pool)
            .await?;

        Ok((rows.into_iter().map(SpeedTestResult::from).collect(), count.0))
    }

    pub async fn get_by_line_id(pool: &SqlitePool, line_id: i32, limit: i64) -> AppResult<Vec<SpeedTestResult>> {
        let rows = sqlx::query_as::<_, SpeedTestResultRow>(
            "SELECT * FROM speed_tests WHERE line_id = $1 ORDER BY created_at DESC LIMIT $2"
        )
        .bind(line_id)
        .bind(limit)
        .fetch_all(pool)
        .await?;
        Ok(rows.into_iter().map(SpeedTestResult::from).collect())
    }

    pub async fn get_latest_by_line_id(pool: &SqlitePool, line_id: i32) -> AppResult<Option<SpeedTestResult>> {
        let row = sqlx::query_as::<_, SpeedTestResultRow>(
            "SELECT * FROM speed_tests WHERE line_id = $1 ORDER BY created_at DESC LIMIT 1"
        )
        .bind(line_id)
        .fetch_optional(pool)
        .await?;
        Ok(row.map(SpeedTestResult::from))
    }

    pub async fn get_by_process_id(pool: &SqlitePool, process_id: Uuid) -> AppResult<Vec<SpeedTestResult>> {
        let rows = sqlx::query_as::<_, SpeedTestResultRow>(
            "SELECT * FROM speed_tests WHERE process_id = $1 ORDER BY created_at DESC"
        )
        .bind(process_id.to_string())
        .fetch_all(pool)
        .await?;
        Ok(rows.into_iter().map(SpeedTestResult::from).collect())
    }

    pub async fn get_latest_for_all_lines(pool: &SqlitePool) -> AppResult<Vec<SpeedTestResult>> {
        let rows = sqlx::query_as::<_, SpeedTestResultRow>(
            r#"
            SELECT * FROM speed_tests WHERE id IN (
                SELECT MAX(id) FROM speed_tests GROUP BY line_id
            )
            ORDER BY line_id
            "#
        )
        .fetch_all(pool)
        .await?;
        Ok(rows.into_iter().map(SpeedTestResult::from).collect())
    }

    pub async fn create(pool: &SqlitePool, req: &CreateSpeedTestResultRequest) -> AppResult<SpeedTestResult> {
        let row = sqlx::query_as::<_, SpeedTestResultRow>(
            r#"
            INSERT INTO speed_tests (line_id, process_id, download_speed, upload_speed, ping, server_name, server_location, public_ip, status, error_message)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#
        )
        .bind(req.line_id)
        .bind(req.process_id.to_string())
        .bind(req.download_speed)
        .bind(req.upload_speed)
        .bind(req.ping)
        .bind(&req.server_name)
        .bind(&req.server_location)
        .bind(&req.public_ip)
        .bind(&req.status)
        .bind(&req.error_message)
        .fetch_one(pool)
        .await?;
        Ok(SpeedTestResult::from(row))
    }

    pub async fn delete_old(pool: &SqlitePool, days: i64) -> AppResult<u64> {
        let result: sqlx::sqlite::SqliteQueryResult = sqlx::query(
            "DELETE FROM speed_tests WHERE created_at < datetime('now', '-' || $1 || ' days')"
        )
        .bind(days)
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }
}
