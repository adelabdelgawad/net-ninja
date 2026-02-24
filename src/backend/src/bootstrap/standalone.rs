use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::app::AppState;
use crate::config::Settings;
use crate::db::{create_pool, run_pending_migrations};
use crate::errors::{AppError, AppResult};
use crate::jobs::JobRunner;
use crate::service::SchedulerLock;

/// Run the application in standalone mode (Tauri desktop app)
pub async fn run(settings: Settings) -> AppResult<()> {
    tracing::info!("Starting NetNinja in STANDALONE mode...");

    // Load encryption key
    let encryption_key = crate::crypto::load_encryption_key().map(std::sync::Arc::new);
    if encryption_key.is_some() {
        tracing::info!("Encryption key loaded successfully");
    } else {
        tracing::warn!("No encryption key found - sensitive data will not be encrypted");
    }

    // Create SQLite database pool
    let pool = match create_pool().await {
        Ok(p) => {
            tracing::info!("SQLite pool created successfully");
            p
        }
        Err(e) => {
            // Determine if this is a recoverable error for fallback mode
            let error_msg = e.to_string();
            let should_fallback = error_msg.contains("database is locked")
                || error_msg.contains("permission denied")
                || error_msg.contains("corrupt")
                || error_msg.contains("disk I/O error");

            if should_fallback {
                tracing::warn!("SQLite unavailable ({}), entering fallback mode", error_msg);
                let state = AppState::new_fallback(settings, error_msg, encryption_key);
                return run_tauri_fallback(state);
            } else {
                return Err(e);
            }
        }
    };

    // Run migrations
    match run_pending_migrations(&pool).await {
        Ok(_) => tracing::info!("Database migrations completed"),
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("database is locked")
                || error_msg.contains("corrupt")
                || error_msg.contains("disk I/O error") {
                tracing::warn!("Migration failed ({}), entering fallback mode", error_msg);
                let state = AppState::new_fallback(settings, error_msg, encryption_key);
                return run_tauri_fallback(state);
            } else {
                return Err(e);
            }
        }
    }

    // Startup safety: reset any tasks/executions stuck in "running" state.
    // Nothing can be running after a restart — these are orphans from a crash or forced quit.
    {
        use crate::repositories::{TaskExecutionRepository, TaskRepository};

        match TaskExecutionRepository::reset_all_unfinished(&pool).await {
            Ok(n) if n > 0 => tracing::info!("Reset {} orphaned execution(s) to 'failed'", n),
            Ok(_) => tracing::debug!("No orphaned executions found"),
            Err(e) => tracing::warn!("Failed to reset orphaned executions: {:?}", e),
        }

        match TaskRepository::reset_all_running(&pool).await {
            Ok(n) if n > 0 => tracing::info!("Reset {} orphaned task(s) from 'running' to 'failed'", n),
            Ok(_) => tracing::debug!("No orphaned running tasks found"),
            Err(e) => tracing::warn!("Failed to reset orphaned tasks: {:?}", e),
        }
    }

    // Create app state for Tauri
    let state = AppState::new(pool.clone(), settings.clone(), encryption_key);

    // Start job runner if scheduler is enabled, but check lock first
    let (job_runner, scheduler_lock, heartbeat_shutdown) = if settings.scheduler.enabled {
        // Initialize scheduler lock
        let scheduler_lock = SchedulerLock::new(pool.clone());
        if let Err(e) = scheduler_lock.initialize().await {
            tracing::warn!("Failed to initialize scheduler lock table: {:?}", e);
            // Continue without lock - non-fatal
        }

        // Check if the service currently holds the lock
        match scheduler_lock.get_lock_holder().await {
            Ok(Some(lock_info)) => {
                // Check if the lock is held by the service and is not stale
                let now = chrono::Utc::now();
                let stale_threshold = now - chrono::Duration::seconds(60);

                if lock_info.holder == "service" && lock_info.heartbeat_at > stale_threshold {
                    tracing::info!(
                        "Scheduler lock is held by Windows service (acquired: {}, last heartbeat: {})",
                        lock_info.acquired_at,
                        lock_info.heartbeat_at
                    );
                    tracing::info!("Desktop scheduler will NOT start - service is managing scheduled tasks");
                    (None, None, None)
                } else {
                    // Lock is stale or held by desktop (stale from previous run)
                    if lock_info.holder == "desktop" {
                        tracing::info!(
                            "Found stale desktop lock from previous session, will reclaim"
                        );
                    } else {
                        tracing::info!(
                            "Found stale service lock (last heartbeat: {}), will reclaim",
                            lock_info.heartbeat_at
                        );
                    }

                    // Attempt to acquire the lock and start scheduler
                    start_scheduler_with_lock(settings.clone(), pool.clone(), scheduler_lock).await?
                }
            }
            Ok(None) => {
                // No lock exists - we can acquire and start
                tracing::debug!("No existing scheduler lock found");
                start_scheduler_with_lock(settings.clone(), pool.clone(), scheduler_lock).await?
            }
            Err(e) => {
                tracing::warn!("Failed to check scheduler lock: {:?}", e);
                // Fall back to starting without lock coordination
                tracing::info!("Starting scheduler without lock coordination due to error");
                let runner = JobRunner::with_pool(Arc::new(settings.clone()), pool.clone()).await?;
                runner.register_jobs().await?;
                runner.start().await?;
                tracing::info!("Job scheduler started (no lock coordination)");
                (Some(runner), None, None)
            }
        }
    } else {
        tracing::info!("Scheduler disabled in settings");
        (None, None, None)
    };

    // Build and run Tauri application
    run_tauri_app(state, job_runner, scheduler_lock, heartbeat_shutdown)
}

