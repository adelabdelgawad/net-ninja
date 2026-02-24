-- Scheduler Lock and Service Metadata Tables

-- Scheduler lock table for ensuring single scheduler instance
-- Used by both Windows service and desktop app to prevent concurrent scheduling
CREATE TABLE IF NOT EXISTS scheduler_lock (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    holder TEXT NOT NULL,
    acquired_at TEXT NOT NULL,
    heartbeat_at TEXT NOT NULL,
    version TEXT
);

-- Service metadata table for storing service-related information
CREATE TABLE IF NOT EXISTS service_info (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
