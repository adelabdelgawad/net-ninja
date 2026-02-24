// Re-export the Tauri adapter for use by the frontend's Tauri app
pub use crate::adapters::tauri;

// Make the crate a library - include all modules needed by the Tauri adapter
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

// Service module - contains scheduler lock (always available) and Windows service
// components (feature-gated within the module)
pub mod service;
