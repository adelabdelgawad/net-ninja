use std::net::IpAddr;

use chrono::Utc;
use futures_util::stream::{self, StreamExt};
use sqlx::SqlitePool;
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::clients::{QuotaDebugLog, SpeedTestClient, WebDriverClient};
use crate::config::Settings;
use crate::errors::{AppError, AppResult};
use crate::models::{
    CreateExecutionLineResultRequest, CreateQuotaResultRequest, CreateSpeedTestResultRequest,
    CreateTaskExecutionRequest, CreateTaskRequest, ExecutionResultSummary, Line,
    LineExecutionResult, LineResponse, Schedule, Task, TaskExecutionResult, TaskResponse,
    TaskTypeResults, UpdateTaskRequest,
};
use crate::repositories::{LineRepository, TaskExecutionRepository, TaskRepository};
use crate::services::{LogService, QuotaCheckService, SpeedTestService};

pub struct TaskService;

impl TaskService {
    /// Get task by ID with populated line information
    pub async fn get_by_id(pool: &SqlitePool, id: i64) -> AppResult<TaskResponse> {
        let task = TaskRepository::get_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Task with id {} not found", id)))?;

        Self::to_response(pool, task).await
    }

    /// Get all tasks with populated line information
    pub async fn get_all(pool: &SqlitePool) -> AppResult<Vec<TaskResponse>> {
        let tasks = TaskRepository::get_all(pool).await?;

        let mut responses = Vec::new();
        for task in tasks {
            responses.push(Self::to_response(pool, task).await?);
        }

        Ok(responses)
    }

    /// Create a new task
    pub async fn create(pool: &SqlitePool, req: CreateTaskRequest) -> AppResult<TaskResponse> {
        // Validate request
        req.validate().map_err(|e| {
            AppError::Validation(format!("Validation failed: {}", e))
        })?;

        // Check task name uniqueness
        let trimmed_name = req.name.trim();
        if trimmed_name.is_empty() {
            return Err(AppError::Validation("Task name is required".to_string()));
        }

        if TaskRepository::name_exists(pool, trimmed_name).await? {
            return Err(AppError::Validation(format!(
                "A task with this name already exists"
            )));
        }

        // Validate all line IDs exist
        for line_id in &req.line_ids {
            if !LineRepository::exists(pool, *line_id as i32).await? {
                return Err(AppError::Validation(format!(
                    "Line with id {} not found",
                    line_id
                )));
            }
        }

        // Validate schedule for scheduled tasks
        let schedule_json = if req.run_mode == "scheduled" {
            if req.schedule.is_none() {
                return Err(AppError::Validation(
                    "Schedule is required for scheduled tasks".to_string()
                ));
            }
            let schedule = req.schedule.as_ref().unwrap();
            Some(serde_json::to_string(schedule).map_err(|e| {
                AppError::Internal(format!("Failed to serialize schedule: {}", e))
            })?)
        } else {
            None
        };

        // Create task
        // Convert task_types array to JSON string
        let task_types_json = serde_json::to_string(&req.task_types)
            .map_err(|e| AppError::Internal(format!("Failed to serialize task_types: {}", e)))?;

        let task = TaskRepository::create(
            pool,
            trimmed_name,
            &task_types_json,
            &req.run_mode,
            schedule_json.as_deref(),
            req.show_browser,
        )
        .await?;

        // Add task-line associations
        TaskRepository::add_lines(pool, task.id, &req.line_ids).await?;

        // Return populated response
        Self::to_response(pool, task).await
    }

    /// Check if task name is available
    pub async fn check_name_available(pool: &SqlitePool, name: &str) -> AppResult<bool> {
        let exists = TaskRepository::name_exists(pool, name).await?;
        Ok(!exists)
    }

    /// Delete a task
    pub async fn delete(pool: &SqlitePool, id: i64) -> AppResult<()> {
        // Check if task exists and is not running
        let task = TaskRepository::get_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Task with id {} not found", id)))?;

        if task.status == "running" {
            return Err(AppError::Validation(
                "Cannot delete a running task".to_string()
            ));
        }

        if !TaskRepository::delete(pool, id).await? {
            return Err(AppError::NotFound(format!("Task with id {} not found", id)));
        }
        Ok(())
    }

    /// Update a task
    pub async fn update(pool: &SqlitePool, id: i64, req: UpdateTaskRequest) -> AppResult<TaskResponse> {
        tracing::info!(
            task_id = id,
            name = %req.name.as_deref().unwrap_or("(unchanged)"),
            "TaskService::update called"
        );

        // Check if task exists and is not running
        let task = TaskRepository::get_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Task with id {} not found", id)))?;

        if task.status == "running" {
            return Err(AppError::Validation(
                "Cannot update a running task".to_string()
            ));
        }

        // Validate request
        req.validate().map_err(|e| {
            AppError::Validation(format!("Validation failed: {}", e))
        })?;

        // Check name uniqueness (excluding current task)
        if let Some(ref name) = req.name {
            let trimmed_name = name.trim();
            if trimmed_name.is_empty() {
                return Err(AppError::Validation("Task name is required".to_string()));
            }
            if TaskRepository::name_exists_excluding(pool, trimmed_name, id).await? {
                return Err(AppError::Validation(format!(
                    "A task with this name already exists"
                )));
            }
        }

        // Validate line IDs if provided
        if let Some(ref line_ids) = req.line_ids {
            for line_id in line_ids {
                if !LineRepository::exists(pool, *line_id as i32).await? {
                    return Err(AppError::Validation(format!(
                        "Line with id {} not found",
                        line_id
                    )));
                }
            }
        }

        // Convert task_types to JSON if provided
        let task_types_json = if let Some(ref types) = req.task_types {
            Some(serde_json::to_string(types)
                .map_err(|e| AppError::Internal(format!("Failed to serialize task_types: {}", e)))?)
        } else {
            None
        };

        // Convert schedule to JSON if provided
        let schedule_json = if let Some(ref schedule) = req.schedule {
            Some(Some(serde_json::to_string(schedule)
                .map_err(|e| AppError::Internal(format!("Failed to serialize schedule: {}", e)))?))
        } else if req.run_mode.is_some() && req.run_mode.as_deref() == Some("one_time") {
            Some(None)  // Clear schedule when switching to one_time
        } else {
            None
        };

        // Update task
        let task = TaskRepository::update(
            pool,
            id,
            req.name.as_deref(),
            task_types_json.as_deref(),
            req.run_mode.as_deref(),
            schedule_json.as_ref().map(|s| s.as_deref()),
            req.show_browser,
        )
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Task with id {} not found", id)))?;

        // Update line associations if provided
        if let Some(line_ids) = req.line_ids {
            // Remove existing associations
            sqlx::query("DELETE FROM task_lines WHERE task_id = $1")
                .bind(id)
                .execute(pool)
                .await?;

            // Add new associations
            TaskRepository::add_lines(pool, id, &line_ids).await?;
        }

        tracing::info!(
            task_id = id,
            name = %task.name,
            "Task updated successfully, returning TaskResponse"
        );

        // Return populated response
        Self::to_response(pool, task).await
    }

    /// Toggle task active status
    pub async fn toggle_active(pool: &SqlitePool, id: i64, is_active: bool) -> AppResult<TaskResponse> {
        let task = TaskRepository::toggle_active(pool, id, is_active)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Task with id {} not found", id)))?;

        Self::to_response(pool, task).await
    }

    /// Execute a task immediately (manual trigger) with optional notification override
    pub async fn execute(
        state: &AppState,
        id: i64,
        notification_override: Option<crate::models::RuntimeNotificationConfigRequest>,
    ) -> AppResult<TaskExecutionResult> {
        Self::execute_with_trigger(state, id, "manual", None, notification_override).await
    }

    /// Execute a task from scheduler
    pub async fn execute_scheduled(state: &AppState, id: i64) -> AppResult<TaskExecutionResult> {
        let scheduled_for = Utc::now().to_rfc3339();
        Self::execute_with_trigger(state, id, "scheduler", Some(scheduled_for), None).await
    }

