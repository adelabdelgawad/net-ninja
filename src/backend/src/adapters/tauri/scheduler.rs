use super::*;
use std::collections::BTreeMap;

// ===== Response Types =====

#[derive(Debug, Clone, serde::Serialize)]
pub struct SchedulerStatusResponse {
    /// "running" (a fresh scheduler lock is held), "idle" (no fresh lock), or
    /// "unknown" (database unavailable / fallback mode).
    pub status: String,
    #[serde(rename = "isRunning")]
    pub is_running: bool,
    #[serde(rename = "lockHolder")]
    pub lock_holder: Option<String>,
    #[serde(rename = "lastHeartbeat")]
    pub last_heartbeat: Option<String>,
    /// Map of job name -> RFC3339 timestamp of its last successful run
    /// (e.g. "quota_check", "speed_test", "cleanup").
    #[serde(rename = "lastRuns")]
    pub last_runs: BTreeMap<String, String>,
}

// ===== Scheduler Commands =====

/// Report scheduler liveness and recent activity, read from the shared database.
///
/// `isRunning` reflects whether a non-stale scheduler lock is currently held
/// (by the service or the desktop app); `lastRuns` surfaces the per-job
/// last-success markers written to `service_info` so an operator can see, e.g.,
/// that the quota check has not succeeded in days.
#[tauri::command]
pub async fn get_scheduler_status(
    state: State<'_, AppState>,
) -> Result<SchedulerStatusResponse, String> {
    let Some(pool) = state.pool.as_ref() else {
        return Ok(SchedulerStatusResponse {
            status: "unknown".to_string(),
            is_running: false,
            lock_holder: None,
            last_heartbeat: None,
            last_runs: BTreeMap::new(),
        });
    };

    let lock = crate::service::SchedulerLock::new(pool.clone());
    let is_running = lock.is_lock_held().await.unwrap_or(false);
    let info = lock.get_lock_holder().await.ok().flatten();

    let mut last_runs = BTreeMap::new();
    if let Ok(rows) = crate::repositories::ServiceInfoRepository::get_all(pool).await {
        use crate::repositories::service_info_repository::LAST_SUCCESS_PREFIX;
        for (key, value, _updated_at) in rows {
            if let Some(job) = key.strip_prefix(LAST_SUCCESS_PREFIX) {
                last_runs.insert(job.to_string(), value);
            }
        }
    }

    Ok(SchedulerStatusResponse {
        status: if is_running { "running" } else { "idle" }.to_string(),
        is_running,
        lock_holder: info.as_ref().map(|i| i.holder.clone()),
        last_heartbeat: info.as_ref().map(|i| i.heartbeat_at.to_rfc3339()),
        last_runs,
    })
}

/// Not supported: the scheduler lifecycle is owned by the host process — the
/// Windows service via the SCM (`netninja-service start`), the desktop via the
/// app lifecycle. Kept as a no-op so existing IPC callers don't error.
#[tauri::command]
pub async fn start_scheduler(_state: State<'_, AppState>) -> Result<(), String> {
    tracing::warn!(
        "start_scheduler is a no-op: control the service via SCM or restart the desktop app"
    );
    Ok(())
}

/// Not supported (see `start_scheduler`). Kept as a no-op for IPC compatibility.
#[tauri::command]
pub async fn stop_scheduler(_state: State<'_, AppState>) -> Result<(), String> {
    tracing::warn!(
        "stop_scheduler is a no-op: control the service via SCM or quit the desktop app"
    );
    Ok(())
}
