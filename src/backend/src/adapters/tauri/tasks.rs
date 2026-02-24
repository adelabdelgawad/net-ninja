use tauri::State;

use crate::app::AppState;
use crate::clients::{DiagnosticReport, NetworkDiagnostics};
use crate::errors::AppError;
use crate::models::{
    CreateTaskRequest, ListExecutionsParams, ResendNotificationRequest, TaskExecutionResponse,
    TaskResponse, TaskExecutionResult, UpdateTaskRequest, RuntimeNotificationConfigRequest,
};

/// Helper: Convert AppError to String for Tauri command results
fn map_err<E: Into<AppError>>(e: E) -> String {
    Into::<AppError>::into(e).to_string()
}

/// Helper: Extract SQLite pool
fn get_sqlite_pool(state: &AppState) -> Result<&sqlx::SqlitePool, String> {
    state.pool.as_ref().ok_or_else(|| {
        "Database not available".to_string()
    })
}

// ===== Task Commands =====

/// Get all tasks with populated line information.
/// Note: No caching — task status can change externally (e.g. by the Windows service scheduler),
/// so we always query the database to reflect the latest state.
#[tauri::command]
pub async fn get_tasks(state: State<'_, AppState>) -> Result<Vec<TaskResponse>, String> {
    let pool = get_sqlite_pool(&state)?;
    crate::services::TaskService::get_all(pool)
        .await
        .map_err(map_err)
}

/// Get a single task by ID
#[tauri::command]
pub async fn get_task(state: State<'_, AppState>, id: i64) -> Result<TaskResponse, String> {
    let pool = get_sqlite_pool(&state)?;
    crate::services::TaskService::get_by_id(pool, id)
        .await
        .map_err(map_err)
}

/// Create a new task
#[tauri::command]
pub async fn create_task(
    state: State<'_, AppState>,
    req: CreateTaskRequest,
) -> Result<TaskResponse, String> {
    let pool = get_sqlite_pool(&state)?;
    crate::services::TaskService::create(pool, req)
        .await
        .map_err(map_err)
}

/// Check if a task name is available
#[tauri::command]
pub async fn check_task_name_available(
    state: State<'_, AppState>,
    name: String,
) -> Result<bool, String> {
    let pool = get_sqlite_pool(&state)?;
    crate::services::TaskService::check_name_available(pool, &name)
        .await
        .map_err(map_err)
}

/// Update an existing task
#[tauri::command]
pub async fn update_task(
    state: State<'_, AppState>,
    id: i64,
    req: UpdateTaskRequest,
) -> Result<TaskResponse, String> {
    tracing::info!(
        task_id = id,
        name = %req.name.as_deref().unwrap_or("(unchanged)"),
        "update_task called"
    );
    let pool = get_sqlite_pool(&state)?;
    crate::services::TaskService::update(pool, id, req)
        .await
        .map_err(map_err)
}

/// Delete a task
#[tauri::command]
pub async fn delete_task(
    state: State<'_, AppState>,
    id: i64,
) -> Result<(), String> {
    let pool = get_sqlite_pool(&state)?;
    crate::services::TaskService::delete(pool, id)
        .await
        .map_err(map_err)
}

/// Toggle task active status
#[tauri::command]
pub async fn toggle_task_active(
    state: State<'_, AppState>,
    id: i64,
    is_active: bool,
) -> Result<TaskResponse, String> {
    let pool = get_sqlite_pool(&state)?;
    crate::services::TaskService::toggle_active(pool, id, is_active)
        .await
        .map_err(map_err)
}

/// Execute a task immediately with optional notification override
#[tauri::command]
pub async fn execute_task(
    state: State<'_, AppState>,
    id: i64,
    notification_override: Option<RuntimeNotificationConfigRequest>,
) -> Result<TaskExecutionResult, String> {
    tracing::info!("[execute_task] Received request for task_id={}, override={:?}", id, notification_override.is_some());
    let result = crate::services::TaskService::execute(&state, id, notification_override)
        .await
        .map_err(map_err);
    match &result {
        Ok(_) => tracing::info!("[execute_task] Command completed successfully for task_id={}", id),
        Err(e) => tracing::error!("[execute_task] Command failed for task_id={}: {}", id, e),
    }
    result
}

/// Stop a running task
#[tauri::command]
pub async fn stop_task(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let pool = get_sqlite_pool(&state)?;
    // Verify task exists
    let task = crate::services::TaskService::get_by_id(pool, id).await.map_err(map_err)?;
    // Verify running
    if task.status != "running" {
        return Err("Task is not running".to_string());
    }
    // Cancel via registry
    if !crate::services::task_runtime::cancel(id).await {
        return Err("Task not found in runtime registry".to_string());
    }
    Ok(())
}

// ===== Network Diagnostics Commands =====

/// Run network diagnostics for speedtest.net connectivity
#[tauri::command]
pub async fn diagnose_speedtest_connectivity() -> Result<DiagnosticReport, String> {
    tracing::info!("[diagnose_speedtest_connectivity] Running diagnostics");

    NetworkDiagnostics::run_diagnostics("www.speedtest.net")
        .await
        .map_err(|e| format!("Diagnostic failed: {}", e))
}

// ===== Task Execution History Commands =====

/// Get execution history for a specific task
#[tauri::command]
pub async fn get_task_executions(
    state: State<'_, AppState>,
    task_id: i64,
    limit: Option<i64>,
) -> Result<Vec<TaskExecutionResponse>, String> {
    let pool = get_sqlite_pool(&state)?;
    crate::services::TaskExecutionService::get_by_task_id(pool, task_id, limit)
        .await
        .map_err(map_err)
}

/// Get all executions with optional filtering
#[tauri::command]
pub async fn get_executions(
    state: State<'_, AppState>,
    params: Option<ListExecutionsParams>,
) -> Result<Vec<TaskExecutionResponse>, String> {
    let pool = get_sqlite_pool(&state)?;
    let params = params.unwrap_or_default();
    crate::services::TaskExecutionService::list(pool, &params)
        .await
        .map_err(map_err)
}

/// Get a single execution by ID
#[tauri::command]
pub async fn get_execution(
    state: State<'_, AppState>,
    id: i64,
) -> Result<TaskExecutionResponse, String> {
    let pool = get_sqlite_pool(&state)?;
    crate::services::TaskExecutionService::get_by_id(pool, id)
        .await
        .map_err(map_err)
}

/// Get the latest execution for a task
#[tauri::command]
pub async fn get_latest_task_execution(
    state: State<'_, AppState>,
    task_id: i64,
) -> Result<Option<TaskExecutionResponse>, String> {
    let pool = get_sqlite_pool(&state)?;
    crate::services::TaskExecutionService::get_latest_for_task(pool, task_id)
        .await
        .map_err(map_err)
}

/// Count total executions (for pagination)
#[tauri::command]
pub async fn count_executions(
    state: State<'_, AppState>,
    params: Option<ListExecutionsParams>,
) -> Result<i64, String> {
    let pool = get_sqlite_pool(&state)?;
    let params = params.unwrap_or_default();
    crate::services::TaskExecutionService::count(pool, &params)
        .await
        .map_err(map_err)
}

/// Resend notification email for a completed execution
#[tauri::command]
pub async fn resend_task_notification(
    state: State<'_, AppState>,
    req: ResendNotificationRequest,
) -> Result<(), String> {
    let pool = get_sqlite_pool(&state)?;
    crate::services::NotificationService::resend_notification(
        pool,
        req,
        state.encryption_key.as_deref(),
    )
    .await
    .map_err(map_err)
}
