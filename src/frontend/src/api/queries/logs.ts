import { createQuery, keepPreviousData } from '@tanstack/solid-query';
import { logsApi } from '~/api/client';
import { queryKeys } from '~/api/queryKeys';

/**
 * Query hook for fetching logs
 *
 * Features:
 * - Stale-while-revalidate for instant navigation
 * - keepPreviousData to show cached data while fetching
 * - Background refresh on mount
 * - Error recovery with fallback empty data
 */
export function useLogsQuery(filters?: { page?: number; pageSize?: number }) {
  const page = filters?.page ?? 1;
  const pageSize = filters?.pageSize ?? 500;

  return createQuery(() => ({
    queryKey: queryKeys.logs.list({ page, pageSize }),
    queryFn: async () => {
      try {
        return await logsApi.list({ page, pageSize });
      } catch (e) {
        console.warn('[useLogsQuery] Failed to load logs:', e);
        return { items: [], total: 0, page, pageSize };
      }
    },
    placeholderData: keepPreviousData,
  }));
}

