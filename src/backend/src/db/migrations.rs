//! Database migration management for SQLite
//!
//! Handles reading, parsing, and applying SQL migration files for SQLite.
//! Migrations are embedded at compile time to avoid runtime path issues.

use std::collections::HashMap;
use std::time::Instant;
use sqlx::SqlitePool;
use crate::errors::AppError;

/// Embedded migrations - these are compiled into the binary
/// to avoid runtime path resolution issues when running via Tauri
static EMBEDDED_MIGRATIONS: &[(&str, &str)] = &[
    ("20260121000000_create_migrations_table", include_str!("../../migrations/20260121000000_create_migrations_table.sql")),
    ("20260121000001_create_lines", include_str!("../../migrations/20260121000001_create_lines.sql")),
    ("20260121000002_create_quota_results", include_str!("../../migrations/20260121000002_create_quota_results.sql")),
    ("20260121000003_create_speed_tests", include_str!("../../migrations/20260121000003_create_speed_tests.sql")),
    ("20260121000004_create_emails", include_str!("../../migrations/20260121000004_create_emails.sql")),
    ("20260121000005_create_logs", include_str!("../../migrations/20260121000005_create_logs.sql")),
    ("20260121000006_create_jobs", include_str!("../../migrations/20260121000006_create_jobs.sql")),
    ("20260121000007_create_smtp_configs", include_str!("../../migrations/20260121000007_create_smtp_configs.sql")),
    ("20260121000008_add_line_fields", include_str!("../../migrations/20260121000008_add_line_fields.sql")),
    ("20260121000009_add_quota_result_fields", include_str!("../../migrations/20260121000009_add_quota_result_fields.sql")),
    ("20260121000010_add_speed_test_fields", include_str!("../../migrations/20260121000010_add_speed_test_fields.sql")),
    ("20260121000011_add_smtp_config_fields", include_str!("../../migrations/20260121000011_add_smtp_config_fields.sql")),
    ("20260121000012_add_line_active", include_str!("../../migrations/20260121000012_add_line_active.sql")),
    ("20260122000001_create_tasks", include_str!("../../migrations/20260122000001_create_tasks.sql")),
    ("20260122000002_create_task_lines", include_str!("../../migrations/20260122000002_create_task_lines.sql")),
    ("20260122000003_add_task_is_active_and_task_types", include_str!("../../migrations/20260122000003_add_task_is_active_and_task_types.sql")),
    ("20260123000001_fix_speed_test_column_types", include_str!("../../migrations/20260123000001_fix_speed_test_column_types.sql")),
    ("20260124000001_create_task_executions", include_str!("../../migrations/20260124000001_create_task_executions.sql")),
    ("20260124000002_drop_scheduled_jobs", include_str!("../../migrations/20260124000002_drop_scheduled_jobs.sql")),
    ("20260124000003_fix_process_id_blob_to_text", include_str!("../../migrations/20260124000003_fix_process_id_blob_to_text.sql")),
    ("20260124000004_create_task_notification_configs", include_str!("../../migrations/20260124000004_create_task_notification_configs.sql")),
    ("20260128000001_add_execution_timeout_fields", include_str!("../../migrations/20260128000001_add_execution_timeout_fields.sql")),
    ("20260203000001_create_scheduler_lock_and_service_info", include_str!("../../migrations/20260203000001_create_scheduler_lock_and_service_info.sql")),
    ("20260207000001_add_logs_line_id", include_str!("../../migrations/20260207000001_add_logs_line_id.sql")),
    ("20260209000001_add_smtp_vendor", include_str!("../../migrations/20260209000001_add_smtp_vendor.sql")),
    ("20260213000001_add_task_show_browser", include_str!("../../migrations/20260213000001_add_task_show_browser.sql")),
];

/// Represents a single migration
struct Migration {
    /// Version number (from filename timestamp)
    version: i64,
    /// Description (from filename)
    description: String,
    /// SQL content
    sql: String,
}

impl Migration {
    /// Parse a migration from an embedded entry (filename, content)
    fn from_embedded(filename: &str, sql: &str) -> Result<Self, AppError> {
        // Parse filename: YYYYMMDDHHMMSS_description
        let parts: Vec<&str> = filename.splitn(2, '_').collect();
        if parts.len() != 2 {
            return Err(AppError::MigrationError(
                format!("Migration filename must be YYYYMMDDHHMMSS_description: {}", filename)
            ));
        }

        let version: i64 = parts[0].parse()
            .map_err(|_| AppError::MigrationError(
                format!("Invalid version number in migration: {}", parts[0])
            ))?;

        let description = parts[1].replace('_', " ");

        Ok(Migration {
            version,
            description,
            sql: sql.to_string(),
        })
    }

