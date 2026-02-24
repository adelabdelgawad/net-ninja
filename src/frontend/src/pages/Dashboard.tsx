import { type Component, createMemo, For, Show } from 'solid-js';
import {
  ArrowDown,
  ArrowUp,
  Gauge,
  Wifi,
  HardDrive,
  Calendar,
  Wallet,
  Clock,
  Play,
} from 'lucide-solid';
import { showToast } from '~/components/ui/toast';
import {
  useDashboardLinesQuery,
  useLatestReportQuery,
  useSpeedTestHistoryQuery,
} from '~/api/queries';
import type { CombinedResult } from '~/types';
import { cn } from '~/lib/utils';
import { formatShortDateTime } from '~/lib/date';
import { SpeedHistoryChart } from '~/components/charts/SpeedHistoryChart';

// --- Color helpers ---

const speedColor = (speed: number | null) => {
  if (speed == null) return 'text-muted-foreground';
  if (speed > 50) return 'text-[#4ec9b0]';
  if (speed > 20) return 'text-[#dcdcaa]';
  return 'text-[#c72e0f]';
};

const pingColor = (ping: number | null) => {
  if (ping == null) return 'text-muted-foreground';
  if (ping < 50) return 'text-[#4ec9b0]';
  if (ping < 100) return 'text-[#dcdcaa]';
  return 'text-[#c72e0f]';
};

const quotaColor = (pct: number | null) => {
  if (pct == null) return 'text-muted-foreground';
  if (pct < 70) return 'text-[#4ec9b0]';
  if (pct < 90) return 'text-[#dcdcaa]';
  return 'text-[#c72e0f]';
};

const quotaBarColor = (pct: number | null) => {
  if (pct == null) return 'bg-muted-foreground/30';
  if (pct < 70) return 'bg-[#4ec9b0]';
  if (pct < 90) return 'bg-[#dcdcaa]';
  return 'bg-[#c72e0f]';
};

const statusDotColor = (r: CombinedResult) => {
  if (r.download == null || r.download === 0) return 'bg-[#c72e0f]';
  if (r.download > 20 && (r.usagePercentage ?? 0) < 90) return 'bg-[#4ec9b0]';
  return 'bg-[#dcdcaa]';
};

