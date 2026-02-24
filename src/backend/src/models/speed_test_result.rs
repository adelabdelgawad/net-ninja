use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Raw database row (process_id stored as TEXT)
#[derive(Debug, Clone, FromRow)]
pub struct SpeedTestResultRow {
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
}

impl From<SpeedTestResultRow> for SpeedTestResult {
    fn from(row: SpeedTestResultRow) -> Self {
        Self {
            id: row.id,
            line_id: row.line_id,
            process_id: Uuid::parse_str(&row.process_id).unwrap_or_default(),
            download_speed: row.download_speed,
            upload_speed: row.upload_speed,
            ping: row.ping,
            server_name: row.server_name,
            server_location: row.server_location,
            public_ip: row.public_ip,
            status: row.status,
            error_message: row.error_message,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SpeedTestResult {
    pub id: i32,
    #[serde(rename = "lineId")]
    pub line_id: i32,
    #[serde(rename = "processId")]
    pub process_id: Uuid,
    #[serde(rename = "downloadSpeed")]
    pub download_speed: Option<f64>,
    #[serde(rename = "uploadSpeed")]
    pub upload_speed: Option<f64>,
    pub ping: Option<f64>,
    #[serde(rename = "serverName")]
    pub server_name: Option<String>,
    #[serde(rename = "serverLocation")]
    pub server_location: Option<String>,
    #[serde(rename = "publicIp")]
    pub public_ip: Option<String>,
    pub status: Option<String>,
    #[serde(rename = "errorMessage")]
    pub error_message: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedTestResultResponse {
    pub id: i32,
    #[serde(rename = "lineId")]
    pub line_id: i32,
    #[serde(rename = "processId")]
    pub process_id: Uuid,
    #[serde(rename = "downloadSpeed")]
    pub download_speed: Option<f64>,
    #[serde(rename = "uploadSpeed")]
    pub upload_speed: Option<f64>,
    pub ping: Option<f64>,
    #[serde(rename = "serverName")]
    pub server_name: Option<String>,
    #[serde(rename = "serverLocation")]
    pub server_location: Option<String>,
    #[serde(rename = "publicIp")]
    pub public_ip: Option<String>,
    pub status: Option<String>,
    #[serde(rename = "errorMessage")]
    pub error_message: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
}

impl From<SpeedTestResult> for SpeedTestResultResponse {
    fn from(r: SpeedTestResult) -> Self {
        Self {
            id: r.id,
            line_id: r.line_id,
            process_id: r.process_id,
            download_speed: r.download_speed,
            upload_speed: r.upload_speed,
            ping: r.ping,
            server_name: r.server_name,
            server_location: r.server_location,
            public_ip: r.public_ip,
            status: r.status,
            error_message: r.error_message,
            created_at: r.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateSpeedTestResultRequest {
    #[serde(rename = "lineId")]
    pub line_id: i32,
    #[serde(rename = "processId")]
    pub process_id: Uuid,
    #[serde(rename = "downloadSpeed")]
    pub download_speed: Option<f64>,
    #[serde(rename = "uploadSpeed")]
    pub upload_speed: Option<f64>,
    pub ping: Option<f64>,
    #[serde(rename = "serverName")]
    pub server_name: Option<String>,
    #[serde(rename = "serverLocation")]
    pub server_location: Option<String>,
    #[serde(rename = "publicIp")]
    pub public_ip: Option<String>,
    pub status: Option<String>,
    #[serde(rename = "errorMessage")]
    pub error_message: Option<String>,
}
