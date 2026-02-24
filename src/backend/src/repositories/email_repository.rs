use sqlx::SqlitePool;

use crate::errors::AppResult;
use crate::models::{CreateEmailRequest, Email, UpdateEmailRequest};

pub struct EmailRepository;

impl EmailRepository {
    pub async fn get_by_id(pool: &SqlitePool, id: i32) -> AppResult<Option<Email>> {
        let email = sqlx::query_as::<_, Email>("SELECT * FROM emails WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;
        Ok(email)
    }

    pub async fn get_all(pool: &SqlitePool) -> AppResult<Vec<Email>> {
        let emails = sqlx::query_as::<_, Email>("SELECT * FROM emails ORDER BY email")
            .fetch_all(pool)
            .await?;
        Ok(emails)
    }

    pub async fn get_active(pool: &SqlitePool) -> AppResult<Vec<Email>> {
        let emails = sqlx::query_as::<_, Email>(
            "SELECT * FROM emails WHERE is_active = true ORDER BY email"
        )
        .fetch_all(pool)
        .await?;
        Ok(emails)
    }

    pub async fn get_paginated(pool: &SqlitePool, offset: i64, limit: i64) -> AppResult<(Vec<Email>, i64)> {
        let emails = sqlx::query_as::<_, Email>(
            "SELECT * FROM emails ORDER BY email LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM emails")
            .fetch_one(pool)
            .await?;

        Ok((emails, count.0))
    }

    pub async fn create(pool: &SqlitePool, req: &CreateEmailRequest) -> AppResult<Email> {
        let email = sqlx::query_as::<_, Email>(
            r#"
            INSERT INTO emails (email, name, is_cc, is_active)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#
        )
        .bind(&req.email)
        .bind(&req.name)
        .bind(req.is_cc.unwrap_or(false))
        .bind(req.is_active.unwrap_or(true))
        .fetch_one(pool)
        .await?;
        Ok(email)
    }

    pub async fn update(pool: &SqlitePool, id: i32, req: &UpdateEmailRequest) -> AppResult<Option<Email>> {
        let email = sqlx::query_as::<_, Email>(
            r#"
            UPDATE emails SET
                email = COALESCE($1, email),
                name = COALESCE($2, name),
                is_cc = COALESCE($3, is_cc),
                is_active = COALESCE($4, is_active),
                updated_at = datetime('now', 'utc')
            WHERE id = $5
            RETURNING *
            "#
        )
        .bind(&req.email)
        .bind(&req.name)
        .bind(req.is_cc)
        .bind(req.is_active)
        .bind(id)
        .fetch_optional(pool)
        .await?;
        Ok(email)
    }

    pub async fn delete(pool: &SqlitePool, id: i32) -> AppResult<bool> {
        let result: sqlx::sqlite::SqliteQueryResult = sqlx::query("DELETE FROM emails WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn get_by_email(pool: &SqlitePool, email: &str) -> AppResult<Option<Email>> {
        let result = sqlx::query_as::<_, Email>("SELECT * FROM emails WHERE email = $1")
            .bind(email)
            .fetch_optional(pool)
            .await?;
        Ok(result)
    }
}
