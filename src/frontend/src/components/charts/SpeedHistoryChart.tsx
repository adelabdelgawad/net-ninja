import { type Component, onMount, onCleanup, createEffect, on } from 'solid-js';
import {
  Chart,
  LineController,
  LineElement,
  PointElement,
  LinearScale,
  CategoryScale,
  Tooltip,
  Legend,
  Filler,
  type ChartConfiguration,
} from 'chart.js';
import type { SpeedTestResult } from '~/types';

// Register required Chart.js components (category scale for string labels on x-axis)
Chart.register(LineController, LineElement, PointElement, LinearScale, CategoryScale, Tooltip, Legend, Filler);

// Distinct colors for up to 10 lines
const LINE_COLORS = [
  '#3584e4', // blue (primary)
  '#4ec9b0', // teal/green
  '#dcdcaa', // yellow
  '#c586c0', // purple
  '#ce9178', // orange
  '#569cd6', // light blue
  '#d16969', // salmon
  '#b5cea8', // lime
  '#d4d4d4', // silver
  '#9cdcfe', // sky
];

export interface SpeedHistoryChartProps {
  results: SpeedTestResult[];
  lineNames: Map<number, string>;
  field: 'downloadSpeed' | 'uploadSpeed';
  label: string;
}

export const SpeedHistoryChart: Component<SpeedHistoryChartProps> = (props) => {
  let canvasRef!: HTMLCanvasElement;
  let chart: Chart | undefined;

  const buildChartData = () => {
    const sevenDaysAgo = new Date();
    sevenDaysAgo.setDate(sevenDaysAgo.getDate() - 7);

    // Filter to last 7 days and successful tests only
    const recent = props.results.filter((r) => {
      const date = new Date(r.createdAt);
      return date >= sevenDaysAgo && r.status === 'success' && r[props.field] != null;
    });

    // Group by lineId
    const byLine = new Map<number, SpeedTestResult[]>();
    for (const r of recent) {
      const arr = byLine.get(r.lineId) ?? [];
      arr.push(r);
      byLine.set(r.lineId, arr);
    }

    // Sort each line's data by time
    for (const [, arr] of byLine) {
      arr.sort((a, b) => new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime());
    }

    // Build unique sorted date labels (short format)
    const allDates = new Set<string>();
    for (const r of recent) {
      allDates.add(formatLabel(r.createdAt));
    }
    const labels = [...allDates].sort((a, b) => parseLabelDate(a) - parseLabelDate(b));

    // Build datasets
    const lineIds = [...byLine.keys()].sort((a, b) => a - b);
    const datasets = lineIds.map((lineId, i) => {
      const color = LINE_COLORS[i % LINE_COLORS.length];
      const points = byLine.get(lineId)!;

      // Map points to label positions
      const dataByLabel = new Map<string, number[]>();
      for (const p of points) {
        const lbl = formatLabel(p.createdAt);
        const arr = dataByLabel.get(lbl) ?? [];
        arr.push(p[props.field]!);
        dataByLabel.set(lbl, arr);
      }

      // Average multiple points at same label
      const data = labels.map((lbl) => {
        const vals = dataByLabel.get(lbl);
        if (!vals || vals.length === 0) return null;
        return vals.reduce((a, b) => a + b, 0) / vals.length;
      });

      return {
        label: props.lineNames.get(lineId) ?? `Line ${lineId}`,
        data,
        borderColor: color,
        backgroundColor: color + '18',
        borderWidth: 2,
        pointRadius: 3,
        pointHoverRadius: 5,
        pointBackgroundColor: color,
        pointBorderColor: 'transparent',
        tension: 0.35,
        fill: false,
        spanGaps: true,
      };
    });

    return { labels, datasets };
  };

  onMount(() => {
    const { labels, datasets } = buildChartData();

    const config: ChartConfiguration = {
      type: 'line',
      data: { labels, datasets },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        interaction: {
          mode: 'index',
          intersect: false,
        },
        plugins: {
          legend: {
            display: true,
            position: 'bottom',
            labels: {
              color: '#868F97',
              font: { family: "'Inter', system-ui, sans-serif", size: 10 },
              boxWidth: 12,
              boxHeight: 2,
              padding: 12,
              usePointStyle: false,
            },
          },
          tooltip: {
            backgroundColor: '#1A1A1F',
            titleColor: '#E6E6E6',
            bodyColor: '#E6E6E6',
            borderColor: '#232323',
            borderWidth: 1,
            titleFont: { family: "'Inter', system-ui, sans-serif", size: 11 },
            bodyFont: { family: "'JetBrains Mono', monospace", size: 11 },
            padding: 8,
            cornerRadius: 6,
            callbacks: {
              label: (ctx) => {
                const val = ctx.parsed.y;
                return val != null ? ` ${ctx.dataset.label}: ${val.toFixed(2)} Mbps` : '';
              },
            },
          },
        },
        scales: {
          x: {
            grid: {
              color: '#232323',
              lineWidth: 0.5,
            },
            ticks: {
              color: '#868F97',
              font: { family: "'Inter', system-ui, sans-serif", size: 9 },
              maxRotation: 0,
            },
            border: { color: '#232323' },
          },
          y: {
            beginAtZero: true,
            grid: {
              color: '#232323',
              lineWidth: 0.5,
            },
            ticks: {
              color: '#868F97',
              font: { family: "'JetBrains Mono', monospace", size: 9 },
              callback: (val) => `${val}`,
            },
            border: { color: '#232323' },
            title: {
              display: true,
              text: 'Mbps',
              color: '#868F97',
              font: { family: "'Inter', system-ui, sans-serif", size: 9 },
            },
          },
        },
      },
    };

    chart = new Chart(canvasRef, config);
  });

  // Reactively update chart data when props change
  createEffect(
    on(
      () => [props.results, props.lineNames, props.field] as const,
      () => {
        if (!chart) return;
        const { labels, datasets } = buildChartData();
        chart.data.labels = labels;
        chart.data.datasets = datasets as any;
        chart.update('none');
      },
    ),
  );

  onCleanup(() => {
    chart?.destroy();
  });

  return (
    <div class="rounded-lg border border-border bg-card overflow-hidden">
      <div class="px-3.5 py-2 border-b border-border">
        <span class="text-[11px] font-medium text-foreground">{props.label}</span>
        <span class="text-[10px] text-muted-foreground ml-2">Last 7 days</span>
      </div>
      <div class="px-3 py-2" style={{ height: '220px' }}>
        <canvas ref={canvasRef!} />
      </div>
    </div>
  );
};

// --- Helpers ---

function formatLabel(dateStr: string): string {
  const d = new Date(dateStr);
  const month = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  const hours = String(d.getHours()).padStart(2, '0');
  const mins = String(d.getMinutes()).padStart(2, '0');
  return `${month}/${day} ${hours}:${mins}`;
}

function parseLabelDate(lbl: string): number {
  // Parse "MM/DD HH:mm" back to a comparable number
  const [datePart, timePart] = lbl.split(' ');
  const [month, day] = datePart.split('/').map(Number);
  const [hours, mins] = timePart.split(':').map(Number);
  // Use current year for comparison
  return new Date(new Date().getFullYear(), month - 1, day, hours, mins).getTime();
}
