use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Task execution database model
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct TaskExecution {
    pub id: i64,
    pub task_id: i64,
    pub execution_id: String,
    pub triggered_by: String,
    pub scheduled_for: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub duration_ms: Option<i64>,
    pub result_summary: Option<String>,
    pub created_at: String,
    /// Deadline by which execution must complete (calculated from line count)
    pub maximum_finish_time: Option<String>,
    /// Whether execution has finished (0 = running, 1 = finished)
    pub is_finished: i32,
}

/// Task execution result (per-line) database model
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct TaskExecutionLineResult {
    pub id: i64,
    pub execution_id: String,
    pub line_id: i64,
    pub task_type: String,
    pub status: String,
    pub error_message: Option<String>,
    pub duration_ms: Option<i64>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

/// Task execution response DTO for API
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskExecutionResponse {
    pub id: i64,
    pub task_id: i64,
    pub task_name: String,
    pub execution_id: String,
    pub triggered_by: String,
    pub scheduled_for: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub duration_ms: Option<i64>,
    pub result_summary: Option<ExecutionResultSummary>,
    pub line_results: Vec<TaskExecutionLineResultResponse>,
}

/// Execution result summary (parsed from JSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionResultSummary {
    pub total_lines: i32,
    pub success_count: i32,
    pub failure_count: i32,
    pub speed_test_count: Option<i32>,
    pub quota_check_count: Option<i32>,
}

/// Per-line execution result response DTO
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskExecutionLineResultResponse {
    pub id: i64,
    pub execution_id: String,
    pub line_id: i64,
    pub line_name: String,
    pub task_type: String,
    pub status: String,
    pub error_message: Option<String>,
    pub duration_ms: Option<i64>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

/// Create task execution request (internal use)
#[derive(Debug, Clone)]
pub struct CreateTaskExecutionRequest {
    pub task_id: i64,
    pub execution_id: String,
    pub triggered_by: String,
    pub scheduled_for: Option<String>,
    /// Number of lines in the task (used to calculate timeout)
    pub line_count: i64,
}

/// Create task execution line result request (internal use)
#[derive(Debug, Clone)]
pub struct CreateExecutionLineResultRequest {
    pub execution_id: String,
    pub line_id: i64,
    pub task_type: String,
    pub status: String,
    pub error_message: Option<String>,
    pub duration_ms: Option<i64>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

/// Query parameters for listing executions
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListExecutionsParams {
    pub task_id: Option<i64>,
    pub status: Option<String>,
    pub triggered_by: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl Default for ListExecutionsParams {
    fn default() -> Self {
        Self {
            task_id: None,
            status: None,
            triggered_by: None,
            limit: Some(50),
            offset: Some(0),
        }
    }
}

/// Timed out execution info (for cleanup jobs)
#[derive(Debug, Clone, FromRow)]
pub struct TimedOutExecution {
    pub id: i64,
    pub task_id: i64,
    pub execution_id: String,
    pub started_at: String,
    pub maximum_finish_time: String,
}
