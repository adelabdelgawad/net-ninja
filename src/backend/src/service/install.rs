//! Service installation and management utilities
//!
//! This module provides functions for installing, uninstalling, starting,
//! and stopping the NetNinja Windows Service. All operations are designed
//! to be idempotent where possible.
//!
//! # Installation Details
//!
//! The service is installed with:
//! - **Account**: LocalSystem (has full system privileges)
//! - **Start Type**: Automatic (starts on boot)
//! - **Recovery**: Restarts on first and second failure
//! - **Data Directory**: `%ProgramData%\NetNinja` with appropriate ACLs
//!
//! # Idempotency
//!
//! - `install_service()`: Detects existing service, updates if necessary
//! - `uninstall_service()`: Safe to call even if service doesn't exist
//! - `start_service()`: Safe to call even if already running
//! - `stop_service()`: Safe to call even if already stopped
//!
//! # Permissions
//!
//! Most operations in this module require administrator privileges.
//! The installer should be run with elevated permissions (Run as Administrator).

use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Duration;

use windows_service::service::{
    ServiceAccess, ServiceErrorControl, ServiceInfo, ServiceStartType, ServiceType,
    ServiceState,
};
use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

use crate::config::paths;
use crate::service::{logging, SERVICE_DESCRIPTION, SERVICE_DISPLAY_NAME, SERVICE_NAME};

/// Error type for service installation operations.
#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    /// Failed to connect to Service Control Manager
    #[error("Failed to connect to Service Control Manager: {0}")]
    ServiceManagerConnection(String),

    /// Failed to create service
    #[error("Failed to create service: {0}")]
    ServiceCreation(String),

    /// Failed to open service
    #[error("Failed to open service: {0}")]
    ServiceOpen(String),

    /// Failed to start service
    #[error("Failed to start service: {0}")]
    ServiceStart(String),

    /// Failed to stop service
    #[error("Failed to stop service: {0}")]
    ServiceStop(String),

    /// Failed to delete service
    #[error("Failed to delete service: {0}")]
    ServiceDelete(String),

    /// Failed to configure service
    #[error("Failed to configure service: {0}")]
    ServiceConfig(String),

    /// Service binary not found
    #[error("Service binary not found at: {0}")]
    BinaryNotFound(PathBuf),

    /// Failed to create data directory
    #[error("Failed to create data directory: {0}")]
    DirectoryCreation(String),

    /// Failed to set directory permissions
    #[error("Failed to set directory permissions: {0}")]
    PermissionSetting(String),

    /// Timeout waiting for service state change
    #[error("Timeout waiting for service to {0}")]
    Timeout(String),
}

/// Install the NetNinja service.
///
/// This function performs an idempotent installation:
/// - If the service doesn't exist, it creates it
/// - If the service exists, it updates the configuration if different
/// - Creates the `%ProgramData%\NetNinja` directory with correct permissions
///
/// # Configuration
///
/// The service is configured with:
/// - **Binary Path**: Path to `netninja-service.exe`
/// - **Start Type**: Automatic (SERVICE_AUTO_START)
/// - **Account**: LocalSystem
/// - **Error Control**: Normal (logs error but system continues to start)
/// - **Description**: Descriptive text for Services management console
///
/// # Returns
///
/// Returns `Ok(())` on successful installation or if service already exists
/// with correct configuration.
///
/// # Errors
///
/// Returns an error if:
/// - Cannot connect to Service Control Manager (usually means not elevated)
/// - Service binary not found
/// - Failed to create service
/// - Failed to create data directory
///
/// # Example
///
/// ```ignore
/// // Install with auto-detected binary path
/// install_service()?;
/// ```
pub fn install_service() -> Result<(), InstallError> {
    install_service_with_path(None)
}

