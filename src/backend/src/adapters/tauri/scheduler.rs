use super::*;

// ===== Response Types =====

#[derive(Debug, Clone, serde::Serialize)]
pub struct SchedulerStatusResponse {
    pub status: String,
    #[serde(rename = "isRunning")]
    pub is_running: bool,
}

// ===== Scheduler Commands =====

#[tauri::command]
pub async fn get_scheduler_status(_state: State<'_, AppState>) -> Result<SchedulerStatusResponse, String> {
    // Placeholder - scheduler status tracking not yet implemented
    Ok(SchedulerStatusResponse {
        status: "unknown".to_string(),
        is_running: false,
    })
}

#[tauri::command]
pub async fn start_scheduler(_state: State<'_, AppState>) -> Result<(), String> {
    // This would interact with the job runner managed by Tauri
    // For now, return a placeholder
    Ok(())
}

#[tauri::command]
pub async fn stop_scheduler(_state: State<'_, AppState>) -> Result<(), String> {
    // This would interact with the job runner managed by Tauri
    // For now, return a placeholder
    Ok(())
}
