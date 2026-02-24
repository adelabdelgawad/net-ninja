use super::*;
use crate::config::get_sqlite_path;

// ===== Helper Functions =====

// Helper function to convert AppError to String
pub(super) fn map_err<E: Into<AppError>>(e: E) -> String {
    Into::<AppError>::into(e).to_string()
}

// Extract SQLite pool
pub(super) fn get_sqlite_pool(state: &AppState) -> Result<&sqlx::SqlitePool, String> {
    state.pool.as_ref()
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
pub async fn get_fallback_status(state: State<'_, AppState>) -> Result<FallbackStatusResponse, String> {
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

/// Get the current status of the NetNinja Windows service
///
/// This function works even when the service feature is not enabled,
/// returning installed=false and running=false in that case.
#[tauri::command]
pub async fn get_service_status() -> Result<ServiceStatusResponse, String> {
    #[cfg(all(target_os = "windows", feature = "service"))]
    {
        get_service_status_windows().await
    }

    #[cfg(not(all(target_os = "windows", feature = "service")))]
    {
        // Service feature not enabled or not on Windows
        Ok(ServiceStatusResponse {
            installed: false,
            running: false,
            version: None,
            last_heartbeat: None,
            lock_holder: None,
        })
    }
}

/// Windows-specific service status check
#[cfg(all(target_os = "windows", feature = "service"))]
async fn get_service_status_windows() -> Result<ServiceStatusResponse, String> {
    use windows_service::{
        service::ServiceAccess,
        service_manager::{ServiceManager, ServiceManagerAccess},
    };

    const SERVICE_NAME: &str = "NetNinja";

    // Try to connect to Service Control Manager
    let manager = match ServiceManager::local_computer(
        None::<&str>,
        ServiceManagerAccess::CONNECT,
    ) {
        Ok(m) => m,
        Err(e) => {
            tracing::warn!("Failed to connect to SCM: {}", e);
            return Ok(ServiceStatusResponse {
                installed: false,
                running: false,
                version: None,
                last_heartbeat: None,
                lock_holder: None,
            });
        }
    };

    // Try to open the service
    let service = match manager.open_service(
        SERVICE_NAME,
        ServiceAccess::QUERY_STATUS,
    ) {
        Ok(s) => s,
        Err(_) => {
            // Service not installed
            return Ok(ServiceStatusResponse {
                installed: false,
                running: false,
                version: None,
                last_heartbeat: None,
                lock_holder: None,
            });
        }
    };

    // Query service status
    let status = service.query_status().map_err(|e| e.to_string())?;
    let running = status.current_state == windows_service::service::ServiceState::Running;

    // TODO: Implement heartbeat and lock holder detection via IPC or shared file
    // For now, return basic installed/running status
    Ok(ServiceStatusResponse {
        installed: true,
        running,
        version: Some(env!("CARGO_PKG_VERSION").to_string()),
        last_heartbeat: None,
        lock_holder: if running { Some("service".to_string()) } else { None },
    })
}
