use leptos::prelude::*;

fn se(msg: impl std::fmt::Display) -> ServerFnError {
    ServerFnError::ServerError(msg.to_string())
}

/// Returns all lines as JSON array (for Dashboard use - no pagination needed).
#[server]
pub async fn get_all_lines() -> Result<String, ServerFnError> {
    use crate::state::WebAppState;
    use crate::auth::session::require_session;
    use net_ninja::services::LineService;

    require_session().await?;

    let Some(state) = use_context::<WebAppState>() else {
        return Err(se("state unavailable"));
    };
    let Some(pool) = state.pool.clone() else {
        return Err(se("database unavailable"));
    };

    let lines = LineService::get_all(&pool)
        .await
        .map_err(|e| se(format!("service error: {e}")))?;

    serde_json::to_string(&lines).map_err(|e| se(format!("serialize error: {e}")))
}

/// Returns paginated lines as JSON (PaginatedResponse<LineResponse>).
#[server]
pub async fn get_lines(page: Option<u32>, page_size: Option<u32>) -> Result<String, ServerFnError> {
    use crate::state::WebAppState;
    use crate::auth::session::require_session;
    use net_ninja::services::LineService;
    use net_ninja::models::PaginationParams;

    require_session().await?;

    let Some(state) = use_context::<WebAppState>() else {
        return Err(se("state unavailable"));
    };
    let Some(pool) = state.pool.clone() else {
        return Err(se("database unavailable"));
    };

    let params = PaginationParams {
        page: page.map(|p| p as i64),
        per_page: page_size.map(|p| p as i64),
    };

    let result = LineService::get_paginated(&pool, &params)
        .await
        .map_err(|e| se(format!("service error: {e}")))?;

    serde_json::to_string(&result).map_err(|e| se(format!("serialize error: {e}")))
}

/// Returns a single line as JSON.
#[server]
pub async fn get_line(id: i32) -> Result<String, ServerFnError> {
    use crate::state::WebAppState;
    use crate::auth::session::require_session;
    use net_ninja::services::LineService;

    require_session().await?;

    let Some(state) = use_context::<WebAppState>() else {
        return Err(se("state unavailable"));
    };
    let Some(pool) = state.pool.clone() else {
        return Err(se("database unavailable"));
    };

    let line = LineService::get_by_id(&pool, id)
        .await
        .map_err(|e| se(format!("service error: {e}")))?;

    serde_json::to_string(&line).map_err(|e| se(format!("serialize error: {e}")))
}

/// Create a new internet line. Returns new line as JSON.
#[allow(clippy::too_many_arguments)]
#[server]
pub async fn create_line(
    name: String,
    line_number: String,
    username: String,
    password: String,
    ip_address: Option<String>,
    isp: Option<String>,
    description: Option<String>,
    gateway_ip: Option<String>,
    is_active: Option<bool>,
) -> Result<String, ServerFnError> {
    use crate::state::WebAppState;
    use crate::auth::session::require_session;
    use net_ninja::services::LineService;
    use net_ninja::models::CreateLineRequest;

    require_session().await?;

    let Some(state) = use_context::<WebAppState>() else {
        return Err(se("state unavailable"));
    };
    let Some(pool) = state.pool.clone() else {
        return Err(se("database unavailable"));
    };

    let req = CreateLineRequest {
        name,
        line_number,
        username,
        password,
        ip_address,
        isp,
        description,
        gateway_ip,
        is_active,
    };

    let line = LineService::create(&pool, req)
        .await
        .map_err(|e| se(format!("service error: {e}")))?;

    serde_json::to_string(&line).map_err(|e| se(format!("serialize error: {e}")))
}

/// Update a line. Returns updated line as JSON.
#[allow(clippy::too_many_arguments)]
#[server]
pub async fn update_line(
    id: i32,
    name: Option<String>,
    line_number: Option<String>,
    username: Option<String>,
    password: Option<String>,
    ip_address: Option<String>,
    isp: Option<String>,
    description: Option<String>,
    gateway_ip: Option<String>,
    is_active: Option<bool>,
) -> Result<String, ServerFnError> {
    use crate::state::WebAppState;
    use crate::auth::session::require_session;
    use net_ninja::services::LineService;
    use net_ninja::models::UpdateLineRequest;

    require_session().await?;

    let Some(state) = use_context::<WebAppState>() else {
        return Err(se("state unavailable"));
    };
    let Some(pool) = state.pool.clone() else {
        return Err(se("database unavailable"));
    };

    let req = UpdateLineRequest {
        name,
        line_number,
        username,
        password,
        ip_address,
        isp,
        description,
        gateway_ip,
        is_active,
    };

    let line = LineService::update(&pool, id, req)
        .await
        .map_err(|e| se(format!("service error: {e}")))?;

    serde_json::to_string(&line).map_err(|e| se(format!("serialize error: {e}")))
}

/// Delete a line.
#[server]
pub async fn delete_line(id: i32) -> Result<(), ServerFnError> {
    use crate::state::WebAppState;
    use crate::auth::session::require_session;
    use net_ninja::services::LineService;

    require_session().await?;

    let Some(state) = use_context::<WebAppState>() else {
        return Err(se("state unavailable"));
    };
    let Some(pool) = state.pool.clone() else {
        return Err(se("database unavailable"));
    };

    LineService::delete(&pool, id)
        .await
        .map_err(|e| se(format!("service error: {e}")))
}
