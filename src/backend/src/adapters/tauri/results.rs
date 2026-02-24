use super::*;
use crate::models::{
    CreateQuotaResultRequest, CreateSpeedTestResultRequest, PaginatedResponse, PaginationParams,
    QuotaResultResponse, SpeedTestResultResponse,
};

// Re-export helper functions from health module
use super::health::{get_sqlite_pool, map_err};

// Type aliases for Tauri command compatibility
type CreateSpeedTestRequest = CreateSpeedTestResultRequest;
type CreateQuotaCheckRequest = CreateQuotaResultRequest;

// ===== Speed Test Commands =====

#[tauri::command]
pub async fn get_speed_tests(
    state: State<'_, AppState>,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Result<PaginatedResponse<SpeedTestResultResponse>, String> {
    let params = PaginationParams {
        page: page.map(|p| p as i64),
        per_page: page_size.map(|p| p as i64),
    };

    let pool = get_sqlite_pool(&state)?;
    crate::services::SpeedTestService::get_paginated(pool, &params)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn get_speed_test(
    state: State<'_, AppState>,
    id: i32,
) -> Result<SpeedTestResultResponse, String> {
    let pool = get_sqlite_pool(&state)?;
    crate::services::SpeedTestService::get_by_id(pool, id)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn create_speed_test(
    state: State<'_, AppState>,
    req: CreateSpeedTestRequest,
) -> Result<SpeedTestResultResponse, String> {
    let pool = get_sqlite_pool(&state)?;
    crate::services::SpeedTestService::create(pool, req)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn delete_speed_test(state: State<'_, AppState>, _id: i32) -> Result<(), String> {
    let pool = get_sqlite_pool(&state)?;
    crate::services::SpeedTestService::cleanup_old(pool, 9999)
        .await
        .map(|_| ())
        .map_err(map_err)
}

#[tauri::command]
pub async fn get_speed_tests_for_line(
    state: State<'_, AppState>,
    line_id: i32,
    limit: Option<i64>,
) -> Result<Vec<SpeedTestResultResponse>, String> {
    let pool = get_sqlite_pool(&state)?;
    crate::services::SpeedTestService::get_by_line_id(pool, line_id, limit)
        .await
        .map_err(map_err)
}

// ===== Quota Check Commands =====

#[tauri::command]
pub async fn get_quota_checks(
    state: State<'_, AppState>,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Result<PaginatedResponse<QuotaResultResponse>, String> {
    let params = PaginationParams {
        page: page.map(|p| p as i64),
        per_page: page_size.map(|p| p as i64),
    };

    let pool = get_sqlite_pool(&state)?;
    crate::services::QuotaCheckService::get_paginated(pool, &params)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn get_quota_check(
    state: State<'_, AppState>,
    id: i32,
) -> Result<QuotaResultResponse, String> {
    let pool = get_sqlite_pool(&state)?;
    crate::services::QuotaCheckService::get_by_id(pool, id)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn create_quota_check(
    state: State<'_, AppState>,
    req: CreateQuotaCheckRequest,
) -> Result<QuotaResultResponse, String> {
    let pool = get_sqlite_pool(&state)?;
    crate::services::QuotaCheckService::create(pool, req)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn delete_quota_check(state: State<'_, AppState>, _id: i32) -> Result<(), String> {
    let pool = get_sqlite_pool(&state)?;
    crate::services::QuotaCheckService::cleanup_old(pool, 9999)
        .await
        .map(|_| ())
        .map_err(map_err)
}

#[tauri::command]
pub async fn get_quota_results_for_line(
    state: State<'_, AppState>,
    line_id: i32,
    limit: Option<i64>,
) -> Result<Vec<QuotaResultResponse>, String> {
    let pool = get_sqlite_pool(&state)?;
    crate::services::QuotaCheckService::get_by_line_id(pool, line_id, limit)
        .await
        .map_err(map_err)
}
