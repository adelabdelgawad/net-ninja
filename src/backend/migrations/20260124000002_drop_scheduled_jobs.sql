-- Drop unused scheduled_jobs and job_executions tables
-- These tables were part of a legacy scheduling system that was never used.
-- The application now uses the tasks table for user-defined scheduled automation.

DROP TABLE IF EXISTS job_executions;
DROP TABLE IF EXISTS scheduled_jobs;
