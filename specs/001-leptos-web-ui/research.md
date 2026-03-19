# Research: Leptos Web Interface (001-leptos-web-ui)

## 1. Leptos SSR Framework

**Decision**: Leptos 0.7 with `leptos_axum` integration for full Server-Side Rendering.

**Rationale**: Leptos is a Rust-native reactive web framework with first-class SSR support.
`leptos_axum` provides the bridge between Axum (HTTP server) and Leptos components. This
is the canonical production SSR stack for Leptos. Server functions (`#[server]`) compile
to HTTP POST endpoints automatically, eliminating hand-written REST routes for mutations.

**Alternatives considered**:
- Actix-web: More mature HTTP ecosystem but no Leptos integration as polished as axum.
- Dioxus: Different reactive model, less production-ready SSR in 2026.
- Full REST + JS frontend: Defeats the purpose of a pure-Rust implementation.

---

## 2. HTTP Server

**Decision**: Axum 0.7 as the HTTP server layer.

**Rationale**: `leptos_axum` is designed around Axum's `Router`. Axum integrates naturally
with Tower middleware (sessions, rate limiting). The existing backend already uses Tokio
1.43, which is Axum's async runtime — no runtime conflict.

**Alternatives considered**:
- Actix-web: Would require a separate runtime thread or compat layer; adds complexity.

---

## 3. Session Management

**Decision**: `tower-sessions` 0.12 with `tower-sessions-sqlx-store` (SQLite backend).

**Rationale**: SQLite-backed sessions survive web server restarts (spec requirement: 24-hour
persistence). The session store uses the same SQLite database as the rest of the app via the
shared `SqlitePool`. `tower-sessions` integrates with Axum as Tower middleware. Sessions are
identified by a random UUID stored in an `HttpOnly` cookie.

**Alternatives considered**:
- In-memory DashMap store: Sessions lost on restart — violates SC-003.
- JWT tokens: Stateless tokens cannot be server-side invalidated on password reset
  (violates clarified requirement Q3-A).
- Redis: External dependency not justified for a single-admin desktop tool.

---

## 4. Password Hashing

**Decision**: `argon2` 0.5 (Argon2id variant, OWASP-recommended parameters).

**Rationale**: Argon2id is the winner of the Password Hashing Competition (2015) and is
recommended by OWASP for new applications. The `argon2` crate is the canonical Rust
implementation. Parameters: memory=19456 KiB, iterations=2, parallelism=1 (OWASP minimum
for interactive logins).

**Alternatives considered**:
- bcrypt (`bcrypt` crate): No known vulnerabilities but lacks side-channel resistance of
  Argon2id; OWASP now recommends Argon2id over bcrypt for new code.
- SHA-256: Not a password hashing function — unsuitable.

---

## 5. Rate Limiting

**Decision**: `governor` 0.6 with a `DashMap`-based per-IP state store held in `WebAppState`.

**Rationale**: `governor` implements the Generic Cell Rate Algorithm (GCRA), which is
accurate, efficient, and idiomatic in Rust. A `DashMap<IpAddr, RateLimiter>` provides
per-IP tracking without an external dependency. The rate limiter is applied at the Axum
layer on the `/login` route only.

**Alternatives considered**:
- `tower-governor` crate: Higher-level integration but less control over the per-IP
  eviction strategy; also introduces an additional indirect dependency chain.
- Database-based rate limiting: Adds DB writes on every failed login — unnecessary load.

---

## 6. Shadcn-Style UI Components

**Decision**: Hand-crafted Tailwind CSS 4 components with Shadcn visual design language.

**Rationale**: There is no official Shadcn port for Leptos. The existing desktop app already
uses Tailwind CSS 4 (same version) giving us a consistent dark-theme baseline. Components
will follow Shadcn's structural patterns (composition-based, variant-driven via CSS classes)
without importing any JavaScript ecosystem dependency. This avoids pulling in a JS runtime.

