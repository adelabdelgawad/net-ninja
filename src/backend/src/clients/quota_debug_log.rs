//! Dedicated debug logger for quota scraping.
//!
//! Writes comprehensive diagnostic information to a separate log file
//! (`quota-debug-YYYY-MM-DD.log`) for troubleshooting CDP / element interaction
//! failures on remote machines.
//!
//! The log captures: timestamps, browser IDs, page URLs, DOM snapshots around
//! target selectors, element visibility states, screenshot paths on failure,
//! and full error chains.

use std::fmt::Write as FmtWrite;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;

use chrono::Local;

use super::WebDriverClient;
use crate::errors::AppResult;

/// Maximum debug log file size (5 MB) before rolling to a new suffix.
const MAX_DEBUG_LOG_SIZE: u64 = 5 * 1024 * 1024;

/// Get the directory for quota debug logs.
///
/// - Service mode (Windows): `%ProgramData%\NetNinja\logs`
/// - Desktop / other:        `<app_data>/netninja/logs`
fn debug_log_dir() -> PathBuf {
    #[cfg(windows)]
    {
        crate::config::paths::get_shared_data_path().join("logs")
    }
    #[cfg(not(windows))]
    {
        platform_dirs::AppDirs::new(Some("netninja"), false)
            .map(|d| d.data_dir.join("logs"))
            .unwrap_or_else(|| PathBuf::from("logs"))
    }
}

/// Resolve the debug log file path, rolling when the file exceeds the size limit.
fn resolve_debug_log_path() -> PathBuf {
    let dir = debug_log_dir();
    let _ = fs::create_dir_all(&dir);
    let date = Local::now().format("%Y-%m-%d").to_string();
    let base = dir.join(format!("quota-debug-{}.log", date));

    if !base.exists() || file_size(&base) < MAX_DEBUG_LOG_SIZE {
        return base;
    }

    let mut suffix = 1u32;
    loop {
        let p = dir.join(format!("quota-debug-{}-{}.log", date, suffix));
        if !p.exists() || file_size(&p) < MAX_DEBUG_LOG_SIZE {
            return p;
        }
        suffix += 1;
    }
}

fn file_size(p: &std::path::Path) -> u64 {
    fs::metadata(p).map(|m| m.len()).unwrap_or(0)
}

/// A per-scrape debug logger that accumulates entries in memory and flushes
/// them to the debug log file when the session ends (or on explicit flush).
pub struct QuotaDebugLog {
    line_name: String,
    browser_id: String,
    start: Instant,
    entries: Mutex<Vec<String>>,
}

impl QuotaDebugLog {
    /// Start a new debug log session for a quota scrape.
    pub fn new(line_name: &str, browser_id: &str) -> Self {
        let log = Self {
            line_name: line_name.to_string(),
            browser_id: browser_id.to_string(),
            start: Instant::now(),
            entries: Mutex::new(Vec::with_capacity(64)),
        };
        log.entry("SESSION_START", &format!(
            "Quota scrape session started | line='{}' browser='{}'",
            line_name, browser_id
        ));
        log
    }

    /// Elapsed milliseconds since session start.
    fn elapsed_ms(&self) -> u128 {
        self.start.elapsed().as_millis()
    }

    /// Append a timestamped entry.
    pub fn entry(&self, tag: &str, msg: &str) {
        let ts = Local::now().format("%H:%M:%S%.3f");
        let line = format!(
            "[{}] [+{}ms] [{}] [{}] [{}] {}",
            ts,
            self.elapsed_ms(),
            self.line_name,
            self.browser_id,
            tag,
            msg
        );
        if let Ok(mut entries) = self.entries.lock() {
            entries.push(line);
        }
    }

    /// Log a step start.
    pub fn step_start(&self, step: &str) {
        self.entry("STEP_START", &format!(">>> {}", step));
    }

    /// Log a step end with result.
    pub fn step_ok(&self, step: &str, detail: &str) {
        self.entry("STEP_OK", &format!("<<< {} | {}", step, detail));
    }

    /// Log a step failure.
    pub fn step_err(&self, step: &str, err: &str) {
        self.entry("STEP_FAIL", &format!("<<< {} | ERROR: {}", step, err));
    }

    /// Log navigation.
    pub fn nav(&self, url: &str) {
        self.entry("NAV", &format!("Navigating to: {}", url));
    }

    /// Log navigation complete with the actual URL the browser landed on.
    pub fn nav_done(&self, actual_url: &str) {
        self.entry("NAV_DONE", &format!("Landed on: {}", actual_url));
    }

    /// Log element interaction attempt.
    pub fn element_action(&self, action: &str, selector: &str) {
        self.entry("ELEMENT", &format!("{} selector='{}'", action, selector));
    }

    /// Log element interaction result.
    pub fn element_result(&self, action: &str, selector: &str, result: &str) {
        self.entry("ELEMENT_RESULT", &format!("{} selector='{}' => {}", action, selector, result));
    }

    /// Log a page diagnostic snapshot (URL, title, body preview).
    pub async fn snapshot(&self, driver: &WebDriverClient, context: &str) {
        let mut info = format!("PAGE_SNAPSHOT for '{}'", context);

        if let Ok(url) = driver.get_current_url().await {
            write!(info, " | url='{}'", url).ok();
        } else {
            write!(info, " | url=UNAVAILABLE").ok();
        }

        if let Ok(title) = driver.get_title().await {
            write!(info, " | title='{}'", title).ok();
        }

        // Get body text preview
        let body_script = r#"
            (function() {
                if (!document.body) return 'NO_BODY';
                var text = document.body.innerText || '';
                return text.substring(0, 300).replace(/\n/g, ' ');
            })()
        "#;
        if let Ok(val) = driver.execute_script(body_script).await {
            if let Some(s) = val.as_str() {
                write!(info, " | body_preview='{}'", s).ok();
            }
        }

        // Get document readyState
        if let Ok(val) = driver.execute_script("document.readyState").await {
            if let Some(s) = val.as_str() {
                write!(info, " | readyState='{}'", s).ok();
            }
        }

        self.entry("SNAPSHOT", &info);
    }

