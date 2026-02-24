use crate::errors::AppResult;
use crate::models::{TaskNotificationConfig, UpsertTaskNotificationConfigRequest};
use sqlx::{Row, SqlitePool};

pub struct TaskNotificationConfigRepository;

impl TaskNotificationConfigRepository {
    pub async fn get_by_task_id(
        pool: &SqlitePool,
        task_id: i32,
    ) -> AppResult<Option<TaskNotificationConfig>> {
        let row = sqlx::query(
            r#"
            SELECT id, task_id, is_enabled, smtp_config_id,
                   email_subject, created_at, updated_at
            FROM task_notification_configs
            WHERE task_id = ?
            "#,
        )
        .bind(task_id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(r) => Ok(Some(TaskNotificationConfig {
                id: r.get("id"),
                task_id: r.get("task_id"),
                is_enabled: r.get::<i32, _>("is_enabled") != 0,
                smtp_config_id: r.get("smtp_config_id"),
                email_subject: r.get("email_subject"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            })),
            None => Ok(None),
        }
    }

    pub async fn get_to_recipients(pool: &SqlitePool, config_id: i32) -> AppResult<Vec<i32>> {
        let rows = sqlx::query(
            r#"
            SELECT email_id
            FROM task_notification_to_recipients
            WHERE task_notification_config_id = ?
            "#,
        )
        .bind(config_id)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.get("email_id")).collect())
    }

    pub async fn get_cc_recipients(pool: &SqlitePool, config_id: i32) -> AppResult<Vec<i32>> {
        let rows = sqlx::query(
            r#"
            SELECT email_id
            FROM task_notification_cc_recipients
            WHERE task_notification_config_id = ?
            "#,
        )
        .bind(config_id)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.get("email_id")).collect())
    }

    pub async fn upsert(
        pool: &SqlitePool,
        task_id: i32,
        req: &UpsertTaskNotificationConfigRequest,
    ) -> AppResult<TaskNotificationConfig> {
        let is_enabled_int = if req.is_enabled { 1 } else { 0 };

        // Upsert the main config
        sqlx::query(
            r#"
            INSERT INTO task_notification_configs (task_id, is_enabled, smtp_config_id, email_subject, updated_at)
            VALUES (?, ?, ?, ?, datetime('now', 'utc'))
            ON CONFLICT(task_id) DO UPDATE SET
                is_enabled = excluded.is_enabled,
                smtp_config_id = excluded.smtp_config_id,
                email_subject = excluded.email_subject,
                updated_at = datetime('now', 'utc')
            "#,
        )
        .bind(task_id)
        .bind(is_enabled_int)
        .bind(req.smtp_config_id)
        .bind(&req.email_subject)
        .execute(pool)
        .await?;

        // Get the config id
        let config = Self::get_by_task_id(pool, task_id)
            .await?
            .expect("Config should exist after upsert");

        // Delete existing recipients
        sqlx::query(
            "DELETE FROM task_notification_to_recipients WHERE task_notification_config_id = ?"
        )
        .bind(config.id)
        .execute(pool)
        .await?;

        sqlx::query(
            "DELETE FROM task_notification_cc_recipients WHERE task_notification_config_id = ?"
        )
        .bind(config.id)
        .execute(pool)
        .await?;

        // Insert new TO recipients
        for email_id in &req.to_recipient_ids {
            sqlx::query(
                "INSERT INTO task_notification_to_recipients (task_notification_config_id, email_id) VALUES (?, ?)"
            )
            .bind(config.id)
            .bind(email_id)
            .execute(pool)
            .await?;
        }

        // Insert new CC recipients
        for email_id in &req.cc_recipient_ids {
            sqlx::query(
                "INSERT INTO task_notification_cc_recipients (task_notification_config_id, email_id) VALUES (?, ?)"
            )
            .bind(config.id)
            .bind(email_id)
            .execute(pool)
            .await?;
        }

        Ok(config)
    }

    pub async fn delete_by_task_id(pool: &SqlitePool, task_id: i32) -> AppResult<()> {
        sqlx::query(
            "DELETE FROM task_notification_configs WHERE task_id = ?"
        )
        .bind(task_id)
        .execute(pool)
        .await?;

        Ok(())
    }
}
