//! Windows Service Control Manager (SCM) integration
//!
//! This module provides the entry point for running NetNinja as a Windows Service.
//! It uses the `windows-service` crate to handle the service lifecycle and
//! communicate with the Windows Service Control Manager.
//!
//! # Service Lifecycle
//!
//! 1. SCM starts the service and calls our entry point
//! 2. We register a service control handler to receive control events
//! 3. We report SERVICE_RUNNING to SCM
//! 4. The scheduler loop runs until shutdown is requested
//! 5. On Stop/Shutdown, we signal the scheduler and wait for graceful shutdown
//! 6. We report SERVICE_STOPPED to SCM
//!
//! # Error Handling
//!
//! Service errors are logged to both the service log file and Windows Event Log.
//! If the service fails to start, we report SERVICE_STOPPED with an error exit code.

use std::ffi::OsString;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult, ServiceStatusHandle},
    service_dispatcher,
};

use crate::errors::AppError;
use crate::service::{logging, scheduler, SERVICE_NAME};

/// Service error type for Windows-specific failures
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    /// Failed to register service control handler
    #[error("Failed to register service control handler: {0}")]
    ControlHandlerRegistration(String),

    /// Failed to set service status
    #[error("Failed to set service status: {0}")]
    StatusUpdate(String),

    /// Scheduler error during service operation
    #[error("Scheduler error: {0}")]
    Scheduler(#[from] AppError),

    /// Service dispatcher error
    #[error("Service dispatcher error: {0}")]
    Dispatcher(String),

    /// Logging initialization failed
    #[error("Logging initialization failed: {0}")]
    LoggingInit(String),
}

impl From<windows_service::Error> for ServiceError {
    fn from(e: windows_service::Error) -> Self {
        ServiceError::Dispatcher(e.to_string())
    }
}

// Define the Windows service entry point using the macro from windows-service crate.
//
// This macro generates the required `extern "system"` function that Windows SCM
// calls when starting the service. The generated function will call our
// `service_main` function.
//
// The service name "NetNinjaScheduler" must match the name used during installation.
define_windows_service!(ffi_service_main, service_main);

/// Run the Windows service.
///
/// This is the main entry point called from the service binary. It registers
/// the service with the Windows Service Dispatcher, which then calls our
/// `service_main` function.
///
/// # Returns
///
/// Returns `Ok(())` if the service ran and stopped successfully, or an error
/// if service registration or execution failed.
///
/// # Example
///
/// ```ignore
/// // In src/bin/service.rs
/// fn main() {
///     if let Err(e) = net_ninja::service::run_service() {
///         eprintln!("Service failed: {}", e);
///         std::process::exit(1);
///     }
/// }
/// ```
pub fn run_service() -> Result<(), ServiceError> {
    // Start the service dispatcher. This function blocks until the service is stopped.
    // The dispatcher will call ffi_service_main, which calls our service_main function.
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
    Ok(())
}

/// Service main function called by Windows SCM via the dispatcher.
///
/// This function is responsible for:
/// 1. Initializing logging
/// 2. Registering the service control handler
/// 3. Reporting service status to SCM
/// 4. Running the scheduler loop
/// 5. Performing graceful shutdown
///
/// # Arguments
///
/// * `_arguments` - Command-line arguments passed to the service (typically unused)
fn service_main(_arguments: Vec<OsString>) {
    // Run the actual service logic and handle any errors
    if let Err(e) = run_service_inner() {
        // Log the error to Windows Event Log since file logging may have failed
        logging::log_to_event_log(
            logging::EventLevel::Error,
            &format!("Service failed with error: {}", e),
        );
    }
}

/// Inner service logic with proper error handling.
///
/// This function contains the main service implementation, separated from
/// `service_main` to allow proper error propagation with the `?` operator.
fn run_service_inner() -> Result<(), ServiceError> {
    // Note: logging is already initialized by handle_run() in bin/service.rs
    // before the service dispatcher is started. Do NOT call init_service_logging()
    // here — the global tracing subscriber can only be set once per process.

    // Write startup marker for log correlation
    logging::write_startup_marker();

    tracing::info!("NetNinja Scheduler Service starting...");
    logging::log_to_event_log(logging::EventLevel::Info, "NetNinja Scheduler Service starting");

    // Create a channel to receive shutdown signals from the control handler
    let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>();

    // Register the service control handler to receive events from SCM
    // The handler will send a message on shutdown_tx when Stop or Shutdown is received
    let status_handle = register_control_handler(shutdown_tx)?;

    // Report that we're starting up
    // Accept Stop and Shutdown controls once we're running
    report_service_status(
        &status_handle,
        ServiceState::StartPending,
        ServiceControlAccept::empty(),
        0,
        Duration::from_secs(10),
    )?;

    // Attempt to run the scheduler loop
    // This will acquire the scheduler lock and start processing jobs
    let scheduler_result = run_scheduler_with_shutdown(&status_handle, shutdown_rx);

    // Report stopped status regardless of how we got here
    // Use appropriate exit code based on whether we stopped cleanly
    let exit_code = match &scheduler_result {
        Ok(()) => ServiceExitCode::Win32(0),
        Err(e) => {
            tracing::error!("Service stopping due to error: {}", e);
            logging::log_to_event_log(
                logging::EventLevel::Error,
                &format!("Service stopping due to error: {}", e),
            );
            ServiceExitCode::Win32(1)
        }
    };

    // Write shutdown marker for log correlation
    logging::write_shutdown_marker();

    report_service_stopped(&status_handle, exit_code)?;

    tracing::info!("NetNinja Scheduler Service stopped");
    logging::log_to_event_log(logging::EventLevel::Info, "NetNinja Scheduler Service stopped");

    scheduler_result
}

