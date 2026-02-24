use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum SmtpVendor {
    Gmail,
    Exchange,
    Outlook365,
}

impl Default for SmtpVendor {
    fn default() -> Self {
        Self::Gmail
    }
}

impl SmtpVendor {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Gmail => "gmail",
            Self::Exchange => "exchange",
            Self::Outlook365 => "outlook365",
        }
    }
}

pub struct VendorConfig {
    pub default_host: &'static str,
    pub default_port: i32,
    pub default_use_tls: bool,
    pub requires_authentication: bool,
}

impl SmtpVendor {
    pub fn config(&self) -> VendorConfig {
        match self {
            Self::Gmail => VendorConfig {
                default_host: "smtp.gmail.com",
                default_port: 465,
                default_use_tls: true,
                requires_authentication: true,
            },
            Self::Exchange => VendorConfig {
                default_host: "",
                default_port: 587,
                default_use_tls: true,
                requires_authentication: true,
            },
            Self::Outlook365 => VendorConfig {
                default_host: "smtp.office365.com",
                default_port: 587,
                default_use_tls: true,
                requires_authentication: true,
            },
        }
    }

    pub fn validate(&self, req: &CreateSmtpConfigRequest) -> Result<(), String> {
        let config = self.config();

        // Vendor-specific host validation
        match self {
            Self::Exchange => {
                if req.host.is_empty() {
                    return Err("Exchange requires explicit host configuration".to_string());
                }
            }
            Self::Gmail | Self::Outlook365 => {
                // Should have been auto-filled, but check anyway
                if req.host.is_empty() {
                    return Err(format!("{} requires host configuration", self.as_str()));
                }
            }
        }

        if config.requires_authentication {
            if req.username.is_none() || req.username.as_ref().map(|s| s.is_empty()).unwrap_or(true)
            {
                return Err(format!("{} requires username", self.as_str()));
            }
            if req.password.is_none() || req.password.as_ref().map(|s| s.is_empty()).unwrap_or(true)
            {
                return Err(format!("{} requires password", self.as_str()));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub id: i32,
    pub name: String,
    pub host: String,
    pub port: i32,
    pub vendor: SmtpVendor,
    pub username: Option<String>,
    #[serde(skip_serializing)]
    pub password: Option<String>,
    #[serde(rename = "senderEmail")]
    pub from_email: String,
    #[serde(rename = "senderName")]
    pub from_name: Option<String>,
    #[serde(rename = "useTls")]
    pub use_tls: bool,
    #[serde(rename = "isDefault")]
    pub is_default: bool,
    #[serde(rename = "isActive")]
    pub is_active: bool,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateSmtpConfigRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(length(min = 1, max = 255))]
    pub host: String,
    #[validate(range(min = 1, max = 65535))]
    pub port: Option<i32>,
    #[serde(rename = "vendor")]
    pub vendor: Option<SmtpVendor>,
    #[validate(length(max = 255))]
    pub username: Option<String>,
    #[validate(length(max = 255))]
    pub password: Option<String>,
    #[validate(email, length(max = 255))]
    #[serde(rename = "senderEmail")]
    pub from_email: String,
    #[validate(length(max = 255))]
    #[serde(rename = "senderName")]
    pub from_name: Option<String>,
    #[serde(rename = "useTls")]
    pub use_tls: Option<bool>,
    #[serde(rename = "isDefault")]
    pub is_default: Option<bool>,
    #[serde(rename = "isActive")]
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateSmtpConfigRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    #[validate(length(min = 1, max = 255))]
    pub host: Option<String>,
    #[validate(range(min = 1, max = 65535))]
    pub port: Option<i32>,
    #[serde(rename = "vendor")]
    pub vendor: Option<SmtpVendor>,
    #[validate(length(max = 255))]
    pub username: Option<String>,
    #[validate(length(max = 255))]
    pub password: Option<String>,
    #[validate(email, length(max = 255))]
    #[serde(rename = "senderEmail")]
    pub from_email: Option<String>,
    #[validate(length(max = 255))]
    #[serde(rename = "senderName")]
    pub from_name: Option<String>,
    #[serde(rename = "useTls")]
    pub use_tls: Option<bool>,
    #[serde(rename = "isDefault")]
    pub is_default: Option<bool>,
    #[serde(rename = "isActive")]
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SmtpConfigTestRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(length(min = 1, max = 255))]
    pub host: String,
    #[validate(range(min = 1, max = 65535))]
    pub port: i32,
    #[serde(rename = "vendor")]
    pub vendor: Option<SmtpVendor>,
    #[validate(length(max = 255))]
    pub username: Option<String>,
    #[validate(length(max = 255))]
    pub password: Option<String>,
    #[validate(email, length(max = 255))]
    #[serde(rename = "senderEmail")]
    pub sender_email: String,
    #[validate(length(max = 255))]
    #[serde(rename = "senderName")]
    pub sender_name: Option<String>,
    #[serde(rename = "useTls")]
    pub use_tls: Option<bool>,
    #[validate(email, length(max = 255))]
    #[serde(rename = "testRecipient")]
    pub test_recipient: String,
}

#[derive(Debug, Serialize)]
pub struct SmtpConfigTestResponse {
    pub success: bool,
    pub message: String,
}
