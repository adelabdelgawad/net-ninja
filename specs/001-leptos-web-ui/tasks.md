# Tasks: Leptos Web Interface with Admin Login & Password Reset

**Input**: Design documents from `/specs/001-leptos-web-ui/`
**Prerequisites**: plan.md ✅ spec.md ✅ research.md ✅ data-model.md ✅ contracts/ ✅ quickstart.md ✅

**Tests**: Not requested — no test tasks generated.

**Organization**: Tasks are grouped by user story to enable independent implementation
and testing of each story. US3 (Line server functions) precedes US2 (Dashboard page)
because the DashboardPage depends on `get_lines()` from `server_fns/lines.rs`.
**Architecture**: src/backend = domain/infra library; src/app = Leptos UI layer (SSR+WASM in one binary, standard cargo-leptos workspace pattern)

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no unresolved dependencies)
- **[Story]**: User story this task belongs to (US1–US7)
- Paths are relative to repo root unless otherwise noted

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create the new Cargo workspace root and the `src/app` crate skeleton.
No user story work can begin until this phase is complete.

- [X] T001 Create root `/Cargo.toml` declaring workspace members `["src/backend", "src/app"]`; add `[workspace.dependencies]` block for shared Leptos 0.7, Axum 0.7, Tokio 1, SQLx versions
- [X] T002 Create `src/app/Cargo.toml` with all required dependencies: `leptos`, `leptos_axum`, `axum`, `tower`, `tower-sessions`, `tower-sessions-sqlx-store`, `argon2`, `governor`, `dashmap`, `net-ninja` (path dep), `sqlx`, `tokio`, `tracing`, `serde`, `chrono`
- [X] T003 [P] Create `src/app/build.rs` for cargo-leptos build support (WASM target detection, Tailwind CSS asset pipeline)
- [X] T004 [P] Create `src/app/style/main.css` as Tailwind CSS 4 input file with base `@import "tailwindcss"` directive and dark-theme design tokens matching existing desktop app
- [X] T005 [P] Create `src/backend/migrations/20260319000001_web_admin_credentials.sql`: `web_admin_credentials` table (`username TEXT PK`, `password_hash TEXT NOT NULL`, `created_at`, `updated_at`)
- [X] T006 [P] Create `src/backend/migrations/20260319000002_web_sessions.sql`: `web_sessions` table (`id TEXT PK`, `data BLOB NOT NULL`, `expiry_date INTEGER NOT NULL`) matching the tower-sessions-sqlx-store schema

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: All infrastructure that MUST be complete before any user story can be
implemented — state, auth primitives, shared UI components, server entry point.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [X] T007 Create `src/app/src/state.rs`: define `WebAppState` struct holding `pool: Option<SqlitePool>`, `session_store: SqliteStore`, `operation_locks: DashMap<i64, bool>`, `rate_limiters: DashMap<IpAddr, RateLimiter>`, `encryption_key: Option<Arc<EncryptionKey>>`; implement `Clone` and Axum `FromRef` extractors
- [X] T008 [P] Create `src/app/src/auth/mod.rs` re-exporting session and rate_limit modules; create `src/app/src/auth/session.rs` with `get_session(session: Session) -> Option<String>` and `require_session` helper that returns `Result<String, Redirect>` — redirects to `/login` if unauthenticated
- [X] T009 [P] Create `src/app/src/auth/rate_limit.rs`: `check_rate_limit(state: &WebAppState, ip: IpAddr) -> bool` using `governor` GCRA; 5 failures per 60 s per IP; entries stored in `state.rate_limiters` DashMap
- [X] T010 [P] Create `src/app/src/components/ui/mod.rs` and implement five primitive components: `src/app/src/components/ui/button.rs` (primary/secondary/destructive variants), `src/app/src/components/ui/input.rs` (label + error display), `src/app/src/components/ui/badge.rs` (success/failure/running status), `src/app/src/components/ui/alert.rs` (warning/info banners), `src/app/src/components/ui/dialog.rs` (confirmation modal with cancel/confirm actions)
- [X] T011 [P] Create `src/app/src/components/spinner.rs`: inline `Spinner` Leptos component accepting `size` prop; used for per-line operation loading state
- [X] T012 [P] Create `src/app/src/components/table.rs`: reusable `DataTable<T>` Leptos component accepting columns config + rows signal; supports pagination via props
- [X] T013 Create `src/app/src/components/layout.rs`: `AppShell` component (sidebar + main slot), `Sidebar` component with nav links (Dashboard, Lines, Tasks, Email Settings, Quota Results, Speed Results, Logs, Logout), `TopBar` component — all responsive down to 375 px
- [X] T014 Create `src/app/src/components/mod.rs` re-exporting `layout`, `table`, `spinner`, and `ui` modules
- [X] T015 Create `src/app/src/server_fns/mod.rs` declaring `pub mod auth; pub mod lines; pub mod tasks; pub mod operations; pub mod email; pub mod history;` — each sub-module file created as empty stubs
- [X] T016 Create `src/app/src/pages/mod.rs` declaring `pub mod login; pub mod dashboard; pub mod lines; pub mod tasks; pub mod email_settings; pub mod quota_results; pub mod speed_results; pub mod logs;` — each sub-module file created as a minimal stub returning `view! { <div/> }`
- [X] T017 Create `src/app/src/startup.rs`: implement `bootstrap_admin(pool: &SqlitePool)` (insert default `admin` / argon2id("admin") credential if `web_admin_credentials` is empty), `reset_orphan_executions(pool: &SqlitePool)` (set running task_executions to failed on startup using `TaskExecutionRepository`), and `check_reset_file(pool: &SqlitePool, store: &SqliteStore)` stub (full logic implemented in US7 Phase 9)
- [X] T018 Create `src/app/src/main.rs`: initialize tracing subscriber; open `SqlitePool` with WAL mode; run `sqlx::migrate!` for shared migrations; call `startup::bootstrap_admin` and `startup::reset_orphan_executions`; initialize `SqliteStore` for tower-sessions; build `WebAppState`; register all Leptos server functions; mount cargo-leptos static file handlers (`/pkg/*`, `/assets/*`); configure session layer (24h expiry, HttpOnly, Secure, SameSite=Strict); attach `leptos_axum::routes_handler` with the Leptos App router; bind on `NETNINJA_WEB_HOST:NETNINJA_WEB_PORT` (defaults `0.0.0.0:8080`); start Tokio runtime — crate compiles but pages are stubs

