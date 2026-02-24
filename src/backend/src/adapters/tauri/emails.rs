use super::*;

// Import helper functions from health module
use super::health::{get_sqlite_pool, map_err};

// ===== Email Commands =====

#[tauri::command]
pub async fn get_emails(
    state: State<'_, AppState>,
) -> Result<Vec<Email>, String> {
    let pool = get_sqlite_pool(&state)?;
    crate::services::EmailService::get_all(pool)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn get_email(state: State<'_, AppState>, id: i32) -> Result<Email, String> {
    let pool = get_sqlite_pool(&state)?;
    crate::services::EmailService::get_by_id(pool, id)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn create_email(
    state: State<'_, AppState>,
    req: CreateEmailRequest,
) -> Result<Email, String> {
    let pool = get_sqlite_pool(&state)?;
    crate::services::EmailService::create(pool, req)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn update_email(
    state: State<'_, AppState>,
    id: i32,
    req: UpdateEmailRequest,
) -> Result<Email, String> {
    let pool = get_sqlite_pool(&state)?;
    crate::services::EmailService::update(pool, id, req)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn delete_email(state: State<'_, AppState>, id: i32) -> Result<(), String> {
    let pool = get_sqlite_pool(&state)?;
    crate::services::EmailService::delete(pool, id)
        .await
        .map_err(map_err)
}
