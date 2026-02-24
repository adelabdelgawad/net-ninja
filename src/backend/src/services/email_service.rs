use sqlx::SqlitePool;

use crate::errors::{AppError, AppResult};
use crate::models::{CreateEmailRequest, Email, PaginatedResponse, PaginationParams, UpdateEmailRequest};
use crate::repositories::EmailRepository;

pub struct EmailService;

impl EmailService {
    pub async fn get_by_id(pool: &SqlitePool, id: i32) -> AppResult<Email> {
        EmailRepository::get_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Email with id {} not found", id)))
    }

    pub async fn get_all(pool: &SqlitePool) -> AppResult<Vec<Email>> {
        EmailRepository::get_all(pool).await
    }

    pub async fn get_active(pool: &SqlitePool) -> AppResult<Vec<Email>> {
        EmailRepository::get_active(pool).await
    }

    pub async fn get_paginated(
        pool: &SqlitePool,
        params: &PaginationParams,
    ) -> AppResult<PaginatedResponse<Email>> {
        let (emails, total) = EmailRepository::get_paginated(pool, params.offset(), params.per_page()).await?;
        Ok(PaginatedResponse::new(emails, total, params))
    }

    pub async fn create(pool: &SqlitePool, req: CreateEmailRequest) -> AppResult<Email> {
        // Check if email already exists
        if EmailRepository::get_by_email(pool, &req.email).await?.is_some() {
            return Err(AppError::Validation(format!(
                "Email {} already exists",
                req.email
            )));
        }

        EmailRepository::create(pool, &req).await
    }

    pub async fn update(pool: &SqlitePool, id: i32, req: UpdateEmailRequest) -> AppResult<Email> {
        // Check if new email conflicts with existing
        if let Some(ref email) = req.email {
            if let Some(existing) = EmailRepository::get_by_email(pool, email).await? {
                if existing.id != id {
                    return Err(AppError::Validation(format!(
                        "Email {} already exists",
                        email
                    )));
                }
            }
        }

        EmailRepository::update(pool, id, &req)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Email with id {} not found", id)))
    }

    pub async fn delete(pool: &SqlitePool, id: i32) -> AppResult<()> {
        if !EmailRepository::delete(pool, id).await? {
            return Err(AppError::NotFound(format!("Email with id {} not found", id)));
        }
        Ok(())
    }
}
