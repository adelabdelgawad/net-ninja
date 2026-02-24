-- Add missing fields to quota_results table for SQLite
ALTER TABLE quota_results ADD COLUMN remaining_quota TEXT;
ALTER TABLE quota_results ADD COLUMN renewal_date TEXT;
ALTER TABLE quota_results ADD COLUMN renewal_cost TEXT;
ALTER TABLE quota_results ADD COLUMN extra_quota TEXT;
