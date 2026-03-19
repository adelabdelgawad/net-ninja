use leptos::prelude::*;

fn se(msg: impl std::fmt::Display) -> ServerFnError {
    ServerFnError::ServerError(msg.to_string())
}

/// T037: Get all tasks (returns JSON array).
#[server]
pub async fn get_tasks() -> Result<String, ServerFnError> {
    use crate::auth::session::require_session;
    use crate::state::WebAppState;

    require_session().await?;

    let Some(state) = use_context::<WebAppState>() else {
        return Err(se("state unavailable"));
    };
    let Some(pool) = state.pool.clone() else {
        return Err(se("database unavailable"));
    };

    let tasks = net_ninja::services::TaskService::get_all(&pool)
        .await
        .map_err(|e| se(e.to_string()))?;

    serde_json::to_string(&tasks).map_err(|e| se(e.to_string()))
}

/// Get a single task by ID (returns JSON).
#[server]
pub async fn get_task(id: i64) -> Result<String, ServerFnError> {
    use crate::auth::session::require_session;
    use crate::state::WebAppState;

    require_session().await?;

    let Some(state) = use_context::<WebAppState>() else {
        return Err(se("state unavailable"));
    };
    let Some(pool) = state.pool.clone() else {
        return Err(se("database unavailable"));
    };

    let task = net_ninja::services::TaskService::get_by_id(&pool, id)
        .await
        .map_err(|e| se(e.to_string()))?;

    serde_json::to_string(&task).map_err(|e| se(e.to_string()))
}

/// Delete a task by ID (cascades to task_executions via FK).
#[server]
pub async fn delete_task(id: i64) -> Result<(), ServerFnError> {
    use crate::auth::session::require_session;
    use crate::state::WebAppState;

    require_session().await?;

    let Some(state) = use_context::<WebAppState>() else {
        return Err(se("state unavailable"));
    };
    let Some(pool) = state.pool.clone() else {
        return Err(se("database unavailable"));
    };

    net_ninja::services::TaskService::delete(&pool, id)
        .await
        .map_err(|e| se(e.to_string()))
}

/// Trigger a task manually. Returns execution_id for polling via get_execution_status.
#[server]
pub async fn trigger_task(task_id: i64) -> Result<i64, ServerFnError> {
    use crate::auth::session::require_session;
    use crate::state::WebAppState;
    use std::time::Instant;
    use uuid::Uuid;

    require_session().await?;

    let Some(state) = use_context::<WebAppState>() else {
        return Err(se("state unavailable"));
    };
    let Some(pool) = state.pool.clone() else {
        return Err(se("database unavailable"));
    };

    // Get the task to know how many lines it has
    let task = net_ninja::services::TaskService::get_by_id(&pool, task_id)
        .await
        .map_err(|e| se(e.to_string()))?;

    let line_count = task.line_ids.len() as i64;

    // Create an execution record
    let execution_id = Uuid::new_v4().to_string();
    let execution = net_ninja::repositories::TaskExecutionRepository::create(
        &pool,
        &net_ninja::models::CreateTaskExecutionRequest {
            task_id,
            execution_id: execution_id.clone(),
            triggered_by: "manual".to_string(),
            scheduled_for: None,
            line_count,
        },
    )
    .await
    .map_err(|e| se(e.to_string()))?;

    let row_id = execution.id;

    // Spawn background execution
    let pool_clone = pool.clone();
    let settings_clone = (*state.settings).clone();
    let enc_key_clone = state.encryption_key.clone();

    tokio::spawn(async move {
        let start = Instant::now();
        let app_state = net_ninja::app::AppState::new_full(pool_clone.clone(), settings_clone, enc_key_clone);

        let exec_result = net_ninja::services::TaskService::execute(&app_state, task_id, None).await;
        let duration_ms = start.elapsed().as_millis() as i64;

        match exec_result {
            Ok(_) => {
                let _ = sqlx::query(
                    "UPDATE task_executions SET status = 'completed', completed_at = datetime('now', 'utc'), duration_ms = $1, is_finished = 1 WHERE id = $2"
                )
                .bind(duration_ms)
                .bind(row_id)
                .execute(&pool_clone)
                .await;
            }
            Err(e) => {
                let _ = sqlx::query(
                    "UPDATE task_executions SET status = 'failed', error_message = $1, completed_at = datetime('now', 'utc'), duration_ms = $2, is_finished = 1 WHERE id = $3"
                )
                .bind(e.to_string())
                .bind(duration_ms)
                .bind(row_id)
                .execute(&pool_clone)
                .await;
            }
        }
    });

    Ok(row_id)
}

/// Get execution history for a task (returns JSON array).
#[server]
pub async fn get_task_executions(task_id: i64) -> Result<String, ServerFnError> {
    use crate::auth::session::require_session;
    use crate::state::WebAppState;

    require_session().await?;

    let Some(state) = use_context::<WebAppState>() else {
        return Err(se("state unavailable"));
    };
    let Some(pool) = state.pool.clone() else {
        return Err(se("database unavailable"));
    };

    let executions = net_ninja::services::TaskExecutionService::get_by_task_id(&pool, task_id, Some(20))
        .await
        .map_err(|e| se(e.to_string()))?;

    serde_json::to_string(&executions).map_err(|e| se(e.to_string()))
}
