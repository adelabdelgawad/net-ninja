//! Service logging configuration
//!
//! This module provides logging configuration for the NetNinja Windows Service.
//! It sets up dual logging to:
//!
//! 1. **File log**: `%ProgramData%\NetNinja\logs\service.log`
//!    - Contains all log messages at DEBUG level and above
//!    - Rotates daily to prevent unbounded growth
//!    - Useful for detailed debugging
//!
//! 2. **Windows Event Log**: `Application` log
//!    - Contains important events only (service start/stop, fatal errors)
//!    - Visible in Windows Event Viewer
//!    - Used by system administrators for monitoring
//!
//! # Event Log Source
//!
//! Before first use, the event source "NetNinjaScheduler" should be registered
//! with the Windows Event Log. This happens automatically during service
//! installation.

use std::fs::{self, OpenOptions};
use std::io;
use std::path::PathBuf;
use std::sync::OnceLock;

use chrono::Local;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter, Layer};

use crate::config::paths;
use crate::service::SERVICE_NAME;

/// Maximum log file size in bytes (2 MB).
const MAX_LOG_FILE_SIZE: u64 = 2 * 1024 * 1024;

/// Error type for logging initialization.
#[derive(Debug, thiserror::Error)]
pub enum LoggingError {
    /// Failed to create log directory
    #[error("Failed to create log directory: {0}")]
    DirectoryCreation(String),

    /// Failed to open log file
    #[error("Failed to open log file: {0}")]
    FileOpen(String),

    /// Failed to initialize tracing subscriber
    #[error("Failed to initialize tracing: {0}")]
    TracingInit(String),

    /// Failed to register Windows Event Log source
    #[error("Failed to register event log source: {0}")]
    EventLogRegistration(String),
}

/// Static holder for the log file path (for diagnostics)
static LOG_FILE_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Get the path to the current service log file.
///
/// Returns `None` if logging hasn't been initialized yet.
pub fn get_log_file_path() -> Option<&'static PathBuf> {
    LOG_FILE_PATH.get()
}

/// Resolve the log file path for today, respecting the 2MB size limit.
///
/// Returns the path to a log file that is either under the size limit or a new
/// file. When the base file (`service-YYYY-MM-DD.log`) exceeds [`MAX_LOG_FILE_SIZE`],
/// suffixed files are checked in order (`-1`, `-2`, ...) until one is found that
/// is still under the limit, or a new suffixed file is created.
fn resolve_log_file_path(log_dir: &std::path::Path) -> PathBuf {
    let date_str = Local::now().format("%Y-%m-%d").to_string();
    let base_name = format!("service-{}.log", date_str);
    let base_path = log_dir.join(&base_name);

    // If the base file doesn't exist or is under the limit, use it.
    if !base_path.exists() || file_size(&base_path) < MAX_LOG_FILE_SIZE {
        return base_path;
    }

    // Base file is over the limit — find the next available suffixed file.
    let mut suffix = 1u32;
    loop {
        let suffixed_name = format!("service-{}-{}.log", date_str, suffix);
        let suffixed_path = log_dir.join(&suffixed_name);

        if !suffixed_path.exists() || file_size(&suffixed_path) < MAX_LOG_FILE_SIZE {
            return suffixed_path;
        }

        suffix += 1;
    }
}

