/**
 * Query key factory for TanStack Query
 *
 * Provides type-safe, hierarchical query keys for cache management.
 * Use these keys for:
 * - Query definitions (queryKey option)
 * - Cache invalidation (queryClient.invalidateQueries)
 * - Optimistic updates (queryClient.setQueryData)
 *
 * Key structure follows TanStack Query best practices:
 * - ['entity'] - All queries for entity
 * - ['entity', 'list'] - List queries
 * - ['entity', 'list', { filters }] - Filtered list queries
 * - ['entity', 'detail', id] - Single entity queries
 */

export const queryKeys = {
  // Speed Tests
  speedTests: {
    all: ['speedTests'] as const,
    lists: () => [...queryKeys.speedTests.all, 'list'] as const,
    list: (filters?: { page?: number; pageSize?: number }) =>
      [...queryKeys.speedTests.lists(), filters] as const,
    forLine: (lineId: number, limit?: number) =>
      [...queryKeys.speedTests.all, 'forLine', lineId, { limit }] as const,
  },

  // Quota Results
  quotaResults: {
    all: ['quotaResults'] as const,
    lists: () => [...queryKeys.quotaResults.all, 'list'] as const,
    list: (filters?: { page?: number; pageSize?: number }) =>
      [...queryKeys.quotaResults.lists(), filters] as const,
    forLine: (lineId: number, limit?: number) =>
      [...queryKeys.quotaResults.all, 'forLine', lineId, { limit }] as const,
  },

  // Logs
  logs: {
    all: ['logs'] as const,
    lists: () => [...queryKeys.logs.all, 'list'] as const,
    list: (filters?: { page?: number; pageSize?: number }) =>
      [...queryKeys.logs.lists(), filters] as const,
    byProcess: (processId: string) =>
      [...queryKeys.logs.all, 'byProcess', processId] as const,
  },

  // Lines
  lines: {
    all: ['lines'] as const,
    lists: () => [...queryKeys.lines.all, 'list'] as const,
    list: () => [...queryKeys.lines.lists()] as const,
    detail: (id: number) => [...queryKeys.lines.all, 'detail', id] as const,
  },

  // Tasks
  tasks: {
    all: ['tasks'] as const,
    lists: () => [...queryKeys.tasks.all, 'list'] as const,
    list: () => [...queryKeys.tasks.lists()] as const,
    detail: (id: number) => [...queryKeys.tasks.all, 'detail', id] as const,
    executions: (taskId: number) =>
      [...queryKeys.tasks.all, 'executions', taskId] as const,
  },

  // Task Executions
  taskExecutions: {
    all: ['taskExecutions'] as const,
    lists: () => [...queryKeys.taskExecutions.all, 'list'] as const,
    list: (params?: { taskId?: number; status?: string; limit?: number }) =>
      [...queryKeys.taskExecutions.lists(), params] as const,
    detail: (id: number) =>
      [...queryKeys.taskExecutions.all, 'detail', id] as const,
    forTask: (taskId: number, limit?: number) =>
      [...queryKeys.taskExecutions.all, 'forTask', taskId, { limit }] as const,
    latestForTask: (taskId: number) =>
      [...queryKeys.taskExecutions.all, 'latest', taskId] as const,
  },

  // Reports
  reports: {
    all: ['reports'] as const,
    latest: () => [...queryKeys.reports.all, 'latest'] as const,
  },

  // Jobs
  jobs: {
    all: ['jobs'] as const,
    lists: () => [...queryKeys.jobs.all, 'list'] as const,
    list: (limit?: number) => [...queryKeys.jobs.lists(), { limit }] as const,
    detail: (id: string) => [...queryKeys.jobs.all, 'detail', id] as const,
  },

  // Fallback Status
  fallback: {
    all: ['fallback'] as const,
    status: () => [...queryKeys.fallback.all, 'status'] as const,
  },

  // Health
  health: {
    all: ['health'] as const,
    check: () => [...queryKeys.health.all, 'check'] as const,
  },

  // Scheduler
  scheduler: {
    all: ['scheduler'] as const,
    status: () => [...queryKeys.scheduler.all, 'status'] as const,
  },

  // SMTP Configs
  smtpConfigs: {
    all: ['smtpConfigs'] as const,
    lists: () => [...queryKeys.smtpConfigs.all, 'list'] as const,
    list: () => [...queryKeys.smtpConfigs.lists()] as const,
    detail: (id: number) =>
      [...queryKeys.smtpConfigs.all, 'detail', id] as const,
    default: () => [...queryKeys.smtpConfigs.all, 'default'] as const,
  },

  // Emails
  emails: {
    all: ['emails'] as const,
    lists: () => [...queryKeys.emails.all, 'list'] as const,
    list: () => [...queryKeys.emails.lists()] as const,
  },

  // Task Notification Configs
  taskNotifications: {
    all: ['taskNotifications'] as const,
    detail: (taskId: number) =>
      [...queryKeys.taskNotifications.all, 'detail', taskId] as const,
  },
} as const;