**Checkpoint**: `cargo check` in `src/app/` passes with stub pages. Foundation ready — user story implementation can begin.

---

## Phase 3: User Story 1 — Secure Admin Login & Session Management (Priority: P1) 🎯 MVP

**Goal**: Admin can log in with `admin/admin`, session persists across refreshes,
protected routes redirect unauthenticated users to `/login`, logout works.

**Independent Test**: Navigate to any protected URL without a session → redirected to `/login`.
Submit valid credentials → redirected to `/dashboard` stub. Refresh → still authenticated.
Click logout → redirected to `/login`; navigating to `/dashboard` redirects again.

- [X] T019 [US1] Implement `src/app/src/server_fns/auth.rs`: `login(username: String, password: String) -> Result<(), ServerFnError>` — extract `IpAddr` from request headers; call `check_rate_limit`; query `web_admin_credentials` by username; verify password with `argon2::Argon2::verify_password`; on success create session with `tower-sessions` and return `Ok(())`; on failure return `ServerFnError::ServerError("invalid_credentials")`; on rate-limit return `ServerFnError::ServerError("rate_limited")`; never log password value
- [X] T020 [P] [US1] Implement `src/app/src/server_fns/auth.rs`: `logout() -> Result<(), ServerFnError>` — call `session.flush()` to destroy server-side session; return `Ok(())`
- [X] T021 [P] [US1] Implement `src/app/src/server_fns/auth.rs`: `change_password(current: String, new_password: String) -> Result<(), ServerFnError>` — enforce `require_session`; verify `current` against stored hash; hash `new_password` with argon2id; update `password_hash` and `updated_at` in `web_admin_credentials`; update session flag `is_default_password = false`; return `ServerFnError::ServerError("wrong_current_password")` if verify fails; never log either password value
- [X] T021b [P] [US1] Implement `src/app/src/pages/change_password.rs` `ChangePasswordPage` component: current-password + new-password form inputs (reuse `ui/input.rs`, `ui/button.rs`); submit calls `change_password()` server function; on success navigate to `/dashboard`; on `wrong_current_password` display inline error message; wrap in `AppShell`; register `<Route path="/change-password" view=ChangePasswordPage/>` in `main.rs`; the warning banner rendered by T044 MUST link to this route so the admin can act on the prompt
- [X] T022 [US1] Implement `src/app/src/pages/login.rs` `LoginPage` component: username + password form inputs (reuse `ui/input.rs`), submit calls `login()` server function, on success navigate to `/dashboard`, display `"Invalid credentials"` error on `invalid_credentials`, display rate-limit message on `rate_limited`, display `"Service unavailable"` on DB error; wrap in centered card layout without `AppShell`
- [X] T023 [US1] Update `src/app/src/main.rs` Leptos Router: register `<Route path="/login" view=LoginPage/>`; add session-guard at router level — any route except `/login` and static assets checks `require_session` and redirects to `/login` if unauthenticated; register `change_password` server function