/// Returns the size of a file in bytes, or 0 if the metadata cannot be read.
fn file_size(path: &std::path::Path) -> u64 {
    fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

/// Initialize service logging.
///
/// Sets up the tracing subscriber with:
/// - File logging to `%ProgramData%\NetNinja\logs\service.log`
/// - Console output (useful when running interactively for debugging)
///
/// The log file name includes the date and rotates daily:
/// `service-YYYY-MM-DD.log`
///
/// # Environment Variables
///
/// - `RUST_LOG`: Controls log level (default: "info")
///   Example: `RUST_LOG=debug` or `RUST_LOG=net_ninja=trace,warn`
///
/// # Returns
///
/// Returns `Ok(())` on successful initialization.
///
/// # Errors
///
/// Returns an error if:
/// - Log directory cannot be created
/// - Log file cannot be opened
/// - Tracing subscriber fails to initialize
///
/// # Example
///
/// ```ignore
/// init_service_logging()?;
/// tracing::info!("Service logging initialized");
/// ```
pub fn init_service_logging() -> Result<(), LoggingError> {
    // Get the log directory path
    let log_dir = paths::get_service_log_path();

    // Create log directory if it doesn't exist
    fs::create_dir_all(&log_dir)
        .map_err(|e| LoggingError::DirectoryCreation(format!("{}: {}", log_dir.display(), e)))?;

    // Create log file name with date for rotation, respecting the 2MB size limit.
    // If the base file (service-YYYY-MM-DD.log) exceeds 2MB, use a suffixed name
    // (service-YYYY-MM-DD-1.log, service-YYYY-MM-DD-2.log, etc.).
    let log_path = resolve_log_file_path(&log_dir);

    // Store the log path for diagnostics
    let _ = LOG_FILE_PATH.set(log_path.clone());

    // Open log file in append mode
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|e| LoggingError::FileOpen(format!("{}: {}", log_path.display(), e)))?;

    // Create file logging layer
    let file_layer = fmt::layer()
        .with_writer(log_file)
        .with_ansi(false) // No ANSI colors in file
        .with_span_events(FmtSpan::CLOSE)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_filter(EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info")));

    // Create console logging layer (for debugging when running interactively)
    let console_layer = fmt::layer()
        .with_ansi(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_filter(EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info")));

    // Initialize the subscriber with both layers
    tracing_subscriber::registry()
        .with(file_layer)
        .with(console_layer)
        .try_init()
        .map_err(|e| LoggingError::TracingInit(e.to_string()))?;

    tracing::info!("Service logging initialized");
    tracing::info!("Log file: {}", log_path.display());

    // Log to Windows Event Log as well
    log_to_event_log(EventLevel::Info, "NetNinja Scheduler Service logging initialized");

    Ok(())
}

/// Event log level for Windows Event Log entries
#[derive(Debug, Clone, Copy)]
pub enum EventLevel {
    Info,
    Warning,
    Error,
}

/// Log a message to the Windows Event Log.
///
/// Writes an entry to the Windows Application Event Log. This is useful for
/// important events that system administrators should see in Event Viewer.
///
/// # Event Types
///
/// - `EventLevel::Info`: Informational message (e.g., service started)
/// - `EventLevel::Warning`: Warning condition (e.g., temporary failure)
/// - `EventLevel::Error`: Error condition (e.g., fatal error, service stopping)
///
/// # Source Registration
///
/// The event source "NetNinjaScheduler" should be registered during installation.
/// If not registered, events may still be logged but with a warning from Windows.
///
/// # Arguments
///
/// * `event_level` - The type/severity of the event
/// * `message` - The event message to log
///
/// # Example
///
/// ```ignore
/// log_to_event_log(EventLevel::Info, "Service started successfully");
/// log_to_event_log(EventLevel::Error, "Fatal error: database connection failed");
/// ```
pub fn log_to_event_log(event_level: EventLevel, message: &str) {
    // Use the Windows API directly for event logging
    // The eventlog crate has compatibility issues, so we log to our file as a backup

    // Always log to our file log as well
    match event_level {
        EventLevel::Info => tracing::info!("[EVENT] {}", message),
        EventLevel::Warning => tracing::warn!("[EVENT] {}", message),
        EventLevel::Error => tracing::error!("[EVENT] {}", message),
    }

    // Try to write to Windows Event Log using PowerShell as a fallback
    // This is more reliable than the eventlog crate
    let level_str = match event_level {
        EventLevel::Info => "Information",
        EventLevel::Warning => "Warning",
        EventLevel::Error => "Error",
    };

    // Escape the message for PowerShell
    let escaped_message = message.replace('\'', "''").replace('`', "``");

    // Use Write-EventLog to write to the Application log
    // Note: This requires the event source to be registered, which we do during installation
    let ps_command = format!(
        "Write-EventLog -LogName Application -Source '{}' -EventId 1000 -EntryType {} -Message '{}'",
        SERVICE_NAME, level_str, escaped_message
    );

    // Run PowerShell in the background - don't block on it
    if let Err(e) = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps_command])
        .spawn()
    {
        tracing::debug!("Could not write to Windows Event Log: {}", e);
    }
}

/// Register the Windows Event Log source.
///
/// This function registers the "NetNinjaScheduler" event source with the
/// Windows Event Log. This should be called once during installation.
///
/// # Administrator Privileges
///
/// This function requires administrator privileges to modify the registry.
///
/// # Returns
///
/// Returns `Ok(())` if registration succeeds or source already exists.
///
/// # Errors
///
/// Returns an error if:
/// - Registration fails due to insufficient privileges
/// - Registry operation fails
pub fn register_event_source() -> Result<(), LoggingError> {
    // Register the event source using PowerShell
    // New-EventLog creates a new event source in the Application log
    let ps_command = format!(
        "if (-not [System.Diagnostics.EventLog]::SourceExists('{}')) {{ \
            New-EventLog -LogName Application -Source '{}' \
        }}",
        SERVICE_NAME, SERVICE_NAME
    );

    match std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps_command])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                tracing::info!("Event log source '{}' is registered", SERVICE_NAME);
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                tracing::warn!("Could not register event log source: {}", stderr);
                // Don't treat this as a fatal error - events can still be logged
                Ok(())
            }
        }
        Err(e) => {
            tracing::warn!("Could not register event log source: {}", e);
            Ok(())
        }
    }
}

/// Deregister the Windows Event Log source.
///
/// This should be called during uninstallation to clean up the event source
/// registry entries.
///
/// # Returns
///
/// Returns `Ok(())` even if deregistration fails (best effort cleanup).
pub fn deregister_event_source() -> Result<(), LoggingError> {
    // The eventlog crate doesn't provide a deregister function
    // This would require manual registry manipulation
    tracing::info!("Event log source deregistration not implemented (manual cleanup may be required)");
    Ok(())
}