export const Dashboard: Component = () => {
  const linesQuery = useDashboardLinesQuery();
  const resultsQuery = useLatestReportQuery();
  const historyQuery = useSpeedTestHistoryQuery();

  const results = () => resultsQuery.data ?? [];
  const historyData = () => historyQuery.data ?? [];

  // Build lineId -> name map for chart legends
  const lineNameMap = createMemo(() => {
    const map = new Map<number, string>();
    for (const line of linesQuery.data ?? []) {
      map.set(line.id, line.name);
    }
    return map;
  });

  const stats = createMemo(() => {
    const data = results();
    if (!data.length) return { lines: 0, avgDown: 0, avgPing: 0, quotaWarnings: 0, errors: 0 };

    const downs = data.filter((r) => r.download != null).map((r) => r.download!);
    const pings = data.filter((r) => r.ping != null).map((r) => r.ping!);

    return {
      lines: linesQuery.data?.length ?? 0,
      avgDown: downs.length ? downs.reduce((a, b) => a + b, 0) / downs.length : 0,
      avgPing: pings.length ? pings.reduce((a, b) => a + b, 0) / pings.length : 0,
      quotaWarnings: data.filter((r) => (r.usagePercentage ?? 0) >= 80).length,
      errors: data.filter((r) => r.download === 0 || r.download == null).length,
    };
  });

  const runFullCheck = () => {
    // TODO: implement full check via task execution
    showToast({ title: 'Full check not yet implemented', variant: 'warning', duration: 3000 });
  };

  return (
    <div class="flex flex-col h-full">
      {/* Header */}
      <div class="px-4 pt-3 pb-3 flex-shrink-0">
        <div class="flex items-center justify-between">
          <div>
            <h1 class="text-[20px] font-semibold text-foreground">Dashboard</h1>
            <p class="text-[12px] text-muted-foreground mt-0.5">Network overview across all internet lines</p>
          </div>
          <button
            type="button"
            class="flex items-center gap-1.5 px-3 py-1.5 text-[11px] font-medium rounded-lg bg-primary text-primary-foreground hover:opacity-90 transition-opacity"
            onClick={runFullCheck}
          >
            <Play class="h-3 w-3" />
            Run Full Check
          </button>
        </div>
      </div>

      {/* Scrollable content */}
      <div class="flex-1 overflow-y-auto px-4 pb-4">
        {/* Summary Stats */}
        <div class="grid grid-cols-5 gap-2.5 mb-4">
          <StatCard label="Lines" value={String(stats().lines)} icon={<Wifi class="h-3.5 w-3.5" />} />
          <StatCard
            label="Avg Download"
            value={`${stats().avgDown.toFixed(1)} Mbps`}
            icon={<ArrowDown class="h-3.5 w-3.5" />}
            valueClass={speedColor(stats().avgDown || null)}
          />
          <StatCard
            label="Avg Ping"
            value={`${stats().avgPing.toFixed(1)} ms`}
            icon={<Gauge class="h-3.5 w-3.5" />}
            valueClass={pingColor(stats().avgPing || null)}
          />
          <StatCard
            label="Quota Warnings"
            value={stats().quotaWarnings > 0 ? String(stats().quotaWarnings) : 'None'}
            icon={<HardDrive class="h-3.5 w-3.5" />}
            valueClass={stats().quotaWarnings === 0 ? 'text-[#4ec9b0]' : 'text-[#dcdcaa]'}
          />
          <StatCard
            label="Issues"
            value={String(stats().errors)}
            icon={<Wifi class="h-3.5 w-3.5" />}
            valueClass={stats().errors === 0 ? 'text-[#4ec9b0]' : 'text-[#c72e0f]'}
          />
        </div>

        {/* Speed History Charts */}
        <Show when={historyData().length > 0}>
          <div class="grid grid-cols-2 gap-3 mb-4">
            <SpeedHistoryChart
              results={historyData()}
              lineNames={lineNameMap()}
              field="downloadSpeed"
              label="Download Speed"
            />
            <SpeedHistoryChart
              results={historyData()}
              lineNames={lineNameMap()}
              field="uploadSpeed"
              label="Upload Speed"
            />
          </div>
        </Show>

        {/* Line Cards */}
        <Show
          when={!resultsQuery.isPending || results().length > 0}
          fallback={
            <div class="grid grid-cols-2 gap-3">
              <For each={[1, 2, 3, 4]}>{() => <SkeletonCard />}</For>
            </div>
          }
        >
          <Show
            when={results().length > 0}
            fallback={
              <div class="flex flex-col items-center justify-center py-16 text-muted-foreground">
                <Wifi class="h-10 w-10 mb-3 opacity-30" />
                <p class="text-sm">No recent results</p>
                <p class="text-xs mt-1">Run a full check to see data here</p>
              </div>
            }
          >
            <div class="grid grid-cols-2 gap-3">
              <For each={results()}>
                {(line) => <LineCard line={line} />}
              </For>
            </div>
          </Show>
        </Show>
      </div>
    </div>
  );
};

// --- Stat Card ---

const StatCard: Component<{
  label: string;
  value: string;
  icon: any;
  valueClass?: string;
}> = (props) => (
  <div class="rounded-lg border border-border bg-card px-3 py-2.5">
    <div class="flex items-center gap-1.5 text-muted-foreground mb-1">
      {props.icon}
      <span class="text-[10px] font-medium uppercase tracking-wider">{props.label}</span>
    </div>
    <div class={cn('text-[15px] font-semibold font-mono', props.valueClass ?? 'text-foreground')}>
      {props.value}
    </div>
  </div>
);

// --- Line Card ---

