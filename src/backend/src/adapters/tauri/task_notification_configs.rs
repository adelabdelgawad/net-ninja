use super::*;

// Import helper functions from health module
use super::health::{get_sqlite_pool, map_err};

// ===== Task Notification Config Commands =====

#[tauri::command]
pub async fn get_task_notification_config(
    state: State<'_, AppState>,
    task_id: i32,
) -> Result<Option<TaskNotificationConfigResponse>, String> {
    let pool = get_sqlite_pool(&state)?;
    crate::services::TaskNotificationConfigService::get_by_task_id(pool, task_id)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn upsert_task_notification_config(
    state: State<'_, AppState>,
    task_id: i32,
    req: UpsertTaskNotificationConfigRequest,
) -> Result<TaskNotificationConfigResponse, String> {
    let pool = get_sqlite_pool(&state)?;
    crate::services::TaskNotificationConfigService::upsert(pool, task_id, req)
        .await
        .map_err(map_err)
}