**Checkpoint**: User Story 1 fully functional. Login, session persistence, rate limiting, and logout all work independently.

---

## Phase 4: User Story 3 — Internet Line Management via Web (Priority: P2)

**Goal**: Admin can view, create, edit, and delete internet lines from the web UI.
Line server functions are implemented here; the dashboard (US2) depends on `get_lines()`.

**Independent Test**: Log in. Navigate to `/lines`. Create a line with valid ISP fields → appears in list. Edit it → changes persist. Delete it → no longer listed. Submit form with missing required fields → validation error shown.

**⚠️ Note**: `get_lines()` implemented in T024 is also required by the US2 DashboardPage (T030). Complete this phase before US2's dashboard page.

- [X] T024 [US3] Implement `src/app/src/server_fns/lines.rs`: `get_lines(page, page_size) -> Result<PaginatedResponse<LineResponse>, ServerFnError>`, `get_line(id) -> Result<LineResponse, ServerFnError>`, `create_line(req: CreateLineRequest) -> Result<LineResponse, ServerFnError>`, `update_line(id, req) -> Result<LineResponse, ServerFnError>`, `delete_line(id) -> Result<(), ServerFnError>` — all enforce `require_session`; delegate to `net_ninja::services::LineService`; `delete_line` returns `ServerFnError::ServerError("has_active_tasks")` if dependent tasks exist (FR-027)
- [X] T025 [US3] Implement `src/app/src/pages/lines.rs` `LinesPage` component: use `DataTable` (reuse `table.rs`) to list lines with ISP/status columns; create/edit modal form (reuse `ui/input.rs`, `ui/button.rs`) with fields matching `CreateLineRequest`/`UpdateLineRequest`; delete confirmation via `ui/dialog.rs`; inline validation error display; full `AppShell` layout with sidebar
- [X] T026 [US3] Register `<Route path="/lines" view=LinesPage/>` in `src/app/src/main.rs` Leptos Router

**Checkpoint**: Line CRUD fully functional via web. User Story 3 independently testable.

---

## Phase 5: User Story 2 — Authenticated Dashboard & Remote Operation Triggers (Priority: P2)

**Goal**: Admin sees all configured lines on dashboard with last-known status, can trigger
quota checks and speed tests per line, result appears inline without page reload.

**Independent Test**: Log in. Dashboard lists lines with status (from US3 `get_lines`).
Click quota-check trigger for one line → spinner appears, button disabled for that line.
Poll resolves → result replaces spinner. Same for speed test. Test at 375px viewport.

**⚠️ Depends on**: T024 (`get_lines` server function from Phase 4 / US3).

- [X] T027 [US2] Implement `src/app/src/server_fns/operations.rs`: `trigger_quota_check(line_id: i64) -> Result<i64, ServerFnError>` — enforce `require_session`; check `state.operation_locks[line_id]`; if locked return `ServerFnError::ServerError("busy")`; otherwise set lock `true`, create `task_execution` row (status `"running"`), spawn `tokio::task` calling `QuotaCheckService`, clear lock on task completion; return `execution_id`
- [X] T028 [P] [US2] Implement `src/app/src/server_fns/operations.rs`: `trigger_speed_test(line_id: i64) -> Result<i64, ServerFnError>` — identical pattern to T027 using `SpeedTestService`
- [X] T029 [P] [US2] Implement `src/app/src/server_fns/operations.rs`: `get_execution_status(execution_id: i64) -> Result<ExecutionStatusResponse, ServerFnError>` — enforce `require_session`; query `task_executions` table; return `ExecutionStatusResponse { status, error_message, finished_at }`
- [X] T030 [US2] Implement `src/app/src/pages/dashboard.rs` `DashboardPage` component: load lines via `get_lines()` resource (SSR initial load); for each line render trigger buttons (quota check, speed test) using `ui/button.rs`; on trigger call `trigger_quota_check` / `trigger_speed_test`, receive `execution_id`, create a Leptos `Resource` polling `get_execution_status` every 3 seconds; show `Spinner` (reuse `spinner.rs`) while status is `"running"`, disable that line's buttons; replace spinner with result data on `"completed"`, error message on `"failed"`, timeout message after 300 s (FR-010a); re-enable buttons when poll resolves; mount in `AppShell`; responsive to 375 px
- [X] T031 [US2] Register `<Route path="/dashboard" view=DashboardPage/>` and `<Route path="/" view=|| view!{ <Redirect path="/dashboard"/> }/>` in `src/app/src/main.rs` Leptos Router

