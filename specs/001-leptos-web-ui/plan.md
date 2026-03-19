# Implementation Plan: Leptos Web Interface with Admin Login & Password Reset

**Branch**: `001-leptos-web-ui` | **Date**: 2026-03-19 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/001-leptos-web-ui/spec.md`

## Summary

Add a Leptos 0.7 SSR web application (`apps/web/`) to the existing NetNinja workspace
that provides full feature-parity with the desktop app for remote browser access.
The web app shares the existing `net-ninja` backend library (services, repositories,
models) without duplicating any business logic. It adds Argon2id-based admin
authentication, server-side SQLite-backed sessions, per-line operation polling with
a loading spinner UX, and a file-based password reset mechanism (`reset_admin_password.bat`
→ resets to `"admin"`). Two new SQLite migrations add `web_admin_credentials` and
`web_sessions` tables.

## Technical Context

**Language/Version**: Rust 2021 edition (same as existing backend)
**Primary Dependencies**: Leptos 0.7, leptos_axum 0.7, Axum 0.7, tower-sessions 0.12
(SQLite store), argon2 0.5, governor 0.6, dashmap 6, cargo-leptos (build tool),
Tailwind CSS 4 (same version as desktop frontend)
**Storage**: SQLite (shared database) — two new tables: `web_admin_credentials`,
`web_sessions`
**Testing**: `cargo test` in `apps/web/`; integration tests against real SQLite
**Target Platform**: Linux + Windows (same as existing app); HTTP server on configurable
port (default 8080)
**Project Type**: SSR web application (new binary alongside existing desktop binary)
**Performance Goals**: Login page < 1 s TTFB; operation trigger acknowledgement < 1 s;
poll result display < 5 s after backend completion (SC-014)
**Constraints**: Must not modify existing Tauri app; must reuse existing service layer;
sessions survive 24-hour inactivity; serves `/login` even when DB unavailable
**Scale/Scope**: Single admin user, low concurrent load (personal network monitoring tool)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design — all gates pass.*

- [x] **Principle I — Strict Layer Isolation**: Web adapter layer (Axum handlers /
  Leptos server functions) calls `net_ninja::services::*` exclusively. No direct
  repository calls from server functions. The Tauri adapter layer is untouched.
- [x] **Principle II — Security by Default**: ISP credentials continue through the
  existing AES-256-GCM encryption path. Admin password stored as Argon2id hash only.
  Sessions delivered via `HttpOnly` cookie. No credential, session ID, or password
  appears in any log entry at any level.
- [x] **Principle III — Resilience & Recovery**: Web app serves `/login` when DB is
  unavailable (fallback mode mirrors existing `AppState::new_fallback` pattern). On
  startup, orphan task executions are reset using the existing `TaskExecutionRepository`
  call (reused from `standalone.rs`).
- [x] **Principle IV — Observability First**: All web-triggered operations (quota check,
  speed test) inherit backend service logging (process ID + line ID). Web-specific
  events (login, logout, password reset) are logged at `INFO` level with actor context
  but without any sensitive value.
- [x] **Principle V — IPC Contract Discipline & Simplicity**: No new `#[tauri::command]`
  is added — this is a separate binary. The three-file registration rule does not apply.
  YAGNI respected: no abstractions beyond what the 7 user stories require.
- [x] **Migrations append-only**: Two new migration files follow timestamp naming
  convention (`20260319000001_*`, `20260319000002_*`). No existing migration modified.

## Project Structure

### Documentation (this feature)