const LineCard: Component<{ line: CombinedResult }> = (props) => {
  const l = () => props.line;

  return (
    <div class="rounded-lg border border-border bg-card overflow-hidden hover:border-[#3584e4]/30 transition-colors">
      {/* Card Header */}
      <div class="flex items-center justify-between px-3.5 py-2.5 border-b border-border">
        <div class="flex items-center gap-2.5 min-w-0">
          <span class={cn('h-2 w-2 rounded-full flex-shrink-0', statusDotColor(l()))} />
          <div class="min-w-0">
            <span class="text-[13px] font-semibold text-foreground truncate block">{l().name}</span>
            <span class="text-[10px] text-muted-foreground">{l().isp}</span>
          </div>
        </div>
        <Show when={l().lastUpdated}>
          <span class="flex items-center gap-1 text-[10px] text-muted-foreground flex-shrink-0">
            <Clock class="h-3 w-3" />
            {formatShortDateTime(l().lastUpdated!)}
          </span>
        </Show>
      </div>

      {/* Metrics Grid */}
      <div class="px-3.5 py-3 grid grid-cols-3 gap-x-3 gap-y-3">
        {/* Download */}
        <Metric
          label="Download"
          icon={<ArrowDown class="h-3 w-3" />}
          value={l().download != null ? `${l().download!.toFixed(2)}` : '--'}
          unit="Mbps"
          valueClass={speedColor(l().download)}
        />
        {/* Upload */}
        <Metric
          label="Upload"
          icon={<ArrowUp class="h-3 w-3" />}
          value={l().upload != null ? `${l().upload!.toFixed(2)}` : '--'}
          unit="Mbps"
        />
        {/* Ping */}
        <Metric
          label="Ping"
          icon={<Gauge class="h-3 w-3" />}
          value={l().ping != null ? `${l().ping!.toFixed(2)}` : '--'}
          unit="ms"
          valueClass={pingColor(l().ping)}
        />
      </div>

      {/* Quota Section */}
      <div class="px-3.5 pb-3">
        <div class="rounded-md bg-secondary/50 px-3 py-2.5">
          {/* Quota bar */}
          <div class="flex items-center justify-between mb-1.5">
            <span class="text-[10px] font-medium text-muted-foreground uppercase tracking-wider">Quota Usage</span>
            <span class={cn('text-[12px] font-semibold font-mono', quotaColor(l().usagePercentage))}>
              {l().usagePercentage != null ? `${l().usagePercentage!.toFixed(1)}%` : '--'}
            </span>
          </div>
          <div class="h-1.5 rounded-full bg-border overflow-hidden mb-2.5">
            <div
              class={cn('h-full rounded-full transition-all', quotaBarColor(l().usagePercentage))}
              style={{ width: `${Math.min(l().usagePercentage ?? 0, 100)}%` }}
            />
          </div>

          {/* Quota details row */}
          <div class="grid grid-cols-3 gap-2">
            <div>
              <span class="text-[9px] text-muted-foreground block">Remaining</span>
              <span class="text-[12px] font-mono font-medium text-foreground">
                {l().dataRemaining != null ? `${l().dataRemaining} GB` : '--'}
              </span>
            </div>
            <div>
              <span class="text-[9px] text-muted-foreground block">Balance</span>
              <span class="text-[12px] font-mono font-medium text-foreground">
                <Show when={l().balance != null} fallback="--">
                  <span class="flex items-center gap-0.5">
                    <Wallet class="h-3 w-3 text-muted-foreground" />
                    {l().balance} EGP
                  </span>
                </Show>
              </span>
            </div>
            <div>
              <span class="text-[9px] text-muted-foreground block">Renewal</span>
              <span class="text-[12px] font-mono font-medium text-foreground">
                <Show when={l().renewalDate} fallback="--">
                  <span class="flex items-center gap-0.5">
                    <Calendar class="h-3 w-3 text-muted-foreground" />
                    {l().renewalDate}
                  </span>
                </Show>
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

// --- Metric ---

const Metric: Component<{
  label: string;
  icon: any;
  value: string;
  unit: string;
  valueClass?: string;
}> = (props) => (
  <div>
    <div class="flex items-center gap-1 text-muted-foreground mb-0.5">
      {props.icon}
      <span class="text-[9px] font-medium uppercase tracking-wider">{props.label}</span>
    </div>
    <div class="flex items-baseline gap-1">
      <span class={cn('text-[16px] font-bold font-mono leading-tight', props.valueClass ?? 'text-foreground')}>
        {props.value}
      </span>
      <span class="text-[9px] text-muted-foreground">{props.unit}</span>
    </div>
  </div>
);

// --- Skeleton ---

const SkeletonCard: Component = () => (
  <div class="rounded-lg border border-border bg-card overflow-hidden animate-pulse">
    <div class="px-3.5 py-2.5 border-b border-border flex items-center gap-2.5">
      <div class="h-2 w-2 rounded-full bg-muted" />
      <div class="h-3 w-24 rounded bg-muted" />
    </div>
    <div class="px-3.5 py-3 grid grid-cols-3 gap-3">
      <div class="space-y-1.5">
        <div class="h-2 w-12 rounded bg-muted" />
        <div class="h-4 w-16 rounded bg-muted" />
      </div>
      <div class="space-y-1.5">
        <div class="h-2 w-12 rounded bg-muted" />
        <div class="h-4 w-16 rounded bg-muted" />
      </div>
      <div class="space-y-1.5">
        <div class="h-2 w-12 rounded bg-muted" />
        <div class="h-4 w-16 rounded bg-muted" />
      </div>
    </div>
    <div class="px-3.5 pb-3">
      <div class="rounded-md bg-secondary/50 px-3 py-2.5">
        <div class="h-1.5 rounded-full bg-muted mb-2" />
        <div class="grid grid-cols-3 gap-2">
          <div class="h-3 w-14 rounded bg-muted" />
          <div class="h-3 w-14 rounded bg-muted" />
          <div class="h-3 w-14 rounded bg-muted" />
        </div>
      </div>
    </div>
  </div>
);