**Checkpoint**: Dashboard fully functional with inline operation polling. User Story 2 independently testable.

---

## Phase 6: User Story 6 — Results & Log History via Web (Priority: P2)

**Goal**: Admin can view historical quota results, speed test results, and operation logs,
filterable by internet line.

**Independent Test**: Log in. Navigate to `/quota-results` — entries listed. Filter by line → only that line's results. Navigate to `/speed-results` → metrics listed. Navigate to `/logs` → timestamped entries with process_id and line.

- [X] T032 [US6] Implement `src/app/src/server_fns/history.rs`: `get_quota_results(line_id: Option<i32>, page: Option<u32>) -> Result<PaginatedResponse<QuotaResult>, ServerFnError>`, `get_speed_results(line_id: Option<i32>, page: Option<u32>) -> Result<PaginatedResponse<SpeedTestResult>, ServerFnError>`, `get_logs(line_id: Option<i64>, page: Option<u32>) -> Result<PaginatedResponse<LogEntry>, ServerFnError>` — all enforce `require_session`; delegate to existing service/repository layer
- [X] T033 [P] [US6] Implement `src/app/src/pages/quota_results.rs` `QuotaResultsPage`: line filter dropdown (populated from `get_lines()`), paginated `DataTable` with line / timestamp / used_gb / total_gb / remaining_gb / status columns; mount in `AppShell`
- [X] T034 [P] [US6] Implement `src/app/src/pages/speed_results.rs` `SpeedResultsPage`: line filter dropdown, paginated `DataTable` with line / timestamp / download_mbps / upload_mbps / latency_ms / status columns; mount in `AppShell`
- [X] T035 [P] [US6] Implement `src/app/src/pages/logs.rs` `LogsPage`: paginated `DataTable` with process_id / line / level / message / created_at columns; `ui/badge.rs` for log level; mount in `AppShell`
- [X] T036 [US6] Register `<Route path="/quota-results" view=QuotaResultsPage/>`, `<Route path="/speed-results" view=SpeedResultsPage/>`, `<Route path="/logs" view=LogsPage/>` in `src/app/src/main.rs` Leptos Router

**Checkpoint**: All history pages functional and filterable. User Story 6 independently testable.

---

## Phase 7: User Story 4 — Task & Schedule Management via Web (Priority: P3)

**Goal**: Admin can create, edit, delete, and manually trigger scheduled tasks; view
per-task execution history with status, duration, and error details.

**Independent Test**: Log in. Navigate to `/tasks`. Create a task targeting an existing line with a cron schedule → appears in list. Manually trigger → execution record created with status and duration. Edit task → changes take effect. Delete → removed from list.

- [X] T037 [US4] Implement `src/app/src/server_fns/tasks.rs`: `get_tasks()`, `get_task(id)`, `create_task(req)`, `update_task(id, req)`, `delete_task(id)`, `trigger_task(task_id) -> Result<i64, ServerFnError>` (returns `execution_id`), `get_task_executions(task_id)` — all enforce `require_session`; delegate to `net_ninja::services::TaskService` and `TaskExecutionService`; `delete_task` MUST cascade-delete all associated `task_executions` rows (FR-022) before removing the task record — verify the underlying service handles this or add the delete step explicitly
- [X] T038 [US4] Implement `src/app/src/pages/tasks.rs` `TasksPage`: task list `DataTable` with name / schedule / target lines / last status columns; create/edit form with schedule JSON input, line multi-select, task_types checkboxes (quota_check / speed_test), run_mode toggle; manual trigger button (reuse `ui/button.rs`) that calls `trigger_task` and polls status via `get_execution_status` (same pattern as dashboard); collapsible execution history per task showing start/end/duration/status/error using `ui/badge.rs`; delete confirmation `ui/dialog.rs`; mount in `AppShell`
- [X] T039 [US4] Register `<Route path="/tasks" view=TasksPage/>` in `src/app/src/main.rs` Leptos Router

**Checkpoint**: Task CRUD, manual trigger, and execution history all functional. User Story 4 independently testable.

