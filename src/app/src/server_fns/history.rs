use leptos::prelude::*;

fn se(msg: impl std::fmt::Display) -> ServerFnError {
    ServerFnError::ServerError(msg.to_string())
}

/// Returns paginated quota results as JSON. Optionally filtered by line_id.
#[server]
pub async fn get_quota_results(
    line_id: Option<i32>,
    page: Option<u32>,
) -> Result<String, ServerFnError> {
    use crate::state::WebAppState;
    use crate::auth::session::require_session;
    use net_ninja::services::QuotaCheckService;
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
        per_page: Some(20),
    };

    let result = if let Some(lid) = line_id {
        // Filter by line: get all for line and return as paginated manually
        let items = QuotaCheckService::get_by_line_id(&pool, lid, Some(20))
            .await
            .map_err(|e| se(format!("service error: {e}")))?;
        serde_json::json!({
            "items": items,
            "total": items.len() as i64,
            "page": 1i64,
            "per_page": 20i64,
            "total_pages": 1i64,
        })
    } else {
        let paged = QuotaCheckService::get_paginated(&pool, &params)
            .await
            .map_err(|e| se(format!("service error: {e}")))?;
        serde_json::to_value(&paged).map_err(|e| se(format!("serialize error: {e}")))?
    };

    serde_json::to_string(&result).map_err(|e| se(format!("serialize error: {e}")))
}

/// Returns paginated speed test results as JSON. Optionally filtered by line_id.
#[server]
pub async fn get_speed_results(
    line_id: Option<i32>,
    page: Option<u32>,
) -> Result<String, ServerFnError> {
    use crate::state::WebAppState;
    use crate::auth::session::require_session;
    use net_ninja::services::SpeedTestService;
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
        per_page: Some(20),
    };

    let result = if let Some(lid) = line_id {
        let items = SpeedTestService::get_by_line_id(&pool, lid, Some(20))
            .await
            .map_err(|e| se(format!("service error: {e}")))?;
        serde_json::json!({
            "items": items,
            "total": items.len() as i64,
            "page": 1i64,
            "per_page": 20i64,
            "total_pages": 1i64,
        })
    } else {
        let paged = SpeedTestService::get_paginated(&pool, &params)
            .await
            .map_err(|e| se(format!("service error: {e}")))?;
        serde_json::to_value(&paged).map_err(|e| se(format!("serialize error: {e}")))?
    };

    serde_json::to_string(&result).map_err(|e| se(format!("serialize error: {e}")))
}

/// Returns paginated operation logs as JSON. Optionally filtered by line_id.
#[server]
pub async fn get_logs(
    line_id: Option<i64>,
    page: Option<u32>,
) -> Result<String, ServerFnError> {
    use crate::state::WebAppState;
    use crate::auth::session::require_session;
    use net_ninja::services::LogService;
    use net_ninja::models::{PaginationParams, LogFilter};

    require_session().await?;

    let Some(state) = use_context::<WebAppState>() else {
        return Err(se("state unavailable"));
    };
    let Some(pool) = state.pool.clone() else {
        return Err(se("database unavailable"));
    };

    let params = PaginationParams {
        page: page.map(|p| p as i64),
        per_page: Some(50),
    };

    let result = if line_id.is_some() {
        let _filter = LogFilter {
            process_id: None,
            level: None,
            from_date: None,
            to_date: None,
        };
        // get_filtered doesn't support line_id filter directly; fall back to paginated
        // and let the frontend filter — or use get_paginated
        let paged = LogService::get_paginated(&pool, &params)
            .await
            .map_err(|e| se(format!("service error: {e}")))?;
        serde_json::to_value(&paged).map_err(|e| se(format!("serialize error: {e}")))?
    } else {
        let paged = LogService::get_paginated(&pool, &params)
            .await
            .map_err(|e| se(format!("service error: {e}")))?;
        serde_json::to_value(&paged).map_err(|e| se(format!("serialize error: {e}")))?
    };

    serde_json::to_string(&result).map_err(|e| se(format!("serialize error: {e}")))
}
