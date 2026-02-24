//! NetNinja Service CLI Entry Point
//!
//! This binary provides the command-line interface for managing the NetNinja
//! Windows Service. It handles installation, uninstallation, starting, stopping,
//! and running as a Windows Service (when invoked by SCM).
//!
//! # Usage
//!
//! ```text
//! netninja-service.exe install     Install the Windows service
//! netninja-service.exe uninstall   Uninstall the Windows service
//! netninja-service.exe start       Start the installed service
//! netninja-service.exe stop        Stop the running service
//! netninja-service.exe run         Run as a service (called by SCM)
//! netninja-service.exe --help      Show this help message
//! netninja-service.exe -h          Show this help message
//! ```
//!
//! # Feature Gate
//!
//! This binary only compiles on Windows with the `service` feature enabled.

#![cfg(all(windows, feature = "service"))]

use std::process::ExitCode;

use net_ninja::config::paths;
use net_ninja::service::{
    self, init_service_logging, install_service, run_service, start_service, stop_service,
    uninstall_service,
};

/// Program name for help and error messages
const PROGRAM_NAME: &str = "netninja-service";

/// Program version from Cargo.toml
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Display usage information
fn print_usage() {
    println!(
        r#"{} v{}

NetNinja Windows Service Manager

USAGE:
    {} <COMMAND>

COMMANDS:
    install     Install the Windows service
    uninstall   Uninstall the Windows service
    start       Start the installed service
    stop        Stop the running service
    run         Run as a service (called by Windows SCM)

OPTIONS:
    -h, --help     Show this help message
    -v, --version  Show version information

EXAMPLES:
    {} install     # Install and register the service
    {} start       # Start the service
    {} stop        # Stop the running service
    {} uninstall   # Remove the service

NOTES:
    - The 'install', 'uninstall', 'start', and 'stop' commands require
      Administrator privileges.
    - The 'run' command is intended to be called by the Windows Service
      Control Manager (SCM) and should not be run manually.
    - Service data is stored in: {}

SERVICE INFO:
    Name:        {}
    Display:     {}
    Description: {}"#,
        PROGRAM_NAME,
        VERSION,
        PROGRAM_NAME,
        PROGRAM_NAME,
        PROGRAM_NAME,
        PROGRAM_NAME,
        PROGRAM_NAME,
        paths::get_shared_data_path().display(),
        service::SERVICE_NAME,
        service::SERVICE_DISPLAY_NAME,
        service::SERVICE_DESCRIPTION
    );
}

/// Display version information
fn print_version() {
    println!("{} v{}", PROGRAM_NAME, VERSION);
}

/// Handle the install command
fn handle_install() -> ExitCode {
    println!("Installing {} service...", service::SERVICE_DISPLAY_NAME);

    // Ensure shared directories exist
    if let Err(e) = paths::ensure_shared_directories() {
        eprintln!("Error: Failed to create data directories: {}", e);
        eprintln!("Path: {}", paths::get_shared_data_path().display());
        return ExitCode::from(1);
    }

    match install_service() {
        Ok(()) => {
            println!("Service installed successfully.");
            println!();
            println!("To start the service, run:");
            println!("    {} start", PROGRAM_NAME);
            println!();
            println!("Or use Windows Services management console (services.msc)");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: Failed to install service: {}", e);
            eprintln!();
            eprintln!("Make sure you are running as Administrator.");
            ExitCode::from(1)
        }
    }
}

/// Handle the uninstall command
fn handle_uninstall() -> ExitCode {
    println!("Uninstalling {} service...", service::SERVICE_DISPLAY_NAME);

    match uninstall_service() {
        Ok(()) => {
            println!("Service uninstalled successfully.");
            println!();
            println!("Note: Service data in {} was not removed.",
                     paths::get_shared_data_path().display());
            println!("You may delete this directory manually if no longer needed.");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: Failed to uninstall service: {}", e);
            eprintln!();
            eprintln!("Make sure you are running as Administrator.");
            eprintln!("If the service is running, stop it first with:");
            eprintln!("    {} stop", PROGRAM_NAME);
            ExitCode::from(1)
        }
    }
}

/// Handle the start command
fn handle_start() -> ExitCode {
    println!("Starting {} service...", service::SERVICE_DISPLAY_NAME);

    match start_service() {
        Ok(()) => {
            println!("Service started successfully.");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: Failed to start service: {}", e);
            eprintln!();
            eprintln!("Make sure you are running as Administrator.");
            eprintln!("Check that the service is installed with:");
            eprintln!("    {} install", PROGRAM_NAME);
            ExitCode::from(1)
        }
    }
}

/// Handle the stop command
fn handle_stop() -> ExitCode {
    println!("Stopping {} service...", service::SERVICE_DISPLAY_NAME);

    match stop_service() {
        Ok(()) => {
            println!("Service stopped successfully.");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: Failed to stop service: {}", e);
            eprintln!();
            eprintln!("Make sure you are running as Administrator.");
            ExitCode::from(1)
        }
    }
}

/// Handle the run command (called by Windows SCM)
fn handle_run() -> ExitCode {
    // Clear previous run's logs and screenshots so only the current session remains
    paths::clear_logs_dir();

    // Initialize service logging before anything else
    // This sets up file logging and Windows Event Log
    if let Err(e) = init_service_logging() {
        // Can't log this error since logging failed, write to stderr
        // (which won't be visible when running as a service)
        eprintln!("Error: Failed to initialize logging: {}", e);
        return ExitCode::from(1);
    }

    // Ensure shared directories exist
    if let Err(e) = paths::ensure_shared_directories() {
        tracing::error!("Failed to create data directories: {}", e);
        return ExitCode::from(1);
    }

    // Run the Windows service main entry point
    // This function blocks until the service is stopped
    match run_service() {
        Ok(()) => {
            tracing::info!("Service exited normally");
            ExitCode::SUCCESS
        }
        Err(e) => {
            tracing::error!("Service failed: {}", e);
            ExitCode::from(1)
        }
    }
}

fn main() -> ExitCode {
    // Collect command line arguments
    let args: Vec<String> = std::env::args().collect();

    // No arguments or help requested
    if args.len() < 2 {
        print_usage();
        return ExitCode::SUCCESS;
    }

    // Parse the command/option
    let command = args[1].as_str();

    match command {
        // Help options
        "-h" | "--help" | "help" | "/?" => {
            print_usage();
            ExitCode::SUCCESS
        }

        // Version options
        "-v" | "--version" | "version" => {
            print_version();
            ExitCode::SUCCESS
        }

        // Service management commands
        "install" => handle_install(),
        "uninstall" => handle_uninstall(),
        "start" => handle_start(),
        "stop" => handle_stop(),
        "run" => handle_run(),

        // Unknown command
        _ => {
            eprintln!("Error: Unknown command '{}'", command);
            eprintln!();
            eprintln!("Run '{} --help' for usage information.", PROGRAM_NAME);
            ExitCode::from(2)
        }
    }
}
