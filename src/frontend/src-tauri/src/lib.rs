use net_ninja::adapters::tauri;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = ::tauri::Builder::default();

    builder
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .setup(|app| {
            // Show window immediately with splash screen visible
            // This prevents the white screen while backend initializes
            if let Some(window) = ::tauri::Manager::get_webview_window(app, "main") {
                let _ = window.show();
            }

            // Initialize AppState synchronously using block_on
            // This ensures state is ready before the app starts handling commands
            let rt = tokio::runtime::Runtime::new().unwrap();
            let state = rt.block_on(async move {
                tauri::initialize_tauri_state().await
            });

            match state {
                Ok(state) => {
                    // Log fallback status BEFORE moving state
                    if state.is_fallback_mode() {
                        log::warn!("App started in fallback mode - database unavailable");
                        if let Some(error) = state.init_error.as_ref() {
                            log::warn!("Fallback reason: {}", error);
                        }
                    } else {
                        log::info!("AppState initialized successfully");
                    }

                    // Store state in app - this will be used by commands
                    ::tauri::Manager::manage(app, state);
                }
                Err(e) => {
                    // Still allow app to start even if state init completely fails
                    // This is a last-resort fallback - should rarely happen
                    log::error!("Critical: Failed to initialize even fallback state: {}", e);
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Critical initialization failure: {}", e),
                    )) as Box<dyn std::error::Error>);
                }
            }

            Ok(())
        })
        .invoke_handler(::tauri::generate_handler![
            // IMPORTANT: This command list must be kept in sync with:
            // src/backend/src/adapters/tauri/mod.rs (build_tauri_app)
            // When adding a new command here, also add it there!

            // Health
            tauri::health::health_check,
            tauri::health::get_fallback_status,
            tauri::health::get_service_status,

            // Lines
            tauri::lines::get_lines,
            tauri::lines::get_line,
            tauri::lines::create_line,
            tauri::lines::update_line,
            tauri::lines::delete_line,

            // Emails
            tauri::emails::get_emails,
            tauri::emails::get_email,
            tauri::emails::create_email,
            tauri::emails::update_email,
            tauri::emails::delete_email,

            // Speed tests
            tauri::results::get_speed_tests,
            tauri::results::get_speed_test,
            tauri::results::create_speed_test,
            tauri::results::delete_speed_test,

            // Quota checks
            tauri::results::get_quota_checks,
            tauri::results::get_quota_check,
            tauri::results::create_quota_check,
            tauri::results::delete_quota_check,
            tauri::results::get_quota_results_for_line,

            // Speed tests per-line
            tauri::results::get_speed_tests_for_line,

            // Logs
            tauri::logs::get_logs,
            tauri::logs::get_log,
            tauri::logs::get_logs_by_process,

            // Reports
            tauri::reports::get_latest_report,

            // SMTP configs
            tauri::smtp_configs::get_smtp_configs,
            tauri::smtp_configs::get_smtp_config,
            tauri::smtp_configs::create_smtp_config,
            tauri::smtp_configs::update_smtp_config,
            tauri::smtp_configs::delete_smtp_config,
            tauri::smtp_configs::test_smtp_config,
            tauri::smtp_configs::get_default_smtp_config,
            tauri::smtp_configs::set_default_smtp_config,
            tauri::smtp_configs::test_smtp_config_inline,

            // Tasks
            tauri::tasks::get_tasks,
            tauri::tasks::get_task,
            tauri::tasks::create_task,
            tauri::tasks::update_task,
            tauri::tasks::delete_task,
            tauri::tasks::toggle_task_active,
            tauri::tasks::execute_task,
            tauri::tasks::stop_task,
            tauri::tasks::check_task_name_available,

            // Task execution history
            tauri::tasks::get_task_executions,
            tauri::tasks::get_executions,
            tauri::tasks::get_execution,
            tauri::tasks::get_latest_task_execution,
            tauri::tasks::count_executions,
            tauri::tasks::resend_task_notification,

            // Task notification configs
            tauri::task_notification_configs::get_task_notification_config,
            tauri::task_notification_configs::upsert_task_notification_config,

            // Network diagnostics
            tauri::tasks::diagnose_speedtest_connectivity,

            // Scheduler
            tauri::scheduler::get_scheduler_status,
            tauri::scheduler::start_scheduler,
            tauri::scheduler::stop_scheduler,

            // Application lifecycle
            tauri::app::app_restart,
            tauri::app::get_database_path,
            tauri::app::get_logs_path,
        ])
        .run(::tauri::generate_context!())
        .expect("error while running tauri application");
}
