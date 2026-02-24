# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/), and this project adheres to [Semantic Versioning](https://semver.org/).

## [0.1.0] - 2026-02-07

### Added
- Automated ISP quota checking via headless Chrome (chaser-oxide)
- Network speed testing with source IP binding for multi-WAN setups
- Task scheduler with cron expressions, timeout handling, and crash recovery
- Email notifications with per-task SMTP configuration
- Multi-line internet connection management
- Line-level logging with process ID tracing
- Desktop UI with dark theme, sidebar navigation, data tables, and charts
- SQLite database with embedded migrations
- AES-256-GCM encryption for stored credentials
- Fallback mode when database is unavailable
- Settings management via native UI
