-- Fix process_id columns: convert BLOB values to TEXT
-- This migration handles data that was incorrectly stored as BLOB UUIDs

-- ===== FIX quota_results TABLE =====
-- Create new table with correct schema
CREATE TABLE quota_results_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    line_id INTEGER NOT NULL,
    process_id TEXT NOT NULL,
    balance TEXT,
    quota_percentage TEXT,
    used_quota TEXT,
    total_quota TEXT,
    remaining_quota TEXT,
    renewal_date TEXT,
    renewal_cost TEXT,
    extra_quota TEXT,
    status TEXT,
    message TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    FOREIGN KEY (line_id) REFERENCES lines(id) ON DELETE CASCADE
);

-- Migrate data, converting BLOB to TEXT hex format if needed
-- SQLite's typeof() returns 'blob' for BLOB values
INSERT INTO quota_results_new (id, line_id, process_id, balance, quota_percentage, used_quota, total_quota, remaining_quota, renewal_date, renewal_cost, extra_quota, status, message, created_at)
SELECT
    id,
    line_id,
    CASE
        WHEN typeof(process_id) = 'blob' THEN
            lower(
                substr(hex(process_id), 1, 8) || '-' ||
                substr(hex(process_id), 9, 4) || '-' ||
                substr(hex(process_id), 13, 4) || '-' ||
                substr(hex(process_id), 17, 4) || '-' ||
                substr(hex(process_id), 21, 12)
            )
        ELSE process_id
    END,
    balance,
    quota_percentage,
    used_quota,
    total_quota,
    remaining_quota,
    renewal_date,
    renewal_cost,
    extra_quota,
    status,
    message,
    created_at
FROM quota_results;

-- Drop old table
DROP TABLE quota_results;

-- Rename new table
ALTER TABLE quota_results_new RENAME TO quota_results;

-- Recreate indexes
CREATE INDEX idx_quota_results_line_id ON quota_results(line_id);
CREATE INDEX idx_quota_results_process_id ON quota_results(process_id);
CREATE INDEX idx_quota_results_created_at ON quota_results(created_at);

-- ===== FIX speed_tests TABLE =====
CREATE TABLE speed_tests_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    line_id INTEGER NOT NULL,
    process_id TEXT NOT NULL,
    download_speed REAL,
    upload_speed REAL,
    ping REAL,
    server_name TEXT,
    server_location TEXT,
    public_ip TEXT,
    status TEXT,
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    FOREIGN KEY (line_id) REFERENCES lines(id) ON DELETE CASCADE
);

INSERT INTO speed_tests_new (id, line_id, process_id, download_speed, upload_speed, ping, server_name, server_location, public_ip, status, error_message, created_at)
SELECT
    id,
    line_id,
    CASE
        WHEN typeof(process_id) = 'blob' THEN
            lower(
                substr(hex(process_id), 1, 8) || '-' ||
                substr(hex(process_id), 9, 4) || '-' ||
                substr(hex(process_id), 13, 4) || '-' ||
                substr(hex(process_id), 17, 4) || '-' ||
                substr(hex(process_id), 21, 12)
            )
        ELSE process_id
    END,
    download_speed,
    upload_speed,
    ping,
    server_name,
    server_location,
    public_ip,
    status,
    error_message,
    created_at
FROM speed_tests;

DROP TABLE speed_tests;
ALTER TABLE speed_tests_new RENAME TO speed_tests;

CREATE INDEX idx_speed_tests_line_id ON speed_tests(line_id);
CREATE INDEX idx_speed_tests_process_id ON speed_tests(process_id);
CREATE INDEX idx_speed_tests_created_at ON speed_tests(created_at);

-- ===== FIX logs TABLE =====
CREATE TABLE logs_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    process_id TEXT NOT NULL,
    level TEXT NOT NULL DEFAULT 'info' CHECK (level IN ('debug', 'info', 'warning', 'error')),
    function_name TEXT,
    message TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc'))
);

INSERT INTO logs_new (id, process_id, level, function_name, message, created_at)
SELECT
    id,
    CASE
        WHEN typeof(process_id) = 'blob' THEN
            lower(
                substr(hex(process_id), 1, 8) || '-' ||
                substr(hex(process_id), 9, 4) || '-' ||
                substr(hex(process_id), 13, 4) || '-' ||
                substr(hex(process_id), 17, 4) || '-' ||
                substr(hex(process_id), 21, 12)
            )
        ELSE process_id
    END,
    level,
    function_name,
    message,
    created_at
FROM logs;

DROP TABLE logs;
ALTER TABLE logs_new RENAME TO logs;

CREATE INDEX idx_logs_process_id ON logs(process_id);
CREATE INDEX idx_logs_level ON logs(level);
CREATE INDEX idx_logs_created_at ON logs(created_at);
