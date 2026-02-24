-- Add missing fields to smtp_configs table for SQLite
ALTER TABLE smtp_configs ADD COLUMN is_active INTEGER NOT NULL DEFAULT 1;