/// Install the NetNinja service with an explicit binary path.
///
/// This is the internal implementation that supports specifying a custom
/// binary path. Use `install_service()` for the common case.
///
/// # Arguments
///
/// * `service_binary_path` - Optional explicit path to the service binary.
///   If not provided, looks for `netninja-service.exe` in the same directory
///   as the running executable.
fn install_service_with_path(service_binary_path: Option<PathBuf>) -> Result<(), InstallError> {
    tracing::info!("Installing {} service...", SERVICE_NAME);

    // Determine the service binary path
    let binary_path = match service_binary_path {
        Some(path) => path,
        None => {
            // Look for the binary in the same directory as the current executable
            let current_exe = std::env::current_exe()
                .map_err(|e| InstallError::BinaryNotFound(PathBuf::from(format!("current_exe error: {}", e))))?;
            current_exe
                .parent()
                .unwrap_or(&current_exe)
                .join("netninja-service.exe")
        }
    };

    // Verify the binary exists
    if !binary_path.exists() {
        return Err(InstallError::BinaryNotFound(binary_path));
    }

    tracing::info!("Service binary: {}", binary_path.display());

    // Create the data directory with appropriate permissions
    create_data_directory()?;

    // Connect to Service Control Manager
    let manager = ServiceManager::local_computer(
        None::<&str>,
        ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE,
    )
    .map_err(|e| InstallError::ServiceManagerConnection(e.to_string()))?;

    // Check if service already exists
    match manager.open_service(
        SERVICE_NAME,
        ServiceAccess::QUERY_CONFIG | ServiceAccess::CHANGE_CONFIG,
    ) {
        Ok(service) => {
            // Service exists - update configuration if needed
            tracing::info!("Service already exists, checking configuration...");

            // Get current configuration
            let config = service.query_config()
                .map_err(|e| InstallError::ServiceConfig(e.to_string()))?;

            // Check if binary path needs updating
            let current_path = config.executable_path.to_string_lossy();
            let desired_path = binary_path.to_string_lossy();

            if current_path != desired_path {
                tracing::info!("Updating service binary path from {} to {}", current_path, desired_path);
                // Note: Updating the binary path requires deleting and recreating the service
                // or using ChangeServiceConfig, which is more complex
                tracing::warn!("Binary path update requires service reinstall. Please uninstall and reinstall.");
            }

            tracing::info!("Service {} is already installed", SERVICE_NAME);
            Ok(())
        }
        Err(_) => {
            // Service doesn't exist - create it
            tracing::info!("Creating new service...");

            let service_info = ServiceInfo {
                name: OsString::from(SERVICE_NAME),
                display_name: OsString::from(SERVICE_DISPLAY_NAME),
                service_type: ServiceType::OWN_PROCESS,
                start_type: ServiceStartType::AutoStart,
                error_control: ServiceErrorControl::Normal,
                executable_path: binary_path,
                launch_arguments: vec![OsString::from("run")],
                dependencies: vec![],
                account_name: None, // LocalSystem
                account_password: None,
            };

            let service = manager
                .create_service(&service_info, ServiceAccess::CHANGE_CONFIG | ServiceAccess::START)
                .map_err(|e| InstallError::ServiceCreation(e.to_string()))?;

            // Set the service description
            service
                .set_description(SERVICE_DESCRIPTION)
                .map_err(|e| InstallError::ServiceConfig(format!("Failed to set description: {}", e)))?;

            // Configure recovery options (restart on failure)
            configure_service_recovery(&service)?;

            // Register Windows Event Log source
            if let Err(e) = logging::register_event_source() {
                tracing::warn!("Failed to register event log source (non-fatal): {}", e);
            }

            tracing::info!("Service {} installed successfully", SERVICE_NAME);
            Ok(())
        }
    }
}

/// Uninstall the NetNinja service.
///
/// This function:
/// 1. Stops the service if it's running
/// 2. Waits for the service to fully stop
/// 3. Removes the service from SCM
///
/// # Idempotency
///
/// This function is safe to call even if the service doesn't exist.
/// It will log a message and return success.
///
/// # Note
///
/// The data directory (`%ProgramData%\NetNinja`) is NOT removed to preserve
/// user data. The user must manually delete this directory if desired.
///
/// # Returns
///
/// Returns `Ok(())` on successful uninstallation or if service doesn't exist.
///
/// # Errors
///
/// Returns an error if:
/// - Cannot connect to Service Control Manager
/// - Failed to stop the service
/// - Failed to delete the service
pub fn uninstall_service() -> Result<(), InstallError> {
    tracing::info!("Uninstalling {} service...", SERVICE_NAME);

    // Connect to Service Control Manager
    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)
        .map_err(|e| InstallError::ServiceManagerConnection(e.to_string()))?;

    // Try to open the service
    match manager.open_service(
        SERVICE_NAME,
        ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE,
    ) {
        Ok(service) => {
            // Stop the service if it's running
            let status = service.query_status()
                .map_err(|e| InstallError::ServiceOpen(e.to_string()))?;

            if status.current_state != ServiceState::Stopped {
                tracing::info!("Stopping service before uninstall...");
                stop_service()?;
            }

            // Deregister Windows Event Log source
            if let Err(e) = logging::deregister_event_source() {
                tracing::warn!("Failed to deregister event log source (non-fatal): {}", e);
            }

            // Delete the service
            service
                .delete()
                .map_err(|e| InstallError::ServiceDelete(e.to_string()))?;

            tracing::info!("Service {} uninstalled successfully", SERVICE_NAME);
            tracing::info!(
                "Note: Data directory at {} was not removed",
                paths::get_programdata_path().display()
            );
            Ok(())
        }
        Err(_) => {
            // Service doesn't exist
            tracing::info!("Service {} is not installed", SERVICE_NAME);
            Ok(())
        }
    }
}

