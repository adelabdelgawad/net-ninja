import { createQuery, createMutation, useQueryClient, keepPreviousData } from '@tanstack/solid-query';
import { linesApi } from '~/api/client';
import { queryKeys } from '~/api/queryKeys';
import type { Line, LineCreate, LineUpdate } from '~/types';

/**
 * Query hook for fetching all lines
 *
 * Features:
 * - Stale-while-revalidate for instant navigation
 * - keepPreviousData to show cached data while fetching
 * - Error recovery with fallback empty array
 */
export function useLinesQuery() {
  return createQuery(() => ({
    queryKey: queryKeys.lines.list(),
    queryFn: async () => {
      try {
        return await linesApi.list();
      } catch (e) {
        console.warn('[useLinesQuery] Failed to load lines:', e);
        return [];
      }
    },
    placeholderData: keepPreviousData,
  }));
}

/**
 * Mutation hook for creating a line
 * Invalidates lines list cache on success
 */
export function useCreateLineMutation() {
  const queryClient = useQueryClient();

  return createMutation(() => ({
    mutationFn: (data: LineCreate) => linesApi.create(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.lines.all });
    },
  }));
}

/**
 * Mutation hook for updating a line
 * Invalidates both the specific line and list caches on success
 */
export function useUpdateLineMutation() {
  const queryClient = useQueryClient();

  return createMutation(() => ({
    mutationFn: ({ id, data }: { id: number; data: LineUpdate }) =>
      linesApi.update(id, data),
    onSuccess: (_result, variables) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.lines.detail(variables.id) });
      queryClient.invalidateQueries({ queryKey: queryKeys.lines.lists() });
    },
  }));
}

/**
 * Mutation hook for deleting a line
 * Uses optimistic update to remove from list immediately
 */
export function useDeleteLineMutation() {
  const queryClient = useQueryClient();

  return createMutation(() => ({
    mutationFn: (id: number) => linesApi.delete(id),
    onMutate: async (id) => {
      // Cancel outgoing refetches
      await queryClient.cancelQueries({ queryKey: queryKeys.lines.lists() });

      // Snapshot previous value
      const previousLines = queryClient.getQueryData<Line[]>(queryKeys.lines.list());

      // Optimistically remove the line
      if (previousLines) {
        queryClient.setQueryData<Line[]>(
          queryKeys.lines.list(),
          previousLines.filter((line) => line.id !== id)
        );
      }

      return { previousLines };
    },
    onError: (_err, _id, context) => {
      // Rollback on error
      if (context?.previousLines) {
        queryClient.setQueryData(queryKeys.lines.list(), context.previousLines);
      }
    },
    onSettled: () => {
      // Always refetch after error or success
      queryClient.invalidateQueries({ queryKey: queryKeys.lines.all });
    },
  }));
}

/**
 * Mutation hook for toggling line active status
 */
export function useToggleLineActiveMutation() {
  const queryClient = useQueryClient();

  return createMutation(() => ({
    mutationFn: ({ id, isActive }: { id: number; isActive: boolean }) =>
      linesApi.update(id, { isActive }),
    onMutate: async ({ id, isActive }) => {
      await queryClient.cancelQueries({ queryKey: queryKeys.lines.lists() });

      const previousLines = queryClient.getQueryData<Line[]>(queryKeys.lines.list());

      if (previousLines) {
        queryClient.setQueryData<Line[]>(
          queryKeys.lines.list(),
          previousLines.map((line) =>
            line.id === id ? { ...line, isActive } : line
          )
        );
      }

      return { previousLines };
    },
    onError: (_err, _variables, context) => {
      if (context?.previousLines) {
        queryClient.setQueryData(queryKeys.lines.list(), context.previousLines);
      }
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.lines.all });
    },
  }));
}
