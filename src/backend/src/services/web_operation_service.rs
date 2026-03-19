//! Service for web-dashboard-initiated operations (quota check, speed test for a single line).
//! These are distinct from scheduler-triggered task executions.

use std::sync::Arc;
use std::time::Instant;

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::config::Settings;
use crate::crypto::EncryptionKey;
use crate::errors::{AppError, AppResult};
use crate::models::CreateTaskExecutionRequest;
use crate::repositories::{LineRepository, TaskExecutionRepository, TaskRepository};

/// Response for execution status polling
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionStatusResponse {
    pub status: String,          // "running" | "completed" | "failed"
    pub error_message: Option<String>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct WebOperationService;

impl WebOperationService {
    /// Trigger a quota check for a specific line.
    /// Creates a temporary task + execution record, spawns the check in background.
    /// Returns the task_execution row id for polling.
    pub async fn trigger_quota_check(
        pool: SqlitePool,
        settings: Arc<Settings>,
        encryption_key: Option<Arc<EncryptionKey>>,
        line_id: i64,
    ) -> AppResult<i64> {
        Self::trigger_operation(pool, settings, encryption_key, line_id, "quota_check").await
    }

    /// Trigger a speed test for a specific line.
    pub async fn trigger_speed_test(
        pool: SqlitePool,
        settings: Arc<Settings>,
        encryption_key: Option<Arc<EncryptionKey>>,
        line_id: i64,
    ) -> AppResult<i64> {
        Self::trigger_operation(pool, settings, encryption_key, line_id, "speed_test").await
    }

    async fn trigger_operation(
        pool: SqlitePool,
        settings: Arc<Settings>,
        encryption_key: Option<Arc<EncryptionKey>>,
        line_id: i64,
        task_type: &'static str,
    ) -> AppResult<i64> {
        // 1. Verify line exists
        let _ = LineRepository::get_by_id(&pool, line_id as i32)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Line {} not found", line_id)))?;

        // 2. Create a temporary one-time task for this operation
        let task_name = format!(
            "web-{}-{}-{}",
            task_type,
            line_id,
            chrono::Utc::now().timestamp_millis()
        );
        let task_types_json = format!(r#"["{}"]"#, task_type);
        let task = TaskRepository::create(
            &pool,
            &task_name,
            &task_types_json,
            "one_time",
            None,
            false,
        )
        .await?;
        TaskRepository::add_lines(&pool, task.id, &[line_id]).await?;

        // 3. Create execution record (status = "running") and get its id
        let execution_id = Uuid::new_v4().to_string();
        let execution = TaskExecutionRepository::create(
            &pool,
            &CreateTaskExecutionRequest {
                task_id: task.id,
                execution_id: execution_id.clone(),
                triggered_by: "manual".to_string(),
                scheduled_for: None,
                line_count: 1,
            },
        )
        .await?;
        let row_id = execution.id;

        // 4. Spawn background execution
        let pool_clone = pool.clone();
        let task_id = task.id;
        tokio::spawn(async move {
            let start = Instant::now();
            let app_state = crate::app::AppState::new_full(
                pool_clone.clone(),
                (*settings).clone(),
                encryption_key,
            );

            let exec_result = crate::services::TaskService::execute(&app_state, task_id, None).await;
            let duration_ms = start.elapsed().as_millis() as i64;

            // Update our pre-created execution record to reflect the result
            match exec_result {
                Ok(ref result) => {
                    let status = if result.results.quota_check.as_ref().or(result.results.speed_test.as_ref())
                        .map(|v| v.iter().all(|r| r.status == "success"))
                        .unwrap_or(true) { "completed" } else { "failed" };
                    let _ = sqlx::query(
                        "UPDATE task_executions SET status = $1, completed_at = datetime('now', 'utc'), duration_ms = $2, is_finished = 1 WHERE id = $3"
                    )
                    .bind(status)
                    .bind(duration_ms)
                    .bind(row_id)
                    .execute(&pool_clone)
                    .await;
                }
                Err(ref e) => {
                    tracing::error!(
                        "[WebOperation] Background {} for line {} failed: {}",
                        task_type,
                        line_id,
                        e
                    );
                    let _ = sqlx::query(
                        "UPDATE task_executions SET status = 'failed', error_message = $1, completed_at = datetime('now', 'utc'), duration_ms = $2, is_finished = 1 WHERE id = $3"
                    )
                    .bind(e.to_string())
                    .bind(duration_ms)
                    .bind(row_id)
                    .execute(&pool_clone)
                    .await;
                }
            }
        });

        Ok(row_id)
    }

    /// Get the execution status for polling.
    pub async fn get_execution_status(
        pool: &SqlitePool,
        execution_id: i64,
    ) -> AppResult<ExecutionStatusResponse> {
        let row = sqlx::query_as::<_, (String, Option<String>, Option<String>)>(
            "SELECT status, error_message, completed_at FROM task_executions WHERE id = $1",
        )
        .bind(execution_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Execution {} not found", execution_id)))?;

        let (status, error_message, finished_at_str) = row;
        let finished_at = finished_at_str.as_deref().and_then(|s| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                .ok()
                .map(|dt| chrono::DateTime::from_naive_utc_and_offset(dt, chrono::Utc))
        });

        Ok(ExecutionStatusResponse {
            status,
            error_message,
            finished_at,
        })
    }
}
