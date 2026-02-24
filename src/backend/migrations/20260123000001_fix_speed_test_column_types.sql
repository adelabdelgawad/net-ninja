-- Fix speed_tests column types: change TEXT to REAL for numeric fields
-- SQLite requires table recreation to change column types

-- Create new table with correct types
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

-- Migrate existing data, converting TEXT to REAL
-- CAST handles NULL and empty strings gracefully
INSERT INTO speed_tests_new (id, line_id, process_id, download_speed, upload_speed, ping, server_name, server_location, public_ip, status, error_message, created_at)
SELECT
    id,
    line_id,
    process_id,
    CASE WHEN download_speed IS NOT NULL AND download_speed != '' THEN CAST(download_speed AS REAL) ELSE NULL END,
    CASE WHEN upload_speed IS NOT NULL AND upload_speed != '' THEN CAST(upload_speed AS REAL) ELSE NULL END,
    CASE WHEN ping IS NOT NULL AND ping != '' THEN CAST(ping AS REAL) ELSE NULL END,
    server_name,
    server_location,
    public_ip,
    status,
    error_message,
    created_at
FROM speed_tests;

-- Drop old table
DROP TABLE speed_tests;

-- Rename new table
ALTER TABLE speed_tests_new RENAME TO speed_tests;

-- Recreate indexes
CREATE INDEX idx_speed_tests_line_id ON speed_tests(line_id);
CREATE INDEX idx_speed_tests_process_id ON speed_tests(process_id);
CREATE INDEX idx_speed_tests_created_at ON speed_tests(created_at);
