use sqlx::SqlitePool;

use crate::errors::AppResult;
use crate::models::{
    CreateExecutionLineResultRequest, CreateTaskExecutionRequest, ListExecutionsParams,
    TaskExecution, TaskExecutionLineResult, TimedOutExecution,
};

pub struct TaskExecutionRepository;

impl TaskExecutionRepository {
    /// Create a new task execution record
    /// Timeout is calculated as: max(line_count * 60 seconds, 60 seconds minimum)
    pub async fn create(
        pool: &SqlitePool,
        req: &CreateTaskExecutionRequest,
    ) -> AppResult<TaskExecution> {
        // Calculate timeout: line_count * 60 seconds (minimum 60 seconds)
        let timeout_seconds = (req.line_count * 60).max(60);

        let execution = sqlx::query_as::<_, TaskExecution>(
            r#"
            INSERT INTO task_executions (
                task_id, execution_id, triggered_by, scheduled_for,
                started_at, status, is_finished, maximum_finish_time
            )
            VALUES (
                $1, $2, $3, $4,
                datetime('now', 'utc'), 'running', 0,
                datetime('now', 'utc', '+' || $5 || ' seconds')
            )
            RETURNING *
            "#,
        )
        .bind(req.task_id)
        .bind(&req.execution_id)
        .bind(&req.triggered_by)
        .bind(&req.scheduled_for)
        .bind(timeout_seconds)
        .fetch_one(pool)
        .await?;

        Ok(execution)
    }

