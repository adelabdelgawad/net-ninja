use super::*;
use super::health::{get_sqlite_pool, map_err};
use crate::models::CombinedResult;

// ===== Report Commands =====

#[tauri::command]
pub async fn get_latest_report(state: State<'_, AppState>) -> Result<Vec<CombinedResult>, String> {
    let pool = get_sqlite_pool(&state)?;
    crate::services::ReportService::get_latest_report(pool)
        .await
        .map_err(map_err)
}
