//! Task Scheduler Job
//!
//! This job runs every minute and checks for scheduled tasks that are due to run.
//!
//! Rather than matching only the *exact* current minute, it finds the most recent
//! scheduled occurrence at or before "now" within a catch-up grace window. This
//! means a run whose scheduled minute was missed — because the process was down,
//! the machine was asleep, or a timer tick was skipped during OS sleep — is still
//! executed once on the next tick/startup, instead of being silently dropped.
//!
//! Catch-up policy: run the *single most recent* missed occurrence per task
//! (never older ones, never the same one twice). Idempotency is guaranteed by the
//! atomic `try_claim_scheduled_execution` compare-and-swap on `last_scheduled_execution`.

use chrono::{DateTime, Datelike, Duration, Local, LocalResult, TimeZone, Timelike};
use sqlx::SqlitePool;

use crate::config::Settings;
use crate::errors::AppResult;
use crate::models::Schedule;
use crate::repositories::TaskRepository;

/// How far back a missed scheduled occurrence may be caught up.
///
/// An occurrence older than this is considered stale and skipped (e.g. the
/// machine was off for days — we don't want to fire a run for a long-past slot).
const CATCH_UP_GRACE_HOURS: i64 = 12;

/// Check and execute scheduled tasks (with catch-up for missed occurrences).
pub async fn run(pool: &SqlitePool, settings: &Settings) -> AppResult<()> {
    let now = Local::now();

    tracing::debug!(
        "Task scheduler check at {} (catch-up grace: {}h)",
        now.format("%Y-%m-%d %H:%M"),
        CATCH_UP_GRACE_HOURS
    );

    // Get all active scheduled tasks
    let tasks = get_active_scheduled_tasks(pool).await?;
    let now_key = minute_key(now);

    for task in tasks {
        // Parse the schedule
        let schedule: Schedule = match task.schedule_json.as_ref() {
            Some(json) => match serde_json::from_str(json) {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!("Failed to parse schedule for task '{}': {}", task.name, e);
                    continue;
                }
            },
            None => {
                tracing::debug!("Skipping task '{}': no schedule defined", task.name);
                continue;
            }
        };

        // Most recent scheduled occurrence at/before now, within the grace window.
        let Some(due) = most_recent_due(&schedule, now) else {
            tracing::trace!(
                "Task '{}' has no due occurrence within the catch-up window",
                task.name
            );
            continue;
        };

        let scheduled_key = minute_key(due);

        // Already executed this occurrence? (cheap pre-check before the CAS)
        if task.last_scheduled_execution.as_deref() == Some(scheduled_key.as_str()) {
            tracing::trace!(
                "Task '{}' already executed occurrence {}",
                task.name,
                scheduled_key
            );
            continue;
        }

        // Atomically claim this occurrence. The CAS prevents a double-fire on the
        // same database (overlapping tick, or a second reader) — the scheduler lock
        // already prevents two scheduler processes from racing here at all.
        let claimed =
            TaskRepository::try_claim_scheduled_execution(pool, task.id, &scheduled_key).await?;
        if !claimed {
            tracing::debug!(
                "Failed to claim occurrence {} for task '{}' (another tick/instance claimed it)",
                scheduled_key,
                task.name
            );
            continue;
        }

        let is_catch_up = scheduled_key != now_key;
        tracing::info!(
            "Executing scheduled task '{}' (id={}, occurrence={}{})",
            task.name,
            task.id,
            scheduled_key,
            if is_catch_up { ", catch-up" } else { "" }
        );

        // Execute off the scheduler loop so a long run never blocks the next tick.
        let pool_clone = pool.clone();
        let settings_clone = settings.clone();
        let task_id = task.id;
        let task_name = task.name.clone();

        tokio::spawn(async move {
            match execute_scheduled_task(&pool_clone, &settings_clone, task_id).await {
                Ok(_) => {
                    tracing::info!("Scheduled task '{}' completed successfully", task_name);
                }
                Err(e) => {
                    tracing::error!("Scheduled task '{}' failed: {:?}", task_name, e);
                }
            }
        });
    }

    Ok(())
}

/// Compute the most recent scheduled occurrence at or before `now` that falls
/// within the catch-up grace window, or `None` if there is no such occurrence.
///
/// The schedule is a weekly recurrence (`days` = weekdays 0..=6 Sun..Sat,
/// `times` = "HH:MM"). DST transitions are tolerated: a time that does not exist
/// (spring-forward gap) is skipped; an ambiguous time (fall-back) resolves to the
/// earlier instant.
fn most_recent_due(schedule: &Schedule, now: DateTime<Local>) -> Option<DateTime<Local>> {
    let earliest = now - Duration::hours(CATCH_UP_GRACE_HOURS);
    // Enough days back to cover the grace window (plus one for safety / partial days).
    let max_days_back = (CATCH_UP_GRACE_HOURS / 24) + 1;

    let mut best: Option<DateTime<Local>> = None;

    for day_offset in 0..=max_days_back {
        let date = now.date_naive() - Duration::days(day_offset);
        let weekday = date.weekday().num_days_from_sunday() as u8;
        if !schedule.days.contains(&weekday) {
            continue;
        }

        for time in &schedule.times {
            let Some((hour, minute)) = parse_hm(&normalize_time(time)) else {
                continue;
            };
            let Some(naive) = date.and_hms_opt(hour, minute, 0) else {
                continue;
            };
            let dt = match Local.from_local_datetime(&naive) {
                LocalResult::Single(dt) => dt,
                LocalResult::Ambiguous(dt, _) => dt, // fall-back hour: take the earlier instant
                LocalResult::None => continue,       // spring-forward gap: this time never occurs
            };

            if dt <= now && dt >= earliest {
                best = Some(match best {
                    Some(b) if b >= dt => b,
                    _ => dt,
                });
            }
        }
    }

    best
}

