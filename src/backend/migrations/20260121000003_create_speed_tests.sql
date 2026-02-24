-- Create speed_tests table for SQLite
CREATE TABLE speed_tests (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    line_id INTEGER NOT NULL,
    process_id TEXT NOT NULL,
    download_speed TEXT,
    upload_speed TEXT,
    ping TEXT,
    server_name TEXT,
    server_location TEXT,
    status TEXT,
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    FOREIGN KEY (line_id) REFERENCES lines(id) ON DELETE CASCADE
);

CREATE INDEX idx_speed_tests_line_id ON speed_tests(line_id);
CREATE INDEX idx_speed_tests_process_id ON speed_tests(process_id);
CREATE INDEX idx_speed_tests_created_at ON speed_tests(created_at);
