//! Execution Timeout Checker Job
//!
//! This job finds and marks executions that have exceeded their maximum_finish_time
//! as failed. This handles cases where:
//! - The app crashed during task execution
//! - A task hung indefinitely
//! - Network or browser issues caused a task to stall
//!
//! The job runs:
//! - On app startup (to clean up orphaned executions from previous sessions)
//! - Every 5 minutes (to catch newly timed-out executions)

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
    }

    Ok(())
}