---

## Phase 8: User Story 5 — SMTP & Email Notification Configuration via Web (Priority: P3)

**Goal**: Admin can configure SMTP server settings and per-task email notification rules
(recipients, custom subject) from the web interface.

**Independent Test**: Log in. Navigate to `/email-settings`. Enter SMTP config and save → persists on reload. Configure notification rule for a task → saved and linked to that task. Submit invalid SMTP config → validation error shown.

- [X] T040 [US5] Implement `src/app/src/server_fns/email.rs`: `get_smtp_configs()`, `create_smtp_config(req)`, `update_smtp_config(id, req)`, `delete_smtp_config(id)`, `get_task_notification_config(task_id)`, `upsert_task_notification_config(task_id, req)` — all enforce `require_session`; delegate to `net_ninja::services::SmtpConfigService` and `TaskNotificationConfigService`
- [X] T041 [US5] Implement `src/app/src/pages/email_settings.rs` `EmailSettingsPage`: SMTP config form (host, port, vendor, username, from_email, use_tls, is_default toggles); per-task notification sub-section with task selector dropdown, to/cc email address list inputs, subject template field; `ui/button.rs` for save/delete; validation error display; mount in `AppShell`
- [X] T042 [US5] Register `<Route path="/email-settings" view=EmailSettingsPage/>` in `src/app/src/main.rs` Leptos Router

**Checkpoint**: SMTP configuration and notification rules fully manageable via web. User Story 5 independently testable.

---

## Phase 9: User Story 7 — Offline Admin Password Reset via .bat File (Priority: P3)

**Goal**: Admin places `reset_admin_password.bat` in the app data directory; on next
startup the password resets to `"admin"`, all sessions are invalidated, the file is
removed, and the event is logged without any password value.

**Independent Test**: Place `reset_admin_password.bat` in `~/.local/share/netninja/`.
Restart `netninja-web`. Verify: file gone, old session rejected, `admin/admin` login succeeds.
Confirm no password value appears in logs. Log in → warning banner shown on dashboard.

**⚠️ Depends on**: T030 (`DashboardPage` from Phase 5 / US2) for the warning banner.

- [X] T043 [US7] Complete `src/app/src/startup.rs` `check_reset_file` function: call `net_ninja::get_shared_data_path()` to locate `reset_admin_password.bat`; if present AND pool is available: (a) hash `"admin"` with `argon2::Argon2::hash_password`; (b) `DELETE FROM web_sessions;` via `SqlitePool` to invalidate all sessions; (c) `INSERT OR REPLACE INTO web_admin_credentials` with new hash; (d) `fs::remove_file` the trigger file; (e) emit `tracing::info!("admin password reset via trigger file")` — never include hash or plaintext; if pool unavailable, skip and leave file intact (retry on next start) (FR-012, FR-013)
- [X] T044 [US7] Add default-password warning banner to `src/app/src/pages/dashboard.rs` `DashboardPage`: after session is verified, load `password_hash` from `web_admin_credentials`; verify it against the argon2id hash of `"admin"`; if it matches, render `ui/alert.rs` warning banner at top of page prompting the admin to change the default password (FR-028); banner disappears after `change_password` is called and session is reloaded

**Checkpoint**: Password reset mechanism fully functional. All 7 user stories complete.

---

## Phase 10: Polish & Cross-Cutting Concerns

**Purpose**: Compilation verification, responsive-layout check, security audit, and
quickstart validation. Each task is independent.

- [X] T045 [P] Run `cargo check` in `src/app/` and resolve all compilation errors; run `cargo clippy -- -D warnings` and fix any warnings in the new crate
- [X] T046 [P] Run `cargo check` in `src/backend/` and confirm zero new errors or warnings were introduced by the workspace restructuring (FR-018)
- [ ] T047 [P] Verify responsive layout at 375 px viewport on all pages (login, dashboard, lines, tasks, email-settings, quota-results, speed-results, logs): confirm no horizontal scrolling, no clipped controls (FR-016, SC-009)
- [X] T048 [P] Audit every `tracing::` and `log::` call in `src/app/src/`: confirm no password values (plaintext or hash) appear in any log statement at any level — refer to FR-014 and SC-008; document audit result in a code comment in `startup.rs`
- [ ] T049 Run quickstart.md validation checklist end-to-end: start server with `cargo leptos watch`, verify `/login` renders, login with `admin/admin` succeeds, `/dashboard` loads lines, session persists across 3 browser refreshes, logout redirects to `/login`, `cargo check` passes in `src/backend/` with no new errors

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 — **BLOCKS all user stories**
- **US1 (Phase 3)**: Depends on Phase 2 — first user story; required by all subsequent phases
- **US3 (Phase 4)**: Depends on Phase 2 + Phase 3 (session auth); `get_lines()` output needed by US2
- **US2 (Phase 5)**: Depends on Phase 4 (T024 `get_lines` for DashboardPage)
- **US6 (Phase 6)**: Depends on Phase 3 (session auth) — independent of US2/US3
- **US4 (Phase 7)**: Depends on Phase 4 (US3 lines must exist for task line-targets)
- **US5 (Phase 8)**: Depends on Phase 7 (notification config references tasks)
- **US7 (Phase 9)**: Core reset logic (T043) depends on Phase 2; warning banner (T044) depends on Phase 5 (US2 DashboardPage)
- **Polish (Phase 10)**: Depends on all desired user stories being complete

