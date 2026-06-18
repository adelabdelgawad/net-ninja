# NetNinja v0.1.4 — Scheduler Reliability Release

## What's New

### 🛡 Missed Scheduled Runs Are No Longer Lost

Scheduled tasks previously fired **only** on their exact minute — if the machine was asleep, powered off, or the process was down at that moment, the run was silently skipped forever. The scheduler now **catches up** the most recent missed occurrence on the next tick or startup.

- Runs the single most-recent missed occurrence per task (no backlog stampede), within a **12-hour grace window**.
- DST-safe: spring-forward gaps are skipped, fall-back hours resolve cleanly.
- Idempotent — the existing atomic claim guarantees a given occurrence never double-fires.

**Before:** PC asleep at the 6 AM quota check → that day's check never happens.
**After:** PC wakes at 9 AM → the 6 AM check runs once, automatically.

---

### ♻️ Crash Recovery on Every Surface

A task left mid-run by a crash used to get stuck in `running` and be **silently excluded from all future scheduling**.

- The **Windows service** now resets orphaned tasks/executions on startup (it previously did *none* — the surface most likely to crash-restart was the least protected).
- The **web admin** now resets stuck `tasks` too, not just executions.
- The timeout sweep's misleading "runs on startup" doc was corrected to match reality.

---

### 🔒 No Two Schedulers at Once

- If the service **loses its lock** (taken over after going stale), it now stops scheduling immediately and exits for a clean SCM restart — instead of logging a warning and continuing to run jobs alongside the new owner.
- On lock loss it **does not** delete the lock row (which now belongs to the new holder).
- The desktop app now **fails closed** on lock-check errors instead of starting a second, uncoordinated scheduler.

---

### 🕐 Correct Local-Time Scheduling

| | Before | After |
|---|---|---|
| Quota check `"6 AM"` | fired 06:00 **UTC** (08:00 / 09:00 Cairo) | fires 06:00 **local** |
| Speed test / cleanup crons | UTC | local |

Config-cron jobs now run on the same local clock the per-task scheduler already used — the two no longer disagree.

---

### 🧹 No Duplicate Checks

When you've defined scheduled tasks, the legacy "all-lines" quota/speed cron jobs are now **skipped** so the modern per-task path owns that work — no more double scrapes/logins of the ISP portal. Fresh installs with no tasks keep the legacy crons as the default engine (no regression).

---

### 🩹 Hung Tasks Are Actually Stopped

The timeout sweep previously only flagged a stalled execution as failed in the database while the real work (e.g. a wedged browser scrape) kept running. It now also **cancels** the in-process run.

---

### 📊 Scheduler Health on the Dashboard

A new **Scheduler** panel surfaces what was previously invisible (the status commands were stubs):

- Live status — **Running / Idle / Unknown** with the lock holder and last heartbeat (auto-refreshes every 15s).
- How scheduling is managed — Windows service + version + run-state, or desktop-managed.
- **Last successful run** per job (Quota Check, Speed Test, Cleanup) with staleness coloring, so a silently-missed job is obvious at a glance.

Backend fix: the service-status check was querying the wrong service name (`"NetNinja"` vs the installed `"netninja-scheduler"`) and always reported *not installed* — now corrected, alongside real lock-holder / heartbeat / version reporting.

---

## ⬆ Upgrade Notes

- **No database migrations required** — uses the existing `scheduler_lock` / `service_info` tables.
- The catch-up grace window is fixed at 12h (internal constant); no config change needed.
- All existing tasks, lines, and email configurations are preserved.
- Scheduled times now fire in **local time** — if you relied on the old (UTC) firing time, expect jobs to shift to the wall-clock time you actually configured.