/// Register the service control handler with Windows SCM.
///
/// The control handler receives events like Stop, Shutdown, Pause, etc.
/// from the Service Control Manager. For NetNinja, we only handle Stop
/// and Shutdown to trigger graceful termination.
///
/// # Arguments
///
/// * `shutdown_tx` - Channel sender to signal the scheduler loop when shutdown is requested
///
/// # Returns
///
/// Returns the status handle used to report service state to SCM.
fn register_control_handler(
    shutdown_tx: mpsc::Sender<()>,
) -> Result<ServiceStatusHandle, ServiceError> {
    // Define the event handler closure
    // This closure is called by Windows whenever a control event is received
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            // Handle Stop request (user clicked Stop in Services, or `sc stop`)
            ServiceControl::Stop => {
                tracing::info!("Received Stop control event");
                // Signal the scheduler to begin shutdown
                // Ignore send errors - the receiver may have already dropped
                let _ = shutdown_tx.send(());
                ServiceControlHandlerResult::NoError
            }

            // Handle Shutdown request (system is shutting down)
            ServiceControl::Shutdown => {
                tracing::info!("Received Shutdown control event (system shutdown)");
                // Signal the scheduler to begin shutdown
                let _ = shutdown_tx.send(());
                ServiceControlHandlerResult::NoError
            }

            // Handle Interrogate request (SCM asking for current status)
            // We respond with NoError; the actual status is reported separately
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,

            // All other control events are not supported
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    // Register the handler with Windows
    service_control_handler::register(SERVICE_NAME, event_handler)
        .map_err(|e| ServiceError::ControlHandlerRegistration(e.to_string()))
}

/// Run the scheduler loop with shutdown signal handling.
///
/// This function:
/// 1. Reports SERVICE_RUNNING to SCM
/// 2. Runs the scheduler loop
/// 3. Handles the shutdown signal from the control handler
/// 4. Performs graceful cleanup
///
/// # Arguments
///
/// * `status_handle` - Handle to report status updates to SCM
/// * `shutdown_rx` - Channel receiver for shutdown signals
fn run_scheduler_with_shutdown(
    status_handle: &ServiceStatusHandle,
    shutdown_rx: Receiver<()>,
) -> Result<(), ServiceError> {
    // Create a tokio runtime for async operations
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| ServiceError::Scheduler(AppError::Internal(e.to_string())))?;

    runtime.block_on(async {
        // Run the scheduler loop
        // This function handles lock acquisition and job execution
        let scheduler_handle = scheduler::run_scheduler_loop().await?;

        // Report that we're now running and accepting Stop/Shutdown controls
        report_service_status(
            status_handle,
            ServiceState::Running,
            ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
            0,
            Duration::ZERO,
        )?;

        tracing::info!("Service is now running");

        // Wait for shutdown signal
        // This blocks until Stop or Shutdown is received
        let _ = shutdown_rx.recv();

        tracing::info!("Shutdown signal received, stopping scheduler...");

        // Report that we're stopping
        report_service_status(
            status_handle,
            ServiceState::StopPending,
            ServiceControlAccept::empty(),
            0,
            Duration::from_secs(30),
        )?;

        // Signal the scheduler to stop and wait for graceful shutdown
        scheduler_handle.shutdown().await?;

        tracing::info!("Scheduler stopped successfully");
        Ok(())
    })
}

/// Report service status to Windows SCM.
///
/// This function updates the service status with the Service Control Manager.
/// It's important to report status changes promptly to prevent SCM from
/// assuming the service has hung.
///
/// # Arguments
///
/// * `status_handle` - Handle to the service status
/// * `state` - Current service state (Running, Stopped, etc.)
/// * `controls_accepted` - Which control events the service will handle
/// * `checkpoint` - Progress indicator for long operations (0 when not pending)
/// * `wait_hint` - Estimated time for pending operation to complete
fn report_service_status(
    status_handle: &ServiceStatusHandle,
    state: ServiceState,
    controls_accepted: ServiceControlAccept,
    checkpoint: u32,
    wait_hint: Duration,
) -> Result<(), ServiceError> {
    let status = ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: state,
        controls_accepted,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint,
        wait_hint,
        process_id: None,
    };

    status_handle
        .set_service_status(status)
        .map_err(|e| ServiceError::StatusUpdate(e.to_string()))
}

/// Report that the service has stopped.
///
/// This is a convenience function for reporting the final stopped state
/// with an appropriate exit code.
///
/// # Arguments
///
/// * `status_handle` - Handle to the service status
/// * `exit_code` - Exit code indicating success (0) or failure (non-zero)
fn report_service_stopped(
    status_handle: &ServiceStatusHandle,
    exit_code: ServiceExitCode,
) -> Result<(), ServiceError> {
    let status = ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code,
        checkpoint: 0,
        wait_hint: Duration::ZERO,
        process_id: None,
    };

    status_handle
        .set_service_status(status)
        .map_err(|e| ServiceError::StatusUpdate(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_name_is_valid() {
        // Service name must not contain spaces or special characters
        assert!(!SERVICE_NAME.contains(' '));
        assert!(!SERVICE_NAME.contains('/'));
        assert!(!SERVICE_NAME.contains('\\'));
    }
}
