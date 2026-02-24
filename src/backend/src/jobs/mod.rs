pub mod quota_check_job;
pub mod speed_test_job;
pub mod cleanup_job;
pub mod task_scheduler_job;
pub mod execution_timeout_job;

use std::sync::Arc;

use sqlx::SqlitePool;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::config::Settings;
use crate::errors::{AppError, AppResult};

pub struct JobRunner {
    scheduler: JobScheduler,
    settings: Arc<Settings>,
    pool: Option<SqlitePool>,
}

impl JobRunner {
    /// Create a new JobRunner with a database pool for task scheduling
    pub async fn with_pool(settings: Arc<Settings>, pool: SqlitePool) -> AppResult<Self> {
        let scheduler = JobScheduler::new()
            .await
            .map_err(|e| AppError::Scheduler(format!("Failed to create scheduler: {}", e)))?;

        Ok(Self {
            scheduler,
            settings,
            pool: Some(pool),
        })
    }

    pub async fn register_jobs(&self) -> AppResult<()> {
        if !self.settings.scheduler.enabled {
            tracing::info!("Scheduler is disabled, skipping job registration");
            return Ok(());
        }

        // Register quota check job
        self.register_quota_check_job().await?;

        // Register speed test job
        self.register_speed_test_job().await?;

        // Register cleanup job
        self.register_cleanup_job().await?;

        // Register task scheduler job (runs every minute to check for scheduled tasks)
        if self.pool.is_some() {
            self.register_task_scheduler_job().await?;
            self.register_execution_timeout_job().await?;
        } else {
            tracing::warn!("Task scheduler job not registered: no database pool available");
        }

        tracing::info!("All scheduled jobs registered");
        Ok(())
    }

    async fn register_quota_check_job(&self) -> AppResult<()> {
        let settings = self.settings.clone();
        let cron = self.settings.quota_check.cron.clone();

        let job = Job::new_async(cron.as_str(), move |_uuid, _lock| {
            let settings = settings.clone();
            Box::pin(async move {
                tracing::info!("Running quota check job");
                if let Err(e) = quota_check_job::run(&settings).await {
                    tracing::error!("Quota check job failed: {:?}", e);
                }
            })
        })
        .map_err(|e| AppError::Scheduler(format!("Failed to create quota check job: {}", e)))?;

        self.scheduler
            .add(job)
            .await
            .map_err(|e| AppError::Scheduler(format!("Failed to add quota check job: {}", e)))?;

        tracing::info!("Quota check job registered with cron: {}", self.settings.quota_check.cron);
        Ok(())
    }

    async fn register_speed_test_job(&self) -> AppResult<()> {
        let settings = self.settings.clone();
        let cron = self.settings.speed_test.cron.clone();

        let job = Job::new_async(cron.as_str(), move |_uuid, _lock| {
            let settings = settings.clone();
            Box::pin(async move {
                tracing::info!("Running speed test job");
                if let Err(e) = speed_test_job::run(&settings).await {
                    tracing::error!("Speed test job failed: {:?}", e);
                }
            })
        })
        .map_err(|e| AppError::Scheduler(format!("Failed to create speed test job: {}", e)))?;

        self.scheduler
            .add(job)
            .await
            .map_err(|e| AppError::Scheduler(format!("Failed to add speed test job: {}", e)))?;

        tracing::info!("Speed test job registered with cron: {}", self.settings.speed_test.cron);
        Ok(())
    }

    async fn register_cleanup_job(&self) -> AppResult<()> {
        let settings = self.settings.clone();
        let cron = self.settings.cleanup.cron.clone();

        let job = Job::new_async(cron.as_str(), move |_uuid, _lock| {
            let settings = settings.clone();
            Box::pin(async move {
                tracing::info!("Running cleanup job");
                if let Err(e) = cleanup_job::run(&settings).await {
                    tracing::error!("Cleanup job failed: {:?}", e);
                }
            })
        })
        .map_err(|e| AppError::Scheduler(format!("Failed to create cleanup job: {}", e)))?;

        self.scheduler
            .add(job)
            .await
            .map_err(|e| AppError::Scheduler(format!("Failed to add cleanup job: {}", e)))?;

        tracing::info!("Cleanup job registered with cron: {}", self.settings.cleanup.cron);
        Ok(())
    }

    /// Register the task scheduler job (runs every minute)
    async fn register_task_scheduler_job(&self) -> AppResult<()> {
        let pool = self.pool.clone().ok_or_else(|| {
            AppError::Scheduler("Cannot register task scheduler without database pool".to_string())
        })?;
        let settings = self.settings.clone();

        // Run every minute: "0 * * * * *" = at second 0 of every minute
        let cron = "0 * * * * *";

        let job = Job::new_async(cron, move |_uuid, _lock| {
            let pool = pool.clone();
            let settings = settings.clone();
            Box::pin(async move {
                tracing::debug!("Running task scheduler check");
                if let Err(e) = task_scheduler_job::run(&pool, &settings).await {
                    tracing::error!("Task scheduler job failed: {:?}", e);
                }
            })
        })
        .map_err(|e| AppError::Scheduler(format!("Failed to create task scheduler job: {}", e)))?;

        self.scheduler
            .add(job)
            .await
            .map_err(|e| AppError::Scheduler(format!("Failed to add task scheduler job: {}", e)))?;

        tracing::info!("Task scheduler job registered (runs every minute)");
        Ok(())
    }

    /// Register the execution timeout job (runs every 5 minutes)
    async fn register_execution_timeout_job(&self) -> AppResult<()> {
        let pool = self.pool.clone().ok_or_else(|| {
            AppError::Scheduler("Cannot register execution timeout job without database pool".to_string())
        })?;

        // Run every 5 minutes: "0 */5 * * * *" = at second 0 of every 5th minute
        let cron = "0 */5 * * * *";

        let job = Job::new_async(cron, move |_uuid, _lock| {
            let pool = pool.clone();
            Box::pin(async move {
                tracing::debug!("Running execution timeout check");
                if let Err(e) = execution_timeout_job::run(&pool).await {
                    tracing::error!("Execution timeout job failed: {:?}", e);
                }
            })
        })
        .map_err(|e| AppError::Scheduler(format!("Failed to create execution timeout job: {}", e)))?;

        self.scheduler
            .add(job)
            .await
            .map_err(|e| AppError::Scheduler(format!("Failed to add execution timeout job: {}", e)))?;

        tracing::info!("Execution timeout job registered (runs every 5 minutes)");
        Ok(())
    }

    pub async fn start(&self) -> AppResult<()> {
        self.scheduler
            .start()
            .await
            .map_err(|e| AppError::Scheduler(format!("Failed to start scheduler: {}", e)))?;

        tracing::info!("Job scheduler started");
        Ok(())
    }

    pub async fn shutdown(&mut self) -> AppResult<()> {
        self.scheduler
            .shutdown()
            .await
            .map_err(|e| AppError::Scheduler(format!("Failed to shutdown scheduler: {}", e)))?;

        tracing::info!("Job scheduler stopped");
        Ok(())
    }
}
