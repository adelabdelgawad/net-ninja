use super::*;
use super::health::{get_sqlite_pool, map_err};

// ===== Log Commands =====

#[tauri::command]
pub async fn get_logs(
    state: State<'_, AppState>,
    page: Option<u32>,
    page_size: Option<u32>,
    filter: Option<LogFilter>,
) -> Result<PaginatedResponse<Log>, String> {
    let params = PaginationParams { page: page.map(|p| p as i64), per_page: page_size.map(|p| p as i64) };
    let filter = filter.unwrap_or_default();

    let pool = get_sqlite_pool(&state)?;
    crate::services::LogService::get_filtered(pool, filter, &params)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn get_log(state: State<'_, AppState>, id: i32) -> Result<Log, String> {
    let pool = get_sqlite_pool(&state)?;
    crate::services::LogService::get_by_id(pool, id)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn get_logs_by_process(
    state: State<'_, AppState>,
    process_id: String,
) -> Result<Vec<Log>, String> {
    let pool = get_sqlite_pool(&state)?;
    let uuid = uuid::Uuid::parse_str(&process_id).map_err(|e| e.to_string())?;
    crate::services::LogService::get_by_process_id(pool, uuid)
        .await
        .map_err(map_err)
}
