//! Platform-specific path utilities for NetNinja
//!
//! This module provides path resolution for shared data directories,
//! particularly for Windows Service mode where data must be accessible
//! by both SYSTEM (service) and user (desktop) processes.

use std::path::PathBuf;

/// Get the shared data path for NetNinja
///
/// On Windows, returns `%ProgramData%\NetNinja` using the Windows API
/// (SHGetKnownFolderPath) for reliable resolution even under SYSTEM account.
///
/// On other platforms, falls back to the platform-specific app data directory.
pub fn get_shared_data_path() -> PathBuf {
    #[cfg(windows)]
    {
        get_programdata_path()
    }

    #[cfg(not(windows))]
    {
        // On non-Windows platforms, use the standard app data directory
        platform_dirs::AppDirs::new(Some("netninja"), false)
            .expect("Failed to get platform directories")
            .data_dir
    }
}

/// Get the Windows ProgramData path for NetNinja
///
/// Uses SHGetKnownFolderPath(FOLDERID_ProgramData) for reliable resolution
/// that works under both user and SYSTEM contexts.
///
/// Returns: `C:\ProgramData\NetNinja` (typical)
#[cfg(windows)]
pub fn get_programdata_path() -> PathBuf {
    use windows::Win32::System::Com::CoTaskMemFree;
    use windows::Win32::UI::Shell::{FOLDERID_ProgramData, SHGetKnownFolderPath, KF_FLAG_DEFAULT};

    unsafe {
        // Call SHGetKnownFolderPath to get ProgramData folder
        // This works reliably under SYSTEM account, unlike environment variables
        // Note: In windows crate 0.58+, this returns Result<PWSTR> directly
        match SHGetKnownFolderPath(&FOLDERID_ProgramData, KF_FLAG_DEFAULT, None) {
            Ok(path_ptr) => {
                // Convert PWSTR to Rust string
                let path_str = path_ptr.to_string().unwrap_or_else(|_| {
                    // Fallback if conversion fails
                    String::from("C:\\ProgramData")
                });

                // Free the memory allocated by SHGetKnownFolderPath
                CoTaskMemFree(Some(path_ptr.as_ptr() as *const _));

                PathBuf::from(path_str).join("NetNinja")
            }
            Err(_) => {
                // Fallback to hardcoded path if API fails
                tracing::warn!("SHGetKnownFolderPath failed, using fallback path");
                PathBuf::from("C:\\ProgramData\\NetNinja")
            }
        }
    }
}

/// Get the SQLite database path for service mode
///
/// Returns: `%ProgramData%\NetNinja\netninja.db` on Windows
#[cfg(windows)]
pub fn get_service_sqlite_path() -> PathBuf {
    get_programdata_path().join("netninja.db")
}

/// Get the service log directory
///
/// Returns: `%ProgramData%\NetNinja\logs` on Windows
#[cfg(windows)]
pub fn get_service_log_path() -> PathBuf {
    get_programdata_path().join("logs")
}

/// Get the Chrome profiles directory for service mode
///
/// Returns: `%ProgramData%\NetNinja\chrome-profiles` on Windows
#[cfg(windows)]
pub fn get_service_chrome_profiles_path() -> PathBuf {
    get_programdata_path().join("chrome-profiles")
}

/// Get the configuration file path for service mode
///
/// Returns: `%ProgramData%\NetNinja\.env` on Windows
#[cfg(windows)]
pub fn get_service_config_path() -> PathBuf {
    get_programdata_path().join(".env")
}

/// Detect if the current process is running as a Windows Service
///
/// This check verifies we're actually running in Session 0 under SCM control,
/// not just that a flag is set. The `NETNINJA_SERVICE_MODE` environment
/// variable is only used as a testing override.
///
/// Returns true ONLY when:
/// 1. The `service` feature is enabled, AND
/// 2. Either:
///    a. Process is running in Session 0 (SCM-launched service), OR
///    b. NETNINJA_SERVICE_MODE=1 is set (testing override)
#[cfg(all(windows, feature = "service"))]
pub fn is_service_mode() -> bool {
    // Check for testing override first
    if std::env::var("NETNINJA_SERVICE_MODE").map(|v| v == "1").unwrap_or(false) {
        tracing::debug!("Service mode enabled via NETNINJA_SERVICE_MODE override");
        return true;
    }

    // Check if we're running in Session 0 (where Windows services run)
    is_session_zero()
}

