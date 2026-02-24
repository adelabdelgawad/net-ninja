use sqlx::SqlitePool;
use uuid::Uuid;

use crate::errors::AppResult;
use crate::models::{CreateLogRequest, Log, LogFilter, LogRow};

pub struct LogRepository;

impl LogRepository {
    pub async fn get_by_id(pool: &SqlitePool, id: i32) -> AppResult<Option<Log>> {
        let row = sqlx::query_as::<_, LogRow>("SELECT * FROM logs WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;
        Ok(row.map(Log::from))
    }

    pub async fn get_paginated(pool: &SqlitePool, offset: i64, limit: i64) -> AppResult<(Vec<Log>, i64)> {
        let rows = sqlx::query_as::<_, LogRow>(
            "SELECT * FROM logs ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM logs")
            .fetch_one(pool)
            .await?;

        Ok((rows.into_iter().map(Log::from).collect(), count.0))
    }

    pub async fn get_by_process_id(pool: &SqlitePool, process_id: Uuid) -> AppResult<Vec<Log>> {
        let rows = sqlx::query_as::<_, LogRow>(
            "SELECT * FROM logs WHERE process_id = $1 ORDER BY created_at ASC"
        )
        .bind(process_id.to_string())
        .fetch_all(pool)
        .await?;
        Ok(rows.into_iter().map(Log::from).collect())
    }

    pub async fn get_filtered(
        pool: &SqlitePool,
        filter: &LogFilter,
        offset: i64,
        limit: i64,
    ) -> AppResult<(Vec<Log>, i64)> {
        let mut query = String::from("SELECT * FROM logs WHERE 1=1");
        let mut count_query = String::from("SELECT COUNT(*) FROM logs WHERE 1=1");

        if filter.process_id.is_some() {
            query.push_str(" AND process_id = $1");
            count_query.push_str(" AND process_id = $1");
        }
        if filter.level.is_some() {
            query.push_str(" AND level = $2");
            count_query.push_str(" AND level = $2");
        }
        if filter.from_date.is_some() {
            query.push_str(" AND created_at >= $3");
            count_query.push_str(" AND created_at >= $3");
        }
        if filter.to_date.is_some() {
            query.push_str(" AND created_at <= $4");
            count_query.push_str(" AND created_at <= $4");
        }

        query.push_str(" ORDER BY created_at DESC LIMIT $5 OFFSET $6");

        let process_id_str = filter.process_id.map(|id| id.to_string());
        let rows = sqlx::query_as::<_, LogRow>(&query)
            .bind(&process_id_str)
            .bind(filter.level)
            .bind(filter.from_date)
            .bind(filter.to_date)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;

        let count: (i64,) = sqlx::query_as(&count_query)
            .bind(&process_id_str)
            .bind(filter.level)
            .bind(filter.from_date)
            .bind(filter.to_date)
            .fetch_one(pool)
            .await?;

        Ok((rows.into_iter().map(Log::from).collect(), count.0))
    }

    pub async fn create(pool: &SqlitePool, req: &CreateLogRequest) -> AppResult<Log> {
        let row = sqlx::query_as::<_, LogRow>(
            r#"
            INSERT INTO logs (process_id, level, function_name, message, line_id)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#
        )
        .bind(req.process_id.to_string())
        .bind(req.level.unwrap_or_default())
        .bind(&req.function_name)
        .bind(&req.message)
        .bind(req.line_id)
        .fetch_one(pool)
        .await?;
        Ok(Log::from(row))
    }

    pub async fn delete_old(pool: &SqlitePool, days: i64) -> AppResult<u64> {
        let result: sqlx::sqlite::SqliteQueryResult = sqlx::query(
            "DELETE FROM logs WHERE created_at < datetime('now', '-' || $1 || ' days')"
        )
        .bind(days)
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }
}