    /// Internal execute method with trigger source tracking
    async fn execute_with_trigger(
        state: &AppState,
        id: i64,
        triggered_by: &str,
        scheduled_for: Option<String>,
        notification_override: Option<crate::models::RuntimeNotificationConfigRequest>,
    ) -> AppResult<TaskExecutionResult> {
        let pool = state.require_pool()?;
        let settings = &state.settings;
        let process_id = Uuid::new_v4();
        let execution_id = process_id.to_string();

        tracing::info!(
            "[TaskService::execute] Starting execution for task_id={}, process_id={}, triggered_by={}",
            id, process_id, triggered_by
        );

        // Get task
        let task = TaskRepository::get_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Task with id {} not found", id)))?;

        tracing::debug!(
            "[TaskService::execute] Task found: name='{}', is_active={}, status='{}'",
            task.name, task.is_active, task.status
        );

        // Check if task is active
        if !task.is_active {
            tracing::warn!("[TaskService::execute] Task is disabled, aborting");
            return Err(AppError::Validation("Task is disabled".to_string()));
        }

        // Check if task is already running
        if task.status == "running" {
            tracing::warn!("[TaskService::execute] Task is already running, aborting");
            return Err(AppError::Validation("Task is already running".to_string()));
        }

        // Parse task_types
        let task_types: Vec<String> = serde_json::from_str(&task.task_types)
            .map_err(|e| AppError::Internal(format!("Failed to parse task_types JSON: {}", e)))?;

        // Get line IDs
        let line_ids = TaskRepository::get_line_ids(pool, id).await?;

        // Get actual line objects
        let lines = LineRepository::get_by_ids(pool, &line_ids).await?;

        // Create execution record with line count for timeout calculation
        let exec_request = CreateTaskExecutionRequest {
            task_id: id,
            execution_id: execution_id.clone(),
            triggered_by: triggered_by.to_string(),
            scheduled_for,
            line_count: line_ids.len() as i64,
        };
        if let Err(e) = TaskExecutionRepository::create(pool, &exec_request).await {
            tracing::warn!("[TaskService::execute] Failed to create execution record: {:?}", e);
        }

        // Set status to running
        TaskRepository::update_status(pool, id, "running").await?;

        // Register cancellation token
        let cancel_token = crate::services::task_runtime::register(id).await;

        // IMPORTANT: From this point on, we MUST ensure task status is updated to a terminal state
        // (completed/failed) before returning, even on error. Use execute_inner and cleanup pattern.
        let result = Self::execute_inner(pool, settings, &task, &task_types, &lines, process_id, &execution_id, cancel_token.clone()).await;

        // Remove task from runtime registry
        crate::services::task_runtime::remove(id).await;

        // Ensure task is never left in "running" state
        match &result {
            Ok(exec_result) => {
                // Record execution completion
                let duration_ms = exec_result.finished_at
                    .map(|f| (f - exec_result.started_at).num_milliseconds())
                    .unwrap_or(0);

                let summary = Self::create_result_summary(&exec_result.results);
                let summary_json = serde_json::to_string(&summary).ok();

                if exec_result.status == "completed" {
                    TaskExecutionRepository::complete(
                        pool,
                        &execution_id,
                        duration_ms,
                        summary_json.as_deref(),
                    ).await.ok();
                } else {
                    TaskExecutionRepository::fail(
                        pool,
                        &execution_id,
                        duration_ms,
                        "Task completed with failures",
                        summary_json.as_deref(),
                    ).await.ok();
                }
            }
            Err(e) => {
                // On any error, ensure task is marked as failed
                tracing::error!(
                    "[TaskService::execute] Task '{}' encountered error, ensuring status is 'failed': {:?}",
                    task.name, e
                );
                if let Err(status_err) = TaskRepository::update_status(pool, id, "failed").await {
                    tracing::error!(
                        "[TaskService::execute] Failed to update task status to 'failed': {:?}",
                        status_err
                    );
                }

                // Record execution failure
                TaskExecutionRepository::fail(
                    pool,
                    &execution_id,
                    0,
                    &format!("{}", e),
                    None,
                ).await.ok();

                // Log the failure
                LogService::error(
                    pool,
                    process_id,
                    "TaskService::execute",
                    &format!("Task '{}' failed with error: {}", task.name, e),
                )
                .await
                .ok();
            }
        }

        // Send notification email (Phase 5)
        // Send notification for both success and failure cases (if we have notification config)
        let exec_result_for_notification = match &result {
            Ok(exec_result) => {
                // Use actual execution result for successful runs
                Some(exec_result.clone())
            }
            Err(e) => {
                // Create a failed execution result for notification purposes
                tracing::debug!(
                    "[TaskService::execute] Creating failed execution result for notification: {}",
                    e
                );
                Some(TaskExecutionResult {
                    task_id: id,
                    task_name: task.name.clone(),
                    status: "failed".to_string(),
                    results: TaskTypeResults {
                        speed_test: None,
                        quota_check: None,
                    },
                    started_at: Utc::now(),
                    finished_at: Some(Utc::now()),
                })
            }
        };

        // Send notification if we have a result (success or failure)
        if let Some(exec_result) = exec_result_for_notification {
            // DIAGNOSTIC: Upgraded to info! for visibility (DEBUG logs are filtered)
            tracing::info!(
                "[TaskService::execute] Attempting to send notification for task '{}', status='{}'",
                task.name,
                exec_result.status
            );

            // Send notification in background - don't fail task if notification fails
            if let Err(e) = crate::services::NotificationService::send_task_notification(
                pool,
                &task,
                &exec_result,
                notification_override,
                process_id,
                state.encryption_key.as_deref(),
            )
            .await
            {
                tracing::warn!(
                    "[TaskService::execute] Failed to send notification for task '{}': {:?}",
                    task.name,
                    e
                );
                // Note: Logging is already done in NotificationService, so we just trace here
            }
        }

        // Cleanup logs older than 7 days after every task execution.
        // Uses existing LogService::cleanup_old which deletes via UTC-based datetime comparison.
        // Errors are logged but never propagated — cleanup must not affect task results.
        const LOG_CLEANUP_RETENTION_DAYS: i64 = 7;
        match LogService::cleanup_old(pool, LOG_CLEANUP_RETENTION_DAYS).await {
            Ok(deleted) => {
                if deleted > 0 {
                    tracing::info!(
                        "[TaskService::execute] Log cleanup: deleted {} entries older than {} days",
                        deleted, LOG_CLEANUP_RETENTION_DAYS
                    );
                }
            }
            Err(e) => {
                tracing::warn!(
                    "[TaskService::execute] Log cleanup failed (non-fatal): {:?}",
                    e
                );
            }
        }

        // Cleanup old service log files (older than 30 days) after every task execution.
        // This prevents unbounded disk usage in %ProgramData%\NetNinja\logs\.
        // Errors are logged but never propagated — cleanup must not affect task results.
        #[cfg(all(windows, feature = "service"))]
        match crate::service::logging::cleanup_old_logs(30) {
            Ok(count) if count > 0 => {
                tracing::info!(
                    "[TaskService::execute] Service log file cleanup: deleted {} files older than 30 days",
                    count
                );
            }
            Ok(_) => {}
            Err(e) => {
                tracing::warn!(
                    "[TaskService::execute] Service log file cleanup failed (non-fatal): {}",
                    e
                );
            }
        }

        result
    }

    /// Create a summary of execution results
    fn create_result_summary(results: &TaskTypeResults) -> ExecutionResultSummary {
        let mut total = 0;
        let mut success = 0;
        let mut failures = 0;
        let mut speed_test_count = None;
        let mut quota_check_count = None;

        if let Some(ref speed_tests) = results.speed_test {
            let count = speed_tests.len() as i32;
            speed_test_count = Some(count);
            total += count;
            success += speed_tests.iter().filter(|r| r.status == "success").count() as i32;
            failures += speed_tests.iter().filter(|r| r.status == "failed").count() as i32;
        }

        if let Some(ref quota_checks) = results.quota_check {
            let count = quota_checks.len() as i32;
            quota_check_count = Some(count);
            total += count;
            success += quota_checks.iter().filter(|r| r.status == "success").count() as i32;
            failures += quota_checks.iter().filter(|r| r.status == "failed").count() as i32;
        }

        ExecutionResultSummary {
            total_lines: total,
            success_count: success,
            failure_count: failures,
            speed_test_count,
            quota_check_count,
        }
    }

    /// Store per-line execution results in the database
    async fn store_line_results(
        pool: &SqlitePool,
        execution_id: &str,
        results: &[LineExecutionResult],
        task_type: &str,
    ) {
        let requests: Vec<CreateExecutionLineResultRequest> = results
            .iter()
            .map(|r| CreateExecutionLineResultRequest {
                execution_id: execution_id.to_string(),
                line_id: r.line_id,
                task_type: task_type.to_string(),
                status: r.status.clone(),
                error_message: r.error_message.clone(),
                duration_ms: Some(r.duration_ms as i64),
                started_at: Some(r.started_at.to_rfc3339()),
                completed_at: Some(r.completed_at.to_rfc3339()),
            })
            .collect();

        if let Err(e) = TaskExecutionRepository::create_line_results_batch(pool, &requests).await {
            tracing::warn!(
                "[TaskService] Failed to store {} line execution results: {:?}",
                task_type,
                e
            );
        }
    }

    /// Inner execution logic - separated to allow cleanup on error
    async fn execute_inner(
        pool: &SqlitePool,
        settings: &Settings,
        task: &Task,
        task_types: &[String],
        lines: &[Line],
        process_id: Uuid,
        execution_id: &str,
        cancel_token: tokio_util::sync::CancellationToken,
    ) -> AppResult<TaskExecutionResult> {
        LogService::info(
            pool,
            process_id,
            "TaskService::execute",
            &format!("Starting task '{}' with {} lines", task.name, lines.len()),
        )
        .await
        .ok(); // Don't fail if logging fails

        tracing::info!("[TaskService::execute] Task '{}' starting with {} lines, process_id={}",
            task.name, lines.len(), process_id);

        let started_at = Utc::now();
        let mut results = TaskTypeResults {
            speed_test: None,
            quota_check: None,
        };
        let mut has_failures = false;

        // Create parallel futures for both task types
        let speed_future = async {
            if task_types.contains(&"speed_test".to_string()) {
                Some(Self::run_speed_test_queue(pool, lines, process_id).await)
            } else {
                None
            }
        };

        let quota_future = async {
            if task_types.contains(&"quota_check".to_string()) {
                Some(Self::run_quota_check_queue(pool, settings, task, lines, process_id).await)
            } else {
                None
            }
        };

        tracing::info!(
            "[TaskService::execute] Starting parallel execution for task '{}'",
            task.name
        );

        // Execute both queues in parallel using tokio::join!, or cancel on token
        let execution_result = tokio::select! {
            results = async { tokio::join!(speed_future, quota_future) } => {
                Ok(results)
            }
            _ = cancel_token.cancelled() => {
                tracing::warn!("[TaskService::execute] Task '{}' was cancelled by user", task.name);
                Err(AppError::Validation("Task stopped by user".to_string()))
            }
        };

        // If cancelled, return error immediately
        let (speed_results, quota_results) = execution_result?;

        tracing::info!(
            "[TaskService::execute] Parallel execution completed for task '{}'",
            task.name
        );

        // Process speed test results
        if let Some(speed_data) = speed_results {
            let failed_count = speed_data.iter().filter(|r| r.status == "failed").count();
            let success_count = speed_data.iter().filter(|r| r.status == "success").count();

            if failed_count > 0 {
                tracing::warn!(
                    "[TaskService::execute] {} of {} speed tests failed",
                    failed_count,
                    speed_data.len()
                );
                has_failures = true;
            }
            if success_count > 0 {
                tracing::info!(
                    "[TaskService::execute] {} of {} speed tests succeeded",
                    success_count,
                    speed_data.len()
                );
            }

            Self::store_line_results(pool, execution_id, &speed_data, "speed_test").await;
            results.speed_test = Some(speed_data);
        }

        // Process quota check results
        if let Some(quota_data) = quota_results {
            let failed_count = quota_data.iter().filter(|r| r.status == "failed").count();
            let success_count = quota_data.iter().filter(|r| r.status == "success").count();

            if failed_count > 0 {
                tracing::warn!(
                    "[TaskService::execute] {} of {} quota checks failed",
                    failed_count,
                    quota_data.len()
                );
                has_failures = true;
            }
            if success_count > 0 {
                tracing::info!(
                    "[TaskService::execute] {} of {} quota checks succeeded",
                    success_count,
                    quota_data.len()
                );
            }

            Self::store_line_results(pool, execution_id, &quota_data, "quota_check").await;
            results.quota_check = Some(quota_data);
        }

        // Update status based on results: failed if any failures, completed otherwise
        // Note: DB constraint only allows: pending, running, completed, failed
        let final_status = if has_failures { "failed" } else { "completed" };

        // Update task status - this is critical, log but don't propagate error
        if let Err(e) = TaskRepository::update_status(pool, task.id, final_status).await {
            tracing::error!(
                "[TaskService::execute] Failed to update task '{}' status to '{}': {:?}",
                task.name, final_status, e
            );
            // Continue anyway - the result will still be returned
        }

        LogService::info(
            pool,
            process_id,
            "TaskService::execute",
            &format!("Task '{}' finished with status: {}", task.name, final_status),
        )
        .await
        .ok(); // Don't fail if logging fails

        let total_elapsed = (Utc::now() - started_at).num_milliseconds();
        tracing::info!("[TaskService::execute] Task '{}' finished with status '{}' in {}ms",
            task.name, final_status, total_elapsed);

        Ok(TaskExecutionResult {
            task_id: task.id,
            task_name: task.name.clone(),
            status: final_status.to_string(),
            results,
            started_at,
            finished_at: Some(Utc::now()),
        })
    }

