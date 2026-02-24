use sqlx::SqlitePool;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::models::{
    CreateSpeedTestResultRequest, PaginatedResponse, PaginationParams,
    SpeedTestResultResponse,
};
use crate::repositories::SpeedTestRepository;

pub struct SpeedTestService;

impl SpeedTestService {
    pub async fn get_by_id(pool: &SqlitePool, id: i32) -> AppResult<SpeedTestResultResponse> {
        let result = SpeedTestRepository::get_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Speed test result with id {} not found", id)))?;
        Ok(result.into())
    }

    pub async fn get_paginated(
        pool: &SqlitePool,
        params: &PaginationParams,
    ) -> AppResult<PaginatedResponse<SpeedTestResultResponse>> {
        let (results, total) =
            SpeedTestRepository::get_paginated(pool, params.offset(), params.per_page()).await?;
        let items = results.into_iter().map(Into::into).collect();
        Ok(PaginatedResponse::new(items, total, params))
    }

    pub async fn get_by_line_id(
        pool: &SqlitePool,
        line_id: i32,
        limit: Option<i64>,
    ) -> AppResult<Vec<SpeedTestResultResponse>> {
        let results = SpeedTestRepository::get_by_line_id(pool, line_id, limit.unwrap_or(10)).await?;
        Ok(results.into_iter().map(Into::into).collect())
    }

    pub async fn get_latest_by_line_id(pool: &SqlitePool, line_id: i32) -> AppResult<Option<SpeedTestResultResponse>> {
        let result = SpeedTestRepository::get_latest_by_line_id(pool, line_id).await?;
        Ok(result.map(Into::into))
    }

    pub async fn get_by_process_id(pool: &SqlitePool, process_id: Uuid) -> AppResult<Vec<SpeedTestResultResponse>> {
        let results = SpeedTestRepository::get_by_process_id(pool, process_id).await?;
        Ok(results.into_iter().map(Into::into).collect())
    }

    pub async fn get_latest_for_all_lines(pool: &SqlitePool) -> AppResult<Vec<SpeedTestResultResponse>> {
        let results = SpeedTestRepository::get_latest_for_all_lines(pool).await?;
        Ok(results.into_iter().map(Into::into).collect())
    }

    pub async fn create(pool: &SqlitePool, req: CreateSpeedTestResultRequest) -> AppResult<SpeedTestResultResponse> {
        let result = SpeedTestRepository::create(pool, &req).await?;
        Ok(result.into())
    }

    pub async fn cleanup_old(pool: &SqlitePool, days: i64) -> AppResult<u64> {
        SpeedTestRepository::delete_old(pool, days).await
    }
}
