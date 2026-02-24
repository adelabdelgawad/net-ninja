//! Task Scheduler Job
//!
//! This job runs every minute and checks for scheduled tasks that are due to run.
//! It uses a polling approach to find tasks whose schedule matches the current day/time.

use chrono::{Datelike, Local, Timelike};
use sqlx::SqlitePool;

use crate::config::Settings;
use crate::errors::AppResult;
use crate::models::Schedule;
use crate::repositories::TaskRepository;

/// Check and execute scheduled tasks
pub async fn run(pool: &SqlitePool, settings: &Settings) -> AppResult<()> {
    let now = Local::now();
    let current_day = now.weekday().num_days_from_sunday() as u8; // 0 = Sunday
    let current_time = format!("{:02}:{:02}", now.hour(), now.minute());
    let scheduled_key = format!("{}-{}-{}", now.date_naive(), current_time, now.timestamp() / 60);

    tracing::debug!(
        "Task scheduler check: day={}, time={}, key={}",
        current_day,
        current_time,
        scheduled_key
    );

    // Get all active scheduled tasks
    let tasks = get_active_scheduled_tasks(pool).await?;

    for task in tasks {
        // Parse the schedule
        let schedule: Schedule = match task.schedule_json.as_ref() {
            Some(json) => match serde_json::from_str(json) {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!(
                        "Failed to parse schedule for task '{}': {}",
                        task.name,
                        e
                    );
                    continue;
                }
            },
            None => {
                tracing::debug!(
                    "Skipping task '{}': no schedule defined",
                    task.name
                );
                continue;
            }
        };

        // Check if current day is in the schedule
        if !schedule.days.contains(&current_day) {
            tracing::trace!(
                "Task '{}' not scheduled for today (day {})",
                task.name,
                current_day
            );
            continue;
        }

        // Check if current time matches any scheduled time
        let time_matches = schedule.times.iter().any(|t| {
            // Normalize time format (e.g., "9:30" -> "09:30")
            let normalized = normalize_time(t);
            normalized == current_time
        });

        if !time_matches {
            tracing::trace!(
                "Task '{}' not scheduled for current time ({})",
                task.name,
                current_time
            );
            continue;
        }

        // Check duplicate prevention using minute-based key
        let minute_key = format!("{}-{}", now.date_naive(), current_time);
        if let Some(ref last_exec) = task.last_scheduled_execution {
            if last_exec.starts_with(&minute_key) {
                tracing::debug!(
                    "Task '{}' already executed this minute (key: {})",
                    task.name,
                    last_exec
                );
                continue;
            }
        }

        // Try to claim execution slot (atomic operation to prevent races)
        let claimed = TaskRepository::try_claim_scheduled_execution(pool, task.id, &minute_key).await?;
        if !claimed {
            tracing::debug!(
                "Failed to claim execution slot for task '{}' (another instance may have claimed it)",
                task.name
            );
            continue;
        }

        tracing::info!(
            "Executing scheduled task '{}' (id={}, time={})",
            task.name,
            task.id,
            current_time
        );

        // Execute the task
        // Note: We use spawn to not block the scheduler loop
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
}
