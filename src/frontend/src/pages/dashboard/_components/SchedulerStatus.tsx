import { type Component, createMemo, For, Show } from 'solid-js';
import { Activity, Server, Clock } from 'lucide-solid';
import { useSchedulerStatusQuery, useServiceStatusQuery } from '~/api/queries';
import { formatShortDateTime } from '~/lib/date';
import { cn } from '~/lib/utils';

// Jobs we surface, in display order. Names must match the backend
// `last_success:<job>` markers written by the cron jobs.
const TRACKED_JOBS: { key: string; label: string }[] = [
  { key: 'quota_check', label: 'Quota Check' },
  { key: 'speed_test', label: 'Speed Test' },
  { key: 'cleanup', label: 'Cleanup' },
];

/** Relative "Xm/h/d ago" for a timestamp, or null if absent/invalid. */
function timeAgo(dateStr: string | null | undefined): string | null {
  if (!dateStr) return null;
  const ts = new Date(dateStr).getTime();
  if (Number.isNaN(ts)) return null;
  const diffMs = Date.now() - ts;
  const mins = Math.floor(diffMs / 60000);
  if (mins < 1) return 'just now';
  if (mins < 60) return `${mins}m ago`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours}h ago`;
  return `${Math.floor(hours / 24)}d ago`;
}

/** Hours since a timestamp, or null. */
function hoursSince(dateStr: string | null | undefined): number | null {
  if (!dateStr) return null;
  const ts = new Date(dateStr).getTime();
  if (Number.isNaN(ts)) return null;
  return (Date.now() - ts) / 3600000;
}

// Generic staleness palette (matches the dashboard's existing colors).
const freshnessColor = (hours: number | null): string => {
  if (hours == null) return 'text-muted-foreground';
  if (hours < 28) return 'text-[#4ec9b0]';
  if (hours < 52) return 'text-[#dcdcaa]';
  return 'text-[#c72e0f]';
};

const statusDot = (status: string): string => {
  if (status === 'running') return 'bg-[#4ec9b0]';
  if (status === 'idle') return 'bg-[#dcdcaa]';
  return 'bg-muted-foreground/40';
};

const lockHolderLabel = (holder: string | null): string => {
  if (holder === 'service') return 'Windows Service';
  if (holder === 'desktop') return 'Desktop App';
  return '—';
};

/**
 * SchedulerStatus — dashboard panel surfacing scheduler liveness and the
 * per-job last-success markers (and Windows service install/run state), so an
 * operator can see at a glance whether scheduled work is actually happening.
 */
export const SchedulerStatus: Component = () => {
  const schedulerQuery = useSchedulerStatusQuery();
  const serviceQuery = useServiceStatusQuery();

  const sched = () => schedulerQuery.data;
  const svc = () => serviceQuery.data;

  const statusLabel = createMemo(() => {
    const s = sched()?.status ?? 'unknown';
    return s.charAt(0).toUpperCase() + s.slice(1);
  });

  // One-line description of how scheduling is being managed.
  const ownerLine = createMemo(() => {
    const service = svc();
    if (service?.installed) {
      const ver = service.version ? ` v${service.version}` : '';
      return `Windows Service${ver} · ${service.running ? 'running' : 'stopped'}`;
    }
    const holder = sched()?.lockHolder;
    if (holder === 'desktop') return 'Desktop-managed (service not installed)';
    return 'Service not installed';
  });

  return (
    <div class="rounded-lg border border-border bg-card">
      {/* Header */}
      <div class="flex items-center justify-between px-3.5 py-2.5 border-b border-border">
        <div class="flex items-center gap-2 min-w-0">
          <Activity class="h-3.5 w-3.5 text-muted-foreground flex-shrink-0" />
          <span class="text-[13px] font-semibold text-foreground">Scheduler</span>
          <span class="flex items-center gap-1.5 ml-1.5">
            <span class={cn('h-2 w-2 rounded-full flex-shrink-0', statusDot(sched()?.status ?? 'unknown'))} />
            <span class="text-[11px] font-medium text-muted-foreground">{statusLabel()}</span>
          </span>
        </div>
        <Show when={sched()?.lastHeartbeat}>
          <span class="flex items-center gap-1 text-[10px] text-muted-foreground flex-shrink-0">
            <Clock class="h-3 w-3" />
            heartbeat {timeAgo(sched()!.lastHeartbeat)}
          </span>
        </Show>
      </div>

      {/* Owner / service line */}
      <div class="flex items-center gap-2 px-3.5 py-2 border-b border-border/60">
        <Server class="h-3 w-3 text-muted-foreground flex-shrink-0" />
        <span class="text-[11px] text-muted-foreground truncate">{ownerLine()}</span>
        <Show when={sched()?.lockHolder}>
          <span class="text-[10px] text-muted-foreground/70 ml-auto flex-shrink-0">
            lock: {lockHolderLabel(sched()!.lockHolder)}
          </span>
        </Show>
      </div>

      {/* Per-job last runs */}
      <div class="px-3.5 py-3 grid grid-cols-3 gap-3">
        <For each={TRACKED_JOBS}>
          {(job) => {
            const ts = () => sched()?.lastRuns?.[job.key] ?? null;
            const rel = () => timeAgo(ts());
            return (
              <div>
                <span class="text-[9px] font-medium uppercase tracking-wider text-muted-foreground block mb-0.5">
                  {job.label}
                </span>
                <span
                  class={cn('text-[12px] font-mono font-medium', freshnessColor(hoursSince(ts())))}
                  title={ts() ? formatShortDateTime(ts()) : 'No successful run recorded'}
                >
                  {rel() ?? 'never'}
                </span>
              </div>
            );
          }}
        </For>
      </div>
    </div>
  );
};
