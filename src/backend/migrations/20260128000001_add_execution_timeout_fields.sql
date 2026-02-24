-- Add timeout tracking fields for detecting stalled/orphaned executions
ALTER TABLE task_executions ADD COLUMN maximum_finish_time TEXT;
ALTER TABLE task_executions ADD COLUMN is_finished INTEGER NOT NULL DEFAULT 0;

-- Backfill completed/failed executions (they are finished)
UPDATE task_executions
SET is_finished = 1,
    maximum_finish_time = COALESCE(completed_at, started_at)
WHERE status IN ('completed', 'failed');

-- Set past deadline for orphaned 'running' executions (will be cleaned up on startup)
UPDATE task_executions
SET is_finished = 0,
    maximum_finish_time = datetime(started_at, '+1 minute')
WHERE status = 'running';

-- Index for efficient timeout queries (only scan non-finished executions)
CREATE INDEX IF NOT EXISTS idx_task_executions_timeout_check
ON task_executions(is_finished, maximum_finish_time)
WHERE is_finished = 0;
