import { type Component, createSignal, onCleanup, createEffect } from 'solid-js';
import { useLocation } from '@solidjs/router';
import { healthApi } from '~/api/client';
import { cn } from '~/lib/utils';

type ConnectionStatus = 'connected' | 'disconnected' | 'checking';

export const StatusBar: Component = () => {
  const location = useLocation();
  const [status, setStatus] = createSignal<ConnectionStatus>('checking');
  const [lastCheck, setLastCheck] = createSignal<Date | null>(null);

  // Check if current route is a Settings route - server logic should NOT run here
  const isSettingsRoute = () => {
    const path = location.pathname;
    return path.startsWith('/settings');
  };

  const checkHealth = async () => {
    // Skip health check entirely on Settings routes
    if (isSettingsRoute()) {
      setStatus('disconnected');
      return;
    }

    try {
      setStatus('checking');
      await healthApi.ready();
      setStatus('connected');
      setLastCheck(new Date());
    } catch {
      setStatus('disconnected');
    }
  };

  // Use createEffect to respond to route changes
  let interval: ReturnType<typeof setInterval> | null = null;

  createEffect(() => {
    const onSettings = isSettingsRoute();

    if (interval) {
      // Clear existing interval when route changes
      clearInterval(interval);
      interval = null;
    }

    if (!onSettings) {
      // Start polling on non-settings routes
      checkHealth();
      interval = setInterval(checkHealth, 30000);
    } else {
      // Set status to disconnected on settings routes
      setStatus('disconnected');
    }
  });

  onCleanup(() => {
    if (interval) clearInterval(interval);
  });

  const formatTime = (date: Date | null) => {
    if (!date) return '--:--:--';
    return date.toLocaleTimeString('en-US', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    });
  };

  return (
    <footer class="flex h-7 items-center justify-between border-t bg-card px-4 text-xs">
      <div class="flex items-center gap-2">
        <div
          class={cn(
            'h-2 w-2 rounded-full',
            status() === 'connected' && 'bg-[#26a269] animate-pulse-glow',
            status() === 'disconnected' && 'bg-[#c72e0f]',
            status() === 'checking' && 'bg-[#e5a50a] animate-pulse'
          )}
        />
        <span
          class={cn(
            'font-medium',
            status() === 'connected' && 'text-[#26a269]',
            status() === 'disconnected' && 'text-[#c72e0f]',
            status() === 'checking' && 'text-[#e5a50a]'
          )}
        >
          {status() === 'connected' && 'Connected'}
          {status() === 'disconnected' && 'Disconnected'}
          {status() === 'checking' && 'Checking...'}
        </span>
      </div>

      <div class="text-muted-foreground font-mono">
        NetNinja Network Monitor
      </div>

      <div class="flex items-center gap-3">
        <span class="text-muted-foreground font-mono">
          Last: {formatTime(lastCheck())}
        </span>
        <button
          class="px-2 py-0.5 text-[11px] hover:bg-[#2d2d2d] rounded-[6px]"
          onClick={checkHealth}
        >
          ↻
        </button>
      </div>
    </footer>
  );
};
