-- Create migrations tracking table for SQLite
CREATE TABLE IF NOT EXISTS _sqlx_migrations (
    version INTEGER PRIMARY KEY,
    description TEXT NOT NULL,
    installed_on TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    success INTEGER NOT NULL,
    checksum BLOB NOT NULL,
    execution_time INTEGER NOT NULL
);
