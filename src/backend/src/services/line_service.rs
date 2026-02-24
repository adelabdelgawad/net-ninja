use sqlx::SqlitePool;

use crate::errors::{AppError, AppResult};
use crate::models::{
    CreateLineRequest, Line, LineResponse, PaginatedResponse, PaginationParams, UpdateLineRequest,
};
use crate::repositories::LineRepository;

pub struct LineService;

impl LineService {
    pub async fn get_by_id(pool: &SqlitePool, id: i32) -> AppResult<LineResponse> {
        let line = LineRepository::get_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Line with id {} not found", id)))?;
        Ok(line.into())
    }

    pub async fn get_all(pool: &SqlitePool) -> AppResult<Vec<LineResponse>> {
        let lines = LineRepository::get_all(pool).await?;
        Ok(lines.into_iter().map(Into::into).collect())
    }

    pub async fn get_paginated(
        pool: &SqlitePool,
        params: &PaginationParams,
    ) -> AppResult<PaginatedResponse<LineResponse>> {
        let (lines, total) = LineRepository::get_paginated(pool, params.offset(), params.per_page()).await?;
        let items = lines.into_iter().map(Into::into).collect();
        Ok(PaginatedResponse::new(items, total, params))
    }

    pub async fn create(pool: &SqlitePool, req: CreateLineRequest) -> AppResult<LineResponse> {
        // Check if line_number already exists
        if LineRepository::get_by_line_number(pool, &req.line_number).await?.is_some() {
            return Err(AppError::Validation(format!(
                "Line with number {} already exists",
                req.line_number
            )));
        }

        let line = LineRepository::create(pool, &req).await?;
        Ok(line.into())
    }

    pub async fn update(pool: &SqlitePool, id: i32, req: UpdateLineRequest) -> AppResult<LineResponse> {
        // Check if new line_number conflicts with existing
        if let Some(ref line_number) = req.line_number {
            if let Some(existing) = LineRepository::get_by_line_number(pool, line_number).await? {
                if existing.id != id {
                    return Err(AppError::Validation(format!(
                        "Line with number {} already exists",
                        line_number
                    )));
                }
            }
        }

        let line = LineRepository::update(pool, id, &req)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Line with id {} not found", id)))?;
        Ok(line.into())
    }

    pub async fn delete(pool: &SqlitePool, id: i32) -> AppResult<()> {
        if !LineRepository::delete(pool, id).await? {
            return Err(AppError::NotFound(format!("Line with id {} not found", id)));
        }
        Ok(())
    }

    pub async fn get_all_with_credentials(pool: &SqlitePool) -> AppResult<Vec<Line>> {
        LineRepository::get_all(pool).await
    }
}
