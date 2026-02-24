# Database Migrations

This directory contains SQLite migration files for NetNinja.

## Structure

```
migrations/
├── 20260121000000_create_migrations_table.sql
├── 20260121000001_create_lines.sql
├── 20260121000002_create_quota_results.sql
├── 20260121000003_create_speed_tests.sql
├── 20240101000001_create_lines.sql
├── 20240101000002_create_quota_results.sql
├── 20240101000003_create_speed_test_results.sql
├── 20240101000004_create_emails.sql
├── 20240101000005_create_logs.sql
├── 20240101000006_create_jobs.sql
├── 20240101000007_create_smtp_configs.sql
└── 20240101000008_create_app_config.sql
```

## Naming Convention

Migration files follow the pattern: `YYYYMMDDHHMMSS_description.sql`

Example: `20260121120000_create_lines_table.sql`

## SQLite Database

NetNinja uses SQLite as its sole database engine. The database file is stored at a platform-specific location:
- **Linux**: `~/.local/share/netninja/netninja.db`
- **macOS**: `~/Library/Application Support/netninja/netninja.db`
- **Windows**: `%APPDATA%\netninja\netninja.db`

The database is created automatically on first launch.

## Applying Migrations

Migrations are automatically applied on application startup via SQLx migrate.

## Creating New Migrations

1. Create a new SQL file with the current timestamp
2. Write SQLite-specific SQL (see existing migrations for reference)
3. Test locally before committing

## SQLite-Specific Notes

- Use `INTEGER PRIMARY KEY AUTOINCREMENT` for auto-incrementing IDs
- Use `TEXT` for string data (not VARCHAR)
- Use `REAL` for floating-point numbers
- Use `datetime('now')` for current timestamp
- Foreign keys are supported but must be enabled with `PRAGMA foreign_keys = ON;`
