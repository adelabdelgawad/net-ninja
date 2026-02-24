-- Create task_executions table for tracking task execution history
CREATE TABLE IF NOT EXISTS task_executions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    execution_id TEXT NOT NULL UNIQUE,
    triggered_by TEXT NOT NULL CHECK(triggered_by IN ('manual', 'scheduler')),
    scheduled_for TEXT,
    started_at TEXT NOT NULL,
    completed_at TEXT,
    status TEXT NOT NULL DEFAULT 'running' CHECK(status IN ('running', 'completed', 'failed')),
    error_message TEXT,
    duration_ms INTEGER,
    result_summary TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Create task_execution_results table for per-line execution results
CREATE TABLE IF NOT EXISTS task_execution_results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    execution_id TEXT NOT NULL,
    line_id INTEGER NOT NULL,
    task_type TEXT NOT NULL CHECK(task_type IN ('speed_test', 'quota_check')),
    status TEXT NOT NULL CHECK(status IN ('success', 'failed')),
    error_message TEXT,
    duration_ms INTEGER,
    started_at TEXT,
    completed_at TEXT,
    FOREIGN KEY (execution_id) REFERENCES task_executions(execution_id) ON DELETE CASCADE
);

-- Create indexes for common queries
CREATE INDEX IF NOT EXISTS idx_task_executions_task_id ON task_executions(task_id);
CREATE INDEX IF NOT EXISTS idx_task_executions_status ON task_executions(status);
CREATE INDEX IF NOT EXISTS idx_task_executions_started_at ON task_executions(started_at DESC);
CREATE INDEX IF NOT EXISTS idx_task_executions_triggered_by ON task_executions(triggered_by);

CREATE INDEX IF NOT EXISTS idx_task_execution_results_execution_id ON task_execution_results(execution_id);
CREATE INDEX IF NOT EXISTS idx_task_execution_results_line_id ON task_execution_results(line_id);
CREATE INDEX IF NOT EXISTS idx_task_execution_results_task_type ON task_execution_results(task_type);