    /// Probe a specific selector and log whether it exists, is visible, its dimensions, etc.
    pub async fn probe_selector(&self, driver: &WebDriverClient, selector: &str) {
        let script = format!(
            r#"
            (function() {{
                var el = document.querySelector('{}');
                if (!el) return JSON.stringify({{found: false}});
                var rect = el.getBoundingClientRect();
                var style = window.getComputedStyle(el);
                return JSON.stringify({{
                    found: true,
                    tag: el.tagName,
                    id: el.id || null,
                    type: el.type || null,
                    disabled: el.disabled || false,
                    readOnly: el.readOnly || false,
                    display: style.display,
                    visibility: style.visibility,
                    opacity: style.opacity,
                    width: Math.round(rect.width),
                    height: Math.round(rect.height),
                    top: Math.round(rect.top),
                    left: Math.round(rect.left),
                    value: (el.value || '').substring(0, 50),
                    innerText: (el.innerText || '').substring(0, 50)
                }});
            }})()
            "#,
            selector.replace('\'', "\\'").replace('\\', "\\\\")
        );

        match driver.execute_script(&script).await {
            Ok(val) => {
                self.entry("PROBE", &format!("selector='{}' => {}", selector, val));
            }
            Err(e) => {
                self.entry("PROBE", &format!("selector='{}' => JS_ERROR: {}", selector, e));
            }
        }
    }

    /// Count how many elements match a selector.
    pub async fn count_selector(&self, driver: &WebDriverClient, selector: &str) {
        let script = format!(
            "document.querySelectorAll('{}').length",
            selector.replace('\'', "\\'")
        );
        match driver.execute_script(&script).await {
            Ok(val) => {
                self.entry("COUNT", &format!("selector='{}' => count={}", selector, val));
            }
            Err(e) => {
                self.entry("COUNT", &format!("selector='{}' => ERROR: {}", selector, e));
            }
        }
    }

    /// Save a screenshot and log its path.
    pub async fn screenshot(&self, driver: &WebDriverClient, label: &str) {
        let dir = debug_log_dir().join("screenshots");
        let _ = fs::create_dir_all(&dir);

        let filename = format!(
            "{}-{}-{}.png",
            self.line_name.replace(' ', "_"),
            label,
            Local::now().format("%H%M%S")
        );
        let path = dir.join(&filename);

        match driver.take_screenshot().await {
            Ok(bytes) => {
                if let Err(e) = fs::write(&path, &bytes) {
                    self.entry("SCREENSHOT", &format!("WRITE_FAILED for '{}': {}", label, e));
                } else {
                    self.entry("SCREENSHOT", &format!("Saved '{}' => {}", label, path.display()));
                }
            }
            Err(e) => {
                self.entry("SCREENSHOT", &format!("CAPTURE_FAILED for '{}': {}", label, e));
            }
        }
    }

    /// Flush all accumulated entries to the debug log file.
    pub fn flush(&self) {
        let entries = {
            let mut guard = match self.entries.lock() {
                Ok(g) => g,
                Err(_) => return,
            };
            std::mem::take(&mut *guard)
        };

        if entries.is_empty() {
            return;
        }

        let path = resolve_debug_log_path();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path);

        match file {
            Ok(mut f) => {
                for entry in &entries {
                    let _ = writeln!(f, "{}", entry);
                }
                let _ = writeln!(f, "");
            }
            Err(e) => {
                tracing::warn!("Failed to write quota debug log to {}: {}", path.display(), e);
            }
        }
    }

    /// End the session — writes a final marker and flushes.
    pub fn end(&self, status: &str) {
        self.entry("SESSION_END", &format!(
            "Quota scrape session ended | status='{}' duration={}ms",
            status,
            self.elapsed_ms()
        ));
        self.flush();
    }
}

impl Drop for QuotaDebugLog {
    fn drop(&mut self) {
        // Safety flush in case end() wasn't called (e.g. panic).
        self.flush();
    }
}

/// Run a full element diagnostic on all selectors used in WE quota scraping.
/// Call this when login or scraping fails to capture the full DOM state.
pub async fn diagnose_we_page(
    dlog: &QuotaDebugLog,
    driver: &WebDriverClient,
    context: &str,
    selectors: &[(&str, &str)],
) {
    dlog.entry("DIAGNOSE", &format!("=== Full page diagnostic: {} ===", context));
    dlog.snapshot(driver, context).await;

    for (_name, sel) in selectors {
        dlog.count_selector(driver, sel).await;
        dlog.probe_selector(driver, sel).await;
    }

    dlog.screenshot(driver, context).await;
    dlog.flush(); // Intermediate flush in case the process crashes
}

/// Helper to log the result of an AppResult and return it unchanged.
pub fn log_result<T>(dlog: &QuotaDebugLog, step: &str, result: &AppResult<T>) {
    match result {
        Ok(_) => dlog.step_ok(step, "OK"),
        Err(e) => dlog.step_err(step, &format!("{}", e)),
    }
}
