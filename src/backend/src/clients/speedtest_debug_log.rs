//! Dedicated debug logger for speed test executions.
//!
//! Writes diagnostic information to a per-execution file at:
//! `%ProgramData%\NetNinja\logs\speedtest\{date}\{process_id}.txt`

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;

use chrono::Local;

/// Get the base logs directory for NetNinja.
fn log_base_dir() -> PathBuf {
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

/// A per-execution debug logger for speed tests.
///
/// Accumulates entries in memory and flushes them to a `.txt` file under
/// `logs/speedtest/{date}/{process_id}.txt` when the session ends or on drop.
pub struct SpeedTestDebugLog {
    line_name: String,
    log_path: PathBuf,
    start: Instant,
    entries: Mutex<Vec<String>>,
}

impl SpeedTestDebugLog {
    /// Create a new debug log session for a speed test execution.
    pub fn new(line_name: &str, process_id: &str) -> Self {
        let base = log_base_dir();
        let date = Local::now().format("%Y-%m-%d").to_string();

        let log_path = base
            .join("speedtest")
            .join(&date)
            .join(format!("{}.txt", process_id));
        let _ = fs::create_dir_all(log_path.parent().unwrap());

        let log = Self {
            line_name: line_name.to_string(),
            log_path,
            start: Instant::now(),
            entries: Mutex::new(Vec::with_capacity(32)),
        };
        log.entry(
            "SESSION_START",
            &format!(
                "Speed test session started | line='{}' process_id='{}'",
                line_name, process_id
            ),
        );
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
            "[{}] [+{}ms] [{}] [{}] {}",
            ts,
            self.elapsed_ms(),
            self.line_name,
            tag,
            msg
        );
        if let Ok(mut entries) = self.entries.lock() {
            entries.push(line);
        }
    }

    /// Flush all accumulated entries to the log file.
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

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path);

        match file {
            Ok(mut f) => {
                for entry in &entries {
                    let _ = writeln!(f, "{}", entry);
                }
                let _ = writeln!(f, "");
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to write speedtest debug log to {}: {}",
                    self.log_path.display(),
                    e
                );
            }
        }
    }

    /// End the session — writes a final marker and flushes.
    pub fn end(&self, status: &str) {
        self.entry(
            "SESSION_END",
            &format!(
                "Speed test session ended | status='{}' duration={}ms",
                status,
                self.elapsed_ms()
            ),
        );
        self.flush();
    }
}

impl Drop for SpeedTestDebugLog {
    fn drop(&mut self) {
        // Safety flush in case end() wasn't called (e.g. panic).
        self.flush();
    }
}
