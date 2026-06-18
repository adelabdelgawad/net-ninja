import { createQuery, keepPreviousData } from '@tanstack/solid-query';
import { schedulerApi } from '~/api/client';
import { queryKeys } from '~/api/queryKeys';
import type { SchedulerStatusResponse, ServiceStatusResponse } from '~/types';

/**
 * Scheduler liveness + per-job last-run markers.
 *
 * Auto-refreshes every 15s (the scheduler heartbeat interval) so the dashboard
 * reflects whether scheduling is currently alive. Returns a safe "unknown"
 * status on failure rather than throwing.
 */
export function useSchedulerStatusQuery() {
  return createQuery(() => ({
    queryKey: queryKeys.scheduler.status(),
    queryFn: async (): Promise<SchedulerStatusResponse> => {
      try {
        return await schedulerApi.getStatus();
      } catch (e) {
        console.warn('[useSchedulerStatusQuery] Failed to load scheduler status:', e);
        return {
          status: 'unknown',
          isRunning: false,
          lockHolder: null,
          lastHeartbeat: null,
          lastRuns: {},
        };
      }
    },
    refetchInterval: 15000,
    placeholderData: keepPreviousData,
  }));
}

/**
 * Windows service install/run state + reported version. Auto-refreshes every 15s.
 */
export function useServiceStatusQuery() {
  return createQuery(() => ({
    queryKey: queryKeys.scheduler.service(),
    queryFn: async (): Promise<ServiceStatusResponse> => {
      try {
        return await schedulerApi.getServiceStatus();
      } catch (e) {
        console.warn('[useServiceStatusQuery] Failed to load service status:', e);
        return {
          installed: false,
          running: false,
          version: null,
          lastHeartbeat: null,
          lockHolder: null,
        };
      }
    },
    refetchInterval: 15000,
    placeholderData: keepPreviousData,
  }));
}
