-- Add is_active field to lines table for SQLite
ALTER TABLE lines ADD COLUMN is_active INTEGER NOT NULL DEFAULT 1;
