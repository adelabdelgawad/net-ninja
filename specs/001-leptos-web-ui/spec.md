# Feature Specification: Leptos Web Interface with Admin Login & Password Reset

**Feature Branch**: `001-leptos-web-ui`
**Created**: 2026-03-19
**Status**: Draft
**Input**: User description: "Add Leptos Web Interface with Admin Login & Password Reset"

## Clarifications

### Session 2026-03-19

- Q: Should the web interface be limited to viewing line status and triggering operations, or should it provide full management feature-parity with the desktop app (lines, tasks, SMTP settings, results history)? → A: Full management — feature-parity with desktop (Option D).
- Q: What should the admin see while a long-running operation (quota check, 30–120 s) is in progress? → A: Inline spinner with polling — trigger button shows a loading spinner immediately; UI polls the server for completion; result replaces the spinner when done; admin can navigate freely (Option A).
- Q: When a file-based password reset runs at startup, should existing active web sessions be invalidated? → A: Yes — all active sessions are immediately invalidated at reset time; admin must log in again with the new password (Option A).
- Q: When an operation is already running for a line, what should happen if the admin clicks the trigger again for the same line? → A: Trigger buttons for that specific line are disabled while its operation is polling; admin cannot trigger a second operation until the first resolves (Option A).
- Q: What is the default admin password and how does the reset file work? → A: Default password is `"admin"` (hashed at rest); the reset trigger is a file named `reset_admin_password.bat` placed in the application data directory; detecting it on startup resets the password back to `"admin"` — no random generation, no output file.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Secure Admin Login & Session Management (Priority: P1)

The admin accesses the web application from any browser, logs in with a username
and password, and remains authenticated across page refreshes and navigation.
Unauthenticated access to any page automatically redirects to the login page.
The admin can explicitly log out, which immediately invalidates the session.

**Why this priority**: Authentication is the security gate for all other
functionality. No feature beyond the login page is accessible without it working
correctly — it is a hard prerequisite for every other story.

**Independent Test**: Navigate to any protected URL without a session — verify
redirect to `/login`. Submit valid credentials — verify redirect to dashboard.
Refresh the browser — verify session persists. Click logout — verify redirect to
`/login` and that navigating back to `/dashboard` redirects to `/login` again.

**Acceptance Scenarios**:

1. **Given** the admin is not authenticated, **When** they visit any protected
   route, **Then** they are immediately redirected to `/login`.
2. **Given** the admin is on `/login`, **When** they submit valid credentials,
   **Then** they are redirected to the dashboard and a session is established.
3. **Given** the admin is on `/login`, **When** they submit incorrect credentials,
   **Then** a generic error message is displayed (not revealing which field was wrong)
   and no session is created.
4. **Given** the admin has submitted incorrect credentials 5 times within 60 seconds,
   **When** they attempt login again, **Then** further attempts are temporarily blocked
   and a rate-limit message is shown; no hint of valid credentials is given.
5. **Given** the admin has an active session, **When** they refresh the browser,
   **Then** they remain on the current page without re-entering credentials.
6. **Given** the admin is authenticated, **When** they click the logout button,
   **Then** their session is invalidated server-side and they are redirected to `/login`.

---

### User Story 2 - Authenticated Dashboard & Remote Operation Triggers (Priority: P2)

Once authenticated, the admin views a dashboard showing all configured internet
lines and their last-known quota and speed-test status. The admin can manually
trigger a quota check or a speed test for any individual line inline. Results
appear without a full page reload. The same underlying backend operations are
invoked as the desktop app — no logic is duplicated.

**Why this priority**: Operational visibility and manual triggering are the most
time-sensitive actions; the admin needs them immediately upon login.

**Independent Test**: Log in. View dashboard — verify lines are listed with status.
Trigger a quota check for one line — verify result or error appears inline. Trigger
a speed test — verify metrics or error appear inline. Test at 375 px viewport width
— verify no horizontal overflow.

**Acceptance Scenarios**:

1. **Given** the admin is authenticated, **When** they visit the dashboard,
   **Then** all configured internet lines are listed with their last-known quota
   and speed-test status.
