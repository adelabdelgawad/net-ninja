use tauri::State;

use crate::app::AppState;
use crate::errors::AppError;
use crate::models::{
    CreateLineRequest, LineResponse, PaginatedResponse, PaginationParams, UpdateLineRequest,
};

/// Helper: Convert AppError to String for Tauri command results
fn map_err<E: Into<AppError>>(e: E) -> String {
    Into::<AppError>::into(e).to_string()
}

/// Helper: Extract SQLite pool
fn get_sqlite_pool(state: &AppState) -> Result<&sqlx::SqlitePool, String> {
    state.pool.as_ref().ok_or_else(|| {
        "Not available in fallback mode - Database connection required".to_string()
    })
}

// ===== Line Commands =====

#[tauri::command]
pub async fn get_lines(
    state: State<'_, AppState>,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Result<PaginatedResponse<LineResponse>, String> {
    let params = PaginationParams {
        page: page.map(|p| p as i64),
        per_page: page_size.map(|p| p as i64),
    };

    let pool = get_sqlite_pool(&state)?;
    crate::services::LineService::get_paginated(pool, &params)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn get_line(state: State<'_, AppState>, id: i32) -> Result<LineResponse, String> {
    let pool = get_sqlite_pool(&state)?;
    crate::services::LineService::get_by_id(pool, id)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn create_line(
    state: State<'_, AppState>,
    req: CreateLineRequest,
) -> Result<LineResponse, String> {
    let pool = get_sqlite_pool(&state)?;
    crate::services::LineService::create(pool, req)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn update_line(
    state: State<'_, AppState>,
    id: i32,
    req: UpdateLineRequest,
) -> Result<LineResponse, String> {
    let pool = get_sqlite_pool(&state)?;
    crate::services::LineService::update(pool, id, req)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn delete_line(state: State<'_, AppState>, id: i32) -> Result<(), String> {
    let pool = get_sqlite_pool(&state)?;
    crate::services::LineService::delete(pool, id)
        .await
        .map_err(map_err)
}
