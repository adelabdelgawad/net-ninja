-- Create scheduled_jobs and job_executions tables for SQLite
-- Note: SQLite doesn't support ENUM, using TEXT with CHECK constraints

CREATE TABLE scheduled_jobs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    task_function TEXT NOT NULL CHECK (task_function IN ('quota_check', 'speed_test', 'send_report', 'cleanup')),
    schedule_type TEXT NOT NULL CHECK (schedule_type IN ('cron', 'interval', 'once')),
    schedule_value TEXT NOT NULL,
    is_enabled INTEGER NOT NULL DEFAULT 1,
    last_run_at TEXT,
    next_run_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc'))
);

CREATE TABLE job_executions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    job_id INTEGER NOT NULL,
    process_id TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'running', 'completed', 'failed', 'cancelled')),
    started_at TEXT,
    completed_at TEXT,
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    FOREIGN KEY (job_id) REFERENCES scheduled_jobs(id) ON DELETE CASCADE
);

CREATE INDEX idx_scheduled_jobs_is_enabled ON scheduled_jobs(is_enabled);
CREATE INDEX idx_scheduled_jobs_next_run_at ON scheduled_jobs(next_run_at);
CREATE INDEX idx_job_executions_job_id ON job_executions(job_id);
CREATE INDEX idx_job_executions_status ON job_executions(status);
CREATE INDEX idx_job_executions_process_id ON job_executions(process_id);

-- Trigger to update updated_at on row changes
CREATE TRIGGER update_scheduled_jobs_updated_at
    AFTER UPDATE ON scheduled_jobs
    FOR EACH ROW
BEGIN
    UPDATE scheduled_jobs SET updated_at = datetime('now', 'utc')
    WHERE id = NEW.id;
END;