    /// Convert Task to TaskResponse with populated lines
    async fn to_response(pool: &SqlitePool, task: Task) -> AppResult<TaskResponse> {
        // Get line IDs for this task
        let line_ids = TaskRepository::get_line_ids(pool, task.id).await?;

        // Fetch line details (no decryption needed - LineResponse excludes passwords)
        let mut lines = Vec::new();
        for line_id in &line_ids {
            if let Some(line) = LineRepository::get_by_id_raw(pool, *line_id as i32).await? {
                lines.push(LineResponse::from(line));
            }
        }

        // Parse schedule JSON if present
        let schedule: Option<Schedule> = if let Some(ref json) = task.schedule_json {
            Some(serde_json::from_str(json).map_err(|e| {
                AppError::Internal(format!("Failed to parse schedule JSON: {}", e))
            })?)
        } else {
            None
        };

        // Parse task_types JSON
        let task_types: Vec<String> = serde_json::from_str(&task.task_types)
            .map_err(|e| AppError::Internal(format!("Failed to parse task_types JSON: {}", e)))?;

        // Get last run time from latest execution
        let last_run_at = match TaskExecutionRepository::get_latest_for_task(pool, task.id).await {
            Ok(Some(exec)) => Some(exec.started_at),
            _ => None,
        };

        // Calculate next run time from schedule
        let next_run_at = if task.run_mode == "scheduled" && task.is_active {
            schedule.as_ref().and_then(Self::compute_next_run)
        } else {
            None
        };

        Ok(TaskResponse {
            id: task.id,
            name: task.name,
            task_types,
            run_mode: task.run_mode,
            schedule,
            status: task.status,
            is_active: task.is_active,
            show_browser: task.show_browser,
            line_ids,
            lines,
            last_run_at,
            next_run_at,
            created_at: task.created_at,
            updated_at: task.updated_at,
        })
    }

    /// Compute the next scheduled run time from a schedule definition.
    /// Returns an RFC3339 string of the next (day, time) occurrence.
    /// Schedule times are interpreted in the host's local timezone.
    fn compute_next_run(schedule: &Schedule) -> Option<String> {
        use chrono::{Datelike, Local, NaiveTime, TimeZone};

        if schedule.days.is_empty() || schedule.times.is_empty() {
            return None;
        }

        let now = Local::now();
        let current_day = now.weekday().num_days_from_sunday() as u8; // 0=Sun
        let current_time = now.time();

        // Parse all schedule times
        let mut parsed_times: Vec<NaiveTime> = schedule
            .times
            .iter()
            .filter_map(|t| NaiveTime::parse_from_str(t, "%H:%M").ok())
            .collect();
        parsed_times.sort();

        if parsed_times.is_empty() {
            return None;
        }

        // Check today and the next 7 days
        for day_offset in 0u8..8 {
            let check_day = (current_day + day_offset) % 7;
            if !schedule.days.contains(&check_day) {
                continue;
            }

            for time in &parsed_times {
                if day_offset == 0 && *time <= current_time {
                    continue; // Skip times already passed today
                }

                let target_date = now.date_naive() + chrono::Duration::days(day_offset as i64);
                let target_dt = target_date.and_time(*time);
                // Convert local datetime to UTC for RFC3339 output
                if let Some(local_dt) = Local.from_local_datetime(&target_dt).single() {
                    return Some(local_dt.with_timezone(&Utc).to_rfc3339());
                }
            }
        }

        None
    }

    /// Execute speed tests in a sequential queue (one at a time)
    async fn run_speed_test_queue(
        pool: &SqlitePool,
        lines: &[Line],
        process_id: Uuid,
    ) -> Vec<LineExecutionResult> {
        LogService::info(
            pool,
            process_id,
            "run_speed_test_queue",
            &format!("Starting speed test queue for {} lines (sequential)", lines.len()),
        )
        .await
        .ok();

        tracing::info!("[SpeedTestQueue] Starting {} speed tests (sequential)", lines.len());
        let queue_start = std::time::Instant::now();

        let results: Vec<LineExecutionResult> = stream::iter(lines.to_vec())
            .map(|line: Line| {
                let pool = pool.clone();
                async move { run_speed_test(&pool, &line, process_id).await }
            })
            .buffer_unordered(1)  // Sequential execution (1 at a time)
            .collect()
            .await;

        let elapsed = queue_start.elapsed();
        tracing::info!(
            "[SpeedTestQueue] Completed {} speed tests in {:?}",
            results.len(),
            elapsed
        );

        results
    }

    /// Execute quota checks sequentially (one browser at a time)
    /// This avoids Chrome's "Opening in existing browser session" issue
    async fn run_quota_check_queue(
        pool: &SqlitePool,
        settings: &Settings,
        task: &Task,
        lines: &[Line],
        process_id: Uuid,
    ) -> Vec<LineExecutionResult> {
        LogService::info(
            pool,
            process_id,
            "run_quota_check_queue",
            &format!("Starting quota check queue for {} lines (sequential)", lines.len()),
        )
        .await
        .ok();

        tracing::info!(
            "[QuotaCheckQueue] Starting {} quota checks (sequential - one at a time)",
            lines.len()
        );
        let queue_start = std::time::Instant::now();

        // Run sequentially - one browser at a time to avoid Chrome singleton issues
        let mut results: Vec<LineExecutionResult> = Vec::with_capacity(lines.len());
        let mut field_counts: Vec<usize> = Vec::with_capacity(lines.len());
        for (i, line) in lines.iter().enumerate() {
            tracing::info!(
                "[QuotaCheckQueue] Processing line {}/{}: '{}'",
                i + 1,
                lines.len(),
                line.name
            );
            let outcome = run_quota_check_core(pool, settings, line, process_id, task.show_browser).await;
            store_quota_result(pool, outcome.db_request, &line.name).await;
            field_counts.push(outcome.data_field_count);
            results.push(outcome.result);
        }

        // Retry failed lines exactly once, keeping the result with more data
        let failed_indices: Vec<usize> = results
            .iter()
            .enumerate()
            .filter(|(_, r)| r.status == "failed")
            .map(|(i, _)| i)
            .collect();

        if !failed_indices.is_empty() {
            tracing::info!(
                "[QuotaCheckQueue] Retrying {} failed quota checks (1 retry per line)",
                failed_indices.len()
            );

            for idx in failed_indices {
                let line = &lines[idx];
                tracing::info!(
                    "[QuotaCheckQueue] RETRY for line '{}' (original had {} data fields, error: {})",
                    line.name,
                    field_counts[idx],
                    results[idx].error_message.as_deref().unwrap_or("unknown error")
                );
                let retry_outcome = run_quota_check_core(pool, settings, line, process_id, task.show_browser).await;

                // Only use the retry result if it's actually better than the original
                if retry_outcome.result.status == "success" || retry_outcome.data_field_count > field_counts[idx] {
                    tracing::info!(
                        "[QuotaCheckQueue] RETRY BETTER for line '{}': {} fields vs {} (using retry)",
                        line.name,
                        retry_outcome.data_field_count,
                        field_counts[idx]
                    );
                    store_quota_result(pool, retry_outcome.db_request, &line.name).await;
                    field_counts[idx] = retry_outcome.data_field_count;
                    results[idx] = retry_outcome.result;
                } else {
                    tracing::info!(
                        "[QuotaCheckQueue] RETRY WORSE/SAME for line '{}': {} fields vs {} (keeping original)",
                        line.name,
                        retry_outcome.data_field_count,
                        field_counts[idx]
                    );
                    // Discard retry result - don't store to DB, keep original
                }
            }
        }

        let elapsed = queue_start.elapsed();
        let final_failures = results.iter().filter(|r| r.status == "failed").count();
        tracing::info!(
            "[QuotaCheckQueue] Completed {} quota checks in {:?} ({} final failures)",
            results.len(),
            elapsed,
            final_failures
        );

        results
    }
}