2. **Given** the admin clicks the trigger button for a quota check or speed test,
   **When** the request is submitted, **Then** a loading spinner appears inline for
   that line and the trigger button is disabled for that line while the poll is active.
3. **Given** an operation is running and the admin navigates to another page,
   **When** they return to the dashboard, **Then** the polling resumes and the result
   is displayed once available.
4. **Given** the admin triggers a quota check for a line and it completes, **When**
   the poll receives the result, **Then** the spinner is replaced by the latest quota
   usage data inline without navigating away.
5. **Given** the admin triggers a speed test for a line and it completes, **When**
   the poll receives the result, **Then** the spinner is replaced by download speed,
   upload speed, and latency metrics inline.
6. **Given** an operation fails, **When** the poll receives the error, **Then** the
   spinner is replaced by a user-readable error message; no internal details or
   credentials are exposed.
5. **Given** the dashboard is viewed on a mobile browser (viewport ≥ 375 px),
   **When** the admin interacts, **Then** all controls are reachable and no content
   scrolls horizontally.

---

### User Story 3 - Internet Line Management via Web (Priority: P2)

The admin can view, add, edit, and delete internet line configurations from the web
interface. This includes ISP type, connection credentials, and associated network
settings. The web interface provides the same line management capabilities as the
desktop app.

**Why this priority**: Lines are the foundational entity — tasks, quotas, and speed
tests all depend on them. Admins may need to add or modify lines remotely without
physical access to the installation device.

**Independent Test**: Log in. Navigate to the Lines page. Create a new line with
valid ISP credentials — verify it appears in the list. Edit the line — verify
changes persist. Delete the line — verify it no longer appears and associated data
is handled gracefully.

**Acceptance Scenarios**:

1. **Given** the admin navigates to the Lines section, **When** the page loads,
   **Then** all configured internet lines are listed with their ISP type and status.
2. **Given** the admin submits a new line form with valid data, **When** the form is
   saved, **Then** the new line appears in the list and is available for tasks.
3. **Given** the admin edits an existing line, **When** changes are saved, **Then**
   the updated values are reflected in the list and in subsequent operations.
4. **Given** the admin deletes a line, **When** the deletion is confirmed, **Then**
   the line no longer appears and no orphaned tasks or results reference it.
5. **Given** the admin submits a line form with missing required fields, **When**
   the form is submitted, **Then** validation errors are displayed inline and no
   record is saved.

---

### User Story 4 - Task & Schedule Management via Web (Priority: P3)

The admin can view, create, edit, and delete scheduled tasks from the web interface.
Each task targets one or more internet lines, runs on a cron schedule (or manually),
executes quota checks and/or speed tests, and can trigger email notifications. The
admin can also manually run a task on demand and view its execution history.

**Why this priority**: Task management is required for long-term automation, but
lines must exist first (P2) and the dashboard provides enough for initial monitoring.

**Independent Test**: Log in. Navigate to Tasks. Create a task for an existing line
with a valid cron expression — verify it appears in the list. Manually trigger the
task — verify an execution record appears with a result. View execution history —
verify past runs with status and duration are listed.

**Acceptance Scenarios**:

1. **Given** the admin navigates to the Tasks section, **When** the page loads,
   **Then** all configured tasks are listed with their schedule, target lines, and
   last execution status.
2. **Given** the admin creates a task with valid schedule and line targets, **When**
   saved, **Then** the task is scheduled and appears in the task list.
3. **Given** the admin manually triggers a task from the web UI, **When** it
   completes, **Then** a new execution record is created with status, duration, and
   result visible in the execution history.
4. **Given** the admin edits a task, **When** changes are saved, **Then** the
   updated schedule and configuration take effect without interrupting other tasks.
5. **Given** the admin deletes a task, **When** deletion is confirmed, **Then** the
   task is removed, its schedule is cancelled, and its execution history is
   accessible or removed per the deletion policy.
6. **Given** the admin views a task's execution history, **When** they expand an
   entry, **Then** they see start time, end time, status (success/failure/timeout),
   duration, and any error message.

