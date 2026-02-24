use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

use super::line::LineResponse;

/// Task database model
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Task {
    pub id: i64,
    pub name: String,
    pub task_types: String,      // JSON array: ["speed_test"], ["quota_check"], or both
    pub run_mode: String,
    pub schedule_json: Option<String>,
    pub status: String,
    pub is_active: bool,
    pub show_browser: bool,      // Show browser window during quota check (false = headless)
    pub last_scheduled_execution: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Schedule definition for scheduled tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub days: Vec<u8>,      // 0-6 for Sunday-Saturday
    pub times: Vec<String>, // "HH:MM" format
}

/// Task response DTO for API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskResponse {
    pub id: i64,
    pub name: String,
    pub task_types: Vec<String>,  // Parsed from JSON
    pub run_mode: String,
    pub schedule: Option<Schedule>,
    pub status: String,
    pub is_active: bool,
    pub show_browser: bool,       // Show browser window during quota check
    pub line_ids: Vec<i64>,
    pub lines: Vec<LineResponse>,
    pub last_run_at: Option<String>,
    pub next_run_at: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create task request DTO
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,

    #[validate(custom(function = "validate_task_types"))]
    pub task_types: Vec<String>,

    #[validate(custom(function = "validate_run_mode"))]
    pub run_mode: String,

    #[validate(custom(function = "validate_schedule"))]
    pub schedule: Option<Schedule>,

    #[validate(length(min = 1))]
    pub line_ids: Vec<i64>,

    #[serde(default)]
    pub show_browser: bool,  // Show browser window during quota check (default: false/headless)
}

/// Update task request DTO
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTaskRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,

    pub task_types: Option<Vec<String>>,

    pub run_mode: Option<String>,

    pub schedule: Option<Schedule>,

    pub line_ids: Option<Vec<i64>>,

    pub show_browser: Option<bool>,  // Show browser window during quota check
}

/// Line execution result for task execution
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LineExecutionResult {
    pub line_id: i64,
    pub line_name: String,
    pub task_type: String,
    pub status: String,           // "success" | "failed"
    pub error_message: Option<String>,
    pub duration_ms: u64,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
}

/// Task execution result
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskExecutionResult {
    pub task_id: i64,
    pub task_name: String,
    pub status: String,           // "running" | "completed" | "failed"
    pub results: TaskTypeResults,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

/// Results grouped by task type
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskTypeResults {
    pub speed_test: Option<Vec<LineExecutionResult>>,
    pub quota_check: Option<Vec<LineExecutionResult>>,
}

// Custom validators
fn validate_task_types(task_types: &[String]) -> Result<(), validator::ValidationError> {
    validate_task_types_impl(task_types, true)
}

fn validate_task_types_impl(task_types: &[String], require_non_empty: bool) -> Result<(), validator::ValidationError> {
    if require_non_empty && task_types.is_empty() {
        let mut err = validator::ValidationError::new("empty_task_types");
        err.message = Some("At least one task type must be selected".into());
        return Err(err);
    }
    let valid_types = ["speed_test", "quota_check"];
    for task_type in task_types {
        if !valid_types.contains(&task_type.as_str()) {
            return Err(validator::ValidationError::new("invalid_task_type"));
        }
    }
    // Check for duplicates
    let unique_types: std::collections::HashSet<_> = task_types.iter().collect();
    if unique_types.len() != task_types.len() {
        let mut err = validator::ValidationError::new("duplicate_task_types");
        err.message = Some("Task types must be unique".into());
        return Err(err);
    }
    Ok(())
}

fn validate_run_mode(run_mode: &str) -> Result<(), validator::ValidationError> {
    if run_mode != "one_time" && run_mode != "scheduled" {
        return Err(validator::ValidationError::new("invalid_run_mode"));
    }
    Ok(())
}

fn validate_schedule(schedule: &Schedule) -> Result<(), validator::ValidationError> {
    validate_schedule_impl(schedule, true)
}

fn validate_schedule_impl(schedule: &Schedule, require_non_empty: bool) -> Result<(), validator::ValidationError> {
    if require_non_empty {
        if schedule.days.is_empty() {
            let mut err = validator::ValidationError::new("empty_days");
            err.message = Some("At least one day must be selected".into());
            return Err(err);
        }
        if schedule.times.is_empty() {
            let mut err = validator::ValidationError::new("empty_times");
            err.message = Some("At least one time must be specified".into());
            return Err(err);
        }
    }
    // Validate day values (0-6)
    for day in &schedule.days {
        if *day > 6 {
            let mut err = validator::ValidationError::new("invalid_day");
            err.message = Some("Day value must be between 0-6".into());
            return Err(err);
        }
    }
    // Validate time format (HH:MM)
    for time in &schedule.times {
        if !is_valid_time_format(time) {
            let mut err = validator::ValidationError::new("invalid_time_format");
            err.message = Some("Time must be in HH:MM format".into());
            return Err(err);
        }
    }
    Ok(())
}

/// Validate HH:MM time format without regex.
fn is_valid_time_format(s: &str) -> bool {
    let bytes = s.as_bytes();
    match bytes.len() {
        // H:MM
        4 => {
            bytes[0].is_ascii_digit()
                && bytes[1] == b':'
                && bytes[2].is_ascii_digit()
                && bytes[3].is_ascii_digit()
                && (bytes[0] - b'0') <= 9
                && (bytes[2] - b'0') * 10 + (bytes[3] - b'0') <= 59
        }
        // HH:MM
        5 => {
            bytes[0].is_ascii_digit()
                && bytes[1].is_ascii_digit()
                && bytes[2] == b':'
                && bytes[3].is_ascii_digit()
                && bytes[4].is_ascii_digit()
                && {
                    let hour = (bytes[0] - b'0') * 10 + (bytes[1] - b'0');
                    let min = (bytes[3] - b'0') * 10 + (bytes[4] - b'0');
                    hour <= 23 && min <= 59
                }
        }
        _ => false,
    }
}
