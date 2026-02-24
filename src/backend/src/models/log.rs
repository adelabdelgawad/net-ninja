use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "log_level", rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

/// Raw database row (process_id stored as TEXT)
#[derive(Debug, Clone, FromRow)]
pub struct LogRow {
    pub id: i32,
    pub process_id: String,
    pub level: LogLevel,
    pub function_name: Option<String>,
    pub message: String,
    pub line_id: Option<i32>,
    pub created_at: DateTime<Utc>,
}

impl From<LogRow> for Log {
    fn from(row: LogRow) -> Self {
        Self {
            id: row.id,
            process_id: Uuid::parse_str(&row.process_id).unwrap_or_default(),
            level: row.level,
            function_name: row.function_name,
            message: row.message,
            line_id: row.line_id,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Log {
    pub id: i32,
    #[serde(rename = "processId")]
    pub process_id: Uuid,
    pub level: LogLevel,
    #[serde(rename = "function")]
    pub function_name: Option<String>,
    pub message: String,
    #[serde(rename = "lineId")]
    pub line_id: Option<i32>,
    #[serde(rename = "timestamp")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLogRequest {
    pub process_id: Uuid,
    pub level: Option<LogLevel>,
    pub function_name: Option<String>,
    pub message: String,
    pub line_id: Option<i32>,
}

#[derive(Debug, Deserialize, Default)]
pub struct LogFilter {
    pub process_id: Option<Uuid>,
    pub level: Option<LogLevel>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
}
