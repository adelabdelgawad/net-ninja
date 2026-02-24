//! Chrome browser detection and auto-installation module.
//!
//! This module provides functionality to:
//! 1. Detect existing Chrome installations on the system
//! 2. Auto-download "Chrome for Testing" if no Chrome is found
//! 3. Support cross-platform (Linux, macOS, Windows)

use std::fs::{self, File};
use std::io::{self, BufReader};
use std::path::PathBuf;

use crate::config::WebDriverSettings;
use crate::errors::{AppError, AppResult};

/// Chrome for Testing download API endpoint
const CHROME_FOR_TESTING_API: &str =
    "https://googlechromelabs.github.io/chrome-for-testing/last-known-good-versions-with-downloads.json";

/// Ensures Chrome is available, downloading it if necessary.
///
/// Returns the path to the Chrome executable.
pub fn ensure_chrome_available(settings: &WebDriverSettings) -> AppResult<PathBuf> {
    // 1. Check user-specified path first
    if let Some(ref custom_path) = settings.chrome_path {
        let path = PathBuf::from(custom_path);
        if path.exists() {
            tracing::info!("Using user-specified Chrome: {:?}", path);
            return Ok(path);
        }
        return Err(AppError::WebDriver(format!(
            "Specified Chrome path does not exist: {}",
            custom_path
        )));
    }

    // 2. Check for system-installed Chrome
    if let Some(path) = find_system_chrome() {
        tracing::info!("Using system Chrome: {:?}", path);
        return Ok(path);
    }

    // 3. Check for previously downloaded Chrome
    if let Some(path) = find_downloaded_chrome() {
        tracing::info!("Using downloaded Chrome: {:?}", path);
        return Ok(path);
    }

    // 4. Auto-install if enabled
    if settings.auto_install {
        tracing::info!("No Chrome found, attempting auto-download...");
        return download_chrome();
    }

    Err(AppError::WebDriver(
        "Chrome not found. Install Chrome or enable auto_install.".to_string(),
    ))
}

/// Find Chrome installed on the system (platform-specific paths).
fn find_system_chrome() -> Option<PathBuf> {
    let candidates = get_system_chrome_paths();

    for path in candidates {
        if path.exists() {
            // Verify it's executable (on Unix)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(metadata) = path.metadata() {
                    if metadata.permissions().mode() & 0o111 != 0 {
                        return Some(path);
                    }
                }
            }
            #[cfg(windows)]
            {
                return Some(path);
            }
        }
    }

    None
}

/// Get platform-specific system Chrome paths.
fn get_system_chrome_paths() -> Vec<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        vec![
            PathBuf::from("/usr/bin/google-chrome"),
            PathBuf::from("/usr/bin/google-chrome-stable"),
            PathBuf::from("/usr/bin/chromium-browser"),
            PathBuf::from("/usr/bin/chromium"),
            PathBuf::from("/snap/bin/chromium"),
            PathBuf::from("/opt/google/chrome/chrome"),
        ]
    }

    #[cfg(target_os = "macos")]
    {
        vec![
            PathBuf::from("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"),
            PathBuf::from("/Applications/Chromium.app/Contents/MacOS/Chromium"),
        ]
    }

    #[cfg(target_os = "windows")]
    {
        use std::env;

        let mut paths = Vec::new();

        // Chrome paths
        if let Ok(program_files) = env::var("PROGRAMFILES") {
            paths.push(PathBuf::from(&program_files).join("Google\\Chrome\\Application\\chrome.exe"));
        }
        if let Ok(program_files_x86) = env::var("PROGRAMFILES(X86)") {
            paths.push(PathBuf::from(&program_files_x86).join("Google\\Chrome\\Application\\chrome.exe"));
        }

        // Local AppData (user install - not available under SYSTEM account)
        if let Ok(local_app_data) = env::var("LOCALAPPDATA") {
            paths.push(PathBuf::from(&local_app_data).join("Google\\Chrome\\Application\\chrome.exe"));
        }

        paths
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        vec![]
    }
}

/// Find previously downloaded Chrome in our app data directory.
fn find_downloaded_chrome() -> Option<PathBuf> {
    let chrome_dir = get_chrome_download_dir()?;
    let chrome_exe = get_chrome_exe_in_dir(&chrome_dir);

    if chrome_exe.exists() {
        Some(chrome_exe)
    } else {
        None
    }
}

/// Get the directory where we store downloaded Chrome.
fn get_chrome_download_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("netninja").join("chrome"))
}

/// Get the path to the Chrome executable within our download directory.
fn get_chrome_exe_in_dir(base_dir: &PathBuf) -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        base_dir.join("chrome-linux64").join("chrome")
    }

    #[cfg(target_os = "macos")]
    {
        base_dir
            .join("chrome-mac-x64")
            .join("Google Chrome for Testing.app")
            .join("Contents")
            .join("MacOS")
            .join("Google Chrome for Testing")
    }

    #[cfg(target_os = "windows")]
    {
        base_dir.join("chrome-win64").join("chrome.exe")
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        base_dir.join("chrome")
    }
}

