import { type ParentComponent } from 'solid-js';
import { QueryClient, QueryClientProvider } from '@tanstack/solid-query';

/**
 * QueryClient configuration for instant navigation with stale-while-revalidate
 *
 * Cache strategy:
 * - staleTime: 2 min - Data considered fresh, instant nav during this window
 * - gcTime: 10 min - Cache retained for back/forward navigation
 * - refetchOnMount: 'always' - Background refresh when returning to page
 * - refetchOnWindowFocus: false - Disabled for desktop app (Tauri)
 */
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 2 * 60 * 1000, // 2 minutes - data fresh for instant navigation
      gcTime: 10 * 60 * 1000, // 10 minutes - cache kept after last use
      refetchOnMount: 'always', // Background refresh when returning to page
      refetchOnWindowFocus: false, // Disabled for desktop app
      retry: 1, // Single retry on failure
      retryDelay: 1000,
    },
  },
});

/**
 * Export queryClient for use in mutations and manual cache operations
 */
export { queryClient };

/**
 * QueryProvider wrapper component
 * Provides TanStack Query context to the application
 */
export const QueryProvider: ParentComponent = (props) => {
  return (
    <QueryClientProvider client={queryClient}>
      {props.children}
    </QueryClientProvider>
  );
};
