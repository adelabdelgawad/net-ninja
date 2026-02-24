import { createQuery, keepPreviousData } from '@tanstack/solid-query';
import { speedTestApi } from '~/api/client';
import { queryKeys } from '~/api/queryKeys';

/**
 * Query hook for fetching speed test results
 *
 * Features:
 * - Stale-while-revalidate for instant navigation
 * - keepPreviousData to show cached data while fetching
 * - Background refresh on mount
 * - Error recovery with fallback empty data
 */
export function useSpeedTestsQuery(filters?: { page?: number; pageSize?: number }) {
  const page = filters?.page ?? 1;
  const pageSize = filters?.pageSize ?? 500;

  return createQuery(() => ({
    queryKey: queryKeys.speedTests.list({ page, pageSize }),
    queryFn: async () => {
      try {
        return await speedTestApi.results({ page, pageSize });
      } catch (e) {
        console.warn('[useSpeedTestsQuery] Failed to load results:', e);
        return { items: [], total: 0, page, pageSize };
      }
    },
    placeholderData: keepPreviousData,
  }));
}

/**
 * Query hook for fetching recent speed tests (last 7 days) for charts
 */
export function useSpeedTestHistoryQuery() {
  return createQuery(() => ({
    queryKey: queryKeys.speedTests.list({ page: 1, pageSize: 1000 }),
    queryFn: async () => {
      try {
        const res = await speedTestApi.results({ page: 1, pageSize: 1000 });
        return res.items;
      } catch (e) {
        console.warn('[useSpeedTestHistoryQuery] Failed to load results:', e);
        return [];
      }
    },
    placeholderData: keepPreviousData,
  }));
}

