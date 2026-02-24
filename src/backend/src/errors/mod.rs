use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Configuration file error: {0}")]
    ConfigFile(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption failed - data may be corrupted or key incorrect")]
    DecryptionFailed,

    #[error("Encryption key required but not configured")]
    EncryptionKeyRequired,

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("DNS resolution failed for {host}: {message}")]
    DnsFailure { host: String, message: String },

    #[error("Connection timeout after {timeout_secs}s to {host}")]
    ConnectionTimeout { host: String, timeout_secs: u64 },

    #[error("Connection refused by {host}:{port}")]
    ConnectionRefused { host: String, port: u16 },

    #[error("TLS/SSL error connecting to {host}: {message}")]
    TlsError { host: String, message: String },

    #[error("HTTP request timeout after {timeout_secs}s")]
    RequestTimeout { timeout_secs: u64 },

    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error("WebDriver error: {0}")]
    WebDriver(String),

    #[error("Scheduler error: {0}")]
    Scheduler(String),

    #[error("Email error: {0}")]
    Email(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Migration error: {0}")]
    MigrationError(String),
}

impl AppError {
    /// Convert error to JSON response for Tauri commands
    pub fn to_json(&self) -> serde_json::Value {
        let (error_type, message) = match self {
            AppError::Config(msg) => ("config_error", msg.clone()),
            AppError::ConfigFile(msg) => ("config_file_error", msg.clone()),
            AppError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                ("database_error", "A database error occurred".to_string())
            }
            AppError::NotFound(msg) => ("not_found", msg.clone()),
            AppError::Validation(msg) => ("validation_error", msg.clone()),
            AppError::BadRequest(msg) => ("bad_request", msg.clone()),
            AppError::Encryption(msg) => {
                tracing::error!("Encryption error: {}", msg);
                ("encryption_error", msg.clone())
            }
            AppError::DecryptionFailed => {
                tracing::error!("Decryption failed - data may be corrupted or key incorrect");
                ("decryption_failed", "Decryption failed - data may be corrupted or key incorrect".to_string())
            }
            AppError::EncryptionKeyRequired => {
                tracing::error!("Encryption key required but not configured");
                ("encryption_key_required", "Encryption key required but not configured".to_string())
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                ("internal_error", "An internal error occurred".to_string())
            }
            AppError::Io(e) => {
                tracing::error!("IO error: {:?}", e);
                ("io_error", e.to_string())
            }
            AppError::DnsFailure { host, message } => {
                tracing::error!("DNS failure for {}: {}", host, message);
                ("dns_failure", format!("DNS resolution failed for {}: {}", host, message))
            }
            AppError::ConnectionTimeout { host, timeout_secs } => {
                tracing::error!("Connection timeout after {}s to {}", timeout_secs, host);
                ("connection_timeout", format!("Connection timeout after {}s to {}", timeout_secs, host))
            }
            AppError::ConnectionRefused { host, port } => {
                tracing::error!("Connection refused by {}:{}", host, port);
                ("connection_refused", format!("Connection refused by {}:{}", host, port))
            }
            AppError::TlsError { host, message } => {
                tracing::error!("TLS/SSL error connecting to {}: {}", host, message);
                ("tls_error", format!("TLS/SSL error connecting to {}: {}", host, message))
            }
            AppError::RequestTimeout { timeout_secs } => {
                tracing::error!("HTTP request timeout after {}s", timeout_secs);
                ("request_timeout", format!("HTTP request timeout after {}s", timeout_secs))
            }
            AppError::HttpClient(e) => {
                tracing::error!("HTTP client error: {:?}", e);
                ("http_client_error", "External service request failed".to_string())
            }
            AppError::WebDriver(msg) => {
                tracing::error!("WebDriver error: {}", msg);
                ("webdriver_error", msg.clone())
            }
            AppError::Scheduler(msg) => {
                tracing::error!("Scheduler error: {}", msg);
                ("scheduler_error", msg.clone())
            }
            AppError::Email(msg) => {
                tracing::error!("Email error: {}", msg);
                ("email_error", msg.clone())
            }
            AppError::Unauthorized(msg) => ("unauthorized", msg.clone()),
            AppError::MigrationError(msg) => {
                tracing::error!("Migration error: {}", msg);
                ("migration_error", msg.clone())
            }
        };

        json!({
            "error": {
                "type": error_type,
                "message": message,
            }
        })
    }
}

pub type AppResult<T> = Result<T, AppError>;

impl From<validator::ValidationErrors> for AppError {
    fn from(errors: validator::ValidationErrors) -> Self {
        AppError::Validation(errors.to_string())
    }
}
