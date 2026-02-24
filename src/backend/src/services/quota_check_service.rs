use sqlx::SqlitePool;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::models::{
    CreateQuotaResultRequest, PaginatedResponse, PaginationParams, QuotaResultResponse,
};
use crate::repositories::QuotaResultRepository;

pub struct QuotaCheckService;

impl QuotaCheckService {
    pub async fn get_by_id(pool: &SqlitePool, id: i32) -> AppResult<QuotaResultResponse> {
        let result = QuotaResultRepository::get_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Quota result with id {} not found", id)))?;
        Ok(result.into())
    }

    pub async fn get_paginated(
        pool: &SqlitePool,
        params: &PaginationParams,
    ) -> AppResult<PaginatedResponse<QuotaResultResponse>> {
        let (results, total) =
            QuotaResultRepository::get_paginated(pool, params.offset(), params.per_page()).await?;
        let items = results.into_iter().map(Into::into).collect();
        Ok(PaginatedResponse::new(items, total, params))
    }

    pub async fn get_by_line_id(
        pool: &SqlitePool,
        line_id: i32,
        limit: Option<i64>,
    ) -> AppResult<Vec<QuotaResultResponse>> {
        let results = QuotaResultRepository::get_by_line_id(pool, line_id, limit.unwrap_or(10)).await?;
        Ok(results.into_iter().map(Into::into).collect())
    }

    pub async fn get_latest_by_line_id(pool: &SqlitePool, line_id: i32) -> AppResult<Option<QuotaResultResponse>> {
        let result = QuotaResultRepository::get_latest_by_line_id(pool, line_id).await?;
        Ok(result.map(Into::into))
    }

    pub async fn get_by_process_id(pool: &SqlitePool, process_id: Uuid) -> AppResult<Vec<QuotaResultResponse>> {
        let results = QuotaResultRepository::get_by_process_id(pool, process_id).await?;
        Ok(results.into_iter().map(Into::into).collect())
    }

    pub async fn get_latest_for_all_lines(pool: &SqlitePool) -> AppResult<Vec<QuotaResultResponse>> {
        let results = QuotaResultRepository::get_latest_for_all_lines(pool).await?;
        Ok(results.into_iter().map(Into::into).collect())
    }

    pub async fn create(pool: &SqlitePool, req: CreateQuotaResultRequest) -> AppResult<QuotaResultResponse> {
        let result = QuotaResultRepository::create(pool, &req).await?;
        Ok(result.into())
    }

    pub async fn cleanup_old(pool: &SqlitePool, days: i64) -> AppResult<u64> {
        QuotaResultRepository::delete_old(pool, days).await
    }
}