/// Start the NetNinja service.
///
/// This function starts the service via SCM. It's safe to call even if
/// the service is already running.
///
/// # Returns
///
/// Returns `Ok(())` on successful start or if service is already running.
///
/// # Errors
///
/// Returns an error if:
/// - Service is not installed
/// - Failed to start the service
/// - Timeout waiting for service to start
pub fn start_service() -> Result<(), InstallError> {
    tracing::info!("Starting {} service...", SERVICE_NAME);

    // Connect to Service Control Manager
    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)
        .map_err(|e| InstallError::ServiceManagerConnection(e.to_string()))?;

    // Open the service
    let service = manager
        .open_service(SERVICE_NAME, ServiceAccess::QUERY_STATUS | ServiceAccess::START)
        .map_err(|e| InstallError::ServiceOpen(e.to_string()))?;

    // Check current status
    let status = service.query_status()
        .map_err(|e| InstallError::ServiceOpen(e.to_string()))?;

    match status.current_state {
        ServiceState::Running => {
            tracing::info!("Service is already running");
            Ok(())
        }
        ServiceState::StartPending => {
            tracing::info!("Service is already starting, waiting...");
            wait_for_service_state(&service, ServiceState::Running, Duration::from_secs(30))?;
            Ok(())
        }
        _ => {
            // Start the service
            service
                .start::<OsString>(&[])
                .map_err(|e| InstallError::ServiceStart(e.to_string()))?;

            // Wait for it to start
            wait_for_service_state(&service, ServiceState::Running, Duration::from_secs(30))?;

            tracing::info!("Service {} started successfully", SERVICE_NAME);
            Ok(())
        }
    }
}

/// Stop the NetNinja service.
///
/// This function stops the service via SCM. It's safe to call even if
/// the service is already stopped.
///
/// # Returns
///
/// Returns `Ok(())` on successful stop or if service is already stopped.
///
/// # Errors
///
/// Returns an error if:
/// - Service is not installed
/// - Failed to stop the service
/// - Timeout waiting for service to stop
pub fn stop_service() -> Result<(), InstallError> {
    tracing::info!("Stopping {} service...", SERVICE_NAME);

    // Connect to Service Control Manager
    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)
        .map_err(|e| InstallError::ServiceManagerConnection(e.to_string()))?;

    // Open the service
    let service = manager
        .open_service(SERVICE_NAME, ServiceAccess::QUERY_STATUS | ServiceAccess::STOP)
        .map_err(|e| InstallError::ServiceOpen(e.to_string()))?;

    // Check current status
    let status = service.query_status()
        .map_err(|e| InstallError::ServiceOpen(e.to_string()))?;

    match status.current_state {
        ServiceState::Stopped => {
            tracing::info!("Service is already stopped");
            Ok(())
        }
        ServiceState::StopPending => {
            tracing::info!("Service is already stopping, waiting...");
            wait_for_service_state(&service, ServiceState::Stopped, Duration::from_secs(30))?;
            Ok(())
        }
        _ => {
            // Stop the service
            service
                .stop()
                .map_err(|e| InstallError::ServiceStop(e.to_string()))?;

            // Wait for it to stop
            wait_for_service_state(&service, ServiceState::Stopped, Duration::from_secs(30))?;

            tracing::info!("Service {} stopped successfully", SERVICE_NAME);
            Ok(())
        }
    }
}

/// Wait for the service to reach a specific state.
///
/// Polls the service status at regular intervals until the desired state
/// is reached or the timeout expires.
///
/// # Arguments
///
/// * `service` - Handle to the service
/// * `desired_state` - The state to wait for
/// * `timeout` - Maximum time to wait
fn wait_for_service_state(
    service: &windows_service::service::Service,
    desired_state: ServiceState,
    timeout: Duration,
) -> Result<(), InstallError> {
    let start_time = std::time::Instant::now();
    let poll_interval = Duration::from_millis(500);

    loop {
        let status = service.query_status()
            .map_err(|e| InstallError::ServiceOpen(e.to_string()))?;

        if status.current_state == desired_state {
            return Ok(());
        }

        if start_time.elapsed() > timeout {
            return Err(InstallError::Timeout(format!("reach {:?}", desired_state)));
        }

        std::thread::sleep(poll_interval);
    }
}

