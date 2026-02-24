-- Create task_lines junction table for many-to-many relationship
CREATE TABLE IF NOT EXISTS task_lines (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id INTEGER NOT NULL,
    line_id INTEGER NOT NULL,
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    FOREIGN KEY (line_id) REFERENCES lines(id) ON DELETE CASCADE,
    UNIQUE(task_id, line_id)
);

-- Create indexes for efficient lookups
CREATE INDEX IF NOT EXISTS idx_task_lines_task_id ON task_lines(task_id);
CREATE INDEX IF NOT EXISTS idx_task_lines_line_id ON task_lines(line_id);
