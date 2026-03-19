use leptos::prelude::*;

pub const SESSION_KEY: &str = "admin_username";

/// Helper: construct a ServerFnError with unambiguous type (NoCustomError default).
fn se(msg: impl std::fmt::Display) -> ServerFnError {
    ServerFnError::ServerError(msg.to_string())
}

/// T019: Login — rate limit, verify credentials, create session.
#[server]
pub async fn login(username: String, password: String) -> Result<(), ServerFnError> {
    use leptos_axum::extract;
    use tower_sessions::Session;
    use std::net::{IpAddr, SocketAddr};
    use axum::extract::ConnectInfo;
    use crate::state::WebAppState;
    use crate::auth::rate_limit::check_rate_limit;
    use crate::startup::verify_password;

    let Some(state) = use_context::<WebAppState>() else {
        return Err(se("state unavailable"));
    };

    let session: Session = extract().await.map_err(|e| se(format!("session error: {e}")))?;

    // Get IP for rate limiting (requires into_make_service_with_connect_info in main.rs)
    let ip: IpAddr = extract::<ConnectInfo<SocketAddr>>().await
        .map(|ci| ci.0.ip())
        .unwrap_or(IpAddr::from([127, 0, 0, 1]));

    if !check_rate_limit(&state, ip) {
        return Err(se("rate_limited"));
    }

    let Some(pool) = state.pool.clone() else {
        return Err(se("database unavailable"));
    };

    let row: Option<(String,)> = sqlx::query_as(
        "SELECT password_hash FROM web_admin_credentials WHERE username = $1"
    )
    .bind(&username)
    .fetch_optional(&pool)
    .await
    .map_err(|e| se(format!("db error: {e}")))?;

    let Some((hash,)) = row else {
        return Err(se("invalid_credentials"));
    };

    if !verify_password(&password, &hash) {
        return Err(se("invalid_credentials"));
    }

    session.insert(SESSION_KEY, username.clone()).await
        .map_err(|e| se(format!("session error: {e}")))?;

    tracing::info!(user = %username, "Admin logged in");
    Ok(())
}

/// T020: Logout — destroy the session.
#[server]
pub async fn logout() -> Result<(), ServerFnError> {
    use leptos_axum::extract;
    use tower_sessions::Session;

    let session: Session = extract().await.map_err(|e| se(format!("session error: {e}")))?;

    let username = session.get::<String>(SESSION_KEY).await
        .ok()
        .flatten()
        .unwrap_or_default();

    session.flush().await.map_err(|e| se(format!("session error: {e}")))?;

    tracing::info!(user = %username, "Admin logged out");
    Ok(())
}

/// T021: Change password — verify current, hash and store new.
#[server]
pub async fn change_password(current: String, new_password: String) -> Result<(), ServerFnError> {
    use crate::state::WebAppState;
    use crate::auth::session::require_session;
    use crate::startup::{hash_password, verify_password};

    let username = require_session().await?;

    let Some(state) = use_context::<WebAppState>() else {
        return Err(se("state unavailable"));
    };

    let Some(pool) = state.pool.clone() else {
        return Err(se("database unavailable"));
    };

    let row: Option<(String,)> = sqlx::query_as(
        "SELECT password_hash FROM web_admin_credentials WHERE username = $1"
    )
    .bind(&username)
    .fetch_optional(&pool)
    .await
    .map_err(|e| se(format!("db error: {e}")))?;

    let Some((hash,)) = row else {
        return Err(se("invalid_credentials"));
    };

    if !verify_password(&current, &hash) {
        return Err(se("wrong_current_password"));
    }

    let new_hash = hash_password(&new_password);

    sqlx::query(
        "UPDATE web_admin_credentials SET password_hash = $1, updated_at = datetime('now') WHERE username = $2"
    )
    .bind(&new_hash)
    .bind(&username)
    .execute(&pool)
    .await
    .map_err(|e| se(format!("db error: {e}")))?;

    tracing::info!(user = %username, "Admin changed password");
    Ok(())
}

/// Used by AppShell auth guard — returns Ok(()) if session is valid.
#[server]
pub async fn check_auth() -> Result<(), ServerFnError> {
    use crate::auth::session::require_session;
    require_session().await.map(|_| ())
}