/// Run a speed test for a single line
async fn run_speed_test(pool: &SqlitePool, line: &Line, process_id: Uuid) -> LineExecutionResult {
    let started_at = Utc::now();

    LogService::info_for_line(
        pool,
        process_id,
        line.id,
        "run_speed_test",
        &format!("Starting speed test for line: {} (IP: {:?})", line.name, line.ip_address),
    )
    .await
    .ok();

    // Create SpeedTest client with source address binding if IP is configured
    let client_result = match &line.ip_address {
        Some(ip_str) => match ip_str.parse::<IpAddr>() {
            Ok(ip) => SpeedTestClient::with_source_address(ip),
            Err(_) => SpeedTestClient::new(),
        },
        None => SpeedTestClient::new(),
    };

    let mut client = match client_result {
        Ok(c) => c,
        Err(e) => {
            let completed_at = Utc::now();
            let duration_ms = (completed_at - started_at).num_milliseconds().max(0) as u64;
            LogService::error_for_line(
                pool,
                process_id,
                line.id,
                "run_speed_test",
                &format!("Failed to create SpeedTest client for {}: {:?}", line.name, e),
            )
            .await
            .ok();

            return LineExecutionResult {
                line_id: line.id as i64,
                line_name: line.name.clone(),
                task_type: "speed_test".to_string(),
                status: "failed".to_string(),
                error_message: Some(format!("Failed to create client: {}", e)),
                duration_ms,
                started_at,
                completed_at,
            };
        }
    };

    tracing::info!("[run_speed_test] Calling SpeedTestClient::run() for line='{}'", line.name);
    let run_start = std::time::Instant::now();

    match client.run().await {
        Ok(result) => {
            let completed_at = Utc::now();
            let duration_ms = (completed_at - started_at).num_milliseconds().max(0) as u64;
            let run_elapsed = run_start.elapsed();

            // Determine status based on result quality
            // Note: "degraded" is not a valid DB status, use "failed" for partial failures
            let (status, error_message) = if result.download_mbps == 0.0 && result.upload_mbps == 0.0 {
                // This shouldn't happen due to validation in SpeedTestClient::run(),
                // but handle it defensively
                ("failed".to_string(), Some("Both download and upload measurements returned 0 Mbps".to_string()))
            } else if result.download_mbps == 0.0 {
                ("failed".to_string(), Some(format!(
                    "Download measurement failed (0 Mbps, {} requests failed), upload succeeded with {:.2} Mbps",
                    result.download_requests_failed, result.upload_mbps
                )))
            } else if result.upload_mbps == 0.0 {
                ("failed".to_string(), Some(format!(
                    "Upload measurement failed (0 Mbps, {} requests failed), download succeeded with {:.2} Mbps",
                    result.upload_requests_failed, result.download_mbps
                )))
            } else {
                ("success".to_string(), None)
            };

            if status == "failed" && error_message.is_some() {
                tracing::warn!(
                    "[run_speed_test] Partial failure in {:?} for line='{}': {:.2} Mbps down, {:.2} Mbps up, {:.2} ms ping. Reason: {:?}",
                    run_elapsed, line.name, result.download_mbps, result.upload_mbps, result.ping_ms, error_message
                );
            } else {
                tracing::info!(
                    "[run_speed_test] Success in {:?} for line='{}': {:.2} Mbps down, {:.2} Mbps up, {:.2} ms ping",
                    run_elapsed, line.name, result.download_mbps, result.upload_mbps, result.ping_ms
                );
            }

            LogService::info_for_line(
                pool,
                process_id,
                line.id,
                "run_speed_test",
                &format!(
                    "Speed test completed for {}: {:.2} Mbps down, {:.2} Mbps up, {:.2} ms ping (status: {})",
                    line.name, result.download_mbps, result.upload_mbps, result.ping_ms, status
                ),
            )
            .await
            .ok();

            // Store the result
            let request = CreateSpeedTestResultRequest {
                line_id: line.id,
                process_id,
                download_speed: Some(result.download_mbps),
                upload_speed: Some(result.upload_mbps),
                ping: Some(result.ping_ms),
                server_name: Some(result.server_name),
                server_location: Some(result.server_location),
                public_ip: Some(result.public_ip),
                status: Some(status.clone()),
                error_message: error_message.clone(),
            };

            if let Err(e) = SpeedTestService::create(pool, request).await {
                tracing::error!("[run_speed_test] Failed to persist speed test result for line '{}': {:?}", line.name, e);
            }

            LineExecutionResult {
                line_id: line.id as i64,
                line_name: line.name.clone(),
                task_type: "speed_test".to_string(),
                status,
                error_message,
                duration_ms,
                started_at,
                completed_at,
            }
        }
        Err(e) => {
            let completed_at = Utc::now();
            let duration_ms = (completed_at - started_at).num_milliseconds().max(0) as u64;
            let run_elapsed = run_start.elapsed();

            tracing::error!("[run_speed_test] Failed after {:?} for line='{}': {:?}", run_elapsed, line.name, e);

            LogService::error_for_line(
                pool,
                process_id,
                line.id,
                "run_speed_test",
                &format!("Speed test failed for {}: {:?}", line.name, e),
            )
            .await
            .ok();

            // Store failed result
            let error_msg = format!("{}", e);
            let request = CreateSpeedTestResultRequest {
                line_id: line.id,
                process_id,
                download_speed: Some(0.0),
                upload_speed: Some(0.0),
                ping: Some(0.0),
                server_name: None,
                server_location: None,
                public_ip: None,
                status: Some("failed".to_string()),
                error_message: Some(error_msg.clone()),
            };

            if let Err(db_err) = SpeedTestService::create(pool, request).await {
                tracing::error!("[run_speed_test] Failed to persist failed speed test result for line '{}': {:?}", line.name, db_err);
            }

            LineExecutionResult {
                line_id: line.id as i64,
                line_name: line.name.clone(),
                task_type: "speed_test".to_string(),
                status: "failed".to_string(),
                error_message: Some(error_msg),
                duration_ms,
                started_at,
                completed_at,
            }
        }
    }
}

/// Run a quota check scrape for a single line without storing to DB.
/// Returns a QuotaCheckOutcome with the result, data quality score, and DB request.
async fn run_quota_check_core(
    pool: &SqlitePool,
    settings: &Settings,
    line: &Line,
    process_id: Uuid,
    show_browser: bool,
) -> QuotaCheckOutcome {
    let started_at = Utc::now();

    LogService::info_for_line(
        pool,
        process_id,
        line.id,
        "run_quota_check",
        &format!("Starting quota check for line: {}", line.name),
    )
    .await
    .ok();

    // Run browser operations directly (chaser-oxide is async)
    let scraping_result = scrape_quota_data_async(settings, line, show_browser).await;

    let completed_at = Utc::now();
    let duration_ms = (completed_at - started_at).num_milliseconds().max(0) as u64;

    let status = scraping_result.determine_status();
    let message = scraping_result.build_message();

    // Calculate total quota and usage percentage from used + remaining
    let (total, quota_percentage) = match (scraping_result.data.used_quota, scraping_result.data.remaining_quota) {
        (Some(used), Some(remaining)) => {
            let total = used + remaining;
            let percentage = if total > 0.0 {
                ((used / total) * 1000.0).round() / 10.0
            } else {
                0.0
            };
            (Some(total), Some(percentage))
        }
        _ => (scraping_result.data.total_quota, None),
    };

    let data_field_count = scraping_result.data.field_count();

    let db_request = CreateQuotaResultRequest {
        line_id: line.id,
        process_id,
        balance: scraping_result.data.balance,
        quota_percentage,
        used_quota: scraping_result.data.used_quota,
        remaining_quota: scraping_result.data.remaining_quota,
        total_quota: total,
        renewal_date: scraping_result.data.renewal_date,
        renewal_cost: scraping_result.data.renewal_cost,
        extra_quota: None,
        status: Some(status.to_string()),
        message: Some(message.clone()),
    };

    // Log based on status (both to console and database)
    match status {
        "success" => {
            tracing::info!("Quota check completed for {}: Balance={:?}, Used={:?}, Remaining={:?}",
                line.name, scraping_result.data.balance, scraping_result.data.used_quota,
                scraping_result.data.remaining_quota);
            LogService::info_for_line(pool, process_id, line.id, "run_quota_check",
                &format!("Quota check completed for {}: Balance={:?}, Used={:?}, Remaining={:?}",
                    line.name, scraping_result.data.balance, scraping_result.data.used_quota,
                    scraping_result.data.remaining_quota)).await.ok();
        }
        "partial_success" => {
            tracing::warn!("Quota check partially succeeded for {}: {}", line.name, message);
            LogService::warning_for_line(pool, process_id, line.id, "run_quota_check",
                &format!("Quota check partially succeeded for {}: {}", line.name, message)).await.ok();
        }
        _ => {
            tracing::error!("Quota check failed for {}: {}", line.name, message);
            LogService::error_for_line(pool, process_id, line.id, "run_quota_check",
                &format!("Quota check failed for {}: {}", line.name, message)).await.ok();
        }
    }

    // For task execution status: treat partial_success as "failed" (conservative approach)
    let execution_status = if status == "success" { "success" } else { "failed" };

    QuotaCheckOutcome {
        result: LineExecutionResult {
            line_id: line.id as i64,
            line_name: line.name.clone(),
            task_type: "quota_check".to_string(),
            status: execution_status.to_string(),
            error_message: if status != "success" { Some(message) } else { None },
            duration_ms,
            started_at,
            completed_at,
        },
        data_field_count,
        db_request,
    }
}

/// Store a quota check result to the database.
async fn store_quota_result(pool: &SqlitePool, request: CreateQuotaResultRequest, line_name: &str) {
    match QuotaCheckService::create(pool, request).await {
        Ok(result) => {
            tracing::info!("[run_quota_check] Successfully stored quota result for line '{}' (id: {})", line_name, result.id);
        }
        Err(e) => {
            tracing::error!("[run_quota_check] Failed to store quota result for line '{}': {:?}", line_name, e);
        }
    }
}

// ========== Quota Scraping Logic (Synchronous for headless_chrome) ==========

const LOGIN_URL: &str = "https://my.te.eg/user/login";
const OVERVIEW_URL: &str = "https://my.te.eg/offering/overview";
const RENEWAL_URL: &str = "https://my.te.eg/echannel/#/overview";

