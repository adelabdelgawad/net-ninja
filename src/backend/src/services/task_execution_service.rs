use sqlx::SqlitePool;

use crate::errors::{AppError, AppResult};
use crate::models::{
    ListExecutionsParams, TaskExecution, TaskExecutionLineResultResponse, TaskExecutionResponse,
};
use crate::repositories::{LineRepository, TaskExecutionRepository, TaskRepository};

pub struct TaskExecutionService;

impl TaskExecutionService {
    /// Get execution by ID with full details
    pub async fn get_by_id(pool: &SqlitePool, id: i64) -> AppResult<TaskExecutionResponse> {
        let execution = TaskExecutionRepository::get_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Execution with id {} not found", id)))?;

        Self::to_response(pool, execution).await
    }

    /// Get execution by execution_id (UUID)
    pub async fn get_by_execution_id(
        pool: &SqlitePool,
        execution_id: &str,
    ) -> AppResult<TaskExecutionResponse> {
        let execution = TaskExecutionRepository::get_by_execution_id(pool, execution_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("Execution with id {} not found", execution_id))
            })?;

        Self::to_response(pool, execution).await
    }

    /// List executions with optional filtering
    pub async fn list(
        pool: &SqlitePool,
        params: &ListExecutionsParams,
    ) -> AppResult<Vec<TaskExecutionResponse>> {
        let executions = TaskExecutionRepository::list(pool, params).await?;

        let mut responses = Vec::new();
        for execution in executions {
            responses.push(Self::to_response(pool, execution).await?);
        }

        Ok(responses)
    }

    /// Get executions for a specific task
    pub async fn get_by_task_id(
        pool: &SqlitePool,
        task_id: i64,
        limit: Option<i64>,
    ) -> AppResult<Vec<TaskExecutionResponse>> {
        let limit = limit.unwrap_or(50);
        let executions = TaskExecutionRepository::get_by_task_id(pool, task_id, limit).await?;

        let mut responses = Vec::new();
        for execution in executions {
            responses.push(Self::to_response(pool, execution).await?);
        }

        Ok(responses)
    }

    /// Get the latest execution for a task
    pub async fn get_latest_for_task(
        pool: &SqlitePool,
        task_id: i64,
    ) -> AppResult<Option<TaskExecutionResponse>> {
        let execution = TaskExecutionRepository::get_latest_for_task(pool, task_id).await?;

        match execution {
            Some(e) => Ok(Some(Self::to_response(pool, e).await?)),
            None => Ok(None),
        }
    }

    /// Count total executions
    pub async fn count(pool: &SqlitePool, params: &ListExecutionsParams) -> AppResult<i64> {
        TaskExecutionRepository::count(pool, params).await
    }

    /// Delete old executions (cleanup)
    pub async fn cleanup_old_executions(pool: &SqlitePool, days: i64) -> AppResult<u64> {
        let deleted = TaskExecutionRepository::delete_older_than_days(pool, days).await?;
        tracing::info!("Deleted {} old execution records (older than {} days)", deleted, days);
        Ok(deleted)
    }

    /// Convert TaskExecution to TaskExecutionResponse
    async fn to_response(
        pool: &SqlitePool,
        execution: TaskExecution,
    ) -> AppResult<TaskExecutionResponse> {
        // Get task name
        let task_name = match TaskRepository::get_by_id(pool, execution.task_id).await? {
            Some(task) => task.name,
            None => format!("Deleted Task ({})", execution.task_id),
        };

        // Get line results
        let line_results =
            TaskExecutionRepository::get_line_results(pool, &execution.execution_id).await?;

        // Convert line results with line names
        let mut line_result_responses = Vec::new();
        for result in line_results {
            let line_name = match LineRepository::get_by_id_raw(pool, result.line_id as i32).await? {
                Some(line) => line.name,
                None => format!("Deleted Line ({})", result.line_id),
            };

            line_result_responses.push(TaskExecutionLineResultResponse {
                id: result.id,
                execution_id: result.execution_id,
                line_id: result.line_id,
                line_name,
                task_type: result.task_type,
                status: result.status,
                error_message: result.error_message,
                duration_ms: result.duration_ms,
                started_at: result.started_at,
                completed_at: result.completed_at,
            });
        }

        // Parse result summary
        let result_summary = execution.result_summary.and_then(|s| serde_json::from_str(&s).ok());

        Ok(TaskExecutionResponse {
            id: execution.id,
            task_id: execution.task_id,
            task_name,
            execution_id: execution.execution_id,
            triggered_by: execution.triggered_by,
            scheduled_for: execution.scheduled_for,
            started_at: execution.started_at,
            completed_at: execution.completed_at,
            status: execution.status,
            error_message: execution.error_message,
            duration_ms: execution.duration_ms,
            result_summary,
            line_results: line_result_responses,
        })
    }
}
