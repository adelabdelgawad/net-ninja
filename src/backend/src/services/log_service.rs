use sqlx::SqlitePool;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::models::{CreateLogRequest, Log, LogFilter, LogLevel, PaginatedResponse, PaginationParams};
use crate::repositories::LogRepository;

pub struct LogService;

impl LogService {
    pub async fn get_by_id(pool: &SqlitePool, id: i32) -> AppResult<Log> {
        LogRepository::get_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Log with id {} not found", id)))
    }

    pub async fn get_paginated(
        pool: &SqlitePool,
        params: &PaginationParams,
    ) -> AppResult<PaginatedResponse<Log>> {
        let (logs, total) = LogRepository::get_paginated(pool, params.offset(), params.per_page()).await?;
        Ok(PaginatedResponse::new(logs, total, params))
    }

    pub async fn get_by_process_id(pool: &SqlitePool, process_id: Uuid) -> AppResult<Vec<Log>> {
        LogRepository::get_by_process_id(pool, process_id).await
    }

    pub async fn get_filtered(
        pool: &SqlitePool,
        filter: LogFilter,
        params: &PaginationParams,
    ) -> AppResult<PaginatedResponse<Log>> {
        let (logs, total) =
            LogRepository::get_filtered(pool, &filter, params.offset(), params.per_page()).await?;
        Ok(PaginatedResponse::new(logs, total, params))
    }

    pub async fn create(pool: &SqlitePool, req: CreateLogRequest) -> AppResult<Log> {
        LogRepository::create(pool, &req).await
    }

    pub async fn info(pool: &SqlitePool, process_id: Uuid, function_name: &str, message: &str) -> AppResult<Log> {
        Self::create(
            pool,
            CreateLogRequest {
                process_id,
                level: Some(LogLevel::Info),
                function_name: Some(function_name.to_string()),
                message: message.to_string(),
                line_id: None,
            },
        )
        .await
    }

    pub async fn info_for_line(pool: &SqlitePool, process_id: Uuid, line_id: i32, function_name: &str, message: &str) -> AppResult<Log> {
        Self::create(
            pool,
            CreateLogRequest {
                process_id,
                level: Some(LogLevel::Info),
                function_name: Some(function_name.to_string()),
                message: message.to_string(),
                line_id: Some(line_id),
            },
        )
        .await
    }

    pub async fn warning(pool: &SqlitePool, process_id: Uuid, function_name: &str, message: &str) -> AppResult<Log> {
        Self::create(
            pool,
            CreateLogRequest {
                process_id,
                level: Some(LogLevel::Warning),
                function_name: Some(function_name.to_string()),
                message: message.to_string(),
                line_id: None,
            },
        )
        .await
    }

    pub async fn warning_for_line(pool: &SqlitePool, process_id: Uuid, line_id: i32, function_name: &str, message: &str) -> AppResult<Log> {
        Self::create(
            pool,
            CreateLogRequest {
                process_id,
                level: Some(LogLevel::Warning),
                function_name: Some(function_name.to_string()),
                message: message.to_string(),
                line_id: Some(line_id),
            },
        )
        .await
    }

    pub async fn error(pool: &SqlitePool, process_id: Uuid, function_name: &str, message: &str) -> AppResult<Log> {
        Self::create(
            pool,
            CreateLogRequest {
                process_id,
                level: Some(LogLevel::Error),
                function_name: Some(function_name.to_string()),
                message: message.to_string(),
                line_id: None,
            },
        )
        .await
    }

    pub async fn error_for_line(pool: &SqlitePool, process_id: Uuid, line_id: i32, function_name: &str, message: &str) -> AppResult<Log> {
        Self::create(
            pool,
            CreateLogRequest {
                process_id,
                level: Some(LogLevel::Error),
                function_name: Some(function_name.to_string()),
                message: message.to_string(),
                line_id: Some(line_id),
            },
        )
        .await
    }

    pub async fn debug(pool: &SqlitePool, process_id: Uuid, function_name: &str, message: &str) -> AppResult<Log> {
        Self::create(
            pool,
            CreateLogRequest {
                process_id,
                level: Some(LogLevel::Debug),
                function_name: Some(function_name.to_string()),
                message: message.to_string(),
                line_id: None,
            },
        )
        .await
    }

    pub async fn cleanup_old(pool: &SqlitePool, days: i64) -> AppResult<u64> {
        LogRepository::delete_old(pool, days).await
    }
}
