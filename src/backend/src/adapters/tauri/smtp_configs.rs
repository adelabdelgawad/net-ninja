use super::*;
use super::health::{get_sqlite_pool, map_err};

// ===== SMTP Config Commands =====

#[tauri::command]
pub async fn get_smtp_configs(
    state: State<'_, AppState>,
) -> Result<Vec<SmtpConfig>, String> {
    let pool = get_sqlite_pool(&state)?;
    let encryption_key = state.encryption_key.as_deref();
    crate::services::SmtpConfigService::get_all(pool, encryption_key)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn get_smtp_config(state: State<'_, AppState>, id: i32) -> Result<SmtpConfig, String> {
    let pool = get_sqlite_pool(&state)?;
    let encryption_key = state.encryption_key.as_deref();
    crate::services::SmtpConfigService::get_by_id(pool, id, encryption_key)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn create_smtp_config(
    state: State<'_, AppState>,
    req: CreateSmtpConfigRequest,
) -> Result<SmtpConfig, String> {
    let pool = get_sqlite_pool(&state)?;
    let encryption_key = state.encryption_key.as_deref();
    crate::services::SmtpConfigService::create(pool, req, encryption_key)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn update_smtp_config(
    state: State<'_, AppState>,
    id: i32,
    req: UpdateSmtpConfigRequest,
) -> Result<SmtpConfig, String> {
    let pool = get_sqlite_pool(&state)?;
    let encryption_key = state.encryption_key.as_deref();
    crate::services::SmtpConfigService::update(pool, id, req, encryption_key)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn delete_smtp_config(state: State<'_, AppState>, id: i32) -> Result<(), String> {
    let pool = get_sqlite_pool(&state)?;
    crate::services::SmtpConfigService::delete(pool, id)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn test_smtp_config(
    state: State<'_, AppState>,
    id: i32,
    test_recipient: String,
) -> Result<SmtpConfigTestResponse, String> {
    let pool = get_sqlite_pool(&state)?;
    let encryption_key = state.encryption_key.as_deref();
    crate::services::SmtpConfigService::test_config_with_recipient(&pool, id, &test_recipient, encryption_key)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn get_default_smtp_config(state: State<'_, AppState>) -> Result<SmtpConfig, String> {
    let pool = get_sqlite_pool(&state)?;
    let encryption_key = state.encryption_key.as_deref();
    crate::services::SmtpConfigService::get_default(pool, encryption_key)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn set_default_smtp_config(state: State<'_, AppState>, id: i32) -> Result<SmtpConfig, String> {
    let pool = get_sqlite_pool(&state)?;
    let encryption_key = state.encryption_key.as_deref();
    crate::services::SmtpConfigService::set_as_default(pool, id, encryption_key)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn test_smtp_config_inline(
    _state: State<'_, AppState>,
    req: SmtpConfigTestRequest,
) -> Result<SmtpConfigTestResponse, String> {
    crate::services::SmtpConfigService::test_config_without_saving(&req)
        .await
        .map_err(map_err)
}
