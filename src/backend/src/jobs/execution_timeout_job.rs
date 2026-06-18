//! Execution Timeout Checker Job
//!
//! This job finds and marks executions that have exceeded their maximum_finish_time
//! as failed. This handles cases where:
//! - The app crashed during task execution
//! - A task hung indefinitely
//! - Network or browser issues caused a task to stall
//!
//! The job runs every 5 minutes (to catch newly timed-out executions).
//!
//! Note: orphaned executions from a previous session are NOT cleaned up by this
//! job at startup — that is handled separately by `reset_all_unfinished` /
//! `reset_all_running`, which every consumer runs on startup (service:
//! `service/scheduler.rs`, desktop: `bootstrap/standalone.rs`, web:
//! `app/src/startup.rs`). This job only sweeps executions that time out while the
//! process is live.

use sqlx::SqlitePool;

use crate::errors::AppResult;
use crate::repositories::{TaskExecutionRepository, TaskRepository};

/// Run the execution timeout check
///
/// Finds all executions that:
/// - Have is_finished = 0 (still running)
/// - Have maximum_finish_time in the past
///
/// For each, marks the execution as failed and updates the parent task status.
pub async fn run(pool: &SqlitePool) -> AppResult<()> {
    let timed_out = TaskExecutionRepository::find_timed_out_executions(pool).await?;

    if timed_out.is_empty() {
        tracing::debug!("No timed out executions found");
        return Ok(());
    }

    tracing::info!(
        "Found {} timed out execution(s), marking as failed",
        timed_out.len()
    );

    for execution in timed_out {
        tracing::warn!(
            "Marking execution {} (task_id={}) as timed out. Started: {}, Deadline: {}",
            execution.execution_id,
            execution.task_id,
            execution.started_at,
            execution.maximum_finish_time
        );

        // Mark execution as failed due to timeout
        match TaskExecutionRepository::mark_as_timed_out(pool, &execution.execution_id).await {
            Ok(true) => {
                tracing::info!(
                    "Successfully marked execution {} as timed out",
                    execution.execution_id
                );
            }
            Ok(false) => {
                tracing::debug!(
                    "Execution {} was already marked (race condition or concurrent update)",
                    execution.execution_id
                );
            }
            Err(e) => {
                tracing::error!(
                    "Failed to mark execution {} as timed out: {:?}",
                    execution.execution_id,
                    e
                );
                continue;
            }
        }

        // Update parent task status to 'failed'
        if let Err(e) = TaskRepository::update_status(pool, execution.task_id, "failed").await {
            tracing::error!(
                "Failed to update task {} status to 'failed': {:?}",
                execution.task_id,
                e
            );
        }

        // Best-effort: abort the actual in-process run. The DB flip above records
        // the timeout regardless, but without this a genuinely hung task (e.g. a
        // wedged browser scrape) keeps consuming resources. cancel() only finds
        // tasks running in *this* process's registry — which is the scheduler-
        // owning process that spawned the run — and is a no-op otherwise.
        if crate::services::task_runtime::cancel(execution.task_id).await {
            tracing::info!(
                "Requested cancellation of hung task {} (execution {})",
                execution.task_id,
                execution.execution_id
            );
        }
    }

    Ok(())
}