/// Always returns false when service feature is not enabled
#[cfg(not(all(windows, feature = "service")))]
pub fn is_service_mode() -> bool {
    false
}

/// Check if the current process is running in Session 0
///
/// Windows services run in Session 0, isolated from user sessions.
/// This is a reliable way to detect service context.
#[cfg(all(windows, feature = "service"))]
fn is_session_zero() -> bool {
    use windows::Win32::System::Threading::{GetCurrentProcess, GetCurrentProcessId};

    // Import the kernel32 function for session ID lookup
    #[link(name = "kernel32")]
    extern "system" {
        fn ProcessIdToSessionId(dwProcessId: u32, pSessionId: *mut u32) -> i32;
    }

    unsafe {
        let _process = GetCurrentProcess();
        let process_id = GetCurrentProcessId();
        let mut session_id: u32 = 0;

        if ProcessIdToSessionId(process_id, &mut session_id) != 0 {
            session_id == 0
        } else {
            // If we can't determine session, assume not service mode
            tracing::warn!("Could not determine session ID, assuming not service mode");
            false
        }
    }
}

/// Clear the logs directory so only the current run's logs remain.
///
/// Removes all files and subdirectories (e.g. `screenshots/`) from
/// `%ProgramData%\NetNinja\logs`. Called at startup before logging is
/// initialised so that each run starts with a clean slate.
#[cfg(windows)]
pub fn clear_logs_dir() {
    let log_dir = get_service_log_path();

    let entries = match std::fs::read_dir(&log_dir) {
        Ok(entries) => entries,
        Err(_) => return, // directory doesn't exist yet — nothing to clean
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let _ = std::fs::remove_dir_all(&path);
        } else {
            let _ = std::fs::remove_file(&path);
        }
    }
}

/// No-op on non-Windows platforms.
#[cfg(not(windows))]
pub fn clear_logs_dir() {}

/// Ensure the shared data directory exists with appropriate permissions
///
/// For Windows service mode, this creates:
/// - %ProgramData%\NetNinja
/// - %ProgramData%\NetNinja\logs
/// - %ProgramData%\NetNinja\chrome-profiles
///
/// On Windows, grants BUILTIN\Users Modify permissions with inheritance
/// so the desktop app (running as a regular user) can write to the database.
pub fn ensure_shared_directories() -> std::io::Result<()> {
    let base_path = get_shared_data_path();
    std::fs::create_dir_all(&base_path)?;

    #[cfg(windows)]
    {
        std::fs::create_dir_all(get_service_log_path())?;
        std::fs::create_dir_all(get_service_chrome_profiles_path())?;
        grant_users_write_access(&base_path);
    }

    Ok(())
}

/// Grant BUILTIN\Users Modify permissions on the directory with inheritance.
///
/// This allows the desktop Tauri app (running as a regular user) to read/write
/// the SQLite database that lives in %ProgramData%\NetNinja. Without this,
/// files created by the SYSTEM service inherit ProgramData's default ACLs
/// which only grant Users Read+Execute access.
#[cfg(windows)]
fn grant_users_write_access(path: &std::path::Path) {
    match std::process::Command::new("icacls")
        .arg(path)
        .args(["/grant", "Users:(OI)(CI)M", "/T", "/Q"])
        .output()
    {
        Ok(output) => {
            if !output.status.success() {
                tracing::warn!(
                    "icacls failed to set permissions on {}: {}",
                    path.display(),
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
        Err(e) => {
            tracing::warn!("Failed to run icacls on {}: {}", path.display(), e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_shared_data_path() {
        let path = get_shared_data_path();
        assert!(path.to_string_lossy().contains("NetNinja") || path.to_string_lossy().contains("netninja"));
    }

    #[cfg(windows)]
    #[test]
    fn test_get_programdata_path() {
        let path = get_programdata_path();
        assert!(path.to_string_lossy().contains("ProgramData"));
        assert!(path.to_string_lossy().ends_with("NetNinja"));
    }

    #[test]
    fn test_is_service_mode_without_feature() {
        // When service feature is not enabled, should always return false
        #[cfg(not(all(windows, feature = "service")))]
        {
            assert!(!is_service_mode());
        }
    }
}
