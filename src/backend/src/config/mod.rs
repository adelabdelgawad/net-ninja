use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::errors::AppError;

pub mod paths;
#[allow(unused_imports)]
pub use paths::*;

/// Get the platform-specific path for the SQLite database file
///
/// Returns the full path to netninja.db:
/// - Windows: %ProgramData%\NetNinja\netninja.db (shared between desktop app and service)
/// - Linux: ~/.local/share/netninja/netninja.db
/// - macOS: ~/Library/Application Support/netninja/netninja.db
pub fn get_sqlite_path() -> PathBuf {
    #[cfg(windows)]
    let app_dir = get_shared_data_path();

    #[cfg(not(windows))]
    let app_dir = platform_dirs::AppDirs::new(Some("netninja"), false)
        .expect("Failed to get platform directories")
        .data_dir;

    // Ensure directory exists
    fs::create_dir_all(&app_dir).unwrap_or_else(|e| {
        tracing::warn!("Failed to create data directory {:?}: {}", app_dir, e);
    });

    app_dir.join("netninja.db")
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
struct RawSettings {
    webdriver: Option<RawWebDriverSettings>,
    scheduler: Option<RawSchedulerSettings>,
    quota_check: Option<RawQuotaCheckSettings>,
    speed_test: Option<RawSpeedTestSettings>,
    cleanup: Option<RawCleanupSettings>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawWebDriverSettings {
    chrome_path: Option<String>,
    headless: Option<bool>,
    auto_install: Option<bool>,
}


#[derive(Debug, Clone, Deserialize)]
struct RawSchedulerSettings {
    enabled: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawQuotaCheckSettings {
    cron: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawSpeedTestSettings {
    cron: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawCleanupSettings {
    cron: Option<String>,
    retention_days: Option<i64>,
}

/// Runtime settings loaded from environment/config.toml
///
/// Note: Database connection settings are now stored in the TOML config
/// via AppConfig. This Settings struct holds operational settings like
/// cron schedules and webdriver configuration. SMTP configuration is
/// managed via the UI and stored in the database.
#[derive(Debug, Clone)]
pub struct Settings {
    pub webdriver: WebDriverSettings,
    pub scheduler: SchedulerSettings,
    pub quota_check: QuotaCheckSettings,
    pub speed_test: SpeedTestSettings,
    pub cleanup: CleanupSettings,
}

#[derive(Debug, Clone)]
pub struct WebDriverSettings {
    /// Optional custom Chrome path (auto-detected if None)
    pub chrome_path: Option<String>,
    /// Run browser in headless mode (default: true)
    pub headless: bool,
    /// Download Chrome automatically if not found (default: true)
    pub auto_install: bool,
}


#[derive(Debug, Clone)]
pub struct SchedulerSettings {
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct QuotaCheckSettings {
    pub cron: String,
}

#[derive(Debug, Clone)]
pub struct SpeedTestSettings {
    pub cron: String,
}

#[derive(Debug, Clone)]
pub struct CleanupSettings {
    pub cron: String,
    pub retention_days: i64,
}

impl Settings {
    /// Load settings from config file and environment variables
    pub fn load() -> Result<Self, AppError> {
        dotenvy::dotenv().ok();

        // Try to load from config file first
        let raw = Self::load_from_file().unwrap_or_default();

        // Convert to Settings, applying environment overrides
        Self::from_raw(raw)
    }

    /// Load settings for Tauri standalone mode
    /// Uses environment variables with defaults suitable for desktop app
    pub fn for_tauri() -> Result<Self, AppError> {
        dotenvy::dotenv().ok();

        Ok(Self {
            webdriver: WebDriverSettings {
                chrome_path: get_env_opt("CHROME_PATH"),
                headless: get_env_or("CHROME_HEADLESS", "true")
                    .parse()
                    .unwrap_or(true),
                auto_install: get_env_or("CHROME_AUTO_INSTALL", "true")
                    .parse()
                    .unwrap_or(true),
            },
            scheduler: SchedulerSettings {
                enabled: get_env_or("SCHEDULER_ENABLED", "true")
                    .parse()
                    .unwrap_or(true),
            },
            quota_check: QuotaCheckSettings {
                cron: get_env_or("QUOTA_CHECK_CRON", "0 0 6 * * *"),
            },
            speed_test: SpeedTestSettings {
                cron: get_env_or("SPEED_TEST_CRON", "0 0 */4 * * *"),
            },
            cleanup: CleanupSettings {
                cron: get_env_or("CLEANUP_CRON", "0 0 0 * * *"),
                retention_days: get_env_or("CLEANUP_RETENTION_DAYS", "90")
                    .parse()
                    .unwrap_or(90),
            },
        })
    }

    fn load_from_file() -> Option<RawSettings> {
        let config_path = Path::new("config.toml");
        if !config_path.exists() {
            return None;
        }

        let contents = fs::read_to_string(config_path).ok()?;
        toml::from_str(&contents).ok()
    }

    fn from_raw(raw: RawSettings) -> Result<Self, AppError> {
        Ok(Self {
            webdriver: WebDriverSettings {
                chrome_path: raw
                    .webdriver
                    .as_ref()
                    .and_then(|w| w.chrome_path.as_ref())
                    .cloned()
                    .or_else(|| get_env_opt("CHROME_PATH")),
                headless: raw
                    .webdriver
                    .as_ref()
                    .and_then(|w| w.headless)
                    .unwrap_or_else(|| get_env_or("CHROME_HEADLESS", "true").parse().unwrap_or(true)),
                auto_install: raw
                    .webdriver
                    .as_ref()
                    .and_then(|w| w.auto_install)
                    .unwrap_or_else(|| get_env_or("CHROME_AUTO_INSTALL", "true").parse().unwrap_or(true)),
            },
            scheduler: SchedulerSettings {
                enabled: raw
                    .scheduler
                    .as_ref()
                    .and_then(|s| s.enabled)
                    .unwrap_or_else(|| get_env_or("SCHEDULER_ENABLED", "true").parse().unwrap_or(true)),
            },
            quota_check: QuotaCheckSettings {
                cron: raw
                    .quota_check
                    .as_ref()
                    .and_then(|q| q.cron.as_ref())
                    .cloned()
                    .unwrap_or_else(|| get_env_or("QUOTA_CHECK_CRON", "0 0 6 * * *")),
            },
            speed_test: SpeedTestSettings {
                cron: raw
                    .speed_test
                    .as_ref()
                    .and_then(|s| s.cron.as_ref())
                    .cloned()
                    .unwrap_or_else(|| get_env_or("SPEED_TEST_CRON", "0 0 */4 * * *")),
            },
            cleanup: CleanupSettings {
                cron: raw
                    .cleanup
                    .as_ref()
                    .and_then(|c| c.cron.as_ref())
                    .cloned()
                    .unwrap_or_else(|| get_env_or("CLEANUP_CRON", "0 0 0 * * *")),
                retention_days: raw
                    .cleanup
                    .as_ref()
                    .and_then(|c| c.retention_days)
                    .unwrap_or_else(|| get_env_or("CLEANUP_RETENTION_DAYS", "90").parse().unwrap_or(90)),
            },
        })
    }
}

impl Default for RawSettings {
    fn default() -> Self {
        Self {
            webdriver: None,
            scheduler: None,
            quota_check: None,
            speed_test: None,
            cleanup: None,
        }
    }
}

fn get_env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn get_env_opt(key: &str) -> Option<String> {
    env::var(key).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_defaults() {
        let settings = Settings {
            webdriver: WebDriverSettings {
                chrome_path: None,
                headless: true,
                auto_install: true,
            },
            scheduler: SchedulerSettings { enabled: true },
            quota_check: QuotaCheckSettings {
                cron: "0 0 6 * * *".into(),
            },
            speed_test: SpeedTestSettings {
                cron: "0 0 */4 * * *".into(),
            },
            cleanup: CleanupSettings {
                cron: "0 0 0 * * *".into(),
                retention_days: 90,
            },
        };

        assert_eq!(settings.scheduler.enabled, true);
    }
}