    /// Update execution status to completed
    pub async fn complete(
        pool: &SqlitePool,
        execution_id: &str,
        duration_ms: i64,
        result_summary: Option<&str>,
    ) -> AppResult<bool> {
        let result = sqlx::query(
            r#"
            UPDATE task_executions
            SET status = 'completed',
                completed_at = datetime('now', 'utc'),
                duration_ms = $1,
                result_summary = $2,
                is_finished = 1
            WHERE execution_id = $3
            "#,
        )
        .bind(duration_ms)
        .bind(result_summary)
        .bind(execution_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Update execution status to failed
    pub async fn fail(
        pool: &SqlitePool,
        execution_id: &str,
        duration_ms: i64,
        error_message: &str,
        result_summary: Option<&str>,
    ) -> AppResult<bool> {
        let result = sqlx::query(
            r#"
            UPDATE task_executions
            SET status = 'failed',
                completed_at = datetime('now', 'utc'),
                duration_ms = $1,
                error_message = $2,
                result_summary = $3,
                is_finished = 1
            WHERE execution_id = $4
            "#,
        )
        .bind(duration_ms)
        .bind(error_message)
        .bind(result_summary)
        .bind(execution_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Get execution by ID
    pub async fn get_by_id(pool: &SqlitePool, id: i64) -> AppResult<Option<TaskExecution>> {
        let execution = sqlx::query_as::<_, TaskExecution>(
            "SELECT * FROM task_executions WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(execution)
    }

    /// Get execution by execution_id (UUID)
    pub async fn get_by_execution_id(
        pool: &SqlitePool,
        execution_id: &str,
    ) -> AppResult<Option<TaskExecution>> {
        let execution = sqlx::query_as::<_, TaskExecution>(
            "SELECT * FROM task_executions WHERE execution_id = $1",
        )
        .bind(execution_id)
        .fetch_optional(pool)
        .await?;

        Ok(execution)
    }

    /// List executions with optional filtering
    pub async fn list(
        pool: &SqlitePool,
        params: &ListExecutionsParams,
    ) -> AppResult<Vec<TaskExecution>> {
        let limit = params.limit.unwrap_or(50);
        let offset = params.offset.unwrap_or(0);

        // Build query dynamically based on filters
        let mut query = String::from(
            "SELECT * FROM task_executions WHERE 1=1"
        );
        let mut bind_count = 0;

        if params.task_id.is_some() {
            bind_count += 1;
            query.push_str(&format!(" AND task_id = ${}", bind_count));
        }
        if params.status.is_some() {
            bind_count += 1;
            query.push_str(&format!(" AND status = ${}", bind_count));
        }
        if params.triggered_by.is_some() {
            bind_count += 1;
            query.push_str(&format!(" AND triggered_by = ${}", bind_count));
        }

        query.push_str(&format!(
            " ORDER BY started_at DESC LIMIT ${} OFFSET ${}",
            bind_count + 1,
            bind_count + 2
        ));

        let mut sqlx_query = sqlx::query_as::<_, TaskExecution>(&query);

        if let Some(task_id) = params.task_id {
            sqlx_query = sqlx_query.bind(task_id);
        }
        if let Some(ref status) = params.status {
            sqlx_query = sqlx_query.bind(status);
        }
        if let Some(ref triggered_by) = params.triggered_by {
            sqlx_query = sqlx_query.bind(triggered_by);
        }

        sqlx_query = sqlx_query.bind(limit).bind(offset);

        let executions = sqlx_query.fetch_all(pool).await?;
        Ok(executions)
    }

    /// Get executions for a specific task
    pub async fn get_by_task_id(
        pool: &SqlitePool,
        task_id: i64,
        limit: i64,
    ) -> AppResult<Vec<TaskExecution>> {
        let executions = sqlx::query_as::<_, TaskExecution>(
            r#"
            SELECT * FROM task_executions
            WHERE task_id = $1
            ORDER BY started_at DESC
            LIMIT $2
            "#,
        )
        .bind(task_id)
        .bind(limit)
        .fetch_all(pool)
        .await?;

        Ok(executions)
    }

    /// Get the latest execution for a task
    pub async fn get_latest_for_task(
        pool: &SqlitePool,
        task_id: i64,
    ) -> AppResult<Option<TaskExecution>> {
        let execution = sqlx::query_as::<_, TaskExecution>(
            r#"
            SELECT * FROM task_executions
            WHERE task_id = $1
            ORDER BY started_at DESC
            LIMIT 1
            "#,
        )
        .bind(task_id)
        .fetch_optional(pool)
        .await?;

        Ok(execution)
    }

    /// Count total executions (for pagination)
    pub async fn count(pool: &SqlitePool, params: &ListExecutionsParams) -> AppResult<i64> {
        let mut query = String::from("SELECT COUNT(*) as count FROM task_executions WHERE 1=1");
        let mut bind_count = 0;

        if params.task_id.is_some() {
            bind_count += 1;
            query.push_str(&format!(" AND task_id = ${}", bind_count));
        }
        if params.status.is_some() {
            bind_count += 1;
            query.push_str(&format!(" AND status = ${}", bind_count));
        }
        if params.triggered_by.is_some() {
            bind_count += 1;
            query.push_str(&format!(" AND triggered_by = ${}", bind_count));
        }

        let mut sqlx_query = sqlx::query_scalar::<_, i64>(&query);

        if let Some(task_id) = params.task_id {
            sqlx_query = sqlx_query.bind(task_id);
        }
        if let Some(ref status) = params.status {
            sqlx_query = sqlx_query.bind(status);
        }
        if let Some(ref triggered_by) = params.triggered_by {
            sqlx_query = sqlx_query.bind(triggered_by);
        }

        let count = sqlx_query.fetch_one(pool).await?;
        Ok(count)
    }

    /// Delete old executions (for cleanup)
    pub async fn delete_older_than_days(pool: &SqlitePool, days: i64) -> AppResult<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM task_executions
            WHERE started_at < datetime('now', 'utc', '-' || $1 || ' days')
            "#,
        )
        .bind(days)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Check if there's a running execution for a task
    pub async fn has_running_execution(pool: &SqlitePool, task_id: i64) -> AppResult<bool> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM task_executions WHERE task_id = $1 AND status = 'running'",
        )
        .bind(task_id)
        .fetch_one(pool)
        .await?;

        Ok(count > 0)
    }

    /// Find executions that have exceeded their maximum_finish_time deadline
    pub async fn find_timed_out_executions(pool: &SqlitePool) -> AppResult<Vec<TimedOutExecution>> {
        let executions = sqlx::query_as::<_, TimedOutExecution>(
            r#"
            SELECT id, task_id, execution_id, started_at, maximum_finish_time
            FROM task_executions
            WHERE is_finished = 0
              AND maximum_finish_time IS NOT NULL
              AND datetime(maximum_finish_time) < datetime('now', 'utc')
            ORDER BY started_at ASC
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(executions)
    }

    /// Mark all unfinished executions as failed (app crash recovery).
    /// Called on startup since no execution can survive a restart.
    pub async fn reset_all_unfinished(pool: &SqlitePool) -> AppResult<u64> {
        let result = sqlx::query(
            r#"
            UPDATE task_executions
            SET status = 'failed',
                completed_at = datetime('now', 'utc'),
                error_message = 'Execution interrupted by app restart',
                is_finished = 1
            WHERE is_finished = 0
            "#,
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }

    /// Mark an execution as timed out (failed due to timeout)
    pub async fn mark_as_timed_out(pool: &SqlitePool, execution_id: &str) -> AppResult<bool> {
        let result = sqlx::query(
            r#"
            UPDATE task_executions
            SET status = 'failed',
                completed_at = datetime('now', 'utc'),
                error_message = 'Task execution timed out',
                is_finished = 1
            WHERE execution_id = $1 AND is_finished = 0
            "#,
        )
        .bind(execution_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    // ===== Line Result Methods =====

    /// Create a line execution result
    pub async fn create_line_result(
        pool: &SqlitePool,
        req: &CreateExecutionLineResultRequest,
    ) -> AppResult<TaskExecutionLineResult> {
        let result = sqlx::query_as::<_, TaskExecutionLineResult>(
            r#"
            INSERT INTO task_execution_results
                (execution_id, line_id, task_type, status, error_message, duration_ms, started_at, completed_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(&req.execution_id)
        .bind(req.line_id)
        .bind(&req.task_type)
        .bind(&req.status)
        .bind(&req.error_message)
        .bind(req.duration_ms)
        .bind(&req.started_at)
        .bind(&req.completed_at)
        .fetch_one(pool)
        .await?;

        Ok(result)
    }

    /// Get all line results for an execution
    pub async fn get_line_results(
        pool: &SqlitePool,
        execution_id: &str,
    ) -> AppResult<Vec<TaskExecutionLineResult>> {
        let results = sqlx::query_as::<_, TaskExecutionLineResult>(
            "SELECT * FROM task_execution_results WHERE execution_id = $1 ORDER BY id",
        )
        .bind(execution_id)
        .fetch_all(pool)
        .await?;

        Ok(results)
    }

    /// Batch create line results
    pub async fn create_line_results_batch(
        pool: &SqlitePool,
        results: &[CreateExecutionLineResultRequest],
    ) -> AppResult<()> {
        for req in results {
            sqlx::query(
                r#"
                INSERT INTO task_execution_results
                    (execution_id, line_id, task_type, status, error_message, duration_ms, started_at, completed_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
            )
            .bind(&req.execution_id)
            .bind(req.line_id)
            .bind(&req.task_type)
            .bind(&req.status)
            .bind(&req.error_message)
            .bind(req.duration_ms)
            .bind(&req.started_at)
            .bind(&req.completed_at)
            .execute(pool)
            .await?;
        }

        Ok(())
    }
}
