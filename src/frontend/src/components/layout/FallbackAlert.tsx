import { type Component, Show, createSignal, onMount } from 'solid-js';
import { Alert, AlertDescription, AlertTitle } from '~/components/ui/alert';
import { Button } from '~/components/ui/button';
import { fallbackApi } from '~/api/client';
import type { FallbackStatusResponse } from '~/types';

export const FallbackAlert: Component = () => {
  const [dismissed, setDismissed] = createSignal(false);
  const [status, setStatus] = createSignal<FallbackStatusResponse | null>(null);
  const [loading, setLoading] = createSignal(true);

  onMount(async () => {
    try {
      const s = await fallbackApi.getStatus();
      setStatus(s);
    } catch (e) {
      console.error('Failed to load fallback status:', e);
    } finally {
      setLoading(false);
    }
  });

  const shouldShow = () => {
    if (dismissed() || loading()) return false;
    const s = status();
    return s?.is_fallback ?? false;
  };

  return (
    <Show when={shouldShow()}>
      <Alert variant="destructive" class="mb-4">
        <div class="flex items-start justify-between gap-4">
          <div class="flex-1">
            <AlertTitle>Running in Limited Mode</AlertTitle>
            <AlertDescription class="mt-2">
              <p class="mb-2">
                The app started in <strong>fallback mode</strong> because the database connection failed.
              </p>
              <Show when={status()?.error}>
                <p class="text-sm opacity-90 mb-2">
                  Error: <code class="bg-background/50 px-1 py-0.5 rounded text-xs">{status()?.error}</code>
                </p>
              </Show>
              <p class="text-sm">
                Features requiring PostgreSQL (lines, quota checks, speed tests, reports) are disabled.
                Only settings management is available.
              </p>
            </AlertDescription>
          </div>
          <Button
            variant="ghost"
            size="sm"
            class="shrink-0"
            onClick={() => setDismissed(true)}
          >
            Dismiss
          </Button>
        </div>
      </Alert>
    </Show>
  );
};
