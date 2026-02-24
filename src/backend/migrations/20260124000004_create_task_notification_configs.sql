-- Task Notification Configuration Tables

CREATE TABLE task_notification_configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id INTEGER NOT NULL UNIQUE,
    is_enabled INTEGER NOT NULL DEFAULT 0,
    smtp_config_id INTEGER,
    email_subject TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    FOREIGN KEY (smtp_config_id) REFERENCES smtp_configs(id) ON DELETE SET NULL
);

CREATE TABLE task_notification_to_recipients (
    task_notification_config_id INTEGER NOT NULL,
    email_id INTEGER NOT NULL,
    PRIMARY KEY (task_notification_config_id, email_id),
    FOREIGN KEY (task_notification_config_id) REFERENCES task_notification_configs(id) ON DELETE CASCADE,
    FOREIGN KEY (email_id) REFERENCES emails(id) ON DELETE CASCADE
);

CREATE TABLE task_notification_cc_recipients (
    task_notification_config_id INTEGER NOT NULL,
    email_id INTEGER NOT NULL,
    PRIMARY KEY (task_notification_config_id, email_id),
    FOREIGN KEY (task_notification_config_id) REFERENCES task_notification_configs(id) ON DELETE CASCADE,
    FOREIGN KEY (email_id) REFERENCES emails(id) ON DELETE CASCADE
);

CREATE INDEX idx_task_notification_configs_task_id ON task_notification_configs(task_id);
