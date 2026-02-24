use tauri::State;

use crate::app::AppState;
use crate::config::Settings;
use crate::db::{create_pool, run_pending_migrations};
use crate::errors::{AppError, AppResult};
use crate::jobs::JobRunner;
use crate::models::*;

// Module declarations
pub mod health;
pub mod lines;
pub mod emails;
pub mod results;
pub mod logs;
pub mod reports;
pub mod scheduler;
pub mod smtp_configs;
pub mod app;
pub mod tasks;
pub mod task_notification_configs;

/// Build the Tauri application with all commands registered
pub fn build_tauri_app(
    state: AppState,
    job_runner: Option<JobRunner>,
) -> Result<tauri::App, Box<dyn std::error::Error>> {
    use tauri::Manager;

    ::tauri::Builder::default()
        .setup(move |app| {
            app.manage(state);
            if let Some(runner) = job_runner {
                app.manage(runner);
            }
            register_commands(app);
            Ok(())
        })
        .invoke_handler(::tauri::generate_handler![
            // IMPORTANT: This command list must be kept in sync with:
            // src/frontend/src-tauri/src/lib.rs (invoke_handler)
            // When adding a new command here, also add it there!

            // Health commands
            health::health_check,
            health::get_fallback_status,
            health::get_service_status,

            // Line commands
            lines::get_lines,
            lines::get_line,
            lines::create_line,
            lines::update_line,
            lines::delete_line,

            // Email commands
            emails::get_emails,
            emails::get_email,
            emails::create_email,
            emails::update_email,
            emails::delete_email,

            // Speed test commands
            results::get_speed_tests,
            results::get_speed_test,
            results::create_speed_test,
            results::delete_speed_test,

            // Quota check commands
            results::get_quota_checks,
            results::get_quota_check,
            results::create_quota_check,
            results::delete_quota_check,
            results::get_quota_results_for_line,

            // Speed test per-line commands
            results::get_speed_tests_for_line,

            // Log commands
            logs::get_logs,
            logs::get_log,
            logs::get_logs_by_process,

            // Report commands
            reports::get_latest_report,

            // Scheduler commands
            scheduler::get_scheduler_status,
            scheduler::start_scheduler,
            scheduler::stop_scheduler,

            // Application lifecycle
            app::app_restart,
            app::get_database_path,
            app::get_logs_path,

            // SMTP config commands
            smtp_configs::get_smtp_configs,
            smtp_configs::get_smtp_config,
            smtp_configs::create_smtp_config,
            smtp_configs::update_smtp_config,
            smtp_configs::delete_smtp_config,
            smtp_configs::test_smtp_config,
            smtp_configs::get_default_smtp_config,
            smtp_configs::set_default_smtp_config,
            smtp_configs::test_smtp_config_inline,

            // Task commands
            tasks::get_tasks,
            tasks::get_task,
            tasks::create_task,
            tasks::update_task,
            tasks::delete_task,
            tasks::toggle_task_active,
            tasks::execute_task,
            tasks::stop_task,
            tasks::check_task_name_available,

            // Task execution history commands
            tasks::get_task_executions,
            tasks::get_executions,
            tasks::get_execution,
            tasks::get_latest_task_execution,
            tasks::count_executions,

            // Network diagnostics commands
            tasks::diagnose_speedtest_connectivity,

            // Resend notification
            tasks::resend_task_notification,

            // Task notification config commands
            task_notification_configs::get_task_notification_config,
            task_notification_configs::upsert_task_notification_config,
        ])
        .build(::tauri::generate_context!())
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

fn register_commands(_app: &mut tauri::App) {
    // Additional setup if needed
}

/// Initialize Tauri application state
/// This creates the AppState with database connections and services
pub async fn initialize_tauri_state() -> AppResult<AppState> {
    tracing::info!("Initializing Tauri state...");

    // Load runtime settings (for non-config values like webdriver, cron schedules)
    let settings = Settings::for_tauri()?;

    // Load encryption key
    let encryption_key = crate::crypto::load_encryption_key().map(std::sync::Arc::new);
    if encryption_key.is_some() {
        tracing::info!("Encryption key loaded successfully");
    } else {
        tracing::warn!("No encryption key found - sensitive data will not be encrypted");
    }

    // Try to connect to SQLite database
    tracing::info!("Attempting database connection...");

    match create_pool().await {
        Ok(pool) => {
            tracing::info!("SQLite pool created");

            // Run migrations
            if let Err(e) = run_pending_migrations(&pool).await {
                tracing::error!("Migration error: {:?}", e);
                // Check if this is a recoverable error
                let error_msg = e.to_string();
                if error_msg.contains("database is locked")
                    || error_msg.contains("corrupt")
                    || error_msg.contains("disk I/O error") {
                    return Ok(AppState::new_fallback(
                        settings,
                        format!("Migration error: {}", e),
                        encryption_key,
                    ));
                } else {
                    return Err(e);
                }
            }
            tracing::info!("Database migrations completed");

            // Startup safety: reset tasks/executions stuck in "running" from a prior crash
            {
                use crate::repositories::{TaskExecutionRepository, TaskRepository};

                match TaskExecutionRepository::reset_all_unfinished(&pool).await {
                    Ok(n) if n > 0 => tracing::info!("Reset {} orphaned execution(s) to 'failed'", n),
                    Ok(_) => {}
                    Err(e) => tracing::warn!("Failed to reset orphaned executions: {:?}", e),
                }
                match TaskRepository::reset_all_running(&pool).await {
                    Ok(n) if n > 0 => tracing::info!("Reset {} orphaned task(s) from 'running' to 'failed'", n),
                    Ok(_) => {}
                    Err(e) => tracing::warn!("Failed to reset orphaned tasks: {:?}", e),
                }
            }

            // Create full app state
            Ok(AppState::new(pool, settings, encryption_key))
        }
        Err(e) => {
            tracing::warn!("Database connection failed: {:?}", e);
            // Check if this is a recoverable error for fallback mode
            let error_msg = e.to_string();
            let should_fallback = error_msg.contains("database is locked")
                || error_msg.contains("permission denied")
                || error_msg.contains("corrupt")
                || error_msg.contains("disk I/O error");

            if should_fallback {
                // Fall back to settings-only mode
                Ok(AppState::new_fallback(
                    settings,
                    format!("Database connection failed: {}", e),
                    encryption_key,
                ))
            } else {
                // Unrecoverable error - fail completely
                Err(e)
            }
        }
    }
}
