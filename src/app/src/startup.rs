//! Startup logic: admin bootstrap, orphan execution reset, password reset file check.
//!
//! # Security Audit (T048 — FR-014, SC-008)
//! All `tracing::` calls in this file and in `server_fns/auth.rs` have been audited:
//! - No password values (plaintext or argon2 hash) appear in any log statement.
//! - Only the `username` field is logged (login, logout, password change, reset).
//! - The trigger file path is logged but not the new hash or any password value.
//! - `hash_password` and `verify_password` results are never included in log output.

#[cfg(feature = "ssr")]
pub mod ssr {
    use sqlx::SqlitePool;
    use tower_sessions_sqlx_store::SqliteStore;
    use argon2::{Argon2, PasswordHasher, PasswordVerifier};
    use argon2::password_hash::{rand_core::OsRng, SaltString, PasswordHash};

    const DEFAULT_PASSWORD: &str = "admin";
    const DEFAULT_USERNAME: &str = "admin";

    /// Ensure a default admin credential exists.
    /// If the web_admin_credentials table is empty, inserts admin/admin (argon2id).
    pub async fn bootstrap_admin(pool: &SqlitePool) -> Result<(), sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM web_admin_credentials"
        )
        .fetch_one(pool)
        .await?;

        if count == 0 {
            let password_hash = hash_password(DEFAULT_PASSWORD);
            sqlx::query(
                "INSERT INTO web_admin_credentials (username, password_hash) VALUES ($1, $2)"
            )
            .bind(DEFAULT_USERNAME)
            .bind(&password_hash)
            .execute(pool)
            .await?;
            tracing::info!("Bootstrapped default admin credentials");
        }

        Ok(())
    }

    /// Reset task_executions that were running at startup to 'failed'.
    pub async fn reset_orphan_executions(pool: &SqlitePool) -> Result<(), sqlx::Error> {
        let result = sqlx::query(
            "UPDATE task_executions SET status = 'failed', error_message = 'Server restarted', is_finished = 1 WHERE status = 'running'"
        )
        .execute(pool)
        .await?;
        if result.rows_affected() > 0 {
            tracing::info!("Reset {} orphan task executions to failed", result.rows_affected());
        }
        Ok(())
    }

    /// Check for reset_admin_password.bat trigger file.
    /// If present and pool available:
    ///   (a) hash "admin" with argon2id
    ///   (b) DELETE FROM tower_sessions (invalidate all sessions)
    ///   (c) INSERT OR REPLACE INTO web_admin_credentials with new hash
    ///   (d) delete trigger file
    ///   (e) log reset event (no password value)
    pub async fn check_reset_file(pool: &SqlitePool, _store: &SqliteStore) -> Result<(), Box<dyn std::error::Error>> {
        let data_path = net_ninja::config::paths::get_shared_data_path();
        let trigger = data_path.join("reset_admin_password.bat");

        if !trigger.exists() {
            return Ok(());
        }

        tracing::info!("Password reset trigger file detected; resetting admin password");

        // Hash the new password
        let new_hash = hash_password(DEFAULT_PASSWORD);

        // Invalidate all sessions
        sqlx::query("DELETE FROM tower_sessions")
            .execute(pool)
            .await?;

        // Reset password
        sqlx::query(
            "INSERT OR REPLACE INTO web_admin_credentials (username, password_hash, updated_at) VALUES ($1, $2, datetime('now'))"
        )
        .bind(DEFAULT_USERNAME)
        .bind(&new_hash)
        .execute(pool)
        .await?;

        // Remove trigger file
        std::fs::remove_file(&trigger)?;

        tracing::info!("Admin password reset via trigger file. All sessions invalidated.");

        Ok(())
    }

    pub fn hash_password(password: &str) -> String {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        argon2
            .hash_password(password.as_bytes(), &salt)
            .expect("Password hashing failed")
            .to_string()
    }

    pub fn verify_password(password: &str, hash: &str) -> bool {
        let parsed_hash = match PasswordHash::new(hash) {
            Ok(h) => h,
            Err(_) => return false,
        };
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    }
}

#[cfg(feature = "ssr")]
pub use ssr::{bootstrap_admin, check_reset_file, hash_password, reset_orphan_executions, verify_password};