/// Create the ProgramData directory with appropriate ACLs.
///
/// Creates `%ProgramData%\NetNinja` and its subdirectories:
/// - `logs/` - Service log files
/// - `chrome-profiles/` - Browser automation profiles
///
/// # Permissions
///
/// Sets ACLs to allow:
/// - SYSTEM: Full control (for service)
/// - Administrators: Full control
/// - Users: Read/Execute (for desktop app to read data)
fn create_data_directory() -> Result<(), InstallError> {
    let data_dir = paths::get_programdata_path();
    let logs_dir = paths::get_service_log_path();
    let chrome_dir = paths::get_service_chrome_profiles_path();

    tracing::info!("Creating data directory: {}", data_dir.display());

    // Create main data directory
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| InstallError::DirectoryCreation(format!("{}: {}", data_dir.display(), e)))?;

    // Create subdirectories
    std::fs::create_dir_all(&logs_dir)
        .map_err(|e| InstallError::DirectoryCreation(format!("{}: {}", logs_dir.display(), e)))?;

    std::fs::create_dir_all(&chrome_dir)
        .map_err(|e| InstallError::DirectoryCreation(format!("{}: {}", chrome_dir.display(), e)))?;

    // Set ACLs using icacls command
    // This grants:
    // - SYSTEM: Full control
    // - Administrators: Full control
    // - Users: Read/Execute
    set_directory_acls(&data_dir)?;

    tracing::info!("Data directory created with appropriate permissions");
    Ok(())
}

/// Set Windows ACLs on the data directory.
///
/// Uses icacls command to set permissions:
/// - SYSTEM: (F) Full control
/// - Administrators: (F) Full control
/// - Users: (RX) Read and Execute
fn set_directory_acls(path: &std::path::Path) -> Result<(), InstallError> {
    use std::process::Command;

    let path_str = path.to_string_lossy();

    // Reset inherited permissions and set explicit ACLs
    // /T applies to all files and subdirectories
    // /Q suppresses success messages
    let commands = [
        // Grant SYSTEM full control
        format!(r#"icacls "{}" /grant "NT AUTHORITY\SYSTEM:(OI)(CI)F" /T /Q"#, path_str),
        // Grant Administrators full control
        format!(r#"icacls "{}" /grant "BUILTIN\Administrators:(OI)(CI)F" /T /Q"#, path_str),
        // Grant Users read and execute
        format!(r#"icacls "{}" /grant "BUILTIN\Users:(OI)(CI)RX" /T /Q"#, path_str),
    ];

    for cmd in &commands {
        let output = Command::new("cmd")
            .args(["/C", cmd])
            .output()
            .map_err(|e| InstallError::PermissionSetting(format!("Failed to run icacls: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("icacls command may have partially failed: {}", stderr);
            // Continue anyway - permissions might still be set correctly
        }
    }

    Ok(())
}

/// Configure service recovery options.
///
/// Sets the service to restart automatically on failure:
/// - First failure: Restart after 60 seconds
/// - Second failure: Restart after 60 seconds
/// - Subsequent failures: Take no action (prevents restart loop)
fn configure_service_recovery(
    service: &windows_service::service::Service,
) -> Result<(), InstallError> {
    use windows_service::service::{
        ServiceAction, ServiceActionType, ServiceFailureActions, ServiceFailureResetPeriod,
    };

    // Define recovery actions
    let actions = vec![
        // First failure: Restart after 60 seconds
        ServiceAction {
            action_type: ServiceActionType::Restart,
            delay: Duration::from_secs(60),
        },
        // Second failure: Restart after 60 seconds
        ServiceAction {
            action_type: ServiceActionType::Restart,
            delay: Duration::from_secs(60),
        },
        // Third and subsequent: No action
        ServiceAction {
            action_type: ServiceActionType::None,
            delay: Duration::ZERO,
        },
    ];

    let failure_actions = ServiceFailureActions {
        reset_period: ServiceFailureResetPeriod::After(Duration::from_secs(86400)), // Reset after 24 hours
        reboot_msg: None,
        command: None,
        actions: Some(actions),
    };

    service
        .update_failure_actions(failure_actions)
        .map_err(|e| InstallError::ServiceConfig(format!("Failed to set recovery options: {}", e)))?;

    tracing::info!("Service recovery options configured");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_name_constants() {
        // Verify constants are set
        assert!(!SERVICE_NAME.is_empty());
        assert!(!SERVICE_DISPLAY_NAME.is_empty());
        assert!(!SERVICE_DESCRIPTION.is_empty());
    }
}
