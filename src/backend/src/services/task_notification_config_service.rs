use crate::errors::AppResult;
use crate::models::{
    TaskNotificationConfigResponse, UpsertTaskNotificationConfigRequest,
};
use crate::repositories::TaskNotificationConfigRepository;
use sqlx::SqlitePool;

pub struct TaskNotificationConfigService;

impl TaskNotificationConfigService {
    pub async fn get_by_task_id(
        pool: &SqlitePool,
        task_id: i32,
    ) -> AppResult<Option<TaskNotificationConfigResponse>> {
        let config = TaskNotificationConfigRepository::get_by_task_id(pool, task_id).await?;

        match config {
            Some(cfg) => {
                let to_recipient_ids =
                    TaskNotificationConfigRepository::get_to_recipients(pool, cfg.id).await?;
                let cc_recipient_ids =
                    TaskNotificationConfigRepository::get_cc_recipients(pool, cfg.id).await?;

                Ok(Some(TaskNotificationConfigResponse {
                    id: cfg.id,
                    task_id: cfg.task_id,
                    is_enabled: cfg.is_enabled,
                    smtp_config_id: cfg.smtp_config_id,
                    email_subject: cfg.email_subject,
                    to_recipient_ids,
                    cc_recipient_ids,
                    created_at: cfg.created_at,
                    updated_at: cfg.updated_at,
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn upsert(
        pool: &SqlitePool,
        task_id: i32,
        req: UpsertTaskNotificationConfigRequest,
    ) -> AppResult<TaskNotificationConfigResponse> {
        // Validate request
        use validator::Validate;
        req.validate()?;

        // Upsert the config
        let config = TaskNotificationConfigRepository::upsert(pool, task_id, &req).await?;

        // Get full response with recipients
        let to_recipient_ids =
            TaskNotificationConfigRepository::get_to_recipients(pool, config.id).await?;
        let cc_recipient_ids =
            TaskNotificationConfigRepository::get_cc_recipients(pool, config.id).await?;

        Ok(TaskNotificationConfigResponse {
            id: config.id,
            task_id: config.task_id,
            is_enabled: config.is_enabled,
            smtp_config_id: config.smtp_config_id,
            email_subject: config.email_subject,
            to_recipient_ids,
            cc_recipient_ids,
            created_at: config.created_at,
            updated_at: config.updated_at,
        })
    }

    pub async fn delete_by_task_id(pool: &SqlitePool, task_id: i32) -> AppResult<()> {
        TaskNotificationConfigRepository::delete_by_task_id(pool, task_id).await
    }
}
