-- Add missing fields to lines table for SQLite
ALTER TABLE lines ADD COLUMN description TEXT;
ALTER TABLE lines ADD COLUMN gateway_ip TEXT;
