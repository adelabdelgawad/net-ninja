-- Add optional line_id column to logs table
ALTER TABLE logs ADD COLUMN line_id INTEGER REFERENCES lines(id) ON DELETE SET NULL;
CREATE INDEX idx_logs_line_id ON logs(line_id);