// CSS Selectors for the ISP portal
const BALANCE_SELECTOR: &str = "#_bes_window > main > div > div > div.ant-row > div:nth-child(2) > div > div > div > div > div:nth-child(3) > div:nth-child(1)";
const USED_SELECTOR: &str = "#_bes_window > main > div > div > div.ant-row > div.ant-col.ant-col-24 > div > div > div.ant-row.ec_accountoverview_primaryBtn_Qyg-Vp > div:nth-child(2) > div > div > div.slick-list > div > div.slick-slide.slick-active.slick-current > div > div > div > div > div:nth-child(2) > div:nth-child(2) > span:nth-child(1)";
const REMAINING_SELECTOR: &str = "#_bes_window > main > div > div > div.ant-row > div.ant-col.ant-col-24 > div > div > div.ant-row.ec_accountoverview_primaryBtn_Qyg-Vp > div:nth-child(2) > div > div > div.slick-list > div > div.slick-slide.slick-active.slick-current > div > div > div > div > div:nth-child(2) > div:nth-child(1) > span:nth-child(1)";
const RENEWAL_COST_SELECTOR: &str = "#_bes_window > main > div > div > div.ant-row > div.ant-col.ant-col-xs-24.ant-col-sm-24.ant-col-md-14.ant-col-lg-14.ant-col-xl-14 > div > div > div > div > div:nth-child(3) > div > span:nth-child(2) > div > div:nth-child(1)";
const RENEWAL_DATE_SELECTOR: &str = "#_bes_window > main > div > div > div.ant-row > div.ant-col.ant-col-xs-24.ant-col-sm-24.ant-col-md-14.ant-col-lg-14.ant-col-xl-14 > div > div > div > div > div:nth-child(4) > div > span";

// Login form selectors
const USERNAME_INPUT: &str = "#login_loginid_input_01";
const PASSWORD_INPUT: &str = "#login_password_input_01";
const LOGIN_TYPE_SELECTOR: &str = "#login_input_type_01";
const LOGIN_TYPE_OPTION: &str = ".ant-select-item-option-active .ant-space-item:nth-child(2) > span";
const LOGIN_BUTTON: &str = "#login-withecare";

// ========== Orange ISP ==========
const ORANGE_LOGIN_URL: &str = "https://www.orange.eg/ar/myaccount/login?ReturnUrl=%2far%2f";
const ORANGE_INTERNET_URL: &str = "https://www.orange.eg/ar/myaccount/internet";

// Orange login selectors
const ORANGE_USERNAME_INPUT: &str = "#PlaceHolderAppsHP_LoginControl_txtDialNumber";
const ORANGE_PASSWORD_INPUT: &str = "#PlaceHolderAppsHP_LoginControl_txtPassword";
const ORANGE_LOGIN_BUTTON: &str = "#GSM_Portal_Login_btnLogin";

// Orange data selectors
const ORANGE_TOTAL_QUOTA_SELECTOR: &str = ".total-consumption";
const ORANGE_USED_QUOTA_SELECTOR: &str = "p.m-0:nth-child(1) > span:nth-child(2)";
const ORANGE_RENEWAL_DATE_SELECTOR: &str = "p.m-0:nth-child(2) > span:nth-child(2)";

#[derive(Debug, Default, Clone)]
struct QuotaData {
    balance: Option<f64>,
    used_quota: Option<f64>,
    remaining_quota: Option<f64>,
    total_quota: Option<f64>,
    renewal_date: Option<chrono::NaiveDate>,
    renewal_cost: Option<f64>,
}

impl QuotaData {
    /// Count how many data fields are populated (non-None).
    /// Used to compare data quality between scraping attempts.
    fn field_count(&self) -> usize {
        [
            self.balance.is_some(),
            self.used_quota.is_some(),
            self.remaining_quota.is_some(),
            self.total_quota.is_some(),
            self.renewal_date.is_some(),
            self.renewal_cost.is_some(),
        ]
        .iter()
        .filter(|&&x| x)
        .count()
    }
}

/// Tracks which scraping steps completed successfully
#[derive(Debug, Default)]
struct ScrapingSteps {
    login: bool,
    overview: bool,
    renewal: bool,
}

impl ScrapingSteps {
    fn any_success(&self) -> bool {
        self.overview || self.renewal
    }

    fn all_success(&self) -> bool {
        self.login && self.overview && self.renewal
    }

    fn to_message(&self) -> String {
        let mut parts = Vec::new();
        if !self.login { parts.push("login failed"); }
        if !self.overview { parts.push("overview page failed"); }
        if !self.renewal { parts.push("renewal page failed"); }

        if parts.is_empty() {
            "All steps completed successfully".to_string()
        } else if self.any_success() {
            format!("Partial success: {}", parts.join(", "))
        } else {
            format!("Complete failure: {}", parts.join(", "))
        }
    }
}

/// Builder for quota scraping results with progressive data collection
#[derive(Debug, Default)]
struct QuotaScrapingResult {
    data: QuotaData,
    steps: ScrapingSteps,
    errors: Vec<String>,
}

impl QuotaScrapingResult {
    fn new() -> Self {
        Self::default()
    }

    fn determine_status(&self) -> &'static str {
        if self.steps.all_success() {
            "success"
        } else if self.steps.any_success() {
            "partial_success"
        } else {
            "failed"
        }
    }

    fn build_message(&self) -> String {
        let step_msg = self.steps.to_message();
        if self.errors.is_empty() {
            step_msg
        } else {
            format!("{}. Errors: {}", step_msg, self.errors.join("; "))
        }
    }
}

/// Result of a quota check scrape before DB storage.
/// Used to compare data quality between first attempt and retry.
struct QuotaCheckOutcome {
    result: LineExecutionResult,
    data_field_count: usize,
    db_request: CreateQuotaResultRequest,
}

/// Async quota scraping using chaser-oxide
async fn scrape_quota_data_async(settings: &Settings, line: &Line, show_browser: bool) -> QuotaScrapingResult {
    let start = std::time::Instant::now();
    let browser_mode = if show_browser { "visible" } else { "headless" };
    tracing::info!(
        "[QuotaScrape:{}] [t=0ms] === STARTING QUOTA CHECK === Creating new browser instance ({})",
        line.name,
        browser_mode
    );

    // Detect service mode for logging and browser selection
    #[cfg(all(windows, feature = "service"))]
    let service_mode = crate::config::paths::is_service_mode();
    #[cfg(not(all(windows, feature = "service")))]
    let service_mode = false;

    // Log execution context for diagnostics
    tracing::info!(
        "[QuotaScrape:{}] Execution context: service_mode={}, temp_dir={:?}",
        line.name,
        service_mode,
        std::env::temp_dir()
    );

    // Create WebDriver with headless mode based on task setting
    // Note: WebDriverSettings.headless is overridden here
    let mut webdriver_settings = settings.webdriver.clone();
    webdriver_settings.headless = !show_browser;  // Invert: show_browser=true means headless=false

    // Detect execution context and use appropriate browser constructor
    #[cfg(all(windows, feature = "service"))]
    let driver = if service_mode {
        let worker_id = format!("task-{}", line.id);
        match WebDriverClient::new_for_service(&webdriver_settings, &worker_id).await {
            Ok(d) => d,
            Err(e) => {
                tracing::error!(
                    "[QuotaScrape:{}] [t={}ms] FAILED to create service browser: {}",
                    line.name,
                    start.elapsed().as_millis(),
                    e
                );
                let mut result = QuotaScrapingResult::new();
                result.errors.push(format!("Failed to create WebDriver: {}", e));
                return result;
            }
        }
    } else {
        match WebDriverClient::new_headless(&webdriver_settings).await {
            Ok(d) => d,
            Err(e) => {
                tracing::error!(
                    "[QuotaScrape:{}] [t={}ms] FAILED to create browser: {}",
                    line.name,
                    start.elapsed().as_millis(),
                    e
                );
                let mut result = QuotaScrapingResult::new();
                result.errors.push(format!("Failed to create WebDriver: {}", e));
                return result;
            }
        }
    };

    #[cfg(not(all(windows, feature = "service")))]
    let driver = match WebDriverClient::new_headless(&webdriver_settings).await {
        Ok(d) => d,
        Err(e) => {
            tracing::error!(
                "[QuotaScrape:{}] [t={}ms] FAILED to create browser: {}",
                line.name,
                start.elapsed().as_millis(),
                e
            );
            let mut result = QuotaScrapingResult::new();
            result.errors.push(format!("Failed to create WebDriver: {}", e));
            return result;
        }
    };

    let browser_id = driver.browser_id();
    let dlog = QuotaDebugLog::new(&line.name, &browser_id.to_string());

    dlog.entry("CONTEXT", &format!(
        "service_mode={} show_browser={} headless={} temp_dir={:?}",
        service_mode, show_browser, webdriver_settings.headless, std::env::temp_dir()
    ));

    tracing::info!(
        "[QuotaScrape:{}] [Browser {}] [t={}ms] Beginning quota check scraping",
        line.name,
        browser_id,
        start.elapsed().as_millis()
    );

    let scrape_start = std::time::Instant::now();
    let result = scrape_quota_data(&driver, line, &dlog).await;

    let status = result.determine_status();
    dlog.entry("SCRAPE_DONE", &format!(
        "status='{}' duration={}ms errors=[{}]",
        status,
        scrape_start.elapsed().as_millis(),
        result.errors.join("; ")
    ));

    tracing::info!(
        "[QuotaScrape:{}] [Browser {}] [t={}ms] Scraping completed in {}ms (status: {})",
        line.name,
        browser_id,
        start.elapsed().as_millis(),
        scrape_start.elapsed().as_millis(),
        status
    );

    // Close browser explicitly
    tracing::info!(
        "[QuotaScrape:{}] [Browser {}] [t={}ms] Closing browser",
        line.name,
        browser_id,
        start.elapsed().as_millis()
    );
    dlog.step_start("BROWSER_QUIT");
    match driver.quit().await {
        Ok(_) => {
            dlog.step_ok("BROWSER_QUIT", "closed cleanly");
            tracing::info!(
                "[QuotaScrape:{}] [Browser {}] [t={}ms] Browser closed successfully",
                line.name,
                browser_id,
                start.elapsed().as_millis()
            );
        }
        Err(e) => {
            dlog.step_err("BROWSER_QUIT", &format!("{}", e));
            tracing::warn!(
                "[QuotaScrape:{}] [Browser {}] [t={}ms] Failed to close browser gracefully: {}",
                line.name,
                browser_id,
                start.elapsed().as_millis(),
                e
            );
        }
    }

    tracing::info!(
        "[QuotaScrape:{}] [t={}ms] === QUOTA CHECK COMPLETE === Total duration: {}ms",
        line.name,
        start.elapsed().as_millis(),
        start.elapsed().as_millis()
    );

    dlog.end(status);
    result
}

