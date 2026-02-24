// Binary crate re-declares the full module tree but only uses bootstrap::run
// and config::Settings directly. The remaining modules are used via the library
// crate by the Tauri frontend. Suppress dead_code here rather than annotating
// every item individually.
#![allow(dead_code)]

mod adapters;
mod app;
mod bootstrap;
mod clients;
mod config;
mod crypto;
mod db;
mod errors;
mod jobs;
mod models;
mod repositories;
mod service;
mod services;
mod templates;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::bootstrap::run;
use crate::config::Settings;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "net_ninja=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting NetNinja...");

    // Clear previous run's debug logs and screenshots
    crate::config::paths::clear_logs_dir();

    // Load configuration
    let settings = match Settings::for_tauri() {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to load configuration: {:?}", e);
            std::process::exit(1);
        }
    };

    tracing::info!("Configuration loaded successfully");

    // Run the standalone Tauri application
    if let Err(e) = run(settings).await {
        tracing::error!("Application failed: {:?}", e);
        std::process::exit(1);
    }
}
