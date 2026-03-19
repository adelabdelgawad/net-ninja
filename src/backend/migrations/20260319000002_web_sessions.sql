-- Web sessions table managed by tower-sessions-sqlx-store
CREATE TABLE IF NOT EXISTS tower_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    data BLOB NOT NULL,
    expiry_date INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_tower_sessions_expiry ON tower_sessions (expiry_date);
