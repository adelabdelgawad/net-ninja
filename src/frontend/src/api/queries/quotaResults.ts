import { createQuery, keepPreviousData } from '@tanstack/solid-query';
import { quotaApi } from '~/api/client';
import { queryKeys } from '~/api/queryKeys';

/**
 * Query hook for fetching quota results
 *
 * Features:
 * - Stale-while-revalidate for instant navigation
 * - keepPreviousData to show cached data while fetching
 * - Background refresh on mount
 * - Error recovery with fallback empty data
 */
export function useQuotaResultsQuery(filters?: { page?: number; pageSize?: number }) {
  const page = filters?.page ?? 1;
  const pageSize = filters?.pageSize ?? 500;

  return createQuery(() => ({
    queryKey: queryKeys.quotaResults.list({ page, pageSize }),
    queryFn: async () => {
      try {
        return await quotaApi.results({ page, pageSize });
      } catch (e) {
        console.warn('[useQuotaResultsQuery] Failed to load results:', e);
        return { items: [], total: 0, page, pageSize };
      }
    },
    placeholderData: keepPreviousData,
  }));
}