**Alternatives considered**:
- `leptonic`: Leptos-native component library but different design system from existing app.
- WASM bindings to shadcn React components: Technically unsound; no viable mechanism.
- Copying Shadcn CSS from CDN: Would introduce version-drift from Tailwind 4 utility model.

---

## 7. Operation Polling Mechanism

**Decision**: Leptos `Resource` with a periodic refetch interval polling a server function
that reads `task_execution` status from the database.

**Rationale**: When a quota check or speed test is triggered, the server function inserts a
`task_execution` row and spawns a Tokio task. The client Leptos `Resource` polls
`get_operation_status(execution_id)` every 3 seconds until the status is no longer
"running". The backend already has `TaskExecutionRepository` and `TaskExecutionService`
that track this state — no new state management is needed.

**Alternatives considered**:
- Server-Sent Events (SSE): More real-time but requires a persistent HTTP connection and
  adds server-push infrastructure. Overkill for 1-admin use.
- WebSockets: Same concern — persistent connection complexity for a feature that fires
  at most a few times per session.
- Long-polling: Less clean than periodic refetch; harder to handle navigation-away.

---

## 8. Workspace Structure

**Decision**: Create a root-level `Cargo.toml` workspace declaring `src/backend` and
`apps/web` as members. The web app at `apps/web/` depends on the backend lib `net-ninja`
via workspace path.

**Rationale**: No root `Cargo.toml` exists currently. Creating one is the standard Cargo
mechanism for sharing a library between multiple binaries. The `net-ninja` library crate
already has a clean `[lib]` section — no changes to its `Cargo.toml` are required other
than potentially removing the `tauri` dependency from `[dependencies]` into a feature flag
(already partially done with `service` feature). The web app Cargo.toml will use
`net-ninja = { path = "../../src/backend" }`.

**Alternatives considered**:
- Re-publish backend as a crate: Unnecessary for a single-repo project.
- Copy service code into apps/web: Violates FR-011 (no duplicate business logic).
- Single binary with feature flags: Would entangle Tauri and Axum runtimes — not feasible.

---

## 9. New Database Migrations

**Decision**: Two new timestamp-based migration files in `src/backend/migrations/`:
- `20260319000001_web_admin_credentials.sql` — `web_admin_credentials` table
- `20260319000002_web_sessions.sql` — `web_sessions` table

**Rationale**: Both the Tauri desktop app and the web binary use the same SQLite database.
Placing migrations in the shared backend library ensures both binaries apply them on startup
via the existing `run_pending_migrations()` call. The timestamp naming convention matches
all existing 24 migration files.

---

## 10. Admin Password Reset File

**Decision**: Detect `reset_admin_password.bat` by file existence in `get_shared_data_path()`
at startup. Reset to `"admin"` (Argon2id hash). No file content is read or executed.

**Rationale**: The `.bat` extension makes it trivially creatable on Windows (right-click →
New → Text Document → rename with `.bat`). The existing `get_shared_data_path()` already
resolves the correct platform path. File is presence-checked, never executed — the `.bat`
extension is purely a naming convention for UX, not a security boundary.

---

## 11. Initial Admin Credential Bootstrap

**Decision**: On web app startup, if the `web_admin_credentials` table is empty, insert the
default `admin` / `argon2id("admin")` record automatically.

**Rationale**: The spec requires the admin can log in immediately with `admin/admin` without
any prior setup step. Bootstrapping on first startup keeps the user experience frictionless
while avoiding a hardcoded insert in the migration file (which would run even if the admin
already changed their password after the desktop app created the record).

---

## 12. Concurrent Operation Guard

**Decision**: Per-line `DashMap<i64, bool>` in `WebAppState` tracking "operation in progress"
flags. Server function checks and sets atomically; the flag is cleared when the operation
completes or times out (5-minute poll cap).

**Rationale**: The spec requires the server to reject a second trigger on the same line
(FR-010b). The `DashMap` in-memory guard handles the common case efficiently. Since the web
app is a single process, in-process state is sufficient; no cross-process coordination is
needed (the desktop app triggers operations differently and the web guard is independent).
