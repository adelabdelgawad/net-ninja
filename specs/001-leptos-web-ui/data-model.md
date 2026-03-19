# Data Model: Leptos Web Interface (001-leptos-web-ui)

## New Entities (web-only)

### WebAdminCredential

Stores the single admin account for web login. One row always exists.

| Column | Type | Constraints | Notes |
|---|---|---|---|
| `username` | TEXT | PRIMARY KEY | Fixed value: `"admin"` |
| `password_hash` | TEXT | NOT NULL | Argon2id hash |
| `created_at` | DATETIME | NOT NULL DEFAULT (datetime('now')) | |
| `updated_at` | DATETIME | NOT NULL DEFAULT (datetime('now')) | |

**Invariants**:
- Exactly one row. If absent on startup, the web app inserts the default (`admin` / hash of `"admin"`).
- `password_hash` is never the plaintext password — only the Argon2id output.
- `username` is immutable; only `password_hash` and `updated_at` are ever modified.

**Migration**: `20260319000001_web_admin_credentials.sql`

---

### WebSession

Server-side session store for active web sessions. Managed by `tower-sessions-sqlx-store`.

| Column | Type | Constraints | Notes |
|---|---|---|---|
| `id` | TEXT | PRIMARY KEY | Random UUID, delivered to browser as HttpOnly cookie |
| `data` | BLOB | NOT NULL | Serialised session data (tower-sessions internal format) |
| `expiry_date` | INTEGER | NOT NULL | Unix timestamp; session invalid after this time |

**Invariants**:
- Rows with `expiry_date < now()` are treated as expired; `tower-sessions` handles cleanup.
- On password reset: all rows in this table are deleted before the new password is written
  (FR-013 step b — invalidate all sessions).

**Migration**: `20260319000002_web_sessions.sql`

---

## Shared Entities (read/written by both web app and desktop app)

These entities already exist. The web app reads and writes them through the shared
`net_ninja` service layer — no schema changes are made to these tables.

### Line (existing: `lines` table)

| Field | Type | Notes |
|---|---|---|
| `id` | INTEGER PK | |
| `name` | TEXT | Display name |
| `line_number` | TEXT | ISP account/line number |
| `username` | TEXT | ISP portal username (encrypted at rest via AES-256-GCM) |
| `password` | TEXT | ISP portal password (encrypted at rest) |
| `ip_address` | TEXT? | Source IP for speed tests |
| `isp` | TEXT? | `"WE"` or `"Orange"` |
| `description` | TEXT? | |
| `gateway_ip` | TEXT? | |
| `is_active` | BOOLEAN | |
| `created_at` | DATETIME | |
| `updated_at` | DATETIME | |

**Web app behaviour**: The web app calls `LineService::*` exactly as the Tauri adapter does.
Credentials pass through the existing `EncryptionKey` path — no change to encryption logic.

---

### Task (existing: `tasks` table)

| Field | Type | Notes |
|---|---|---|
| `id` | INTEGER PK | |
| `name` | TEXT | |
| `task_types` | TEXT | JSON array: `["quota_check"]`, `["speed_test"]`, or both |
| `run_mode` | TEXT | `"one_time"` or `"scheduled"` |
| `schedule_json` | TEXT? | Serialised `Schedule` (days + times) |
| `status` | TEXT | `"idle"`, `"running"`, `"failed"`, `"completed"` |
| `is_active` | BOOLEAN | |
| `show_browser` | BOOLEAN | Headless (false) or visible Chrome (true) |
| `last_scheduled_execution` | TEXT? | |
| `created_at` | DATETIME | |
| `updated_at` | DATETIME | |

Related: `task_lines` join table (task_id, line_id).

---

### TaskExecution (existing: `task_executions` table)

Used by the polling mechanism (FR-009, FR-010).

| Field | Type | Notes |
|---|---|---|
| `id` | INTEGER PK | Returned to client after trigger; polled for status |
| `task_id` | INTEGER FK | |
| `status` | TEXT | `"running"` → `"completed"` / `"failed"` / `"timeout"` |
| `started_at` | DATETIME | |
| `finished_at` | DATETIME? | |
| `duration_ms` | INTEGER? | |
| `error_message` | TEXT? | |

**Polling contract**: After triggering an operation, the client polls
`get_execution_status(execution_id)` every 3 seconds. When `status ≠ "running"`, polling
stops and the result is rendered. After 5 minutes (300 s) the client stops polling
regardless and shows a timeout error (FR-010a).

---

