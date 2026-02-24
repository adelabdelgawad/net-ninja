use sqlx::SqlitePool;

use crate::errors::AppResult;
use crate::models::Task;

pub struct TaskRepository;

impl TaskRepository {
    /// Get task by ID
    pub async fn get_by_id(pool: &SqlitePool, id: i64) -> AppResult<Option<Task>> {
        let task = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;
        Ok(task)
    }

    /// Get all tasks ordered by creation date (newest first)
    pub async fn get_all(pool: &SqlitePool) -> AppResult<Vec<Task>> {
        let tasks = sqlx::query_as::<_, Task>("SELECT * FROM tasks ORDER BY created_at DESC")
            .fetch_all(pool)
            .await?;
        Ok(tasks)
    }

    /// Create a new task
    pub async fn create(
        pool: &SqlitePool,
        name: &str,
        task_types: &str,  // JSON array string
        run_mode: &str,
        schedule_json: Option<&str>,
        show_browser: bool,
    ) -> AppResult<Task> {
        let task = sqlx::query_as::<_, Task>(
            r#"
            INSERT INTO tasks (name, task_types, run_mode, schedule_json, status, is_active, show_browser)
            VALUES ($1, $2, $3, $4, 'pending', 1, $5)
            RETURNING *
            "#
        )
        .bind(name)
        .bind(task_types)
        .bind(run_mode)
        .bind(schedule_json)
        .bind(show_browser as i32)
        .fetch_one(pool)
        .await?;
        Ok(task)
    }

    /// Update a task
    pub async fn update(
        pool: &SqlitePool,
        id: i64,
        name: Option<&str>,
        task_types: Option<&str>,
        run_mode: Option<&str>,
        schedule_json: Option<Option<&str>>,
        show_browser: Option<bool>,
    ) -> AppResult<Option<Task>> {
        tracing::debug!(
            task_id = id,
            name = name.unwrap_or("(unchanged)"),
            "TaskRepository::update called"
        );

        // Get current task to have a baseline
        let current = Self::get_by_id(pool, id).await?
            .ok_or_else(|| crate::errors::AppError::NotFound(format!("Task {} not found", id)))?;

        // Use provided values or fall back to current values
        let final_name = name.unwrap_or(&current.name);
        let final_task_types = task_types.unwrap_or(&current.task_types);
        let final_run_mode = run_mode.unwrap_or(&current.run_mode);
        let final_schedule_json = match schedule_json {
            Some(Some(sj)) => Some(sj),
            Some(None) => None,
            None => current.schedule_json.as_deref(),
        };
        let final_show_browser = show_browser.unwrap_or(current.show_browser);

        let task = sqlx::query_as::<_, Task>(
            r#"
            UPDATE tasks
            SET name = $1,
                task_types = $2,
                run_mode = $3,
                schedule_json = $4,
                show_browser = $5,
                status = 'pending',
                updated_at = datetime('now', 'utc')
            WHERE id = $6
            RETURNING *
            "#
        )
        .bind(final_name)
        .bind(final_task_types)
        .bind(final_run_mode)
        .bind(final_schedule_json)
        .bind(final_show_browser as i32)
        .bind(id)
        .fetch_optional(pool)
        .await?;

        match &task {
            Some(t) => {
                tracing::info!(
                    task_id = id,
                    name = %t.name,
                    "Task updated successfully"
                );
            }
            None => {
                tracing::warn!(
                    task_id = id,
                    "Task update returned no rows (task not found)"
                );
            }
        }

        Ok(task)
    }

    /// Toggle is_active flag
    pub async fn toggle_active(pool: &SqlitePool, id: i64, is_active: bool) -> AppResult<Option<Task>> {
        let task = sqlx::query_as::<_, Task>(
            "UPDATE tasks SET is_active = $1 WHERE id = $2 RETURNING *"
        )
        .bind(is_active as i32)
        .bind(id)
        .fetch_optional(pool)
        .await?;
        Ok(task)
    }

    /// Try to claim a scheduled execution slot (atomic duplicate prevention)
    pub async fn try_claim_scheduled_execution(
        pool: &SqlitePool,
        id: i64,
        scheduled_key: &str,
    ) -> AppResult<bool> {
        let result = sqlx::query(
            "UPDATE tasks SET last_scheduled_execution = $1
             WHERE id = $2 AND (last_scheduled_execution IS NULL OR last_scheduled_execution != $1)"
        )
        .bind(scheduled_key)
        .bind(id)
        .execute(pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Update task status
    pub async fn update_status(pool: &SqlitePool, id: i64, status: &str) -> AppResult<bool> {
        let result = sqlx::query("UPDATE tasks SET status = $1 WHERE id = $2")
            .bind(status)
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Reset all tasks that are stuck in "running" status to "failed".
    /// Called on startup since nothing can actually be running after a restart.
    pub async fn reset_all_running(pool: &SqlitePool) -> AppResult<u64> {
        let result = sqlx::query("UPDATE tasks SET status = 'failed' WHERE status = 'running'")
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    /// Check if task name exists (case-insensitive)
    pub async fn name_exists(pool: &SqlitePool, name: &str) -> AppResult<bool> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM tasks WHERE LOWER(TRIM(name)) = LOWER(TRIM($1))"
        )
        .bind(name)
        .fetch_one(pool)
        .await?;
        Ok(count.0 > 0)
    }

    /// Check if task name exists excluding a specific ID (for updates)
    pub async fn name_exists_excluding(pool: &SqlitePool, name: &str, exclude_id: i64) -> AppResult<bool> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM tasks WHERE LOWER(TRIM(name)) = LOWER(TRIM($1)) AND id != $2"
        )
        .bind(name)
        .bind(exclude_id)
        .fetch_one(pool)
        .await?;
        Ok(count.0 > 0)
    }

    /// Get line IDs for a task
    pub async fn get_line_ids(pool: &SqlitePool, task_id: i64) -> AppResult<Vec<i64>> {
        let line_ids: Vec<(i64,)> = sqlx::query_as(
            "SELECT line_id FROM task_lines WHERE task_id = $1 ORDER BY line_id"
        )
        .bind(task_id)
        .fetch_all(pool)
        .await?;
        Ok(line_ids.into_iter().map(|row| row.0).collect())
    }

    /// Add task-line associations
    pub async fn add_lines(pool: &SqlitePool, task_id: i64, line_ids: &[i64]) -> AppResult<()> {
        for line_id in line_ids {
            sqlx::query("INSERT INTO task_lines (task_id, line_id) VALUES ($1, $2)")
                .bind(task_id)
                .bind(line_id)
                .execute(pool)
                .await?;
        }
        Ok(())
    }

    /// Delete a task (cascade will remove task_lines)
    pub async fn delete(pool: &SqlitePool, id: i64) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM tasks WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
