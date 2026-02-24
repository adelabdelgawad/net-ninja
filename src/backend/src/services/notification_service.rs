use chrono::{DateTime, Utc};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

use crate::crypto::EncryptionKey;
use crate::errors::{AppError, AppResult};
use crate::models::{
    QuotaResult, QuotaResultRow, ResendNotificationRequest, RuntimeNotificationConfigRequest,
    SpeedTestResult, SpeedTestResultRow, Task, TaskExecutionResult, TaskNotificationConfigResponse,
};
use crate::repositories::{EmailRepository, TaskExecutionRepository, TaskNotificationConfigRepository, TaskRepository};
use crate::services::{LogService, SmtpConfigService};
use crate::templates::notification_email::{build_task_notification_email, LineInfo};

/// Speed test row with line info from JOIN (avoids N+1 queries)
#[derive(Debug, Clone, FromRow)]
struct SpeedTestResultRowWithName {
    pub id: i32,
    pub line_id: i32,
    pub process_id: String,
    pub download_speed: Option<f64>,
    pub upload_speed: Option<f64>,
    pub ping: Option<f64>,
    pub server_name: Option<String>,
    pub server_location: Option<String>,
    pub public_ip: Option<String>,
    pub status: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub line_name: String,
    pub line_number: String,
    pub line_isp: Option<String>,
    pub line_description: Option<String>,
}

/// Quota result row with line info from JOIN (avoids N+1 queries)
#[derive(Debug, Clone, FromRow)]
struct QuotaResultRowWithName {
    pub id: i32,
    pub line_id: i32,
    pub process_id: String,
    pub balance: Option<String>,
    pub quota_percentage: Option<String>,
    pub used_quota: Option<String>,
    pub total_quota: Option<String>,
    pub remaining_quota: Option<String>,
    pub renewal_date: Option<String>,
    pub renewal_cost: Option<String>,
    pub extra_quota: Option<String>,
    pub status: Option<String>,
    pub message: Option<String>,
    pub created_at: String,
    pub line_name: String,
    pub line_number: String,
    pub line_isp: Option<String>,
    pub line_description: Option<String>,
}

pub struct NotificationService;

