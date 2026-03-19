// Re-export the Tauri adapter only when tauri-app feature is enabled
#[cfg(feature = "tauri-app")]
pub use crate::adapters::tauri;

// All modules still public for the web app
pub mod adapters;
pub mod app;
pub mod bootstrap;
pub mod clients;
pub mod config;
pub mod crypto;
pub mod db;
pub mod errors;
pub mod jobs;
pub mod models;
pub mod repositories;
pub mod services;
pub mod templates;
pub mod service;