/// Duplicate-prevention / occurrence key: "YYYY-MM-DD-HH:MM" in local time.
fn minute_key(dt: DateTime<Local>) -> String {
    format!("{}-{:02}:{:02}", dt.date_naive(), dt.hour(), dt.minute())
}

/// Parse a normalized "HH:MM" string into (hour, minute), validating ranges.
fn parse_hm(hm: &str) -> Option<(u32, u32)> {
    let (h, m) = hm.split_once(':')?;
    let hour: u32 = h.parse().ok()?;
    let minute: u32 = m.parse().ok()?;
    (hour <= 23 && minute <= 59).then_some((hour, minute))
}

/// Get all active tasks with scheduled run mode
async fn get_active_scheduled_tasks(pool: &SqlitePool) -> AppResult<Vec<crate::models::Task>> {
    let tasks = sqlx::query_as::<_, crate::models::Task>(
        r#"
        SELECT * FROM tasks
        WHERE is_active = 1
          AND run_mode = 'scheduled'
          AND status != 'running'
        ORDER BY id
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(tasks)
}

/// Execute a scheduled task
async fn execute_scheduled_task(
    pool: &SqlitePool,
    settings: &Settings,
    task_id: i64,
) -> AppResult<()> {
    use crate::app::AppState;
    use crate::services::TaskService;

    // Load encryption key for this execution
    let encryption_key = crate::crypto::load_encryption_key().map(std::sync::Arc::new);

    // Create a minimal AppState for task execution
    let state = AppState::new(pool.clone(), settings.clone(), encryption_key);

    // Execute the task using TaskService
    // Note: TaskService.execute() handles status updates and logging
    match TaskService::execute_scheduled(&state, task_id).await {
        Ok(result) => {
            tracing::info!(
                "Scheduled task {} finished with status: {}",
                task_id,
                result.status
            );
            Ok(())
        }
        Err(e) => {
            tracing::error!("Scheduled task {} failed: {:?}", task_id, e);
            Err(e)
        }
    }
}

/// Normalize time string to HH:MM format
fn normalize_time(time: &str) -> String {
    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() == 2 {
        let hour: u32 = parts[0].parse().unwrap_or(0);
        let minute: u32 = parts[1].parse().unwrap_or(0);
        format!("{:02}:{:02}", hour, minute)
    } else {
        time.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_time() {
        assert_eq!(normalize_time("9:30"), "09:30");
        assert_eq!(normalize_time("09:30"), "09:30");
        assert_eq!(normalize_time("14:05"), "14:05");
        assert_eq!(normalize_time("0:00"), "00:00");
    }

    #[test]
    fn test_parse_hm() {
        assert_eq!(parse_hm("06:00"), Some((6, 0)));
        assert_eq!(parse_hm("23:59"), Some((23, 59)));
        assert_eq!(parse_hm("24:00"), None);
        assert_eq!(parse_hm("12:60"), None);
        assert_eq!(parse_hm("nope"), None);
    }

    /// An earlier-today occurrence within the grace window is returned (the
    /// missed-run catch-up case).
    #[test]
    fn test_most_recent_due_catches_earlier_today() {
        let now = Local
            .with_ymd_and_hms(2026, 6, 18, 13, 0, 0)
            .single()
            .unwrap();
        let wd = now.date_naive().weekday().num_days_from_sunday() as u8;
        let schedule = Schedule {
            days: vec![wd],
            times: vec!["06:00".into()],
        };
        let due = most_recent_due(&schedule, now).expect("06:00 is 7h ago, within 12h grace");
        assert_eq!(due.hour(), 6);
        assert_eq!(due.minute(), 0);
        assert_eq!(due.date_naive(), now.date_naive());
    }

    /// An occurrence older than the grace window is NOT caught up.
    #[test]
    fn test_most_recent_due_skips_beyond_grace() {
        let now = Local
            .with_ymd_and_hms(2026, 6, 18, 20, 0, 0)
            .single()
            .unwrap();
        let wd = now.date_naive().weekday().num_days_from_sunday() as u8;
        let schedule = Schedule {
            days: vec![wd],
            times: vec!["06:00".into()], // 14h ago > 12h grace
        };
        assert!(most_recent_due(&schedule, now).is_none());
    }

    /// A purely future time today (no past occurrence within grace) returns None.
    #[test]
    fn test_most_recent_due_ignores_future() {
        let now = Local
            .with_ymd_and_hms(2026, 6, 18, 5, 0, 0)
            .single()
            .unwrap();
        let wd = now.date_naive().weekday().num_days_from_sunday() as u8;
        let schedule = Schedule {
            days: vec![wd],
            times: vec!["06:00".into()], // 1h in the future; prior occurrence is 7 days ago
        };
        assert!(most_recent_due(&schedule, now).is_none());
    }

    /// The exact current minute is "due" (the normal, non-catch-up case).
    #[test]
    fn test_most_recent_due_matches_now() {
        let now = Local
            .with_ymd_and_hms(2026, 6, 18, 6, 0, 0)
            .single()
            .unwrap();
        let wd = now.date_naive().weekday().num_days_from_sunday() as u8;
        let schedule = Schedule {
            days: vec![wd],
            times: vec!["06:00".into()],
        };
        let due = most_recent_due(&schedule, now).expect("06:00 == now");
        assert_eq!(minute_key(due), minute_key(now));
    }
}
