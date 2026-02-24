-- Create smtp_configs table for SQLite
CREATE TABLE smtp_configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    host TEXT NOT NULL,
    port INTEGER NOT NULL DEFAULT 587,
    username TEXT,
    password TEXT,
    from_email TEXT NOT NULL,
    from_name TEXT,
    use_tls INTEGER NOT NULL DEFAULT 1,
    is_default INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc'))
);

-- Trigger to update updated_at on row changes
CREATE TRIGGER update_smtp_configs_updated_at
    AFTER UPDATE ON smtp_configs
    FOR EACH ROW
BEGIN
    UPDATE smtp_configs SET updated_at = datetime('now', 'utc')
    WHERE id = NEW.id;
END;
