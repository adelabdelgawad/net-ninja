#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::config::get_configuration;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
    use std::str::FromStr;
    use std::sync::Arc;
    use tower::ServiceBuilder;
    use tower_sessions::Expiry;
    use tower_sessions::SessionManagerLayer;
    use tower_sessions_sqlx_store::SqliteStore;
    use dashmap::DashMap;

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "netninja_web=info,tower_http=warn".into()),
        )
        .init();

    tracing::info!("Starting NetNinja Web...");

    // Get config from leptos metadata
    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;

    // Database path
    let data_path = net_ninja::config::paths::get_shared_data_path();
    std::fs::create_dir_all(&data_path).expect("Failed to create data directory");
    let db_path = data_path.join("netninja.db");

    // Connect to SQLite with WAL mode
    let connect_options = SqliteConnectOptions::from_str(&format!("sqlite://{}", db_path.display()))
        .expect("Invalid SQLite URL")
        .journal_mode(SqliteJournalMode::Wal)
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options)
        .await
        .expect("Failed to connect to database");

    // Run migrations
    sqlx::migrate!("../../src/backend/migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Load settings
    let settings = net_ninja::config::Settings::load()
        .unwrap_or_else(|_| net_ninja::config::Settings {
            webdriver: net_ninja::config::WebDriverSettings {
                chrome_path: None,
                headless: true,
                auto_install: true,
            },
            scheduler: net_ninja::config::SchedulerSettings { enabled: true },
            quota_check: net_ninja::config::QuotaCheckSettings {
                cron: "0 0 6 * * *".to_string(),
            },
            speed_test: net_ninja::config::SpeedTestSettings {
                cron: "0 0 */4 * * *".to_string(),
            },
            cleanup: net_ninja::config::CleanupSettings {
                cron: "0 0 0 * * *".to_string(),
                retention_days: 90,
            },
        });
    let settings = Arc::new(settings);

    // Load encryption key
    let encryption_key = net_ninja::crypto::load_encryption_key().map(Arc::new);

    // Startup tasks
    netninja_web::startup::bootstrap_admin(&pool)
        .await
        .expect("Failed to bootstrap admin");
    netninja_web::startup::reset_orphan_executions(&pool)
        .await
        .expect("Failed to reset orphan executions");

    // Setup session store
    let session_store = SqliteStore::new(pool.clone());
    session_store.migrate().await.expect("Failed to migrate session store");

    // Check password reset file
    let _ = netninja_web::startup::check_reset_file(&pool, &session_store).await;

    // Build WebAppState
    let web_state = netninja_web::state::WebAppState {
        pool: Some(pool.clone()),
        session_store: session_store.clone(),
        operation_locks: Arc::new(DashMap::new()),
        rate_limiters: Arc::new(DashMap::new()),
        encryption_key,
        settings,
    };

    // Session layer: 24h inactivity, HttpOnly cookie
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // set to true in production with HTTPS
        .with_expiry(Expiry::OnInactivity(tower_sessions::cookie::time::Duration::hours(24)));

    // Generate route list
    let routes = generate_route_list(netninja_web::App);

    // Build axum router
    let app = Router::new()
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            {
                let web_state = web_state.clone();
                move || {
                    provide_context(web_state.clone());
                }
            },
            {
                let leptos_options = leptos_options.clone();
                move || netninja_web::shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler(netninja_web::shell))
        .layer(
            ServiceBuilder::new()
                .layer(session_layer)
        )
        .with_state(leptos_options);

    tracing::info!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // Client-side entry: replaced by the wasm-bindgen generated code
}
