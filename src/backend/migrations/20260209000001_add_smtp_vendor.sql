-- Add vendor column to smtp_configs table with safe default for backward compatibility
ALTER TABLE smtp_configs ADD COLUMN vendor TEXT NOT NULL DEFAULT 'gmail';

-- Create index for vendor-based queries (performance optimization)
CREATE INDEX IF NOT EXISTS idx_smtp_configs_vendor ON smtp_configs(vendor);

-- Explicitly backfill existing records to 'gmail'
UPDATE smtp_configs SET vendor = 'gmail' WHERE vendor = 'gmail';