impl NotificationService {
    /// Send task notification email
    ///
    /// # Arguments
    /// * `pool` - Database pool
    /// * `task` - The task that was executed
    /// * `execution_result` - Results from task execution
    /// * `notification_override` - Optional runtime notification config override
    /// * `process_id` - Process ID for logging
    /// * `encryption_key` - Optional encryption key for SMTP password decryption
    pub async fn send_task_notification(
        pool: &SqlitePool,
        task: &Task,
        execution_result: &TaskExecutionResult,
        notification_override: Option<RuntimeNotificationConfigRequest>,
        process_id: Uuid,
        encryption_key: Option<&EncryptionKey>,
    ) -> AppResult<()> {
        tracing::debug!(
            "[NotificationService] Entered send_task_notification for task '{}', override={}",
            task.name,
            notification_override.is_some()
        );

        // Determine which notification config to use
        let config = if let Some(override_config) = notification_override {
            // Use override config
            if !override_config.is_enabled {
                tracing::debug!(
                    "[NotificationService] Notifications disabled by override for task '{}'",
                    task.name
                );
                return Ok(());
            }

            // Convert RuntimeNotificationConfigRequest to TaskNotificationConfigResponse
            TaskNotificationConfigResponse {
                id: 0, // Not used for override
                task_id: task.id as i32,
                is_enabled: override_config.is_enabled,
                smtp_config_id: override_config.smtp_config_id,
                email_subject: override_config.email_subject,
                to_recipient_ids: override_config.to_recipient_ids,
                cc_recipient_ids: override_config.cc_recipient_ids,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }
        } else {
            // Use saved config
            tracing::debug!(
                "[NotificationService] Fetching notification config for task_id={}",
                task.id
            );

            let db_result = TaskNotificationConfigRepository::get_by_task_id(pool, task.id as i32).await;

            tracing::debug!(
                "[NotificationService] Database query returned: {}",
                if db_result.is_ok() { "Ok" } else { "Err" }
            );

            match db_result? {
                Some(saved_config) => {
                    tracing::debug!(
                        "[NotificationService] Found saved config id={}, is_enabled={}",
                        saved_config.id,
                        saved_config.is_enabled
                    );

                    tracing::debug!("[NotificationService] Fetching TO recipients...");
                    let to_recipient_ids =
                        TaskNotificationConfigRepository::get_to_recipients(pool, saved_config.id)
                            .await?;

                    tracing::debug!("[NotificationService] Fetching CC recipients...");
                    let cc_recipient_ids =
                        TaskNotificationConfigRepository::get_cc_recipients(pool, saved_config.id)
                            .await?;

                    tracing::debug!(
                        "[NotificationService] Retrieved {} TO and {} CC recipient IDs",
                        to_recipient_ids.len(),
                        cc_recipient_ids.len()
                    );

                    let response = TaskNotificationConfigResponse {
                        id: saved_config.id,
                        task_id: saved_config.task_id,
                        is_enabled: saved_config.is_enabled,
                        smtp_config_id: saved_config.smtp_config_id,
                        email_subject: saved_config.email_subject,
                        to_recipient_ids,
                        cc_recipient_ids,
                        created_at: saved_config.created_at,
                        updated_at: saved_config.updated_at,
                    };

                    if !response.is_enabled {
                        tracing::debug!(
                            "[NotificationService] Notifications disabled for task '{}'",
                            task.name
                        );
                        return Ok(());
                    }

                    response
                }
                None => {
                    tracing::debug!(
                        "[NotificationService] No notification config found for task '{}'",
                        task.name
                    );
                    return Ok(());
                }
            }
        };

        tracing::debug!(
            "[NotificationService] Final config: to_recipients={}, cc_recipients={}, smtp_config_id={:?}",
            config.to_recipient_ids.len(),
            config.cc_recipient_ids.len(),
            config.smtp_config_id
        );

        // Validate config has required fields
        if config.to_recipient_ids.is_empty() {
            tracing::debug!("[NotificationService] TO recipients is empty, exiting");
            LogService::warning(
                pool,
                process_id,
                "NotificationService",
                &format!("No TO recipients configured for task '{}', skipping notification", task.name),
            )
            .await
            .ok();
            return Ok(());
        }

        tracing::debug!("[NotificationService] TO recipients check passed");

        let smtp_config_id = match config.smtp_config_id {
            Some(id) => {
                tracing::debug!("[NotificationService] SMTP config ID found: {}", id);
                id
            }
            None => {
                // No SMTP config assigned, try to use default
                tracing::debug!("[NotificationService] No SMTP config assigned, attempting to use default");

                match crate::services::SmtpConfigService::get_default(pool, encryption_key).await {
                    Ok(default_smtp) => {
                        tracing::debug!(
                            "[NotificationService] Using default SMTP config: id={}, host={}",
                            default_smtp.id,
                            default_smtp.host
                        );
                        default_smtp.id
                    }
                    Err(e) => {
                        tracing::warn!(
                            "[NotificationService] No SMTP config assigned and no default found: {:?}",
                            e
                        );
                        LogService::warning(
                            pool,
                            process_id,
                            "NotificationService",
                            &format!(
                                "No SMTP config specified for task '{}' and no default SMTP config available, skipping notification",
                                task.name
                            ),
                        )
                        .await
                        .ok();
                        return Ok(());
                    }
                }
            }
        };

        LogService::info(
            pool,
            process_id,
            "NotificationService",
            &format!("Queuing notification for task '{}'", task.name),
        )
        .await
        .ok();

        // Fetch actual speed test and quota results from database
        let speed_tests = Self::fetch_speed_test_results(pool, process_id).await?;
        let quota_checks = Self::fetch_quota_results(pool, process_id).await?;

        // Build email body
        let email_body = Self::build_email_body(task, execution_result, &speed_tests, &quota_checks);

        // Get recipient email addresses
        let to_emails = Self::get_recipient_emails(pool, &config.to_recipient_ids).await?;
        let cc_emails = Self::get_recipient_emails(pool, &config.cc_recipient_ids).await?;

        if to_emails.is_empty() {
            LogService::warning(
                pool,
                process_id,
                "NotificationService",
                &format!("No valid TO recipient emails for task '{}', skipping notification", task.name),
            )
            .await
            .ok();
            return Ok(());
        }

        // Determine email subject
        let subject = config
            .email_subject
            .as_deref()
            .unwrap_or("NetNinja Task Results");

        LogService::info(
            pool,
            process_id,
            "NotificationService",
            &format!(
                "Sending notification to {} recipients (CC: {}) via SMTP config {}",
                to_emails.len(),
                cc_emails.len(),
                smtp_config_id
            ),
        )
        .await
        .ok();

        // Send email
        match SmtpConfigService::send_email(
            pool,
            smtp_config_id,
            &to_emails,
            &cc_emails,
            subject,
            &email_body,
            encryption_key,
        )
        .await
        {
            Ok(_) => {
                LogService::info(
                    pool,
                    process_id,
                    "NotificationService",
                    &format!("Notification sent successfully for task '{}'", task.name),
                )
                .await
                .ok();
                Ok(())
            }
            Err(e) => {
                LogService::error(
                    pool,
                    process_id,
                    "NotificationService",
                    &format!("Notification failed for task '{}': {}", task.name, e),
                )
                .await
                .ok();
                Err(AppError::Email(format!("Failed to send notification: {}", e)))
            }
        }
    }