---

### User Story 5 - SMTP & Email Notification Configuration via Web (Priority: P3)

The admin can configure the SMTP server settings and per-task email notification
rules from the web interface. This matches the capability currently available in the
desktop app's email settings screen.

**Why this priority**: Email notifications are a secondary concern; tasks and lines
must be configured first. This is a convenience feature for remote setup.

**Independent Test**: Log in. Navigate to Email Settings. Enter SMTP server details
and save — verify settings persist. Configure a notification rule for an existing
task (to/cc recipients, subject) — verify it is saved. Trigger the task — verify
(if SMTP is reachable) a notification is sent to the configured recipients.

**Acceptance Scenarios**:

1. **Given** the admin navigates to Email Settings, **When** the page loads,
   **Then** any existing SMTP configuration is shown pre-populated in the form.
2. **Given** the admin saves a valid SMTP configuration, **When** the form is
   submitted, **Then** the settings are persisted and used for future notifications.
3. **Given** the admin configures per-task notification rules (to/cc recipients,
   custom subject), **When** the task executes, **Then** the notification is sent
   using the configured rules.
4. **Given** the admin submits invalid SMTP settings, **When** the form is saved,
   **Then** a validation error is displayed; no invalid configuration is persisted.

---

### User Story 6 - Results & Log History via Web (Priority: P2)

The admin can view historical quota results, speed test results, and operation logs
from the web interface. Results can be filtered by line. This provides the same
visibility as the desktop app's QuotaResults, SpeedResults, and Logs pages.

**Why this priority**: Historical data is needed to monitor trends and diagnose
issues; it is equally important as the operational triggers on the dashboard.

**Independent Test**: Log in. Navigate to Quota Results — verify entries from
previous runs are listed. Filter by a specific line — verify only that line's
results appear. Navigate to Speed Results — verify entries with metrics are listed.
Navigate to Logs — verify timestamped log entries are displayed.

**Acceptance Scenarios**:

1. **Given** the admin navigates to Quota Results, **When** the page loads,
   **Then** historical quota check results are listed with line, timestamp, and
   usage data.
2. **Given** the admin filters Quota Results by a specific internet line, **When**
   the filter is applied, **Then** only results for that line are shown.
3. **Given** the admin navigates to Speed Results, **When** the page loads,
   **Then** historical speed tests are listed with line, timestamp, and metrics.
4. **Given** the admin navigates to Logs, **When** the page loads,
   **Then** timestamped log entries are displayed with process ID and associated
   line (where applicable), matching the 7-day retention policy.

---

### User Story 7 - Offline Admin Password Reset via .bat File (Priority: P3)

When the admin cannot log in (forgotten password), they place a file named
`reset_admin_password.bat` in the application data directory on the host machine.
On the next application startup, the system detects the file, resets the admin
password back to the default `"admin"`, removes the trigger file, invalidates all
active sessions, and logs the event. The admin can then log in with `"admin"` and
change the password.

**Why this priority**: Recovery from a locked-out state is critical for long-term
maintainability. The `.bat` extension makes the trigger easy to create on Windows
(the file can be double-clicked, though execution is not required — only presence
is checked).

