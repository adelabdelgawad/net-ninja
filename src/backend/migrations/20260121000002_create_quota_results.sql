-- Create quota_results table for SQLite
CREATE TABLE quota_results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    line_id INTEGER NOT NULL,
    process_id TEXT NOT NULL,
    balance TEXT,
    quota_percentage TEXT,
    used_quota TEXT,
    total_quota TEXT,
    status TEXT,
    message TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    FOREIGN KEY (line_id) REFERENCES lines(id) ON DELETE CASCADE
);

CREATE INDEX idx_quota_results_line_id ON quota_results(line_id);
CREATE INDEX idx_quota_results_process_id ON quota_results(process_id);
CREATE INDEX idx_quota_results_created_at ON quota_results(created_at);
