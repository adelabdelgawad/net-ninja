-- Web admin credentials table (single admin account for web login)
CREATE TABLE IF NOT EXISTS web_admin_credentials (
    username TEXT PRIMARY KEY,
    password_hash TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT (datetime('now')),
    updated_at DATETIME NOT NULL DEFAULT (datetime('now'))
);