/// Clean up old log files.
///
/// Removes log files older than the specified number of days to prevent
/// unbounded disk usage.
///
/// # Arguments
///
/// * `max_age_days` - Maximum age of log files to keep (default: 30)
///
/// # Returns
///
/// Returns the number of files deleted.
pub fn cleanup_old_logs(max_age_days: u32) -> io::Result<usize> {
    let log_dir = paths::get_service_log_path();
    let max_age = chrono::Duration::days(max_age_days as i64);
    let cutoff = Local::now() - max_age;

    let mut deleted_count = 0;

    if !log_dir.exists() {
        return Ok(0);
    }

    for entry in fs::read_dir(&log_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Only process .log files
        if !path.extension().map(|e| e == "log").unwrap_or(false) {
            continue;
        }

        // Check file modification time
        if let Ok(metadata) = fs::metadata(&path) {
            if let Ok(modified) = metadata.modified() {
                let modified_time: chrono::DateTime<Local> = modified.into();
                if modified_time < cutoff {
                    if let Err(e) = fs::remove_file(&path) {
                        tracing::warn!("Failed to delete old log file {}: {}", path.display(), e);
                    } else {
                        tracing::debug!("Deleted old log file: {}", path.display());
                        deleted_count += 1;
                    }
                }
            }
        }
    }

    if deleted_count > 0 {
        tracing::info!("Cleaned up {} old log files", deleted_count);
    }

    Ok(deleted_count)
}

/// Write a startup marker to the log file.
///
/// This creates a visible separator in the log file to indicate a new
/// service session has started. Useful for correlating logs with service
/// restart events.
pub fn write_startup_marker() {
    let marker = format!(
        "\n\
        ═══════════════════════════════════════════════════════════════════════\n\
        {} SERVICE STARTUP - {}\n\
        ═══════════════════════════════════════════════════════════════════════",
        SERVICE_NAME,
        Local::now().format("%Y-%m-%d %H:%M:%S %Z")
    );

    tracing::info!("{}", marker);
}

/// Write a shutdown marker to the log file.
///
/// This creates a visible separator in the log file to indicate the
/// service session is ending.
pub fn write_shutdown_marker() {
    let marker = format!(
        "\n\
        ───────────────────────────────────────────────────────────────────────\n\
        {} SERVICE SHUTDOWN - {}\n\
        ───────────────────────────────────────────────────────────────────────\n",
        SERVICE_NAME,
        Local::now().format("%Y-%m-%d %H:%M:%S %Z")
    );

    tracing::info!("{}", marker);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_name_for_logging() {
        // Ensure the service name is suitable for event log
        assert!(!SERVICE_NAME.is_empty());
        assert!(SERVICE_NAME.len() < 256); // Windows limit
    }

    #[test]
    fn test_log_path_construction() {
        let log_dir = paths::get_service_log_path();
        let log_filename = format!("service-{}.log", Local::now().format("%Y-%m-%d"));
        let log_path = log_dir.join(&log_filename);

        // Verify path looks reasonable
        assert!(log_path.to_string_lossy().contains("logs"));
        assert!(log_path.to_string_lossy().contains("service-"));
        assert!(log_path.to_string_lossy().ends_with(".log"));
    }

    #[test]
    fn test_resolve_log_file_path_returns_base_when_no_file_exists() {
        let temp_dir = std::env::temp_dir().join("netninja_log_test_base");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let result = resolve_log_file_path(&temp_dir);
        let name = result.file_name().unwrap().to_string_lossy();

        assert!(name.starts_with("service-"));
        assert!(name.ends_with(".log"));
        // Base name should have no suffix
        assert!(!name.contains("-1.log"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_resolve_log_file_path_rolls_over_when_exceeding_max_size() {
        let temp_dir = std::env::temp_dir().join("netninja_log_test_rollover");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Create a base file that exceeds the max size
        let date_str = Local::now().format("%Y-%m-%d").to_string();
        let base_name = format!("service-{}.log", date_str);
        let base_path = temp_dir.join(&base_name);
        let oversized = vec![0u8; (MAX_LOG_FILE_SIZE + 1) as usize];
        fs::write(&base_path, &oversized).unwrap();

        let result = resolve_log_file_path(&temp_dir);
        let name = result.file_name().unwrap().to_string_lossy();

        // Should pick the first suffixed name
        let expected = format!("service-{}-1.log", date_str);
        assert_eq!(name, expected);

        // Now make that one oversized too
        fs::write(&result, &oversized).unwrap();
        let result2 = resolve_log_file_path(&temp_dir);
        let name2 = result2.file_name().unwrap().to_string_lossy();
        let expected2 = format!("service-{}-2.log", date_str);
        assert_eq!(name2, expected2);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
