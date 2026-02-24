use sqlx::SqlitePool;

use crate::crypto::{EncryptionKey, maybe_encrypt, maybe_decrypt};
use crate::errors::AppResult;
use crate::models::{CreateSmtpConfigRequest, SmtpConfig, UpdateSmtpConfigRequest};

pub struct SmtpConfigRepository;

fn decrypt_smtp_config(mut config: SmtpConfig, key: Option<&EncryptionKey>) -> AppResult<SmtpConfig> {
    if let Some(ref password) = config.password {
        config.password = Some(maybe_decrypt(password, key)?);
    }
    Ok(config)
}

impl SmtpConfigRepository {
    pub async fn get_by_id(pool: &SqlitePool, id: i32, encryption_key: Option<&EncryptionKey>) -> AppResult<Option<SmtpConfig>> {
        let config = sqlx::query_as::<_, SmtpConfig>("SELECT * FROM smtp_configs WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;

        match config {
            Some(cfg) => Ok(Some(decrypt_smtp_config(cfg, encryption_key)?)),
            None => Ok(None),
        }
    }

    pub async fn get_all(pool: &SqlitePool, encryption_key: Option<&EncryptionKey>) -> AppResult<Vec<SmtpConfig>> {
        let configs = sqlx::query_as::<_, SmtpConfig>("SELECT * FROM smtp_configs ORDER BY name")
            .fetch_all(pool)
            .await?;

        configs.into_iter()
            .map(|cfg| decrypt_smtp_config(cfg, encryption_key))
            .collect()
    }

    pub async fn get_default(pool: &SqlitePool, encryption_key: Option<&EncryptionKey>) -> AppResult<Option<SmtpConfig>> {
        let config = sqlx::query_as::<_, SmtpConfig>(
            "SELECT * FROM smtp_configs WHERE is_default = true LIMIT 1"
        )
        .fetch_optional(pool)
        .await?;

        match config {
            Some(cfg) => Ok(Some(decrypt_smtp_config(cfg, encryption_key)?)),
            None => Ok(None),
        }
    }

    pub async fn create(pool: &SqlitePool, req: &CreateSmtpConfigRequest, encryption_key: Option<&EncryptionKey>) -> AppResult<SmtpConfig> {
        // If this is set as default, unset other defaults first
        if req.is_default.unwrap_or(false) {
            let _: sqlx::sqlite::SqliteQueryResult = sqlx::query("UPDATE smtp_configs SET is_default = false WHERE is_default = true")
                .execute(pool)
                .await?;
        }

        // Encrypt password if provided
        let encrypted_password = req.password.as_ref().map(|p| maybe_encrypt(p, encryption_key));

        let config = sqlx::query_as::<_, SmtpConfig>(
            r#"
            INSERT INTO smtp_configs (name, host, port, username, password, from_email, from_name, use_tls, is_default, is_active, vendor)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#
        )
        .bind(&req.name)
        .bind(&req.host)
        .bind(req.port.unwrap_or(587))
        .bind(&req.username)
        .bind(&encrypted_password)
        .bind(&req.from_email)
        .bind(&req.from_name)
        .bind(req.use_tls.unwrap_or(true))
        .bind(req.is_default.unwrap_or(false))
        .bind(req.is_active.unwrap_or(true))
        .bind(&req.vendor)
        .fetch_one(pool)
        .await?;

        decrypt_smtp_config(config, encryption_key)
    }

    pub async fn update(pool: &SqlitePool, id: i32, req: &UpdateSmtpConfigRequest, encryption_key: Option<&EncryptionKey>) -> AppResult<Option<SmtpConfig>> {
        // If this is set as default, unset other defaults first
        if req.is_default == Some(true) {
            let _: sqlx::sqlite::SqliteQueryResult = sqlx::query("UPDATE smtp_configs SET is_default = false WHERE is_default = true AND id != $1")
                .bind(id)
                .execute(pool)
                .await?;
        }

        // Encrypt password if provided
        let encrypted_password = req.password.as_ref().map(|p| maybe_encrypt(p, encryption_key));

        let config = sqlx::query_as::<_, SmtpConfig>(
            r#"
            UPDATE smtp_configs SET
                name = COALESCE($1, name),
                host = COALESCE($2, host),
                port = COALESCE($3, port),
                username = COALESCE($4, username),
                password = COALESCE($5, password),
                from_email = COALESCE($6, from_email),
                from_name = COALESCE($7, from_name),
                use_tls = COALESCE($8, use_tls),
                is_default = COALESCE($9, is_default),
                is_active = COALESCE($10, is_active),
                vendor = COALESCE($11, vendor),
                updated_at = datetime('now', 'utc')
            WHERE id = $12
            RETURNING *
            "#
        )
        .bind(&req.name)
        .bind(&req.host)
        .bind(req.port)
        .bind(&req.username)
        .bind(&encrypted_password)
        .bind(&req.from_email)
        .bind(&req.from_name)
        .bind(req.use_tls)
        .bind(req.is_default)
        .bind(req.is_active)
        .bind(&req.vendor)
        .bind(id)
        .fetch_optional(pool)
        .await?;

        match config {
            Some(cfg) => Ok(Some(decrypt_smtp_config(cfg, encryption_key)?)),
            None => Ok(None),
        }
    }

    pub async fn delete(pool: &SqlitePool, id: i32) -> AppResult<bool> {
        let result: sqlx::sqlite::SqliteQueryResult = sqlx::query("DELETE FROM smtp_configs WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
