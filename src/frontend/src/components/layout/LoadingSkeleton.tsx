import { type Component, Index } from 'solid-js';

export const LoadingSkeleton: Component = () => {
  return (
    <div class="flex h-screen w-screen overflow-hidden">
      {/* Sidebar Skeleton */}
      <div class="sidebar-nav flex h-full w-56 flex-col bg-sidebar">
        {/* Header */}
        <div class="flex h-14 items-center px-4">
          <div class="h-5 w-20 animate-pulse rounded bg-white/10" />
        </div>

        {/* Navigation section */}
        <div class="flex flex-1 flex-col px-3 py-1">
          <div class="flex flex-col gap-0.5">
            <Index each={[1, 2, 3, 4, 5, 6, 7]}>
              {() => (
                <div class="flex items-center gap-3 rounded-lg px-3 py-2">
                  <div class="h-[18px] w-[18px] animate-pulse rounded bg-white/10" />
                  <div class="h-4 w-20 animate-pulse rounded bg-white/10" />
                </div>
              )}
            </Index>
          </div>
        </div>

        {/* Bottom section */}
        <div class="px-3 pb-3 space-y-0.5">
          <div class="flex items-center gap-3 rounded-lg px-3 py-2">
            <div class="h-[18px] w-[18px] animate-pulse rounded bg-white/10" />
            <div class="h-4 w-16 animate-pulse rounded bg-white/10" />
          </div>
          <div class="flex items-center gap-3 rounded-lg px-3 py-2">
            <div class="h-[18px] w-[18px] animate-pulse rounded bg-white/10" />
            <div class="h-4 w-12 animate-pulse rounded bg-white/10" />
          </div>
        </div>
      </div>

      {/* Main content area */}
      <div class="flex flex-1 flex-col overflow-hidden">
        {/* Main content with padding */}
        <main class="flex-1 overflow-auto p-6">
          {/* Title placeholder */}
          <div class="mb-6">
            <div class="mb-2 h-8 w-48 animate-pulse rounded bg-muted" />
            <div class="h-4 w-96 animate-pulse rounded bg-muted/70" />
          </div>

          {/* Content placeholders */}
          <div class="space-y-4">
            {/* First row of cards */}
            <div class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
              <Index each={[1, 2, 3]}>
                {() => (
                  <div class="rounded-lg border bg-card p-6">
                    <div class="mb-4 h-5 w-32 animate-pulse rounded bg-muted" />
                    <div class="mb-2 h-8 w-24 animate-pulse rounded bg-muted/70" />
                    <div class="h-4 w-40 animate-pulse rounded bg-muted/50" />
                  </div>
                )}
              </Index>
            </div>

            {/* Second row - larger card */}
            <div class="rounded-lg border bg-card p-6">
              <div class="mb-4 h-5 w-40 animate-pulse rounded bg-muted" />
              <div class="space-y-3">
                <Index each={[1, 2, 3, 4]}>
                  {() => (
                    <div class="flex items-center gap-4">
                      <div class="h-10 w-10 animate-pulse rounded bg-muted" />
                      <div class="flex-1 space-y-2">
                        <div class="h-4 w-48 animate-pulse rounded bg-muted/70" />
                        <div class="h-3 w-32 animate-pulse rounded bg-muted/50" />
                      </div>
                    </div>
                  )}
                </Index>
              </div>
            </div>

            {/* Third row - content blocks */}
            <div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
              <Index each={[1, 2]}>
                {() => (
                  <div class="rounded-lg border bg-card p-6">
                    <div class="mb-4 h-5 w-36 animate-pulse rounded bg-muted" />
                    <div class="space-y-2">
                      <Index each={[1, 2, 3]}>
                        {() => <div class="h-4 w-full animate-pulse rounded bg-muted/50" />}
                      </Index>
                    </div>
                  </div>
                )}
              </Index>
            </div>
          </div>
        </main>

        {/* Status bar skeleton */}
        <footer class="flex h-7 items-center justify-between border-t bg-card px-4 text-xs">
          <div class="flex items-center gap-2">
            <div class="h-2 w-2 animate-pulse rounded-full bg-warning" />
            <div class="h-4 w-20 animate-pulse rounded bg-muted" />
          </div>
          <div class="h-4 w-48 animate-pulse rounded bg-muted/70 font-mono" />
          <div class="flex items-center gap-3">
            <div class="h-4 w-24 animate-pulse rounded bg-muted font-mono" />
            <div class="h-5 w-6 animate-pulse rounded bg-muted" />
          </div>
        </footer>
      </div>
    </div>
  );
};