/// ISP dispatch - routes to the appropriate scraper based on line.isp
async fn scrape_quota_data(driver: &WebDriverClient, line: &Line, dlog: &QuotaDebugLog) -> QuotaScrapingResult {
    dlog.entry("DISPATCH", &format!("ISP={:?}", line.isp));
    match line.isp.as_deref() {
        Some("Orange") => scrape_orange_quota_data(driver, line, dlog).await,
        _ => scrape_we_quota_data(driver, line, dlog).await, // Default to WE (backward compatible)
    }
}

/// Main scraping orchestrator for WE (Telecom Egypt) - always returns a result, even on partial failure
async fn scrape_we_quota_data(driver: &WebDriverClient, line: &Line, dlog: &QuotaDebugLog) -> QuotaScrapingResult {
    let mut result = QuotaScrapingResult::new();

    // Step 1: Login (required for all subsequent steps)
    dlog.step_start("WE_LOGIN");
    match quota_login(driver, line, dlog).await {
        Ok(_) => {
            result.steps.login = true;
            dlog.step_ok("WE_LOGIN", "login succeeded");
            tracing::info!("Login step completed successfully");
        }
        Err(e) => {
            let error_msg = format!("Login failed: {}", e);
            dlog.step_err("WE_LOGIN", &error_msg);

            // Full diagnostic on login failure
            let login_selectors: &[(&str, &str)] = &[
                ("USERNAME_INPUT", USERNAME_INPUT),
                ("PASSWORD_INPUT", PASSWORD_INPUT),
                ("LOGIN_TYPE_SELECTOR", LOGIN_TYPE_SELECTOR),
                ("LOGIN_BUTTON", LOGIN_BUTTON),
            ];
            crate::clients::quota_debug_log::diagnose_we_page(
                dlog, driver, "LOGIN_FAILURE", login_selectors
            ).await;

            tracing::error!("{}", error_msg);
            result.errors.push(error_msg);
            return result;
        }
    }

    // Step 2: Scrape overview page (independent from renewal page)
    dlog.step_start("WE_OVERVIEW");
    if let Err(e) = scrape_overview_page(driver, &mut result.data, dlog).await {
        let msg = format!("Overview page failed: {}", e);
        dlog.step_err("WE_OVERVIEW", &msg);
        result.errors.push(msg);
    } else {
        dlog.step_ok("WE_OVERVIEW", &format!(
            "balance={:?} used={:?} remaining={:?}",
            result.data.balance, result.data.used_quota, result.data.remaining_quota
        ));
        result.steps.overview = true;
    }

    // Step 3: Scrape renewal page (independent from overview page)
    dlog.step_start("WE_RENEWAL");
    if let Err(e) = scrape_renewal_page(driver, &mut result.data, dlog).await {
        let msg = format!("Renewal page failed: {}", e);
        dlog.step_err("WE_RENEWAL", &msg);
        result.errors.push(msg);
    } else {
        dlog.step_ok("WE_RENEWAL", &format!(
            "renewal_cost={:?} renewal_date={:?}",
            result.data.renewal_cost, result.data.renewal_date
        ));
        result.steps.renewal = true;
    }

    result
}

async fn quota_login(driver: &WebDriverClient, line: &Line, dlog: &QuotaDebugLog) -> AppResult<()> {
    let step_start = std::time::Instant::now();
    tracing::info!("[{}] [LOGIN] Starting login process", line.name);

    // Navigate to login page
    dlog.nav(LOGIN_URL);
    driver.navigate(LOGIN_URL).await?;
    tracing::info!("[{}] [LOGIN] [t={}ms] Login page loaded",
        line.name, step_start.elapsed().as_millis());
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Snapshot page state after navigation
    dlog.snapshot(driver, "after_login_nav").await;

    tracing::debug!("Waiting for username input element: {}", USERNAME_INPUT);
    dlog.element_action("WAIT_FOR", USERNAME_INPUT);

    // Probe the username input before waiting
    dlog.probe_selector(driver, USERNAME_INPUT).await;

    // If element not found, get page info for debugging
    if let Err(e) = driver.wait_for_element(USERNAME_INPUT, 10).await {
        dlog.element_result("WAIT_FOR", USERNAME_INPUT, &format!("TIMEOUT: {}", e));

        tracing::error!("Username input not found! Checking page state...");

        // Get current URL
        if let Ok(url) = driver.get_current_url().await {
            tracing::error!("Current URL when login failed: {}", url);
            dlog.nav_done(&url);
        }

        // Get page title
        if let Ok(title) = driver.get_title().await {
            tracing::error!("Page title when login failed: {}", title);
        }

        // Try to get page body text to see what's actually loaded
        let body_check_script = r#"
            document.body ? document.body.innerText.substring(0, 500) : "NO BODY"
        "#;
        if let Ok(result) = driver.execute_script(body_check_script).await {
            tracing::error!("Page body content (first 500 chars): {:?}", result);
        }

        // Full diagnostic dump
        dlog.screenshot(driver, "login_wait_failed").await;
        dlog.flush();

        return Err(e);
    }
    dlog.element_result("WAIT_FOR", USERNAME_INPUT, "found");

    // Enter username — type naturally first (anti-detection), then force React state sync
    tracing::debug!("Entering username");
    dlog.element_action("CLICK_AND_TYPE", USERNAME_INPUT);
    dlog.probe_selector(driver, USERNAME_INPUT).await;
    let username_result = driver.click_and_type(USERNAME_INPUT, &line.username).await;
    if let Err(ref e) = username_result {
        dlog.element_result("CLICK_AND_TYPE", USERNAME_INPUT, &format!("FAILED: {}", e));
        dlog.snapshot(driver, "username_type_failed").await;
        dlog.screenshot(driver, "username_type_failed").await;
        dlog.flush();
    } else {
        dlog.element_result("CLICK_AND_TYPE", USERNAME_INPUT, "OK");
    }
    username_result?;

    // Force React state sync for username (CDP type_str may not trigger React onChange)
    dlog.element_action("REACT_SET_VALUE", USERNAME_INPUT);
    if let Err(e) = driver.set_react_input_value(USERNAME_INPUT, &line.username).await {
        dlog.element_result("REACT_SET_VALUE", USERNAME_INPUT, &format!("FAILED: {}", e));
        tracing::warn!("React value sync failed for username (continuing): {}", e);
    } else {
        dlog.element_result("REACT_SET_VALUE", USERNAME_INPUT, "OK");
    }

    // Conditionally select login type (WE serves different page versions — some have this dropdown, some don't)
    dlog.step_start("LOGIN_TYPE_CHECK");
    let login_type_script = format!(
        "document.querySelector('{}') !== null",
        LOGIN_TYPE_SELECTOR.replace('\'', "\\'")
    );
    let has_login_type = match driver.execute_script(&login_type_script).await {
        Ok(val) => val.as_bool().unwrap_or(false),
        Err(_) => false,
    };
    if has_login_type {
        dlog.step_ok("LOGIN_TYPE_CHECK", "element present — selecting login type");
        dlog.element_action("CLICK", LOGIN_TYPE_SELECTOR);
        if let Err(e) = driver.click(LOGIN_TYPE_SELECTOR).await {
            dlog.element_result("CLICK", LOGIN_TYPE_SELECTOR, &format!("FAILED: {}", e));
            // Non-fatal: continue anyway — the default type may already be correct
            tracing::warn!("Failed to click login type selector (continuing): {}", e);
        } else {
            dlog.element_result("CLICK", LOGIN_TYPE_SELECTOR, "OK");
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            dlog.element_action("CLICK", LOGIN_TYPE_OPTION);
            if let Err(e) = driver.click(LOGIN_TYPE_OPTION).await {
                dlog.element_result("CLICK", LOGIN_TYPE_OPTION, &format!("FAILED: {}", e));
                tracing::warn!("Failed to click login type option (continuing): {}", e);
            } else {
                dlog.element_result("CLICK", LOGIN_TYPE_OPTION, "OK");
            }
        }
    } else {
        dlog.step_ok("LOGIN_TYPE_CHECK", "element absent — skipping login type selection");
        tracing::info!("[{}] Login type dropdown not present, skipping", line.name);
    }

    // Enter password — type naturally first, then force React state sync
    tracing::debug!("Entering password");
    dlog.element_action("CLICK_AND_TYPE", PASSWORD_INPUT);
    dlog.probe_selector(driver, PASSWORD_INPUT).await;
    let password_result = driver.click_and_type(PASSWORD_INPUT, &line.password).await;
    if let Err(ref e) = password_result {
        dlog.element_result("CLICK_AND_TYPE", PASSWORD_INPUT, &format!("FAILED: {}", e));
        dlog.snapshot(driver, "password_type_failed").await;
        dlog.screenshot(driver, "password_type_failed").await;
        dlog.flush();
    } else {
        dlog.element_result("CLICK_AND_TYPE", PASSWORD_INPUT, "OK");
    }
    password_result?;

    // Force React state sync for password
    dlog.element_action("REACT_SET_VALUE", PASSWORD_INPUT);
    if let Err(e) = driver.set_react_input_value(PASSWORD_INPUT, &line.password).await {
        dlog.element_result("REACT_SET_VALUE", PASSWORD_INPUT, &format!("FAILED: {}", e));
        tracing::warn!("React value sync failed for password (continuing): {}", e);
    } else {
        dlog.element_result("REACT_SET_VALUE", PASSWORD_INPUT, "OK");
    }

    // Wait for login button to become enabled (form validation must pass first)
    dlog.element_action("WAIT_ENABLED", LOGIN_BUTTON);
    match driver.wait_for_element_enabled(LOGIN_BUTTON, 5).await {
        Ok(_) => {
            dlog.element_result("WAIT_ENABLED", LOGIN_BUTTON, "enabled");
        }
        Err(e) => {
            dlog.element_result("WAIT_ENABLED", LOGIN_BUTTON, &format!("STILL DISABLED: {}", e));
            dlog.probe_selector(driver, LOGIN_BUTTON).await;
            dlog.screenshot(driver, "login_button_disabled").await;
            tracing::warn!("[{}] Login button still disabled after waiting — clicking anyway", line.name);
        }
    }

    // Click login button
    tracing::debug!("Clicking login button");
    dlog.element_action("CLICK_HUMAN", LOGIN_BUTTON);
    dlog.probe_selector(driver, LOGIN_BUTTON).await;
    let login_click_result = driver.click_human(LOGIN_BUTTON).await;
    if let Err(ref e) = login_click_result {
        dlog.element_result("CLICK_HUMAN", LOGIN_BUTTON, &format!("FAILED: {}", e));
        dlog.screenshot(driver, "login_button_failed").await;
        dlog.flush();
    } else {
        dlog.element_result("CLICK_HUMAN", LOGIN_BUTTON, "OK");
    }
    login_click_result?;

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Post-login snapshot
    dlog.snapshot(driver, "after_login_submit").await;
    dlog.screenshot(driver, "after_login_submit").await;

    // Verify login actually succeeded by checking URL moved away from #/login
    if let Ok(url) = driver.get_current_url().await {
        if url.contains("#/login") {
            tracing::error!("[{}] Login verification FAILED — still on login page: {}", line.name, url);
            dlog.step_err("LOGIN_VERIFY", &format!("still on login page: {}", url));
            dlog.flush();
            return Err(crate::errors::AppError::WebDriver(
                format!("Login failed for '{}' — page stayed at login URL: {}", line.name, url)
            ));
        }
        tracing::info!("[{}] Login verified — redirected to: {}", line.name, url);
    }

    tracing::info!("Login successful for {}", line.name);
    dlog.flush();

    Ok(())
}