    /// Calculate checksum for the migration
    fn checksum(&self) -> Vec<u8> {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(self.sql.as_bytes());
        hasher.finalize().to_vec()
    }
}

/// Load all migrations from embedded data
fn load_migrations() -> Result<Vec<Migration>, AppError> {
    let mut migrations = Vec::new();

    for (filename, sql) in EMBEDDED_MIGRATIONS {
        migrations.push(Migration::from_embedded(filename, sql)?);
    }

    // Sort by version
    migrations.sort_by_key(|m| m.version);

    Ok(migrations)
}

/// Status of a migration
enum MigrationStatus {
    /// Migration has not been applied
    Pending,
    /// Migration has been applied successfully
    Applied,
    /// Migration checksum mismatch (file changed after application)
    Modified,
}

/// Migration with status information
struct MigrationInfo {
    migration: Migration,
    status: MigrationStatus,
}

/// Get list of applied migrations from database
async fn get_applied_migrations(pool: &SqlitePool) -> Result<Vec<(i64, Vec<u8>)>, AppError> {
    use sqlx::Row;

    let rows = sqlx::query(
        "SELECT version, checksum FROM _sqlx_migrations WHERE success = 1 ORDER BY version"
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.iter()
        .map(|row| {
            let version: i64 = row.get("version");
            let checksum: Vec<u8> = row.get("checksum");
            (version, checksum)
        })
        .collect())
}

/// Get migration status by comparing filesystem and database
async fn get_migration_status(pool: &SqlitePool) -> Result<Vec<MigrationInfo>, AppError> {
    let all_migrations = load_migrations()?;

    // Try to get applied migrations; if table doesn't exist, treat all as pending
    let applied = match get_applied_migrations(pool).await {
        Ok(migrations) => migrations,
        Err(_) => {
            // Table doesn't exist yet, all migrations are pending
            tracing::debug!("Migration tracking table not found, treating all migrations as pending");
            Vec::new()
        }
    };

    let applied_map: HashMap<i64, Vec<u8>> = applied.into_iter().collect();

    let mut info_list = Vec::new();

    for migration in all_migrations {
        let status = match applied_map.get(&migration.version) {
            None => MigrationStatus::Pending,
            Some(stored_checksum) => {
                let current_checksum = migration.checksum();
                if &current_checksum == stored_checksum {
                    MigrationStatus::Applied
                } else {
                    MigrationStatus::Modified
                }
            }
        };

        info_list.push(MigrationInfo {
            migration,
            status,
        });
    }

    Ok(info_list)
}

/// Get only pending migrations
async fn get_pending_migrations(pool: &SqlitePool) -> Result<Vec<Migration>, AppError> {
    let statuses = get_migration_status(pool).await?;

    Ok(statuses.into_iter()
        .filter(|info| matches!(info.status, MigrationStatus::Pending))
        .map(|info| info.migration)
        .collect())
}

/// Execute a single migration
async fn apply_migration(pool: &SqlitePool, migration: &Migration) -> Result<(), AppError> {
    let checksum = migration.checksum();
    let start_time = Instant::now();

    let mut tx = pool.begin().await?;

    // Execute migration SQL
    sqlx::query(&migration.sql)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::MigrationError(
            format!("Failed to execute migration {}: {}", migration.version, e)
        ))?;

    // Record migration
    let execution_time = start_time.elapsed().as_millis() as i64;
    sqlx::query(
        "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time)
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind(migration.version)
    .bind(&migration.description)
    .bind(1) // SQLite uses INTEGER for boolean
    .bind(&checksum[..])
    .bind(execution_time)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    tracing::info!(
        "Applied migration {}: {} in {}ms",
        migration.version,
        migration.description,
        start_time.elapsed().as_millis()
    );

    Ok(())
}

/// Run all pending migrations
pub async fn run_pending_migrations(pool: &SqlitePool) -> Result<usize, AppError> {
    let pending = get_pending_migrations(pool).await?;

    if pending.is_empty() {
        tracing::info!("No pending migrations");
        return Ok(0);
    }

    tracing::info!("Found {} pending migrations", pending.len());

    for migration in &pending {
        apply_migration(pool, migration).await?;
    }

    tracing::info!("Successfully applied {} migrations", pending.len());

    Ok(pending.len())
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_migration_filename_parsing() {
        // Test the parsing logic
        let filename = "20260121000001_create_lines";
        let parts: Vec<&str> = filename.splitn(2, '_').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "20260121000001");
        assert_eq!(parts[1], "create_lines");
    }
}
