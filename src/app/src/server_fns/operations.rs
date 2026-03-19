use leptos::prelude::*;

fn se(msg: impl std::fmt::Display) -> ServerFnError {
    ServerFnError::ServerError(msg.to_string())
}

/// T027: Trigger a quota check for a specific line.
/// Returns the execution_id (row id) for polling via get_execution_status.
#[server]
pub async fn trigger_quota_check(line_id: i64) -> Result<i64, ServerFnError> {
    use crate::auth::session::require_session;
    use crate::state::WebAppState;

    require_session().await?;

    let Some(state) = use_context::<WebAppState>() else {
        return Err(se("state unavailable"));
    };

    // Check operation lock
    if state.operation_locks.get(&line_id).map(|v| *v).unwrap_or(false) {
        return Err(se("busy"));
    }
    state.operation_locks.insert(line_id, true);

    let Some(pool) = state.pool.clone() else {
        state.operation_locks.remove(&line_id);
        return Err(se("database unavailable"));
    };

    let result = net_ninja::services::WebOperationService::trigger_quota_check(
        pool,
        state.settings.clone(),
        state.encryption_key.clone(),
        line_id,
    )
    .await;

    state.operation_locks.remove(&line_id);

    result.map_err(|e| se(e.to_string()))
}

/// T028: Trigger a speed test for a specific line.
/// Returns the execution_id (row id) for polling.
#[server]
pub async fn trigger_speed_test(line_id: i64) -> Result<i64, ServerFnError> {
    use crate::auth::session::require_session;
    use crate::state::WebAppState;

    require_session().await?;

    let Some(state) = use_context::<WebAppState>() else {
        return Err(se("state unavailable"));
    };

    // Check operation lock
    if state.operation_locks.get(&line_id).map(|v| *v).unwrap_or(false) {
        return Err(se("busy"));
    }
    state.operation_locks.insert(line_id, true);

    let Some(pool) = state.pool.clone() else {
        state.operation_locks.remove(&line_id);
        return Err(se("database unavailable"));
    };

    let result = net_ninja::services::WebOperationService::trigger_speed_test(
        pool,
        state.settings.clone(),
        state.encryption_key.clone(),
        line_id,
    )
    .await;

    state.operation_locks.remove(&line_id);

    result.map_err(|e| se(e.to_string()))
}

/// T029: Poll execution status by execution_id. Returns JSON of ExecutionStatusResponse.
#[server]
pub async fn get_execution_status(execution_id: i64) -> Result<String, ServerFnError> {
    use crate::auth::session::require_session;
    use crate::state::WebAppState;

    require_session().await?;

    let Some(state) = use_context::<WebAppState>() else {
        return Err(se("state unavailable"));
    };

    let Some(pool) = state.pool.clone() else {
        return Err(se("database unavailable"));
    };

    let status = net_ninja::services::WebOperationService::get_execution_status(&pool, execution_id)
        .await
        .map_err(|e| se(e.to_string()))?;

    serde_json::to_string(&status).map_err(|e| se(e.to_string()))
}
