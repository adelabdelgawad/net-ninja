-- Add missing fields to speed_tests table for SQLite
ALTER TABLE speed_tests ADD COLUMN public_ip TEXT;
