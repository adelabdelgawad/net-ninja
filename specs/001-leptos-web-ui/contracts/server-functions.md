# Server Function Contracts: Leptos Web Interface

Leptos `#[server]` functions compile to HTTP POST endpoints under `/api/*`.
All are invoked via Leptos's internal serialisation (by default JSON over HTTP POST).
Authentication is enforced server-side in every function except `login`.

Convention: `→ Ok(T)` on success, `→ Err(ServerFnError)` on failure.
`ServerFnError` variants: `ServerError(String)` for domain errors, automatic
HTTP 500 for unexpected panics (never exposed raw to client).

---

## Authentication

### `login(username: String, password: String) → Result<(), ServerFnError>`

- **Route (auto)**: `POST /api/login`
- **Auth required**: No (public)
- **Rate limit**: Yes — 5 failures per 60 s per IP; returns `ServerFnError::ServerError("rate_limited")` when exceeded
- **Success**: Creates session, sets `HttpOnly` `Secure` cookie, returns `Ok(())`; client redirects to `/`
- **Failure (bad creds)**: Returns `ServerFnError::ServerError("invalid_credentials")` — same message regardless of which field is wrong
- **Failure (rate limited)**: Returns `ServerFnError::ServerError("rate_limited")`
- **Failure (DB unavailable)**: Returns `ServerFnError::ServerError("service_unavailable")`

### `logout() → Result<(), ServerFnError>`

- **Route (auto)**: `POST /api/logout`
- **Auth required**: Yes
- **Success**: Deletes server-side session, clears cookie, returns `Ok(())`; client redirects to `/login`

### `change_password(current: String, new_password: String) → Result<(), ServerFnError>`

- **Route (auto)**: `POST /api/change_password`
- **Auth required**: Yes
- **Validation**: `current` must match stored hash; `new_password` min length 1
- **Success**: Updates `web_admin_credentials.password_hash`, returns `Ok(())`
- **Failure**: `ServerFnError::ServerError("invalid_current_password")`

---

## Lines

### `get_lines(page: Option<u32>, page_size: Option<u32>) → Result<PaginatedResponse<LineResponse>, ServerFnError>`

- **Auth required**: Yes
- Delegates to `LineService::get_paginated`

### `get_line(id: i32) → Result<LineResponse, ServerFnError>`

- **Auth required**: Yes

### `create_line(req: CreateLineRequest) → Result<LineResponse, ServerFnError>`

- **Auth required**: Yes
- Delegates to `LineService::create` (handles credential encryption)

### `update_line(id: i32, req: UpdateLineRequest) → Result<LineResponse, ServerFnError>`

- **Auth required**: Yes

### `delete_line(id: i32) → Result<(), ServerFnError>`

- **Auth required**: Yes
- Fails with `"has_active_tasks"` if dependent tasks exist and user has not confirmed cascade

---

## Operations (Trigger & Poll)

### `trigger_quota_check(line_id: i64) → Result<i64, ServerFnError>`

- **Auth required**: Yes
- **Busy guard**: If `operation_locks[line_id] == true`, returns `ServerFnError::ServerError("busy")`
- **Success**: Sets `operation_locks[line_id] = true`, creates `task_execution` row (status=`"running"`),
  spawns Tokio task executing `QuotaCheckService`, returns `execution_id: i64`
- **Failure (DB unavailable)**: `ServerFnError::ServerError("service_unavailable")`

### `trigger_speed_test(line_id: i64) → Result<i64, ServerFnError>`

- Identical pattern to `trigger_quota_check` but calls `SpeedTestService`

### `get_execution_status(execution_id: i64) → Result<ExecutionStatusResponse, ServerFnError>`

- **Auth required**: Yes
- **Returns**: `ExecutionStatusResponse { status: String, error_message: Option<String>, finished_at: Option<DateTime<Utc>> }`
- **Polling contract**: Client polls every 3 s; stops when `status ≠ "running"` or 300 s elapsed

```rust
pub struct ExecutionStatusResponse {
    pub status: String,          // "running" | "completed" | "failed" | "timeout"
    pub error_message: Option<String>,
    pub finished_at: Option<DateTime<Utc>>,
}
```

---

## Tasks

### `get_tasks() → Result<Vec<TaskResponse>, ServerFnError>`

- **Auth required**: Yes

### `get_task(id: i64) → Result<TaskResponse, ServerFnError>`

- **Auth required**: Yes

### `create_task(req: CreateTaskRequest) → Result<TaskResponse, ServerFnError>`

- **Auth required**: Yes

### `update_task(id: i64, req: UpdateTaskRequest) → Result<TaskResponse, ServerFnError>`

- **Auth required**: Yes

### `delete_task(id: i64) → Result<(), ServerFnError>`

- **Auth required**: Yes

### `trigger_task(task_id: i64) → Result<i64, ServerFnError>`

- **Auth required**: Yes
- Manually runs a task; returns `execution_id`

### `get_task_executions(task_id: i64) → Result<Vec<TaskExecutionResponse>, ServerFnError>`

- **Auth required**: Yes

---

## Email Settings

### `get_smtp_configs() → Result<Vec<SmtpConfig>, ServerFnError>`

- **Auth required**: Yes

### `create_smtp_config(req: CreateSmtpConfigRequest) → Result<SmtpConfig, ServerFnError>`

- **Auth required**: Yes

### `update_smtp_config(id: i32, req: UpdateSmtpConfigRequest) → Result<SmtpConfig, ServerFnError>`

- **Auth required**: Yes

### `delete_smtp_config(id: i32) → Result<(), ServerFnError>`

- **Auth required**: Yes

### `get_task_notification_config(task_id: i64) → Result<Option<TaskNotificationConfig>, ServerFnError>`

- **Auth required**: Yes

### `upsert_task_notification_config(task_id: i64, req: UpsertNotificationConfigRequest) → Result<TaskNotificationConfig, ServerFnError>`

- **Auth required**: Yes

---

## History (Read-only)

### `get_quota_results(line_id: Option<i32>, page: Option<u32>) → Result<PaginatedResponse<QuotaResult>, ServerFnError>`

- **Auth required**: Yes

### `get_speed_results(line_id: Option<i32>, page: Option<u32>) → Result<PaginatedResponse<SpeedTestResult>, ServerFnError>`

- **Auth required**: Yes

### `get_logs(line_id: Option<i64>, page: Option<u32>) → Result<PaginatedResponse<LogEntry>, ServerFnError>`

- **Auth required**: Yes

---

## Route Map (Axum + Leptos SSR)

All HTML page routes are rendered server-side by Leptos. Server functions are
POST endpoints auto-registered by `leptos_axum`.

| Method | Path | Handler | Auth |
|--------|------|---------|------|
| GET | `/login` | Leptos SSR: LoginPage | No |
| GET | `/` → redirect → `/dashboard` | | Yes |
| GET | `/dashboard` | Leptos SSR: DashboardPage | Yes |
| GET | `/lines` | Leptos SSR: LinesPage | Yes |
| GET | `/tasks` | Leptos SSR: TasksPage | Yes |
| GET | `/email-settings` | Leptos SSR: EmailSettingsPage | Yes |
| GET | `/quota-results` | Leptos SSR: QuotaResultsPage | Yes |
| GET | `/speed-results` | Leptos SSR: SpeedResultsPage | Yes |
| GET | `/logs` | Leptos SSR: LogsPage | Yes |
| POST | `/api/*` | Leptos server functions (auto) | Per fn |
| GET | `/pkg/*` | Static WASM + JS assets | No |
| GET | `/assets/*` | CSS, fonts | No |