/// Start the scheduler with lock acquisition and heartbeat task
async fn start_scheduler_with_lock(
    settings: Settings,
    pool: sqlx::SqlitePool,
    scheduler_lock: SchedulerLock,
) -> AppResult<(Option<JobRunner>, Option<SchedulerLock>, Option<Arc<AtomicBool>>)> {
    // Get application version for lock metadata
    let version = option_env!("CARGO_PKG_VERSION").map(String::from);

    // Attempt to acquire the lock
    match scheduler_lock.try_acquire("desktop", version.as_deref()).await {
        Ok(true) => {
            tracing::info!("Scheduler lock acquired by desktop application");

            // Create shutdown signal for heartbeat task
            let shutdown_signal = Arc::new(AtomicBool::new(false));
            let shutdown_clone = shutdown_signal.clone();

            // Clone pool for heartbeat task
            let heartbeat_pool = pool.clone();

            // Start heartbeat background task
            tokio::spawn(async move {
                let lock = SchedulerLock::new(heartbeat_pool);
                let interval = tokio::time::Duration::from_secs(15);

                tracing::debug!("Scheduler lock heartbeat task started (interval: 15s)");

                loop {
                    tokio::time::sleep(interval).await;

                    // Check if shutdown was requested
                    if shutdown_clone.load(Ordering::SeqCst) {
                        tracing::debug!("Heartbeat task received shutdown signal");
                        break;
                    }

                    // Send heartbeat
                    match lock.heartbeat().await {
                        Ok(true) => {
                            tracing::trace!("Scheduler lock heartbeat sent");
                        }
                        Ok(false) => {
                            tracing::warn!("Scheduler lock heartbeat failed - lock may have been lost");
                            // Don't break here - let the main app decide what to do
                        }
                        Err(e) => {
                            tracing::warn!("Failed to send scheduler lock heartbeat: {:?}", e);
                        }
                    }
                }

                tracing::debug!("Scheduler lock heartbeat task stopped");
            });

            // Start the job runner
            let runner = JobRunner::with_pool(Arc::new(settings), pool).await?;
            runner.register_jobs().await?;
            runner.start().await?;
            tracing::info!("Job scheduler started with task scheduling enabled");

            Ok((Some(runner), Some(scheduler_lock), Some(shutdown_signal)))
        }
        Ok(false) => {
            // Lock acquisition failed - another scheduler is running
            tracing::info!("Could not acquire scheduler lock - another scheduler is active");
            tracing::info!("Desktop scheduler will NOT start");
            Ok((None, None, None))
        }
        Err(e) => {
            tracing::warn!("Error acquiring scheduler lock: {:?}", e);
            // Fall back to starting without lock
            tracing::info!("Starting scheduler without lock coordination due to error");
            let runner = JobRunner::with_pool(Arc::new(settings), pool).await?;
            runner.register_jobs().await?;
            runner.start().await?;
            tracing::info!("Job scheduler started (no lock coordination)");
            Ok((Some(runner), None, None))
        }
    }
}

/// Run Tauri in full mode
fn run_tauri_app(
    state: AppState,
    job_runner: Option<JobRunner>,
    scheduler_lock: Option<SchedulerLock>,
    heartbeat_shutdown: Option<Arc<AtomicBool>>,
) -> AppResult<()> {
    use crate::adapters::tauri::build_tauri_app;
    let app = build_tauri_app(state, job_runner).map_err(|e| AppError::Internal(e.to_string()))?;
    app.run(move |_app_handle, event| {
        // Handle shutdown event to release scheduler lock
        if let tauri::RunEvent::Exit = event {
            // Signal heartbeat task to stop
            if let Some(ref shutdown) = heartbeat_shutdown {
                shutdown.store(true, Ordering::SeqCst);
                tracing::debug!("Signaled heartbeat task to stop");
            }

            // Release the scheduler lock
            if let Some(ref lock) = scheduler_lock {
                // We need to block on this since we're in a sync context
                // Use a new runtime since Tauri's runtime is shutting down
                if let Ok(rt) = tokio::runtime::Runtime::new() {
                    rt.block_on(async {
                        match lock.release().await {
                            Ok(()) => tracing::info!("Scheduler lock released on shutdown"),
                            Err(e) => tracing::warn!("Failed to release scheduler lock on shutdown: {:?}", e),
                        }
                    });
                } else {
                    tracing::warn!("Could not create runtime to release scheduler lock");
                }
            }
        }
    });
    Ok(())
}

/// Run Tauri in fallback mode (no database)
fn run_tauri_fallback(state: AppState) -> AppResult<()> {
    use crate::adapters::tauri::build_tauri_app;
    let app = build_tauri_app(state, None).map_err(|e| AppError::Internal(e.to_string()))?;
    app.run(|_app_handle, _event| {
        // Handle Tauri events here if needed
    });
    Ok(())
}
