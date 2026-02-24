-- SQLite requires table recreation to modify columns
-- This migration adds: is_active, task_types (array), last_scheduled_execution
-- Also changes status CHECK constraint values

CREATE TABLE tasks_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    task_types TEXT NOT NULL CHECK (
        task_types LIKE '%speed_test%' OR task_types LIKE '%quota_check%'
    ),
    run_mode TEXT NOT NULL CHECK(run_mode IN ('one_time', 'scheduled')),
    schedule_json TEXT,
    status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending', 'running', 'completed', 'failed')),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK(is_active IN (0, 1)),
    last_scheduled_execution TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Migrate existing data:
-- - Convert task_type string to JSON array
-- - Convert status 'active' to 'pending'
-- - Set is_active = 1 for all existing tasks
-- - Set last_scheduled_execution to NULL
INSERT INTO tasks_new (id, name, task_types, run_mode, schedule_json, status, is_active, last_scheduled_execution, created_at, updated_at)
SELECT id, name, '["' || task_type || '"]', run_mode, schedule_json,
       CASE WHEN status = 'active' THEN 'pending' ELSE status END,
       1, NULL, created_at, updated_at
FROM tasks;

-- Drop old table and rename new table
DROP TABLE tasks;
ALTER TABLE tasks_new RENAME TO tasks;

-- Recreate indexes
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON tasks(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_tasks_is_active ON tasks(is_active);

-- Recreate trigger to automatically update updated_at timestamp
CREATE TRIGGER IF NOT EXISTS tasks_updated_at
AFTER UPDATE ON tasks
FOR EACH ROW
BEGIN
    UPDATE tasks SET updated_at = datetime('now') WHERE id = NEW.id;
END;
