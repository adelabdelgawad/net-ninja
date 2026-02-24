use uuid::Uuid;

use crate::config::Settings;
use crate::db::create_pool;
use crate::errors::AppResult;
use crate::services::{LogService, QuotaCheckService, SpeedTestService};

pub async fn run(settings: &Settings) -> AppResult<()> {
    let process_id = Uuid::new_v4();
    let retention_days = settings.cleanup.retention_days;

    // Create database pool for this job run
    let pool = create_pool().await?;

    LogService::info(
        &pool,
        process_id,
        "cleanup_job::run",
        &format!("Starting cleanup job (retention: {} days)", retention_days),
    )
    .await?;

    // Clean up old quota results
    let quota_deleted = QuotaCheckService::cleanup_old(&pool, retention_days).await?;
    tracing::info!("Deleted {} old quota results", quota_deleted);

    // Clean up old speed test results
    let speed_deleted = SpeedTestService::cleanup_old(&pool, retention_days).await?;
    tracing::info!("Deleted {} old speed test results", speed_deleted);

    // Clean up old logs
    let logs_deleted = LogService::cleanup_old(&pool, retention_days).await?;
    tracing::info!("Deleted {} old logs", logs_deleted);

    LogService::info(
        &pool,
        process_id,
        "cleanup_job::run",
        &format!(
            "Cleanup completed: {} quota, {} speed tests, {} logs deleted",
            quota_deleted, speed_deleted, logs_deleted
        ),
    )
    .await?;

    Ok(())
}
