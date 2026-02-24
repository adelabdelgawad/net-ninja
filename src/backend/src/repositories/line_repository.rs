use sqlx::SqlitePool;

use crate::errors::AppResult;
use crate::models::{CreateLineRequest, Line, UpdateLineRequest};

pub struct LineRepository;

impl LineRepository {
    pub async fn get_by_id(pool: &SqlitePool, id: i32) -> AppResult<Option<Line>> {
        let line = sqlx::query_as::<_, Line>("SELECT * FROM lines WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;
        Ok(line)
    }

    pub async fn get_all(pool: &SqlitePool) -> AppResult<Vec<Line>> {
        let lines = sqlx::query_as::<_, Line>("SELECT * FROM lines ORDER BY name")
            .fetch_all(pool)
            .await?;
        Ok(lines)
    }

    pub async fn get_paginated(
        pool: &SqlitePool,
        offset: i64,
        limit: i64,
    ) -> AppResult<(Vec<Line>, i64)> {
        let lines = sqlx::query_as::<_, Line>(
            "SELECT * FROM lines ORDER BY name LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM lines")
            .fetch_one(pool)
            .await?;

        Ok((lines, count.0))
    }

    pub async fn create(pool: &SqlitePool, req: &CreateLineRequest) -> AppResult<Line> {
        let line = sqlx::query_as::<_, Line>(
            r#"
            INSERT INTO lines (name, line_number, username, password, ip_address, isp, description, gateway_ip, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#
        )
        .bind(&req.name)
        .bind(&req.line_number)
        .bind(&req.username)
        .bind(&req.password)
        .bind(&req.ip_address)
        .bind(&req.isp)
        .bind(&req.description)
        .bind(&req.gateway_ip)
        .bind(req.is_active.unwrap_or(true))
        .fetch_one(pool)
        .await?;

        Ok(line)
    }

    pub async fn update(
        pool: &SqlitePool,
        id: i32,
        req: &UpdateLineRequest,
    ) -> AppResult<Option<Line>> {
        // Treat empty strings as None for credentials to prevent accidental overwrites
        let username = req.username.as_deref().filter(|s| !s.is_empty()).map(String::from);
        let password = req.password.as_deref().filter(|s| !s.is_empty()).map(String::from);

        let line = sqlx::query_as::<_, Line>(
            r#"
            UPDATE lines SET
                name = COALESCE($1, name),
                line_number = COALESCE($2, line_number),
                username = COALESCE($3, username),
                password = COALESCE($4, password),
                ip_address = COALESCE($5, ip_address),
                isp = COALESCE($6, isp),
                description = COALESCE($7, description),
                gateway_ip = COALESCE($8, gateway_ip),
                is_active = COALESCE($9, is_active),
                updated_at = datetime('now', 'utc')
            WHERE id = $10
            RETURNING *
            "#
        )
        .bind(&req.name)
        .bind(&req.line_number)
        .bind(&username)
        .bind(&password)
        .bind(&req.ip_address)
        .bind(&req.isp)
        .bind(&req.description)
        .bind(&req.gateway_ip)
        .bind(req.is_active)
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(line)
    }

    pub async fn exists(pool: &SqlitePool, id: i32) -> AppResult<bool> {
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM lines WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await?;
        Ok(result.0 > 0)
    }

    pub async fn delete(pool: &SqlitePool, id: i32) -> AppResult<bool> {
        let result: sqlx::sqlite::SqliteQueryResult = sqlx::query("DELETE FROM lines WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn get_by_line_number(
        pool: &SqlitePool,
        line_number: &str,
    ) -> AppResult<Option<Line>> {
        let line = sqlx::query_as::<_, Line>("SELECT * FROM lines WHERE line_number = $1")
            .bind(line_number)
            .fetch_optional(pool)
            .await?;
        Ok(line)
    }

    pub async fn get_by_ip_address(
        pool: &SqlitePool,
        ip_address: &str,
    ) -> AppResult<Option<Line>> {
        let line = sqlx::query_as::<_, Line>("SELECT * FROM lines WHERE ip_address = $1")
            .bind(ip_address)
            .fetch_optional(pool)
            .await?;
        Ok(line)
    }

    /// Get a line by ID (alias for get_by_id for backward compatibility)
    pub async fn get_by_id_raw(pool: &SqlitePool, id: i32) -> AppResult<Option<Line>> {
        Self::get_by_id(pool, id).await
    }

    pub async fn get_by_ids(pool: &SqlitePool, ids: &[i64]) -> AppResult<Vec<Line>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders: Vec<String> = (1..=ids.len())
            .map(|i| format!("${}", i))
            .collect();
        let query = format!(
            "SELECT * FROM lines WHERE id IN ({}) ORDER BY name",
            placeholders.join(", ")
        );

        let mut query_builder = sqlx::query_as::<_, Line>(&query);
        for id in ids {
            query_builder = query_builder.bind(*id as i32);
        }

        let lines = query_builder.fetch_all(pool).await?;
        Ok(lines)
    }
}
