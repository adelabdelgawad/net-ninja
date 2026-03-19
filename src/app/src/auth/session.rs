//! Session helpers for web auth.

use leptos::server_fn::error::ServerFnError;
use tower_sessions::Session;

pub const SESSION_KEY: &str = "admin_username";

/// Get the current session username, if authenticated.
pub async fn get_session() -> Option<String> {
    use leptos_axum::extract;
    let session: Session = extract().await.ok()?;
    session.get::<String>(SESSION_KEY).await.ok().flatten()
}

/// Require an authenticated session. Returns the username or a ServerFnError.
/// The caller should return the error to trigger client-side redirect to /login.
pub async fn require_session() -> Result<String, ServerFnError> {
    get_session().await.ok_or_else(|| {
        ServerFnError::ServerError("unauthenticated".to_string())
    })
}
