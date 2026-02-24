use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Line {
    pub id: i32,
    pub name: String,
    #[serde(rename = "lineNumber")]
    pub line_number: String,
    #[serde(rename = "portalUsername")]
    pub username: String,
    #[serde(rename = "portalPassword")]
    pub password: String,
    #[serde(rename = "ipAddress")]
    pub ip_address: Option<String>,
    pub isp: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "gatewayIp")]
    pub gateway_ip: Option<String>,
    #[serde(rename = "isActive")]
    pub is_active: bool,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineResponse {
    pub id: i32,
    pub name: String,
    #[serde(rename = "lineNumber")]
    pub line_number: String,
    #[serde(rename = "portalUsername")]
    pub username: String,
    #[serde(rename = "portalPassword")]
    pub password: String,
    #[serde(rename = "ipAddress")]
    pub ip_address: Option<String>,
    pub isp: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "gatewayIp")]
    pub gateway_ip: Option<String>,
    #[serde(rename = "isActive")]
    pub is_active: bool,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

impl From<Line> for LineResponse {
    fn from(line: Line) -> Self {
        Self {
            id: line.id,
            name: line.name,
            line_number: line.line_number,
            username: line.username,
            password: line.password,
            ip_address: line.ip_address,
            isp: line.isp,
            description: line.description,
            gateway_ip: line.gateway_ip,
            is_active: line.is_active,
            created_at: line.created_at,
            updated_at: line.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateLineRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(length(min = 1, max = 100))]
    #[serde(rename = "lineNumber")]
    pub line_number: String,
    #[validate(length(min = 1, max = 255))]
    #[serde(rename = "portalUsername")]
    pub username: String,
    #[validate(length(min = 1, max = 255))]
    #[serde(rename = "portalPassword")]
    pub password: String,
    #[validate(length(max = 45))]
    #[serde(rename = "ipAddress")]
    pub ip_address: Option<String>,
    #[validate(length(max = 100))]
    pub isp: Option<String>,
    #[validate(length(max = 500))]
    pub description: Option<String>,
    #[validate(length(max = 45))]
    #[serde(rename = "gatewayIp")]
    pub gateway_ip: Option<String>,
    #[serde(rename = "isActive")]
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateLineRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    #[validate(length(min = 1, max = 100))]
    #[serde(rename = "lineNumber")]
    pub line_number: Option<String>,
    #[validate(length(min = 1, max = 255))]
    #[serde(rename = "portalUsername")]
    pub username: Option<String>,
    #[validate(length(min = 1, max = 255))]
    #[serde(rename = "portalPassword")]
    pub password: Option<String>,
    #[validate(length(max = 45))]
    #[serde(rename = "ipAddress")]
    pub ip_address: Option<String>,
    #[validate(length(max = 100))]
    pub isp: Option<String>,
    #[validate(length(max = 500))]
    pub description: Option<String>,
    #[validate(length(max = 45))]
    #[serde(rename = "gatewayIp")]
    pub gateway_ip: Option<String>,
    #[serde(rename = "isActive")]
    pub is_active: Option<bool>,
}