### User Story Dependencies

| Story | Phase | Depends On | Can Parallel With |
|-------|-------|------------|-------------------|
| US1 (P1) | 3 | Foundation | — |
| US3 (P2) | 4 | US1 (auth) | US6 |
| US2 (P2) | 5 | US3 T024 (get_lines) | US6 |
| US6 (P2) | 6 | US1 (auth) | US3, US2 |
| US4 (P3) | 7 | US3 (lines entity) | US5 |
| US5 (P3) | 8 | US4 (task entity) | — |
| US7 (P3) | 9 | US2 T030 (dashboard for banner) | — |

### Parallel Opportunities

Within Phase 2 (Foundation), these can run in parallel:
- T008 (auth/session.rs) ∥ T009 (auth/rate_limit.rs)
- T010 (ui/ components) ∥ T011 (spinner.rs) ∥ T012 (table.rs)

Within US1 (Phase 3):
- T020 (logout) ∥ T021 (change_password) — after T019 (login) is done

Within US2 (Phase 5):
- T028 (trigger_speed_test) ∥ T029 (get_execution_status) — after T027 (trigger_quota_check)

Within US6 (Phase 6):
- T033 (quota_results page) ∥ T034 (speed_results page) ∥ T035 (logs page) — after T032 (history server fns)

Polish (Phase 10):
- T045 ∥ T046 ∥ T047 ∥ T048 — all independent

---

## Parallel Example: US6 History Pages

```bash
# After T032 (history server functions) is complete:
# Launch all three page implementations in parallel:

Task A: "Implement pages/quota_results.rs QuotaResultsPage"   # T033
Task B: "Implement pages/speed_results.rs SpeedResultsPage"   # T034
Task C: "Implement pages/logs.rs LogsPage"                    # T035
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1 (Setup)
2. Complete Phase 2 (Foundational — **critical blocker**)
3. Complete Phase 3 (US1 — login, session, logout)
4. **STOP and VALIDATE**: log in with `admin/admin`, session persists, logout works
5. Web app is secure and deployable for basic admin access

### Incremental Delivery

1. Phase 1 + 2: Foundation ready
2. Phase 3 (US1): Login works → **MVP**
3. Phase 4 (US3 lines): Line CRUD available remotely
4. Phase 5 (US2 dashboard): Operations triggerable remotely → **Core Feature**
5. Phase 6 (US6 history): Historical data visible remotely
6. Phase 7 (US4 tasks): Task automation configurable remotely
7. Phase 8 (US5 email): Notification rules configurable remotely
8. Phase 9 (US7 reset): Password recovery mechanism in place → **Feature Complete**
9. Phase 10: Polish, compile verification, security audit

### Single Developer Strategy

Work sequentially in phase order. Within each phase, tasks marked **[P]** can be
batched into a single `Agent` subagent call or worked in parallel with a second window.

---

## Summary

| Metric | Count |
|--------|-------|
| Total tasks | 50 |
| Setup + Foundation + Polish | 23 |
| US1 (Login) tasks | 6 |
| US2 (Dashboard) tasks | 5 |
| US3 (Lines) tasks | 3 |
| US4 (Tasks) tasks | 3 |
| US5 (Email) tasks | 3 |
| US6 (History) tasks | 5 |
| US7 (Reset) tasks | 2 |
| Parallelizable tasks [P] | 21 |

**MVP scope**: Phases 1–3 (US1 Login) — 24 tasks — delivers a working authenticated web
interface. All other stories add management capabilities on top of this foundation.
