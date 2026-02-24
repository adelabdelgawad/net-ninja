use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Email {
    pub id: i32,
    #[serde(rename = "recipient")]
    pub email: String,
    pub name: Option<String>,
    #[serde(rename = "isCc")]
    pub is_cc: bool,
    #[serde(rename = "isActive")]
    pub is_active: bool,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateEmailRequest {
    #[validate(email)]
    #[serde(rename = "recipient")]
    pub email: String,
    #[validate(length(max = 255))]
    pub name: Option<String>,
    #[serde(rename = "isCc")]
    pub is_cc: Option<bool>,
    #[serde(rename = "isActive")]
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateEmailRequest {
    #[validate(email)]
    #[serde(rename = "recipient")]
    pub email: Option<String>,
    #[validate(length(max = 255))]
    pub name: Option<String>,
    #[serde(rename = "isCc")]
    pub is_cc: Option<bool>,
    #[serde(rename = "isActive")]
    pub is_active: Option<bool>,
}