**Independent Test**: Place `reset_admin_password.bat` in the application data
directory (e.g., `%ProgramData%\NetNinja\`). Restart the application. Verify:
(a) trigger file is gone, (b) login with username `admin` and password `admin`
succeeds, (c) any previously active session no longer works.

**Acceptance Scenarios**:

1. **Given** a file named `reset_admin_password.bat` exists in the application data
   directory, **When** the application starts, **Then** the admin password is reset
   to `"admin"` (hashed at rest) before any web requests are served.
2. **Given** a reset is triggered, **When** processing completes, **Then** the
   trigger file is removed and all active web sessions are invalidated.
3. **Given** a reset has occurred, **When** the admin submits username `admin` and
   password `admin` on `/login`, **Then** login succeeds.
4. **Given** a reset has occurred, **When** application logs are inspected, **Then**
   the reset event is recorded but no password value appears in any log line.
5. **Given** no trigger file exists, **When** the application starts, **Then** the
   existing admin password is unchanged.
6. **Given** the admin account does not yet exist, **When** a reset is triggered,
   **Then** the admin account is created with the hashed `"admin"` password
   (first-time setup path).
7. **Given** the admin has an active session when the application restarts with the
   trigger file present, **When** they next make any request, **Then** their session
   is rejected and they are redirected to `/login`.
8. **Given** a reset has occurred and the admin logs in with `"admin"`, **When** they
   are on the dashboard, **Then** a prominent warning is displayed prompting them to
   change the default password to a stronger one.

---

### Edge Cases

- What happens when two web requests simultaneously trigger the same operation on
  the same line? The UI prevents this by disabling the trigger buttons for a line
  while its operation is polling (FR-009, FR-010). If a duplicate request reaches
  the server despite this (e.g., two browser tabs), the server MUST reject the
  second with a "busy" response and not start a second concurrent operation on
  the same line.
- What happens when the web server starts but the database is unavailable?
  The `/login` page must still render; all operational routes return a graceful
  "service temporarily unavailable" message without crashing.
- What happens when a session cookie is tampered with or replayed after logout?
  The request must be treated as unauthenticated and redirected to `/login`; no
  server error or internal detail is exposed.
- What happens when the reset trigger file exists but the database is unavailable
  at startup? The trigger file must NOT be removed; the reset should be retried on
  the next startup when the database is available.
- What happens when the admin deletes a line that has active scheduled tasks?
  The system must either prevent deletion with a clear error or cascade-cancel the
  associated tasks before removing the line.
- What happens when a polled operation exceeds the 5-minute timeout? The UI must
  display a timeout error inline for that line, stop polling, and allow the admin
  to retry the trigger. The backend operation itself may still run to completion
  and the result will be visible in Results History.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide a `/login` route with a username and password
  input form accessible to unauthenticated users.
- **FR-002**: System MUST store admin passwords as hashes only; plaintext passwords
  MUST NOT be persisted in the database, logs, or any file.
- **FR-003**: System MUST create a server-side session on successful login and
  deliver the session identifier to the browser via a secure, `HttpOnly` cookie.
- **FR-004**: System MUST redirect all unauthenticated requests to protected routes
  to `/login`.
- **FR-005**: System MUST invalidate the server-side session on logout and redirect
  the browser to `/login`.
- **FR-006**: System MUST enforce rate limiting on login attempts: after 5 failed
  attempts within 60 seconds from the same source, further attempts MUST be blocked
  and a user-visible message displayed.
- **FR-007**: System MUST NOT indicate in error messages whether the submitted
  username or password specifically was incorrect.
- **FR-008**: System MUST provide an authenticated dashboard listing all configured
  internet lines with their last-known quota and speed-test status.
- **FR-009**: System MUST allow the admin to trigger a quota check for any line from
  the web dashboard; upon trigger, the UI MUST display an inline loading spinner and
  poll the server for the result, replacing the spinner with the result when the
  operation completes. The admin MUST be able to navigate freely during polling.
- **FR-010**: System MUST allow the admin to trigger a speed test for any line from
  the web dashboard; upon trigger, the UI MUST display an inline loading spinner and
  poll the server for the result, replacing the spinner with download speed, upload
  speed, and latency metrics when the operation completes.
- **FR-010a**: If a polled operation exceeds a configurable timeout (default: 5
  minutes), the UI MUST display a timeout error inline for that line, stop polling,
  and re-enable the trigger buttons for that line.
- **FR-010b**: While an operation is actively running for a line, the server MUST
  reject any additional trigger request for the same line with a "busy" error,
  even if multiple browser sessions are open simultaneously. The UI MUST disable
  the quota-check and speed-test trigger buttons for a line as soon as its poll
  begins, and re-enable them only when the poll resolves or times out.
- **FR-011**: System MUST reuse the existing backend service layer for all quota,
  speed-test, line, task, email, and scheduling operations; duplicate implementations
  of the same logic are forbidden.
- **FR-012**: System MUST check for a file named `reset_admin_password.bat` in the
  application data directory on every startup, before serving any requests. Only
  the file's presence is checked; its content is not read or executed.
- **FR-013**: When the trigger file is found, the system MUST: (a) update or create
  the admin credential record with the hashed value of the default password `"admin"`,
  (b) invalidate all existing web sessions, (c) remove the trigger file, and
  (d) log that a reset occurred without logging any password value.
- **FR-014**: System MUST NOT include any password value — plaintext or hashed — in
  any log output at any log level.
- **FR-028**: When the admin logs in using the default password `"admin"`, the system
  MUST display a prominent in-app warning on the dashboard prompting them to change
  their password to a stronger value. This warning MUST persist on every login until
  the password is changed from the default.
- **FR-015**: Sessions MUST remain valid for at least 24 hours of inactivity before
  automatic expiry.
- **FR-016**: The web interface layout MUST be usable on viewport widths from 375 px
  to 1920 px without horizontal scrolling.
- **FR-017**: The web application MUST start and accept requests independently; it
  MUST NOT require the Tauri desktop process to be running.
- **FR-018**: The web application MUST NOT modify, break, or interfere with the
  existing Tauri desktop application, its IPC commands, or its frontend.
- **FR-019**: All sensitive mutations (login, logout, triggering operations, saving
  credentials) MUST be validated server-side; client-side validation alone is
  insufficient.
- **FR-020**: The web application listen port MUST be configurable via environment
  variable or configuration file, with a documented default value.
- **FR-021**: System MUST provide a Lines management section where the admin can
  view, create, edit, and delete internet line configurations including ISP type,
  credentials, and network settings.
- **FR-022**: System MUST provide a Tasks management section where the admin can
  view, create, edit, delete, and manually trigger scheduled tasks, and view per-task
  execution history with status, duration, and error details. When a task is deleted,
  its associated `task_executions` records MUST be cascade-deleted.
- **FR-023**: System MUST provide an Email Settings section where the admin can
  configure the SMTP server and per-task notification rules (to/cc recipients and
  custom subjects).
- **FR-024**: System MUST provide a Quota Results history page listing past quota
  check results filterable by internet line.
- **FR-025**: System MUST provide a Speed Test Results history page listing past
  speed tests filterable by internet line.
- **FR-026**: System MUST provide a Logs page displaying timestamped operation log
  entries with process ID and line association, respecting the 7-day retention policy.
- **FR-027**: When the admin attempts to delete a line that has associated active
  tasks, the system MUST either prevent deletion with a clear error message or
  require explicit confirmation that dependent tasks will also be removed.

### Key Entities *(include if feature involves data)*

- **AdminCredential**: Single record holding the username (`admin`, fixed) and the
  hashed password. Created on first setup or password reset if the record is absent.
- **WebSession**: Server-side session record managed by `tower-sessions`. Persisted
  as `(id TEXT PK, data BLOB NOT NULL, expiry_date INTEGER NOT NULL)` in the
  `web_sessions` SQLite table. Session contents (including activity tracking) are
  opaque to application code; the middleware handles expiry after 24 hours of
  inactivity (FR-015).
- **PasswordResetTrigger**: Presence-based file sentinel (`reset_admin_password.bat`)
  in the OS application data directory. Only its presence is checked; content is
  ignored. Consumed (deleted) on successful processing.
- **NetworkOperationResult**: Outcome of a quota check or speed test triggered from
  the web interface: status (success / failure / timeout), result data or error
  message, and timestamp.
- **Line** *(shared with desktop)*: Internet connection configuration — ISP type,
  credentials (encrypted at rest), and network settings.
- **Task** *(shared with desktop)*: Scheduled unit targeting one or more lines, with
  a cron expression, operation types (quota / speed test), and notification rules.
- **TaskExecution** *(shared with desktop)*: Record of a single task run — start/end
  time, status, duration, and error message.
- **SmtpConfig** *(shared with desktop)*: SMTP server configuration for email delivery.
- **TaskNotificationConfig** *(shared with desktop)*: Per-task email notification
  rules — recipients, subject template.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The admin completes the full login flow (navigate to any URL → redirect
  to `/login` → enter credentials → land on dashboard) in under 10 seconds on a
  local-area network connection.
- **SC-002**: 100% of requests to protected routes without a valid session result in
  a redirect to `/login`; no authenticated content is served to unauthenticated
  requests.
- **SC-003**: After a successful login, the session persists through at least 5
  consecutive browser refreshes without requiring re-authentication.
- **SC-004**: After 5 failed login attempts within 60 seconds, subsequent attempts
  are blocked; the blocking is visible to the admin (a message is displayed) and
  verifiable without server log access.
- **SC-005**: A quota check or speed test triggered from the web dashboard produces
  the same result data as the equivalent operation triggered from the desktop app for
  the same line.
- **SC-006**: A password reset triggered by `reset_admin_password.bat` completes
  (trigger file removed, password reset to `"admin"`, all sessions invalidated)
  within 10 seconds of startup.
- **SC-007**: After a file-triggered reset, login with `admin / admin` succeeds on
  the first attempt and `reset_admin_password.bat` no longer exists in the data
  directory.
- **SC-008**: A full-text search of application log output after any password event
  returns zero matches for any password value in plaintext or hashed form.
- **SC-009**: The dashboard and all management pages are fully navigable and operable
  on a 375 px-wide viewport — no horizontal scrolling, no clipped controls.
- **SC-010**: The web server starts and serves the `/login` page successfully even
  when the SQLite database is unavailable.
- **SC-011**: A line created via the web interface is immediately available for task
  scheduling and appears in the desktop app's Lines page on next refresh (shared DB).
- **SC-012**: A task created or modified via the web interface takes effect on the
  next scheduled trigger without requiring a desktop app restart.
- **SC-013**: Historical quota results, speed test results, and logs visible in the
  desktop app are also visible in the web interface without delay (shared DB read).
- **SC-014**: After triggering a quota check or speed test, the loading spinner
  appears within 1 second of the button click; the result replaces the spinner within
  5 seconds of the backend operation completing; the admin can navigate to another
  page and return without losing the in-progress polling state.

## Assumptions

The following assumptions have been made where the feature description left gaps.
Raise a clarification before moving to planning if any assumption is incorrect.

- **"Order execution"** is interpreted as manually triggering a network operation
  (quota check or speed test) for a specific internet line. Full management (lines,
  tasks, settings) is covered by US3–US6 per clarification Q1.
- **Feature parity with desktop** means all management capabilities currently exposed
  through the Tauri IPC layer (lines, tasks, task executions, SMTP config, task
  notification config, quota results, speed test results, logs) are accessible from
  the web UI. The About page and purely local diagnostics are not required.
- **Application data directory** reuses the existing `get_shared_data_path()`
  resolution: `%ProgramData%\NetNinja\` on Windows, platform-standard XDG path on
  Linux/macOS. No new path strategy is introduced.
- **Session delivery** uses a server-set `HttpOnly`, `Secure` cookie. TLS
  termination is the responsibility of a reverse proxy in production; the web app
  itself listens over plain HTTP.
- **Rate limiting** is tracked per source IP address.
- **Initial admin password** is `"admin"`, stored as a hash. The credential record
  is created on first startup if absent. The admin can log in immediately with
  `admin / admin` and is warned to change the default password.
- **Default web port** is `8080`, configurable via the environment variable
  `NETNINJA_WEB_PORT`.
- **Output file permissions** are mode `0600` (owner-read-only) on POSIX systems
  and an equivalent ACL on Windows. A warning is logged if this cannot be applied,
  but the reset still completes.
- **The existing Tauri desktop app** continues to operate unchanged. The web app
  shares the SQLite database and backend service crate but does not alter any Tauri
  IPC commands, command registrations, or the SolidJS frontend.
- **Shared database**: Both the web app and the desktop app read from and write to
  the same SQLite database. Concurrent writes are managed by SQLite's WAL mode;
  no additional locking layer is assumed.
