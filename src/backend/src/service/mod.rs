//! Windows Service module for NetNinja
//!
//! This module provides Windows Service integration for running the NetNinja
//! scheduler as a background service. It handles service lifecycle, installation,
//! and logging.
//!
//! # Module Structure
//!
//! - `lock.rs`: SQLite-based advisory lock for scheduler coordination (always available)
//! - `windows.rs`: Windows Service Control Manager (SCM) integration
//! - `scheduler.rs`: Scheduler loop with lock acquisition and graceful shutdown
//! - `install.rs`: Idempotent service installation and management
//! - `logging.rs`: File and Windows Event Log logging
//!
//! # Feature Gate
//!
//! Most of this module is only compiled when both:
//! - Target platform is Windows (`cfg(windows)`)
//! - The `service` feature is enabled (`feature = "service"`)
//!
//! This prevents compilation errors on non-Windows platforms and keeps the
//! desktop application binary free of service-related code when not needed.
//!
//! **Exception**: The `lock` module is always available since both the desktop
//! application and the service need to coordinate scheduler access.

// Lock module is always available - both desktop and service need it
pub mod lock;

#[cfg(all(windows, feature = "service"))]
pub mod windows;

#[cfg(all(windows, feature = "service"))]
pub mod scheduler;

#[cfg(all(windows, feature = "service"))]
pub mod install;

#[cfg(all(windows, feature = "service"))]
pub mod logging;

// Re-export lock module items (always available)
pub use lock::SchedulerLock;

// Re-export commonly used items when the feature is enabled
#[cfg(all(windows, feature = "service"))]
pub use windows::run_service;

#[cfg(all(windows, feature = "service"))]
pub use scheduler::run_scheduler_loop;

#[cfg(all(windows, feature = "service"))]
pub use install::{install_service, uninstall_service, start_service, stop_service};

#[cfg(all(windows, feature = "service"))]
pub use logging::init_service_logging;

/// Service name as registered with Windows SCM
pub const SERVICE_NAME: &str = "netninja-scheduler";

/// Display name shown in Services management console
pub const SERVICE_DISPLAY_NAME: &str = "NetNinja Scheduler";

/// Service description
pub const SERVICE_DESCRIPTION: &str =
    "Background service for NetNinja that performs scheduled quota checks, speed tests, and notifications.";
