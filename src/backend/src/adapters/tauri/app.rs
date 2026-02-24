// ===== Application Lifecycle Commands =====

/// Restart the Tauri application
#[tauri::command]
pub async fn app_restart(app: tauri::AppHandle) -> Result<(), String> {
    tracing::info!("[app_restart] User triggered application restart");

    // Tauri's restart method closes the app and relaunches it
    // This is a diverging function - it terminates the process
    app.restart();
}

/// Get the database file path
#[tauri::command]
pub async fn get_database_path() -> Result<String, String> {
    use crate::config::get_sqlite_path;
    Ok(get_sqlite_path().to_string_lossy().to_string())
}

/// Get the logs directory path
#[tauri::command]
pub async fn get_logs_path() -> Result<String, String> {
    use crate::config::paths::get_shared_data_path;
    Ok(get_shared_data_path().join("logs").to_string_lossy().to_string())
}
