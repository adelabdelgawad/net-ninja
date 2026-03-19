use leptos::prelude::*;

fn se(msg: impl std::fmt::Display) -> ServerFnError {
    ServerFnError::ServerError(msg.to_string())
}

/// T040: Get all SMTP configs (returns JSON array).
#[server]
pub async fn get_smtp_configs() -> Result<String, ServerFnError> {
    use crate::auth::session::require_session;
    use crate::state::WebAppState;

    require_session().await?;

    let Some(state) = use_context::<WebAppState>() else {
        return Err(se("state unavailable"));
    };
    let Some(pool) = state.pool.clone() else {
        return Err(se("database unavailable"));
    };

    let enc_key = state.encryption_key.as_deref();

    let configs = net_ninja::services::SmtpConfigService::get_all(&pool, enc_key)
        .await
        .map_err(|e| se(e.to_string()))?;

    serde_json::to_string(&configs).map_err(|e| se(e.to_string()))
}

/// Create a new SMTP config from JSON-encoded CreateSmtpConfigRequest.
#[allow(clippy::too_many_arguments)]
#[server]
pub async fn create_smtp_config(
    name: String,
    host: String,
    port: i32,
    from_email: String,
    username: Option<String>,
    password: Option<String>,
    use_tls: bool,
    is_default: bool,
) -> Result<String, ServerFnError> {
    use crate::auth::session::require_session;
    use crate::state::WebAppState;

    require_session().await?;

    let Some(state) = use_context::<WebAppState>() else {
        return Err(se("state unavailable"));
    };
    let Some(pool) = state.pool.clone() else {
        return Err(se("database unavailable"));
    };

    let enc_key = state.encryption_key.as_deref();

    let req = net_ninja::models::CreateSmtpConfigRequest {
        name,
        host,
        port: Some(port),
        vendor: None,
        username,
        password,
        from_email,
        from_name: None,
        use_tls: Some(use_tls),
        is_default: Some(is_default),
        is_active: Some(true),
    };

    let config = net_ninja::services::SmtpConfigService::create(&pool, req, enc_key)
        .await
        .map_err(|e| se(e.to_string()))?;

    serde_json::to_string(&config).map_err(|e| se(e.to_string()))
}

/// Update an existing SMTP config.
#[allow(clippy::too_many_arguments)]
#[server]
pub async fn update_smtp_config(
    id: i32,
    host: Option<String>,
    port: Option<i32>,
    from_email: Option<String>,
    username: Option<String>,
    password: Option<String>,
    use_tls: Option<bool>,
    is_default: Option<bool>,
) -> Result<String, ServerFnError> {
    use crate::auth::session::require_session;
    use crate::state::WebAppState;

    require_session().await?;

    let Some(state) = use_context::<WebAppState>() else {
        return Err(se("state unavailable"));
    };
    let Some(pool) = state.pool.clone() else {
        return Err(se("database unavailable"));
    };

    let enc_key = state.encryption_key.as_deref();

    let req = net_ninja::models::UpdateSmtpConfigRequest {
        name: None,
        host,
        port,
        vendor: None,
        username,
        password,
        from_email,
        from_name: None,
        use_tls,
        is_default,
        is_active: None,
    };

    let config = net_ninja::services::SmtpConfigService::update(&pool, id, req, enc_key)
        .await
        .map_err(|e| se(e.to_string()))?;

    serde_json::to_string(&config).map_err(|e| se(e.to_string()))
}

/// Delete an SMTP config.
#[server]
pub async fn delete_smtp_config(id: i32) -> Result<(), ServerFnError> {
    use crate::auth::session::require_session;
    use crate::state::WebAppState;

    require_session().await?;

    let Some(state) = use_context::<WebAppState>() else {
        return Err(se("state unavailable"));
    };
    let Some(pool) = state.pool.clone() else {
        return Err(se("database unavailable"));
    };

    net_ninja::services::SmtpConfigService::delete(&pool, id)
        .await
        .map_err(|e| se(e.to_string()))
}

/// Get notification config for a task (returns JSON or null).
#[server]
pub async fn get_task_notification_config(task_id: i64) -> Result<String, ServerFnError> {
    use crate::auth::session::require_session;
    use crate::state::WebAppState;

    require_session().await?;

    let Some(state) = use_context::<WebAppState>() else {
        return Err(se("state unavailable"));
    };
    let Some(pool) = state.pool.clone() else {
        return Err(se("database unavailable"));
    };

    let config = net_ninja::services::TaskNotificationConfigService::get_by_task_id(&pool, task_id as i32)
        .await
        .map_err(|e| se(e.to_string()))?;

    serde_json::to_string(&config).map_err(|e| se(e.to_string()))
}

/// Upsert notification config for a task.
#[server]
pub async fn upsert_task_notification_config(
    task_id: i64,
    smtp_config_id: i32,
    is_enabled: bool,
    email_subject: Option<String>,
    to_ids: Vec<i32>,
    cc_ids: Vec<i32>,
) -> Result<String, ServerFnError> {
    use crate::auth::session::require_session;
    use crate::state::WebAppState;

    require_session().await?;

    let Some(state) = use_context::<WebAppState>() else {
        return Err(se("state unavailable"));
    };
    let Some(pool) = state.pool.clone() else {
        return Err(se("database unavailable"));
    };

    let req = net_ninja::models::UpsertTaskNotificationConfigRequest {
        smtp_config_id: Some(smtp_config_id),
        is_enabled,
        email_subject,
        to_recipient_ids: to_ids,
        cc_recipient_ids: cc_ids,
    };

    let config = net_ninja::services::TaskNotificationConfigService::upsert(&pool, task_id as i32, req)
        .await
        .map_err(|e| se(e.to_string()))?;

    serde_json::to_string(&config).map_err(|e| se(e.to_string()))
}