async fn scrape_overview_page(driver: &WebDriverClient, data: &mut QuotaData, dlog: &QuotaDebugLog) -> AppResult<()> {
    tracing::info!("Navigating to overview page");
    dlog.nav(OVERVIEW_URL);
    driver.navigate(OVERVIEW_URL).await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    dlog.snapshot(driver, "after_overview_nav").await;

    dlog.element_action("WAIT_FOR", BALANCE_SELECTOR);
    if let Err(e) = driver.wait_for_element(BALANCE_SELECTOR, 7).await {
        dlog.element_result("WAIT_FOR", BALANCE_SELECTOR, &format!("TIMEOUT: {}", e));
        let overview_selectors: &[(&str, &str)] = &[
            ("BALANCE", BALANCE_SELECTOR),
            ("USED", USED_SELECTOR),
            ("REMAINING", REMAINING_SELECTOR),
        ];
        crate::clients::quota_debug_log::diagnose_we_page(
            dlog, driver, "OVERVIEW_WAIT_FAILED", overview_selectors
        ).await;
        return Err(e);
    }
    dlog.element_result("WAIT_FOR", BALANCE_SELECTOR, "found");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let mut extraction_errors = Vec::new();

    // Extract balance
    dlog.probe_selector(driver, BALANCE_SELECTOR).await;
    match driver.get_text(BALANCE_SELECTOR).await {
        Ok(text) => {
            dlog.entry("EXTRACT", &format!("BALANCE raw='{}' parsed={:?}", text, parse_number(&text)));
            data.balance = parse_number(&text);
        }
        Err(e) => {
            extraction_errors.push("balance");
            dlog.entry("EXTRACT", &format!("BALANCE FAILED: {}", e));
            tracing::warn!("Failed to extract balance: {}", e);
        }
    }

    // Extract used quota
    dlog.probe_selector(driver, USED_SELECTOR).await;
    match driver.get_text(USED_SELECTOR).await {
        Ok(text) => {
            dlog.entry("EXTRACT", &format!("USED raw='{}' parsed={:?}", text, parse_number(&text)));
            data.used_quota = parse_number(&text);
        }
        Err(e) => {
            extraction_errors.push("used_quota");
            dlog.entry("EXTRACT", &format!("USED FAILED: {}", e));
            tracing::warn!("Failed to extract used quota: {}", e);
        }
    }

    // Extract remaining quota
    dlog.probe_selector(driver, REMAINING_SELECTOR).await;
    match driver.get_text(REMAINING_SELECTOR).await {
        Ok(text) => {
            dlog.entry("EXTRACT", &format!("REMAINING raw='{}' parsed={:?}", text, parse_number(&text)));
            data.remaining_quota = parse_number(&text);
        }
        Err(e) => {
            extraction_errors.push("remaining_quota");
            dlog.entry("EXTRACT", &format!("REMAINING FAILED: {}", e));
            tracing::warn!("Failed to extract remaining quota: {}", e);
        }
    }

    // Calculate total quota and percentage immediately after extraction
    if let (Some(used), Some(remaining)) = (data.used_quota, data.remaining_quota) {
        let total = used + remaining;
        data.total_quota = Some(total);
        let percentage = (used / total) * 100.0;
        dlog.entry("CALC", &format!(
            "Used={:.2} Remaining={:.2} Total={:.2} Usage={:.2}%",
            used, remaining, total, percentage
        ));
        tracing::info!(
            "Calculated quota totals: Used={:.2} GB, Remaining={:.2} GB, Total={:.2} GB, Usage={:.2}%",
            used, remaining, total, percentage
        );
    }

    tracing::info!(
        "Extracted: Balance={:?}, Used={:?}, Remaining={:?}, Total={:?}",
        data.balance, data.used_quota, data.remaining_quota, data.total_quota
    );

    dlog.screenshot(driver, "overview_extracted").await;

    // Success if we got at least one field
    if data.balance.is_some() || data.used_quota.is_some() || data.remaining_quota.is_some() {
        Ok(())
    } else {
        let err_msg = format!("Failed to extract any data: {}", extraction_errors.join(", "));
        dlog.entry("OVERVIEW_FAIL", &err_msg);
        Err(AppError::WebDriver(err_msg))
    }
}

async fn scrape_renewal_page(driver: &WebDriverClient, data: &mut QuotaData, dlog: &QuotaDebugLog) -> AppResult<()> {
    tracing::info!("Navigating to renewal page");

    // CRITICAL: Use JavaScript for hash-based navigation to avoid CDP timeout
    dlog.nav(RENEWAL_URL);
    dlog.entry("NAV_METHOD", "JavaScript window.location.href (hash-based SPA)");
    let nav_script = format!("window.location.href = '{}';", RENEWAL_URL);
    driver.execute_script(&nav_script).await?;

    // Give SPA time to initialize and route to the hash URL
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    dlog.snapshot(driver, "after_renewal_nav").await;

    tracing::info!("Waiting for renewal page elements to load");
    dlog.element_action("WAIT_FOR", RENEWAL_COST_SELECTOR);
    if let Err(e) = driver.wait_for_element(RENEWAL_COST_SELECTOR, 10).await {
        dlog.element_result("WAIT_FOR", RENEWAL_COST_SELECTOR, &format!("TIMEOUT: {}", e));
        let renewal_selectors: &[(&str, &str)] = &[
            ("RENEWAL_COST", RENEWAL_COST_SELECTOR),
            ("RENEWAL_DATE", RENEWAL_DATE_SELECTOR),
        ];
        crate::clients::quota_debug_log::diagnose_we_page(
            dlog, driver, "RENEWAL_WAIT_FAILED", renewal_selectors
        ).await;
        return Err(e);
    }
    dlog.element_result("WAIT_FOR", RENEWAL_COST_SELECTOR, "found");

    // Additional delay to ensure all elements are rendered
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let mut extraction_errors = Vec::new();

    tracing::info!("Extracting renewal cost");
    dlog.probe_selector(driver, RENEWAL_COST_SELECTOR).await;
    match driver.get_text(RENEWAL_COST_SELECTOR).await {
        Ok(text) => {
            dlog.entry("EXTRACT", &format!("RENEWAL_COST raw='{}' parsed={:?}", text, parse_number(&text)));
            tracing::debug!("Raw renewal cost text: '{}'", text);
            data.renewal_cost = parse_number(&text);
        }
        Err(e) => {
            extraction_errors.push("renewal_cost");
            dlog.entry("EXTRACT", &format!("RENEWAL_COST FAILED: {}", e));
            tracing::warn!("Failed to extract renewal cost: {}", e);
        }
    }

    tracing::info!("Extracting renewal date");
    dlog.probe_selector(driver, RENEWAL_DATE_SELECTOR).await;
    match driver.get_text(RENEWAL_DATE_SELECTOR).await {
        Ok(text) => {
            dlog.entry("EXTRACT", &format!("RENEWAL_DATE raw='{}' parsed={:?}", text, parse_renewal_date(&text)));
            tracing::debug!("Raw renewal date text: '{}'", text);
            data.renewal_date = parse_renewal_date(&text);
        }
        Err(e) => {
            extraction_errors.push("renewal_date");
            dlog.entry("EXTRACT", &format!("RENEWAL_DATE FAILED: {}", e));
            tracing::warn!("Failed to extract renewal date: {}", e);
        }
    }

    tracing::info!(
        "Extracted: Renewal Cost={:?}, Renewal Date={:?}",
        data.renewal_cost, data.renewal_date
    );

    dlog.screenshot(driver, "renewal_extracted").await;

    // Success if we got at least one field
    if data.renewal_cost.is_some() || data.renewal_date.is_some() {
        Ok(())
    } else {
        let err_msg = format!("Failed to extract any data: {}", extraction_errors.join(", "));
        dlog.entry("RENEWAL_FAIL", &err_msg);
        Err(AppError::WebDriver(err_msg))
    }
}

fn parse_number(text: &str) -> Option<f64> {
    let cleaned = text.replace(',', "").trim().to_string();
    cleaned.parse().ok()
}

fn parse_renewal_date(text: &str) -> Option<chrono::NaiveDate> {
    // Format: "Renewal Date: DD-MM-YYYY, X Remaining Days"
    if let Some(start) = text.find("Renewal Date: ") {
        let start = start + "Renewal Date: ".len();
        if text.len() >= start + 10 {
            let date_str = &text[start..start + 10];
            return chrono::NaiveDate::parse_from_str(date_str, "%d-%m-%Y").ok();
        }
    }
    None
}

// ========== Orange ISP Scraping Functions ==========