/// Download and install Chrome for Testing.
fn download_chrome() -> AppResult<PathBuf> {
    let download_dir = get_chrome_download_dir().ok_or_else(|| {
        AppError::WebDriver("Could not determine app data directory".to_string())
    })?;

    // Ensure download directory exists
    fs::create_dir_all(&download_dir).map_err(|e| {
        AppError::WebDriver(format!("Failed to create Chrome directory: {}", e))
    })?;

    // Fetch the download URL from Chrome for Testing API
    let download_url = get_chrome_download_url()?;
    tracing::info!("Downloading Chrome from: {}", download_url);

    // Download the zip file
    let zip_path = download_dir.join("chrome.zip");
    download_file(&download_url, &zip_path)?;

    // Extract the zip
    tracing::info!("Extracting Chrome...");
    extract_zip(&zip_path, &download_dir)?;

    // Clean up zip file
    let _ = fs::remove_file(&zip_path);

    // Make executable on Unix
    #[cfg(unix)]
    {
        let chrome_exe = get_chrome_exe_in_dir(&download_dir);
        if chrome_exe.exists() {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&chrome_exe)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&chrome_exe, perms)?;
        }
    }

    let chrome_path = get_chrome_exe_in_dir(&download_dir);
    if chrome_path.exists() {
        tracing::info!("Chrome installed successfully: {:?}", chrome_path);
        Ok(chrome_path)
    } else {
        Err(AppError::WebDriver(
            "Chrome download succeeded but executable not found".to_string(),
        ))
    }
}

/// Get the download URL for the current platform from Chrome for Testing API.
fn get_chrome_download_url() -> AppResult<String> {
    // Use blocking HTTP request since we're in sync context
    let response = reqwest::blocking::get(CHROME_FOR_TESTING_API).map_err(|e| {
        AppError::WebDriver(format!("Failed to fetch Chrome download info: {}", e))
    })?;

    let json: serde_json::Value = response.json().map_err(|e| {
        AppError::WebDriver(format!("Failed to parse Chrome download info: {}", e))
    })?;

    let platform = get_platform_name();

    // Navigate: channels.Stable.downloads.chrome[platform].url
    let url = json
        .get("channels")
        .and_then(|c: &serde_json::Value| c.get("Stable"))
        .and_then(|s: &serde_json::Value| s.get("downloads"))
        .and_then(|d: &serde_json::Value| d.get("chrome"))
        .and_then(|c: &serde_json::Value| c.as_array())
        .and_then(|arr: &Vec<serde_json::Value>| {
            arr.iter()
                .find(|item: &&serde_json::Value| {
                    item.get("platform").and_then(|p: &serde_json::Value| p.as_str()) == Some(platform)
                })
        })
        .and_then(|item: &serde_json::Value| item.get("url"))
        .and_then(|u: &serde_json::Value| u.as_str())
        .ok_or_else(|| {
            AppError::WebDriver(format!("No Chrome download available for platform: {}", platform))
        })?;

    Ok(url.to_string())
}

/// Get the platform name for Chrome for Testing API.
fn get_platform_name() -> &'static str {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        "linux64"
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        "mac-x64"
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "mac-arm64"
    }
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        "win64"
    }
    #[cfg(all(target_os = "windows", target_arch = "x86"))]
    {
        "win32"
    }
    #[cfg(not(any(
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "windows", target_arch = "x86_64"),
        all(target_os = "windows", target_arch = "x86"),
    )))]
    {
        "linux64" // fallback
    }
}

/// Download a file from URL to the specified path.
///
/// Streams the response body directly to disk to avoid buffering
/// the entire file (~200 MB for Chrome) into memory.
fn download_file(url: &str, dest: &PathBuf) -> AppResult<()> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| AppError::WebDriver(format!("Failed to create HTTP client: {}", e)))?;

    let mut response = client.get(url).send().map_err(|e| {
        AppError::WebDriver(format!("Failed to download Chrome: {}", e))
    })?;

    if !response.status().is_success() {
        return Err(AppError::WebDriver(format!(
            "Chrome download failed with status: {}",
            response.status()
        )));
    }

    let mut file = File::create(dest).map_err(|e| {
        AppError::WebDriver(format!("Failed to create download file: {}", e))
    })?;

    // Stream directly to disk instead of buffering entire response in memory
    response.copy_to(&mut file).map_err(|e| {
        AppError::WebDriver(format!("Failed to write Chrome download to disk: {}", e))
    })?;

    Ok(())
}

/// Extract a zip file to the specified directory.
fn extract_zip(zip_path: &PathBuf, dest_dir: &PathBuf) -> AppResult<()> {
    let file = File::open(zip_path).map_err(|e| {
        AppError::WebDriver(format!("Failed to open zip file: {}", e))
    })?;

    let mut archive = zip::ZipArchive::new(BufReader::new(file)).map_err(|e| {
        AppError::WebDriver(format!("Failed to read zip archive: {}", e))
    })?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| {
            AppError::WebDriver(format!("Failed to read zip entry: {}", e))
        })?;

        let outpath = match file.enclosed_name() {
            Some(path) => dest_dir.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath).ok();
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent).ok();
            }
            let mut outfile = File::create(&outpath).map_err(|e| {
                AppError::WebDriver(format!("Failed to create extracted file: {}", e))
            })?;
            io::copy(&mut file, &mut outfile).map_err(|e| {
                AppError::WebDriver(format!("Failed to extract file: {}", e))
            })?;
        }

        // Set permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).ok();
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_system_chrome_paths_not_empty() {
        let paths = get_system_chrome_paths();
        // Should have at least one candidate path on supported platforms
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        assert!(!paths.is_empty());
    }

    #[test]
    fn test_get_platform_name() {
        let platform = get_platform_name();
        // Should return a non-empty string
        assert!(!platform.is_empty());
    }

    #[test]
    fn test_chrome_download_dir() {
        let dir = get_chrome_download_dir();
        // Should be able to determine app data dir
        assert!(dir.is_some());
    }
}