```text
specs/001-leptos-web-ui/
├── plan.md              # This file
├── research.md          # Technology decisions + rationale
├── data-model.md        # Entity schemas + state transitions
├── quickstart.md        # Dev/prod run instructions
├── contracts/
│   └── server-functions.md   # All Leptos server function signatures + route map
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
/                                   ← repo root
├── Cargo.toml                      ← NEW: Cargo workspace (members: src/backend, apps/web)
│
├── src/
│   ├── backend/                    ← UNCHANGED: existing net-ninja library + Tauri binary
│   │   ├── migrations/
│   │   │   ├── ...existing 24 files...
│   │   │   ├── 20260319000001_web_admin_credentials.sql  ← NEW
│   │   │   └── 20260319000002_web_sessions.sql           ← NEW
│   │   └── src/                    ← UNCHANGED (all services, models, repos, adapters)
│   └── frontend/                   ← UNCHANGED: existing SolidJS Tauri frontend
│
└── apps/
    └── web/                        ← NEW: Leptos SSR web application
        ├── Cargo.toml
        ├── build.rs                ← cargo-leptos build support
        ├── src/
        │   ├── main.rs             ← Axum server entry point + cargo-leptos hydration
        │   ├── state.rs            ← WebAppState (pool, sessions, rate limiters, op locks)
        │   ├── startup.rs          ← Reset file check, admin bootstrap, orphan recovery
        │   ├── auth/
        │   │   ├── mod.rs
        │   │   ├── session.rs      ← Session helpers (get_session, require_session)
        │   │   └── rate_limit.rs   ← Per-IP governor rate limiter
        │   ├── server_fns/
        │   │   ├── mod.rs
        │   │   ├── auth.rs         ← login(), logout(), change_password()
        │   │   ├── lines.rs        ← get_lines(), create_line(), update_line(), delete_line()
        │   │   ├── tasks.rs        ← get_tasks(), create_task(), ..., trigger_task()
        │   │   ├── operations.rs   ← trigger_quota_check(), trigger_speed_test(),
        │   │   │                     get_execution_status()
        │   │   ├── email.rs        ← SMTP config + notification config CRUD
        │   │   └── history.rs      ← get_quota_results(), get_speed_results(), get_logs()
        │   ├── pages/
        │   │   ├── mod.rs
        │   │   ├── login.rs        ← /login page component
        │   │   ├── dashboard.rs    ← / dashboard with per-line trigger buttons + spinner
        │   │   ├── lines.rs        ← /lines CRUD page
        │   │   ├── tasks.rs        ← /tasks CRUD + execution history page
        │   │   ├── email_settings.rs  ← /email-settings page
        │   │   ├── quota_results.rs   ← /quota-results page
        │   │   ├── speed_results.rs   ← /speed-results page
        │   │   └── logs.rs         ← /logs page
        │   └── components/
        │       ├── mod.rs
        │       ├── layout.rs       ← AppShell, Sidebar, TopBar
        │       ├── table.rs        ← Reusable data table (Shadcn-style)
        │       ├── spinner.rs      ← Loading spinner for operation polling
        │       └── ui/
        │           ├── mod.rs
        │           ├── button.rs   ← Button variants (primary, secondary, destructive)
        │           ├── input.rs    ← Form input with label + error display
        │           ├── badge.rs    ← Status badge (success/failure/running)
        │           ├── alert.rs    ← Alert/banner (warning for default password)
        │           └── dialog.rs   ← Confirmation dialog (delete line/task)
        └── style/
            └── main.css            ← Tailwind CSS 4 input file
```

**Structure Decision**: New workspace crate at `apps/web/` (not a Tauri plugin, not a
sub-module of `src/backend`). This keeps the web binary fully independent of the Tauri
runtime and allows `cargo leptos build` tooling to work without conflict. The shared
backend library is imported via workspace path dependency — no code duplication.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|--------------------------------------|
| New Cargo workspace root | Required to share `net-ninja` lib between Tauri binary and new web binary | Single binary: entangles Tauri and Axum runtimes (incompatible); copy code: violates FR-011 |
| New binary (`apps/web`) | Feature requirement — web access must not replace existing Tauri app | Feature flag in existing binary: would pull Axum/Leptos into Tauri build, adding ~10 MB to installer |
| New crate dependencies (Leptos, Axum, tower-sessions, argon2, governor) | Each satisfies a specific spec requirement with no available substitute in the existing stack | No existing Rust web framework is already in the dependency tree |
