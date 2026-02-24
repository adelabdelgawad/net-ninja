use std::sync::Arc;

use sqlx::SqlitePool;
use crate::config::Settings;
use crate::crypto::EncryptionKey;

/// Describes the mode the app is running in after initialization
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum InitMode {
    /// Full mode with SQLite database and all features
    Full,
    /// Fallback mode - no database connection, settings-only mode
    Fallback,
}

#[derive(Clone)]
pub struct AppState {
    /// SQLite database connection pool - None in fallback mode
    pub pool: Option<SqlitePool>,
    /// Runtime settings (from environment/config.toml in working directory)
    pub settings: Arc<Settings>,
    /// Initialization mode - Full or Fallback
    pub init_mode: InitMode,
    /// Original error message that caused fallback mode (if applicable)
    pub init_error: Option<Arc<String>>,
    /// Encryption key for encrypting/decrypting sensitive data
    pub encryption_key: Option<Arc<EncryptionKey>>,
}

impl AppState {
    /// Create AppState in full mode with database available
    pub fn new_full(
        pool: SqlitePool,
        settings: Settings,
        encryption_key: Option<Arc<EncryptionKey>>,
    ) -> Self {
        Self {
            pool: Some(pool),
            settings: Arc::new(settings),
            init_mode: InitMode::Full,
            init_error: None,
            encryption_key,
        }
    }

    /// Create AppState in fallback mode (no SQLite)
    pub fn new_fallback(
        settings: Settings,
        error: String,
        encryption_key: Option<Arc<EncryptionKey>>,
    ) -> Self {
        Self {
            pool: None,
            settings: Arc::new(settings),
            init_mode: InitMode::Fallback,
            init_error: Some(Arc::new(error)),
            encryption_key,
        }
    }

    /// Create AppState (legacy constructor for backward compatibility)
    pub fn new(
        pool: SqlitePool,
        settings: Settings,
        encryption_key: Option<Arc<EncryptionKey>>,
    ) -> Self {
        Self::new_full(pool, settings, encryption_key)
    }

    /// Check if the app is running in fallback mode
    pub fn is_fallback_mode(&self) -> bool {
        self.init_mode == InitMode::Fallback
    }

    /// Get the database pool, returning an error if in fallback mode
    pub fn require_pool(&self) -> crate::errors::AppResult<&SqlitePool> {
        self.pool.as_ref()
            .ok_or_else(|| crate::errors::AppError::Validation(
                "Not available in fallback mode - database connection required".to_string()
            ))
    }
}
