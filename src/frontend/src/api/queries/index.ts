/**
 * Query hooks barrel export
 *
 * Re-exports all query hooks for convenient importing:
 * import { useSpeedTestsQuery, useLinesQuery } from '~/api/queries';
 */

// Speed Tests
export { useSpeedTestsQuery, useSpeedTestHistoryQuery } from './speedTests';

// Quota Results
export { useQuotaResultsQuery } from './quotaResults';

// Logs
export { useLogsQuery } from './logs';

// Lines
export {
  useLinesQuery,
  useCreateLineMutation,
  useUpdateLineMutation,
  useDeleteLineMutation,
  useToggleLineActiveMutation,
} from './lines';

// Tasks
export {
  useTasksQuery,
  useCreateTaskMutation,
  useUpdateTaskMutation,
  useDeleteTaskMutation,
  useToggleTaskActiveMutation,
  useExecuteTaskMutation,
  useStopTaskMutation,
  useTaskNotificationQuery,
  useToggleTaskNotificationMutation,
} from './tasks';

// Dashboard
export {
  useDashboardLinesQuery,
  useLatestReportQuery,
  useRecentExecutionsQuery,
  useScheduledTasksQuery,
} from './dashboard';
