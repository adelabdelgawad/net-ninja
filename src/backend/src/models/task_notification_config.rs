use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskNotificationConfig {
    pub id: i32,
    pub task_id: i32,
    pub is_enabled: bool,
    pub smtp_config_id: Option<i32>,
    pub email_subject: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskNotificationConfigResponse {
    pub id: i32,
    pub task_id: i32,
    pub is_enabled: bool,
    pub smtp_config_id: Option<i32>,
    pub email_subject: Option<String>,
    pub to_recipient_ids: Vec<i32>,
    pub cc_recipient_ids: Vec<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpsertTaskNotificationConfigRequest {
    pub is_enabled: bool,
    pub smtp_config_id: Option<i32>,
    #[validate(length(max = 255))]
    pub email_subject: Option<String>,
    pub to_recipient_ids: Vec<i32>,
    pub cc_recipient_ids: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeNotificationConfigRequest {
    pub is_enabled: bool,
    pub smtp_config_id: Option<i32>,
    #[validate(length(max = 255))]
    pub email_subject: Option<String>,
    pub to_recipient_ids: Vec<i32>,
    pub cc_recipient_ids: Vec<i32>,
}

/// Request to resend a notification for a completed execution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResendNotificationRequest {
    pub execution_id: String,
    pub smtp_config_id: i32,
    pub email_subject: String,
    pub to_recipient_ids: Vec<i32>,
    pub cc_recipient_ids: Vec<i32>,
}
