use super::*;
use crate::config::get_sqlite_path;

// ===== Helper Functions =====

// Helper function to convert AppError to String
pub(super) fn map_err<E: Into<AppError>>(e: E) -> String {
    Into::<AppError>::into(e).to_string()
}

// Extract SQLite pool
pub(super) fn get_sqlite_pool(state: &AppState) -> Result<&sqlx::SqlitePool, String> {
    state
        .pool
        .as_ref()
        .ok_or_else(|| "Not available in fallback mode - Database connection required".to_string())
}

// ===== Health Commands =====

/// Health check response
#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthCheckResponse {
    pub status: String,
    #[serde(rename = "databaseConnected")]
    pub database_connected: bool,
    #[serde(rename = "databasePath")]
    pub database_path: String,
    #[serde(rename = "initMode")]
    pub init_mode: String,
}

#[tauri::command]
pub async fn health_check(state: State<'_, AppState>) -> Result<HealthCheckResponse, String> {
    let db_path = get_sqlite_path();

    Ok(HealthCheckResponse {
        status: "OK".to_string(),
        database_connected: state.pool.is_some(),
        database_path: db_path.display().to_string(),
        init_mode: format!("{:?}", state.init_mode),
    })
}

// ===== Fallback Status =====

/// Response for fallback status check
#[derive(Debug, Clone, serde::Serialize)]
pub struct FallbackStatusResponse {
    pub is_fallback: bool,
    pub init_mode: String,
    pub error: Option<String>,
}

/// Check if the app is running in fallback mode
#[tauri::command]
pub async fn get_fallback_status(
    state: State<'_, AppState>,
) -> Result<FallbackStatusResponse, String> {
    Ok(FallbackStatusResponse {
        is_fallback: state.is_fallback_mode(),
        init_mode: format!("{:?}", state.init_mode),
        error: state.init_error.as_ref().map(|s| s.to_string()),
    })
}

// ===== Service Status =====

/// Response for service status check
#[derive(Debug, Clone, serde::Serialize)]
pub struct ServiceStatusResponse {
    /// Whether the service is registered with the Service Control Manager
    pub installed: bool,
    /// Whether the service is currently running
    pub running: bool,
    /// Service version if available
    pub version: Option<String>,
    /// ISO timestamp of last heartbeat
    #[serde(rename = "lastHeartbeat")]
    pub last_heartbeat: Option<String>,
    /// Who holds the lock ("service" or "desktop")
    #[serde(rename = "lockHolder")]
    pub lock_holder: Option<String>,
}

/// Get the current status of the NetNinja Windows service.
///
/// Lock holder / heartbeat / version are read from the shared SQLite database
/// (the `scheduler_lock` and `service_info` tables) and are meaningful on any
/// platform. Installed/running come from the Windows Service Control Manager and
/// are false when off-Windows or the `service` feature is disabled.
#[tauri::command]
pub async fn get_service_status(
    state: State<'_, AppState>,
) -> Result<ServiceStatusResponse, String> {
    let (lock_holder, last_heartbeat, version) = match state.pool.as_ref() {
        Some(pool) => {
            let lock = crate::service::SchedulerLock::new(pool.clone());
            let info = lock.get_lock_holder().await.ok().flatten();
            let holder = info.as_ref().map(|i| i.holder.clone());
            let heartbeat = info.as_ref().map(|i| i.heartbeat_at.to_rfc3339());
            let ver = info.as_ref().and_then(|i| i.version.clone());

            // Prefer the persisted service version stamp if present.
            let ver = match crate::repositories::ServiceInfoRepository::get(pool, "service_version")
                .await
            {
                Ok(Some((v, _))) => Some(v),
                _ => ver,
            };
            (holder, heartbeat, ver)
        }
        None => (None, None, None),
    };

    let (installed, running) = service_installed_running();

    Ok(ServiceStatusResponse {
        installed,
        running,
        version,
        last_heartbeat,
        lock_holder,
    })
}

/// Query the Service Control Manager for (installed, running).
///
/// Returns `(false, false)` off-Windows or when the `service` feature is off.
fn service_installed_running() -> (bool, bool) {
    #[cfg(all(target_os = "windows", feature = "service"))]
    {
        use windows_service::{
            service::{ServiceAccess, ServiceState},
            service_manager::{ServiceManager, ServiceManagerAccess},
        };

        let manager =
            match ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT) {
                Ok(m) => m,
                Err(e) => {
                    tracing::warn!("Failed to connect to SCM: {}", e);
                    return (false, false);
                }
            };

        // Use the canonical service name (was previously hard-coded to "NetNinja",
        // which never matched the installed "netninja-scheduler" → always false).
        match manager.open_service(crate::service::SERVICE_NAME, ServiceAccess::QUERY_STATUS) {
            Ok(service) => match service.query_status() {
                Ok(status) => (true, status.current_state == ServiceState::Running),
                Err(e) => {
                    tracing::warn!("Failed to query service status: {}", e);
                    (true, false)
                }
            },
            Err(_) => (false, false), // not installed
        }
    }

    #[cfg(not(all(target_os = "windows", feature = "service")))]
    {
        (false, false)
    }
}
