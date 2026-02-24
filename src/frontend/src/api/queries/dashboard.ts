import { createQuery, keepPreviousData } from '@tanstack/solid-query';
import { linesApi, reportsApi, taskExecutionsApi, tasksApi } from '~/api/client';
import { queryKeys } from '~/api/queryKeys';
import type { TaskExecutionResponse, Task } from '~/types';

/**
 * Query hook for fetching lines (dashboard version)
 * Same as useLinesQuery but exported separately for dashboard use
 */
export function useDashboardLinesQuery() {
  return createQuery(() => ({
    queryKey: queryKeys.lines.list(),
    queryFn: async () => {
      try {
        return await linesApi.list();
      } catch (e) {
        console.warn('[useDashboardLinesQuery] Failed to load lines:', e);
        return [];
      }
    },
    placeholderData: keepPreviousData,
  }));
}

/**
 * Query hook for fetching latest report results
 */
export function useLatestReportQuery() {
  return createQuery(() => ({
    queryKey: queryKeys.reports.latest(),
    queryFn: async () => {
      try {
        return await reportsApi.latest();
      } catch (e) {
        console.warn('[useLatestReportQuery] Failed to load results:', e);
        return [];
      }
    },
    placeholderData: keepPreviousData,
  }));
}

/**
 * Query hook for fetching recent task executions for the dashboard.
 *
 * Features:
 * - Fetches last 8 executions
 * - Auto-refresh every 30 seconds for live updates
 * - Returns empty array on failure (no error throws)
 * - keepPreviousData for smooth transitions
 *
 * @param enabled - Whether the query should run (set false when in fallback mode)
 */
export function useRecentExecutionsQuery(enabled: boolean) {
  return createQuery(() => ({
    queryKey: queryKeys.taskExecutions.list({ limit: 8 }),
    queryFn: async (): Promise<TaskExecutionResponse[]> => {
      try {
        return await taskExecutionsApi.list({ limit: 8 });
      } catch (e) {
        console.warn('[useRecentExecutionsQuery] Failed to load executions:', e);
        return [];
      }
    },
    enabled,
    refetchInterval: 30000, // Auto-refresh every 30 seconds
    placeholderData: keepPreviousData,
  }));
}

/**
 * Query hook for fetching scheduled tasks for the dashboard.
 *
 * Features:
 * - Fetches tasks with runMode === 'scheduled' && isActive === true
 * - Returns empty array on failure (no error throws)
 * - keepPreviousData for smooth transitions
 *
 * @param enabled - Whether the query should run (set false when in fallback mode)
 */
export function useScheduledTasksQuery(enabled: boolean) {
  return createQuery(() => ({
    queryKey: [...queryKeys.tasks.lists(), { scheduled: true, active: true }] as const,
    queryFn: async (): Promise<Task[]> => {
      try {
        const allTasks = await tasksApi.list();
        // Filter for scheduled and active tasks only
        return allTasks.filter(task => task.runMode === 'scheduled' && task.isActive);
      } catch (e) {
        console.warn('[useScheduledTasksQuery] Failed to load scheduled tasks:', e);
        return [];
      }
    },
    enabled,
    placeholderData: keepPreviousData,
  }));
}