    /// Resend notification email for a completed execution
    pub async fn resend_notification(
        pool: &SqlitePool,
        req: ResendNotificationRequest,
        encryption_key: Option<&EncryptionKey>,
    ) -> AppResult<()> {
        let process_id = uuid::Uuid::parse_str(&req.execution_id)
            .map_err(|e| AppError::Validation(format!("Invalid execution_id: {}", e)))?;

        // Get execution record to find the task
        let execution = TaskExecutionRepository::get_by_execution_id(pool, &req.execution_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Execution '{}' not found", req.execution_id)))?;

        // Get task
        let task = TaskRepository::get_by_id(pool, execution.task_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Task with id {} not found", execution.task_id)))?;

        // Parse started_at from execution record (SQLite stores as "YYYY-MM-DD HH:MM:SS")
        let started_at = chrono::DateTime::parse_from_rfc3339(&execution.started_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(&execution.started_at, "%Y-%m-%d %H:%M:%S")
                    .map(|ndt| ndt.and_utc())
                    .unwrap_or_else(|_| chrono::Utc::now())
            });

        // Fetch speed test and quota results by process_id
        let speed_tests = Self::fetch_speed_test_results(pool, process_id).await?;
        let quota_checks = Self::fetch_quota_results(pool, process_id).await?;

        // Build a minimal execution result for the email template
        let exec_result = TaskExecutionResult {
            task_id: execution.task_id,
            task_name: task.name.clone(),
            status: execution.status.clone(),
            results: crate::models::TaskTypeResults {
                speed_test: None,
                quota_check: None,
            },
            started_at,
            finished_at: execution.completed_at.as_ref().and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
        };

        // Build email body
        let email_body = Self::build_email_body(&task, &exec_result, &speed_tests, &quota_checks);

        // Get recipient email addresses
        let to_emails = Self::get_recipient_emails(pool, &req.to_recipient_ids).await?;
        let cc_emails = Self::get_recipient_emails(pool, &req.cc_recipient_ids).await?;

        if to_emails.is_empty() {
            return Err(AppError::Validation("No valid TO recipients".to_string()));
        }

        // Send email
        SmtpConfigService::send_email(
            pool,
            req.smtp_config_id,
            &to_emails,
            &cc_emails,
            &req.email_subject,
            &email_body,
            encryption_key,
        )
        .await
        .map_err(|e| AppError::Email(format!("Failed to send notification: {}", e)))?;

        tracing::info!(
            "[NotificationService] Resent notification for execution '{}', task '{}'",
            req.execution_id,
            task.name
        );

        Ok(())
    }

    /// Fetch speed test results from database by process_id
    async fn fetch_speed_test_results(
        pool: &SqlitePool,
        process_id: Uuid,
    ) -> AppResult<Vec<(LineInfo, SpeedTestResult)>> {
        // Use a single JOIN query to get both the speed test data and line info,
        // avoiding N+1 queries that would re-fetch each line individually.
        let rows = sqlx::query_as::<_, SpeedTestResultRowWithName>(
            r#"
            SELECT st.id, st.line_id, st.process_id, st.download_speed, st.upload_speed,
                   st.ping, st.server_name, st.server_location, st.public_ip,
                   st.status, st.error_message, st.created_at,
                   l.name as line_name, l.line_number, l.isp as line_isp,
                   l.description as line_description
            FROM speed_tests st
            JOIN lines l ON l.id = st.line_id
            WHERE st.process_id = ?
            ORDER BY l.name
            "#,
        )
        .bind(process_id.to_string())
        .fetch_all(pool)
        .await?;

        let results = rows
            .into_iter()
            .map(|row| {
                let info = LineInfo {
                    name: row.line_name.clone(),
                    number: row.line_number.clone(),
                    isp: row.line_isp.clone().unwrap_or_default(),
                    description: row.line_description.clone().unwrap_or_default(),
                };
                let speed_test_row = SpeedTestResultRow {
                    id: row.id,
                    line_id: row.line_id,
                    process_id: row.process_id,
                    download_speed: row.download_speed,
                    upload_speed: row.upload_speed,
                    ping: row.ping,
                    server_name: row.server_name,
                    server_location: row.server_location,
                    public_ip: row.public_ip,
                    status: row.status,
                    error_message: row.error_message,
                    created_at: row.created_at,
                };
                (info, speed_test_row.into())
            })
            .collect();

        Ok(results)
    }

    /// Fetch quota results from database by process_id
    async fn fetch_quota_results(
        pool: &SqlitePool,
        process_id: Uuid,
    ) -> AppResult<Vec<(LineInfo, QuotaResult)>> {
        // Use a single JOIN query to get both the quota data and line info,
        // avoiding N+1 queries that would re-fetch each line individually.
        let rows = sqlx::query_as::<_, QuotaResultRowWithName>(
            r#"
            SELECT qr.id, qr.line_id, qr.process_id, qr.balance, qr.quota_percentage,
                   qr.used_quota, qr.total_quota, qr.remaining_quota, qr.renewal_date,
                   qr.renewal_cost, qr.extra_quota, qr.status, qr.message, qr.created_at,
                   l.name as line_name, l.line_number, l.isp as line_isp,
                   l.description as line_description
            FROM quota_results qr
            JOIN lines l ON l.id = qr.line_id
            WHERE qr.process_id = ?
            ORDER BY l.name
            "#,
        )
        .bind(process_id.to_string())
        .fetch_all(pool)
        .await?;

        let results = rows
            .into_iter()
            .map(|row| {
                let info = LineInfo {
                    name: row.line_name.clone(),
                    number: row.line_number.clone(),
                    isp: row.line_isp.clone().unwrap_or_default(),
                    description: row.line_description.clone().unwrap_or_default(),
                };
                let quota_row = QuotaResultRow {
                    id: row.id,
                    line_id: row.line_id,
                    process_id: row.process_id,
                    balance: row.balance,
                    quota_percentage: row.quota_percentage,
                    used_quota: row.used_quota,
                    total_quota: row.total_quota,
                    remaining_quota: row.remaining_quota,
                    renewal_date: row.renewal_date,
                    renewal_cost: row.renewal_cost,
                    extra_quota: row.extra_quota,
                    status: row.status,
                    message: row.message,
                    created_at: row.created_at,
                };
                (info, quota_row.into())
            })
            .collect();

        Ok(results)
    }

    /// Build HTML email body from task results
    fn build_email_body(
        task: &Task,
        execution_result: &TaskExecutionResult,
        speed_tests: &[(LineInfo, SpeedTestResult)],
        quota_checks: &[(LineInfo, QuotaResult)],
    ) -> String {
        // Convert to format expected by template
        let speed_test_data: Vec<(LineInfo, Option<&SpeedTestResult>)> = speed_tests
            .iter()
            .map(|(info, result)| (info.clone(), Some(result)))
            .collect();

        let quota_check_data: Vec<(LineInfo, Option<&QuotaResult>)> = quota_checks
            .iter()
            .map(|(info, result)| (info.clone(), Some(result)))
            .collect();

        let speed_tests_opt = if speed_test_data.is_empty() {
            None
        } else {
            Some(speed_test_data.as_slice())
        };

        let quota_checks_opt = if quota_check_data.is_empty() {
            None
        } else {
            Some(quota_check_data.as_slice())
        };

        build_task_notification_email(
            &task.name,
            task.id,
            execution_result.started_at,
            speed_tests_opt,
            quota_checks_opt,
        )
    }

    /// Get recipient email addresses from IDs
    async fn get_recipient_emails(
        pool: &SqlitePool,
        recipient_ids: &[i32],
    ) -> AppResult<Vec<String>> {
        let mut emails = Vec::new();

        for &id in recipient_ids {
            if let Some(email_record) = EmailRepository::get_by_id(pool, id).await? {
                if email_record.is_active {
                    emails.push(email_record.email);
                }
            }
        }

        Ok(emails)
    }
}
