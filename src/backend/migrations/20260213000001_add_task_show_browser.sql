-- Add show_browser column to tasks table
-- When false (default), browser runs in headless mode (invisible)
-- When true, browser window is visible (useful for debugging)
ALTER TABLE tasks ADD COLUMN show_browser BOOLEAN NOT NULL DEFAULT 0;
