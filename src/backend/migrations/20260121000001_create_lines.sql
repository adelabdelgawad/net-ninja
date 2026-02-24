-- Create lines table for SQLite
CREATE TABLE lines (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    line_number TEXT NOT NULL UNIQUE,
    username TEXT,
    password TEXT,
    ip_address TEXT,
    isp TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc'))
);

CREATE INDEX idx_lines_name ON lines(name);
CREATE INDEX idx_lines_line_number ON lines(line_number);

-- Trigger to update updated_at on row changes
CREATE TRIGGER update_lines_updated_at
    AFTER UPDATE ON lines
    FOR EACH ROW
BEGIN
    UPDATE lines SET updated_at = datetime('now', 'utc')
    WHERE id = NEW.id;
END;