/// Main scraping orchestrator for Orange - always returns a result, even on partial failure
async fn scrape_orange_quota_data(driver: &WebDriverClient, line: &Line, dlog: &QuotaDebugLog) -> QuotaScrapingResult {
    let mut result = QuotaScrapingResult::new();

    // Step 1: Login (required for all subsequent steps)
    dlog.step_start("ORANGE_LOGIN");
    match orange_login(driver, line, dlog).await {
        Ok(_) => {
            result.steps.login = true;
            dlog.step_ok("ORANGE_LOGIN", "login succeeded");
            tracing::info!("[Orange] Login step completed successfully for line '{}'", line.name);
        }
        Err(e) => {
            let error_msg = format!("Orange login failed: {}", e);
            dlog.step_err("ORANGE_LOGIN", &error_msg);
            tracing::error!("{}", error_msg);
            result.errors.push(error_msg);
            return result; // Cannot proceed without login
        }
    }

    // Step 2: Scrape internet page (Orange gets renewal date from the internet page, no separate step)
    dlog.step_start("ORANGE_INTERNET");
    match scrape_orange_internet_page(driver, &mut result.data, dlog).await {
        Ok(_) => {
            result.steps.overview = true;
            result.steps.renewal = true; // Orange gets renewal from internet page
            dlog.step_ok("ORANGE_INTERNET", &format!(
                "total={:?} used={:?} renewal_date={:?}",
                result.data.total_quota, result.data.used_quota, result.data.renewal_date
            ));
            tracing::info!("[Orange] Internet page scraping completed successfully for line '{}'", line.name);
        }
        Err(e) => {
            dlog.step_err("ORANGE_INTERNET", &format!("{}", e));
            result.errors.push(format!("Orange internet page failed: {}", e));
        }
    }

    result
}

async fn orange_login(driver: &WebDriverClient, line: &Line, dlog: &QuotaDebugLog) -> AppResult<()> {
    let step_start = std::time::Instant::now();
    tracing::info!("[Orange] [LOGIN] Starting login process for line '{}'", line.name);

    dlog.nav(ORANGE_LOGIN_URL);
    driver.navigate(ORANGE_LOGIN_URL).await?;
    tracing::info!("[Orange] [LOGIN] [t={}ms] Login page loaded for line '{}'",
        step_start.elapsed().as_millis(), line.name);
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    dlog.snapshot(driver, "after_orange_login_nav").await;

    // Wait for username input element with timeout
    dlog.element_action("WAIT_FOR", ORANGE_USERNAME_INPUT);
    if let Err(e) = driver.wait_for_element(ORANGE_USERNAME_INPUT, 10).await {
        dlog.element_result("WAIT_FOR", ORANGE_USERNAME_INPUT, &format!("TIMEOUT: {}", e));
        dlog.screenshot(driver, "orange_login_wait_failed").await;
        dlog.flush();
        return Err(e);
    }
    dlog.element_result("WAIT_FOR", ORANGE_USERNAME_INPUT, "found");

    // Enter username
    dlog.element_action("CLICK_AND_TYPE", ORANGE_USERNAME_INPUT);
    driver.click_and_type(ORANGE_USERNAME_INPUT, &line.username).await?;
    dlog.element_result("CLICK_AND_TYPE", ORANGE_USERNAME_INPUT, "OK");

    // Enter password
    dlog.element_action("CLICK_AND_TYPE", ORANGE_PASSWORD_INPUT);
    driver.click_and_type(ORANGE_PASSWORD_INPUT, &line.password).await?;
    dlog.element_result("CLICK_AND_TYPE", ORANGE_PASSWORD_INPUT, "OK");

    // Click login button
    dlog.element_action("CLICK_HUMAN", ORANGE_LOGIN_BUTTON);
    driver.click_human(ORANGE_LOGIN_BUTTON).await?;
    dlog.element_result("CLICK_HUMAN", ORANGE_LOGIN_BUTTON, "OK");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Wait for URL to change away from the login page
    let login_start = std::time::Instant::now();
    let max_wait = std::time::Duration::from_secs(15);

    while login_start.elapsed() < max_wait {
        if let Ok(url) = driver.get_current_url().await {
            if !url.contains("login") {
                dlog.nav_done(&url);
                tracing::info!("[Orange] Login successful for line '{}', redirected to: {}", line.name, url);
                dlog.screenshot(driver, "orange_after_login").await;
                return Ok(());
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    dlog.entry("WARN", "URL did not change after login within 15s timeout");
    dlog.snapshot(driver, "orange_login_timeout").await;
    dlog.screenshot(driver, "orange_login_timeout").await;
    tracing::warn!("[Orange] Login may have succeeded but URL didn't change within timeout for line '{}'", line.name);
    Ok(())
}

async fn scrape_orange_internet_page(driver: &WebDriverClient, data: &mut QuotaData, dlog: &QuotaDebugLog) -> AppResult<()> {
    tracing::info!("[Orange] Navigating to internet page");
    dlog.nav(ORANGE_INTERNET_URL);
    driver.navigate(ORANGE_INTERNET_URL).await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    dlog.snapshot(driver, "after_orange_internet_nav").await;

    // Wait for total quota element with timeout
    dlog.element_action("WAIT_FOR", ORANGE_TOTAL_QUOTA_SELECTOR);
    if let Err(e) = driver.wait_for_element(ORANGE_TOTAL_QUOTA_SELECTOR, 15).await {
        dlog.element_result("WAIT_FOR", ORANGE_TOTAL_QUOTA_SELECTOR, &format!("TIMEOUT: {}", e));
        dlog.screenshot(driver, "orange_internet_wait_failed").await;
        dlog.flush();
        return Err(e);
    }
    dlog.element_result("WAIT_FOR", ORANGE_TOTAL_QUOTA_SELECTOR, "found");

    // Additional delay to ensure all elements are rendered
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let mut extraction_errors = Vec::new();

    // Extract total quota
    match driver.get_text(ORANGE_TOTAL_QUOTA_SELECTOR).await {
        Ok(text) => {
            dlog.entry("EXTRACT", &format!("[Orange] TOTAL_QUOTA raw='{}' parsed={:?}", text, parse_orange_quota(&text)));
            data.total_quota = parse_orange_quota(&text);
        }
        Err(e) => {
            extraction_errors.push("total_quota");
            dlog.entry("EXTRACT", &format!("[Orange] TOTAL_QUOTA FAILED: {}", e));
            tracing::warn!("[Orange] Failed to extract total quota: {}", e);
        }
    }

    // Extract used quota
    match driver.get_text(ORANGE_USED_QUOTA_SELECTOR).await {
        Ok(text) => {
            dlog.entry("EXTRACT", &format!("[Orange] USED_QUOTA raw='{}' parsed={:?}", text, parse_orange_quota(&text)));
            data.used_quota = parse_orange_quota(&text);
        }
        Err(e) => {
            extraction_errors.push("used_quota");
            dlog.entry("EXTRACT", &format!("[Orange] USED_QUOTA FAILED: {}", e));
            tracing::warn!("[Orange] Failed to extract used quota: {}", e);
        }
    }

    // Extract renewal date
    match driver.get_text(ORANGE_RENEWAL_DATE_SELECTOR).await {
        Ok(text) => {
            dlog.entry("EXTRACT", &format!("[Orange] RENEWAL_DATE raw='{}' parsed={:?}", text, parse_arabic_date(&text)));
            data.renewal_date = parse_arabic_date(&text);
        }
        Err(e) => {
            extraction_errors.push("renewal_date");
            dlog.entry("EXTRACT", &format!("[Orange] RENEWAL_DATE FAILED: {}", e));
            tracing::warn!("[Orange] Failed to extract renewal date: {}", e);
        }
    }

    // Calculate remaining quota if we have total and used
    if let (Some(total), Some(used)) = (data.total_quota, data.used_quota) {
        data.remaining_quota = Some((total - used).max(0.0));
        dlog.entry("CALC", &format!(
            "[Orange] Total={:.2} Used={:.2} Remaining={:.2}",
            total, used, data.remaining_quota.unwrap()
        ));
    }

    dlog.screenshot(driver, "orange_internet_extracted").await;

    // Success if we got at least one field
    if data.total_quota.is_some() || data.used_quota.is_some() || data.renewal_date.is_some() {
        Ok(())
    } else {
        let err_msg = format!("[Orange] Failed to extract any data: {}", extraction_errors.join(", "));
        dlog.entry("ORANGE_INTERNET_FAIL", &err_msg);
        Err(AppError::WebDriver(err_msg))
    }
}

// ========== Orange ISP Parsing Helpers ==========

/// Parse Orange quota values like "720GB" or "562863MBs"
/// Returns value in GB
fn parse_orange_quota(text: &str) -> Option<f64> {
    // Strip non-numeric chars (except .) to get the number
    let text = text.trim();
    let number_str: String = text
        .chars()
        .filter(|c| c.is_numeric() || *c == '.')
        .collect();

    let mut value: f64 = number_str.parse().ok()?;

    // If text contains "MB" (case-insensitive), divide by 1024 to convert to GB
    if text.to_uppercase().contains("MB") {
        value /= 1024.0;
    }

    Some(value)
}

/// Parse Arabic dates like "22 فبراير 2026"
fn parse_arabic_date(text: &str) -> Option<chrono::NaiveDate> {
    let text = text.trim();

    // Map Arabic month names to month numbers
    let arabic_months = [
        ("يناير", 1),   // January
        ("فبراير", 2),  // February
        ("مارس", 3),    // March
        ("أبريل", 4),   // April
        ("مايو", 5),    // May
        ("يونيو", 6),   // June
        ("يوليو", 7),   // July
        ("أغسطس", 8),   // August
        ("سبتمبر", 9),  // September
        ("أكتوبر", 10), // October
        ("نوفمبر", 11), // November
        ("ديسمبر", 12), // December
    ];

    // Split into parts: day, Arabic month name, year
    let parts: Vec<&str> = text.split_whitespace().collect();
    if parts.len() >= 3 {
        let day: u32 = parts[0].parse().ok()?;
        let year: i32 = parts[2].parse().ok()?;

        // Find month number from Arabic month name
        let month = arabic_months
            .iter()
            .find(|(name, _)| parts[1].contains(name))
            .map(|(_, num)| *num)?;

        chrono::NaiveDate::from_ymd_opt(year, month, day)
    } else {
        None
    }
}
