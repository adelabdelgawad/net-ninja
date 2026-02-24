import { createQuery, createMutation, useQueryClient, keepPreviousData } from '@tanstack/solid-query';
import { tasksApi, taskNotificationApi } from '~/api/client';
import { queryKeys } from '~/api/queryKeys';
import type { Task, CreateTaskRequest, UpdateTaskRequest, UpsertTaskNotificationConfig, RuntimeNotificationConfig } from '~/types';

/**
 * Query hook for fetching all tasks
 *
 * Features:
 * - Stale-while-revalidate for instant navigation
 * - keepPreviousData to show cached data while fetching (no flicker)
 * - Auto-polls every 5s when any task is "running" to pick up status changes
 * - Structural sharing prevents unnecessary re-renders
 */
export function useTasksQuery() {
  return createQuery(() => ({
    queryKey: queryKeys.tasks.list(),
    queryFn: async () => {
      try {
        return await tasksApi.list();
      } catch (e) {
        console.warn('[useTasksQuery] Failed to load tasks:', e);
        return [];
      }
    },
    placeholderData: keepPreviousData,
    refetchInterval: 5000,
    refetchIntervalInBackground: false,
  }));
}

/**
 * Mutation hook for creating a task
 * Invalidates tasks list cache on success
 */
export function useCreateTaskMutation() {
  const queryClient = useQueryClient();

  return createMutation(() => ({
    mutationFn: (data: CreateTaskRequest) => tasksApi.create(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.tasks.all });
    },
  }));
}

/**
 * Mutation hook for updating a task
 */
export function useUpdateTaskMutation() {
  const queryClient = useQueryClient();

  return createMutation(() => ({
    mutationFn: ({ id, data }: { id: number; data: UpdateTaskRequest }) =>
      tasksApi.update(id, data),
    onSuccess: (_result, variables) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.tasks.detail(variables.id) });
      queryClient.invalidateQueries({ queryKey: queryKeys.tasks.lists() });
    },
  }));
}

/**
 * Mutation hook for deleting a task
 * Uses optimistic update to remove from list immediately
 */
export function useDeleteTaskMutation() {
  const queryClient = useQueryClient();

  return createMutation(() => ({
    mutationFn: (id: number) => tasksApi.delete(id),
    onMutate: async (id) => {
      await queryClient.cancelQueries({ queryKey: queryKeys.tasks.lists() });

      const previousTasks = queryClient.getQueryData<Task[]>(queryKeys.tasks.list());

      if (previousTasks) {
        queryClient.setQueryData<Task[]>(
          queryKeys.tasks.list(),
          previousTasks.filter((task) => task.id !== id)
        );
      }

      return { previousTasks };
    },
    onError: (_err, _id, context) => {
      if (context?.previousTasks) {
        queryClient.setQueryData(queryKeys.tasks.list(), context.previousTasks);
      }
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.tasks.all });
    },
  }));
}

/**
 * Mutation hook for toggling task active status
 */
export function useToggleTaskActiveMutation() {
  const queryClient = useQueryClient();

  return createMutation(() => ({
    mutationFn: ({ id, isActive }: { id: number; isActive: boolean }) =>
      tasksApi.toggleActive(id, isActive),
    onMutate: async ({ id, isActive }) => {
      await queryClient.cancelQueries({ queryKey: queryKeys.tasks.lists() });

      const previousTasks = queryClient.getQueryData<Task[]>(queryKeys.tasks.list());

      if (previousTasks) {
        queryClient.setQueryData<Task[]>(
          queryKeys.tasks.list(),
          previousTasks.map((task) =>
            task.id === id ? { ...task, isActive } : task
          )
        );
      }

      return { previousTasks };
    },
    onError: (_err, _variables, context) => {
      if (context?.previousTasks) {
        queryClient.setQueryData(queryKeys.tasks.list(), context.previousTasks);
      }
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.tasks.all });
    },
  }));
}

/**
 * Mutation hook for executing a task with optional notification override
 * Uses optimistic update to immediately show "running" status
 */
export function useExecuteTaskMutation() {
  const queryClient = useQueryClient();

  return createMutation(() => ({
    mutationFn: ({ id, notificationOverride }: { id: number; notificationOverride?: RuntimeNotificationConfig }) =>
      tasksApi.execute(id, notificationOverride),
    onMutate: async ({ id }) => {
      // Cancel any outgoing refetches so they don't overwrite our optimistic update
      await queryClient.cancelQueries({ queryKey: queryKeys.tasks.lists() });

      const previousTasks = queryClient.getQueryData<Task[]>(queryKeys.tasks.list());

      // Optimistically set the task status to "running"
      if (previousTasks) {
        queryClient.setQueryData<Task[]>(
          queryKeys.tasks.list(),
          previousTasks.map((task) =>
            task.id === id ? { ...task, status: 'running' } : task
          )
        );
      }

      return { previousTasks };
    },
    onError: (_err, _variables, context) => {
      // Roll back to previous state on error
      if (context?.previousTasks) {
        queryClient.setQueryData(queryKeys.tasks.list(), context.previousTasks);
      }
    },
    onSettled: (_result, _error, { id }) => {
      // Always refetch after mutation settles (success or error)
      queryClient.invalidateQueries({ queryKey: queryKeys.tasks.detail(id) });
      queryClient.invalidateQueries({ queryKey: queryKeys.tasks.lists() });
      queryClient.invalidateQueries({ queryKey: queryKeys.taskExecutions.forTask(id) });
      queryClient.invalidateQueries({ queryKey: queryKeys.speedTests.all });
      queryClient.invalidateQueries({ queryKey: queryKeys.quotaResults.all });
    },
  }));
}

/**
 * Mutation hook for stopping a running task
 * Invalidates task queries on success
 */
export function useStopTaskMutation() {
  const queryClient = useQueryClient();

  return createMutation(() => ({
    mutationFn: (id: number) => tasksApi.stop(id),
    onSettled: (_result, _error, id) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.tasks.detail(id) });
      queryClient.invalidateQueries({ queryKey: queryKeys.tasks.lists() });
      queryClient.invalidateQueries({ queryKey: queryKeys.taskExecutions.forTask(id) });
    },
  }));
}

/**
 * Query hook for fetching task notification config
 */
export function useTaskNotificationQuery(taskId: number) {
  return createQuery(() => ({
    queryKey: queryKeys.taskNotifications.detail(taskId),
    queryFn: () => taskNotificationApi.get(taskId),
    enabled: taskId > 0,
  }));
}

/**
 * Mutation hook for toggling task notification enabled status
 * Quick toggle - only updates isEnabled flag, preserves other settings
 */
export function useToggleTaskNotificationMutation() {
  const queryClient = useQueryClient();

  return createMutation(() => ({
    mutationFn: async ({ taskId, isEnabled }: { taskId: number; isEnabled: boolean }) => {
      // Get current config or create default
      const currentConfig = await taskNotificationApi.get(taskId);
      const config: UpsertTaskNotificationConfig = currentConfig
        ? {
            isEnabled,
            smtpConfigId: currentConfig.smtpConfigId ?? undefined,
            emailSubject: currentConfig.emailSubject ?? undefined,
            toRecipientIds: currentConfig.toRecipientIds,
            ccRecipientIds: currentConfig.ccRecipientIds,
          }
        : {
            isEnabled,
            toRecipientIds: [],
            ccRecipientIds: [],
          };

      return taskNotificationApi.upsert(taskId, config);
    },
    onSuccess: (_result, { taskId }) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.taskNotifications.detail(taskId) });
    },
  }));
}