### SmtpConfig (existing: `smtp_configs` table)

| Field | Type | Notes |
|---|---|---|
| `id` | INTEGER PK | |
| `name` | TEXT | |
| `host` | TEXT | |
| `port` | INTEGER | |
| `vendor` | TEXT | `"gmail"`, `"exchange"`, `"outlook365"` |
| `username` | TEXT? | |
| `password` | TEXT? | Stored encrypted |
| `from_email` | TEXT | |
| `from_name` | TEXT? | |
| `use_tls` | BOOLEAN | |
| `is_default` | BOOLEAN | |
| `is_active` | BOOLEAN | |
| `created_at` | DATETIME | |
| `updated_at` | DATETIME | |

---

### TaskNotificationConfig (existing: `task_notification_configs` table)

| Field | Type | Notes |
|---|---|---|
| `id` | INTEGER PK | |
| `task_id` | INTEGER FK | |
| `smtp_config_id` | INTEGER FK | |
| `to_emails` | TEXT | JSON array of recipient addresses |
| `cc_emails` | TEXT? | JSON array |
| `subject_template` | TEXT? | |
| `created_at` | DATETIME | |
| `updated_at` | DATETIME | |

---

### QuotaResult (existing: `quota_results` table)

Read-only from web UI. Written by `QuotaCheckService`.

| Field | Type | Notes |
|---|---|---|
| `id` | INTEGER PK | |
| `line_id` | INTEGER FK | |
| `task_execution_id` | INTEGER FK? | |
| `used_gb` | REAL? | |
| `total_gb` | REAL? | |
| `remaining_gb` | REAL? | |
| `status` | TEXT | `"success"` / `"failed"` |
| `error_message` | TEXT? | |
| `checked_at` | DATETIME | |

---

### SpeedTestResult (existing: `speed_test_results` table)

Read-only from web UI. Written by `SpeedTestService`.

| Field | Type | Notes |
|---|---|---|
| `id` | INTEGER PK | |
| `line_id` | INTEGER FK | |
| `task_execution_id` | INTEGER FK? | |
| `download_mbps` | REAL? | |
| `upload_mbps` | REAL? | |
| `latency_ms` | REAL? | |
| `status` | TEXT | `"success"` / `"failed"` |
| `error_message` | TEXT? | |
| `tested_at` | DATETIME | |

---

### Log (existing: `logs` table)

Read-only from web UI. 7-day retention enforced by cleanup job.

| Field | Type | Notes |
|---|---|---|
| `id` | INTEGER PK | |
| `process_id` | TEXT | Unique identifier per run/operation |
| `line_id` | INTEGER FK? | Associated line (nullable for global events) |
| `level` | TEXT | `"info"`, `"warn"`, `"error"` |
| `message` | TEXT | |
| `created_at` | DATETIME | |

---

## In-Memory State (WebAppState — not persisted)

| Field | Type | Notes |
|---|---|---|
| `operation_locks` | `DashMap<i64, bool>` | Per-line operation guard (FR-010b) |
| `rate_limiters` | `DashMap<IpAddr, RateLimiter>` | Per-IP login rate limiting (FR-006) |
| `pool` | `Option<SqlitePool>` | None in fallback mode |
| `session_store` | `SqliteStore` | tower-sessions store backed by `web_sessions` table |
| `encryption_key` | `Option<Arc<EncryptionKey>>` | Reused from backend lib |

---

## State Transitions

### Session lifecycle

```
[no cookie] → login(admin, admin) → [session created, cookie set]
     ↓                                        ↓
[redirect /login]              [active session, 24h inactivity window]
                                         ↓                   ↓
                               logout / reset file      expiry_date reached
                                         ↓                   ↓
                               [session deleted]      [session expired]
                                         ↓
                               [redirect /login]
```

### Operation lifecycle (per line)

```
[idle: lock=false, no spinner]
   → admin clicks trigger
   → server fn creates task_execution (status="running"), sets lock=true
   → client shows spinner, polls every 3s
      ↓                                ↓ (after 5 min)
   status≠"running"              timeout error shown
   lock=false                    lock=false
   result rendered inline        admin may retry
```

### Password reset lifecycle

```
startup: check for reset_admin_password.bat
   ↓ file absent            ↓ file present
   (no action)     delete all web_sessions
                   update web_admin_credentials (hash of "admin")
                   delete reset_admin_password.bat
                   log reset event (no password value)
                   → admin logs in with admin/admin
                   → warning banner shown until password changed
```
