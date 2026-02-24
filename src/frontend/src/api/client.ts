import type {
  Line,
  LineCreate,
  LineUpdate,
  QuotaResult,
  SpeedTestResult,
  Email,
  EmailCreate,
  EmailUpdate,
  Log,
  CombinedResult,
  HealthResponse,
  PaginatedResponse,
  ResultFilterParams,
  LogFilterParams,
  SmtpConfig,
  SmtpConfigCreate,
  SmtpConfigUpdate,
  SmtpConfigTestRequest,
  SmtpConfigTestResponse,
  FallbackStatusResponse,
  Task,
  CreateTaskRequest,
  UpdateTaskRequest,
  TaskExecutionResult,
  TaskExecutionResponse,
  ListExecutionsParams,
  TaskNotificationConfig,
  UpsertTaskNotificationConfig,
  RuntimeNotificationConfig,
  ResendNotificationRequest,
} from '../types';

import { invoke } from '@tauri-apps/api/core';

// Generic Tauri command wrapper with better error handling
async function tauriInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    const result = await invoke(command, args);
    return result as T;
  } catch (error) {
    console.error('[Tauri] Error for command', command, ':', error);
    throw new Error(`Tauri command '${command}' failed: ${error}`);
  }
}

// Health endpoints
export const healthApi = {
  check: () => tauriInvoke<HealthResponse>('health_check'),
  ready: () => tauriInvoke<HealthResponse>('health_check'),
};

// Fallback status endpoints
export const fallbackApi = {
  getStatus: () => tauriInvoke<FallbackStatusResponse>('get_fallback_status'),
};

// Lines endpoints
export const linesApi = {
  list: async () => {
    const response = await tauriInvoke<PaginatedResponse<Line>>('get_lines');
    return response.items;
  },
  get: (id: number) => tauriInvoke<Line>('get_line', { id }),
  create: (data: LineCreate) => tauriInvoke<Line>('create_line', { req: data }),
  update: (id: number, data: LineUpdate) => tauriInvoke<Line>('update_line', { id, req: data }),
  delete: (id: number) => tauriInvoke<void>('delete_line', { id }),
};

// Quota endpoints
export const quotaApi = {
  results: (params: ResultFilterParams = {}) =>
    tauriInvoke<PaginatedResponse<QuotaResult>>('get_quota_checks', {
      page: params.page,
      pageSize: params.pageSize,
    }),
  resultsForLine: (lineId: number, limit = 10) =>
    tauriInvoke<QuotaResult[]>('get_quota_results_for_line', { line_id: lineId, limit }),
  run: () => tauriInvoke<QuotaResult>('create_quota_check', {}),
};

// Speed test endpoints
export const speedTestApi = {
  results: (params: ResultFilterParams = {}) =>
    tauriInvoke<PaginatedResponse<SpeedTestResult>>('get_speed_tests', {
      page: params.page,
      pageSize: params.pageSize,
    }),
  resultsForLine: (lineId: number, limit = 10) =>
    tauriInvoke<SpeedTestResult[]>('get_speed_tests_for_line', { line_id: lineId, limit }),
  run: () => tauriInvoke<SpeedTestResult>('create_speed_test', {}),
};

// Email endpoints
export const emailsApi = {
  list: () => tauriInvoke<Email[]>('get_emails'),
  create: (data: EmailCreate) => tauriInvoke<Email>('create_email', { req: data }),
  update: (id: number, data: EmailUpdate) =>
    tauriInvoke<Email>('update_email', { id, req: data }),
  delete: (id: number) => tauriInvoke<void>('delete_email', { id }),
};

// Reports endpoints
export const reportsApi = {
  latest: () => tauriInvoke<CombinedResult[]>('get_latest_report'),
};

// Logs endpoints
export const logsApi = {
  list: (params: LogFilterParams = {}) =>
    tauriInvoke<PaginatedResponse<Log>>('get_logs', {
      page: params.page,
      pageSize: params.pageSize,
    }),
  byProcess: (processId: string) =>
    tauriInvoke<Log[]>('get_logs_by_process', { processId }),
};

// SMTP Config endpoints
export const smtpConfigsApi = {
  list: () => tauriInvoke<SmtpConfig[]>('get_smtp_configs'),
  get: (id: number) => tauriInvoke<SmtpConfig>('get_smtp_config', { id }),
  getDefault: () => tauriInvoke<SmtpConfig>('get_default_smtp_config', {}),
  create: (data: SmtpConfigCreate) => tauriInvoke<SmtpConfig>('create_smtp_config', { req: data }),
  update: (id: number, data: SmtpConfigUpdate) =>
    tauriInvoke<SmtpConfig>('update_smtp_config', { id, req: data }),
  delete: (id: number) => tauriInvoke<void>('delete_smtp_config', { id }),
  setDefault: (id: number) => tauriInvoke<SmtpConfig>('set_default_smtp_config', { id }),
  test: (data: SmtpConfigTestRequest) =>
    tauriInvoke<SmtpConfigTestResponse>('test_smtp_config_inline', { req: data }),
  testExisting: (id: number, testRecipient: string) =>
    tauriInvoke<SmtpConfigTestResponse>('test_smtp_config', { id, testRecipient }),
};

// Tasks endpoints
export const tasksApi = {
  list: () => tauriInvoke<Task[]>('get_tasks'),
  get: (id: number) => tauriInvoke<Task>('get_task', { id }),
  create: (data: CreateTaskRequest) => tauriInvoke<Task>('create_task', { req: data }),
  update: (id: number, data: UpdateTaskRequest) => tauriInvoke<Task>('update_task', { id, req: data }),
  delete: (id: number) => tauriInvoke<void>('delete_task', { id }),
  toggleActive: (id: number, isActive: boolean) => tauriInvoke<Task>('toggle_task_active', { id, isActive }),
  execute: (id: number, notificationOverride?: RuntimeNotificationConfig) =>
    tauriInvoke<TaskExecutionResult>('execute_task', {
      id,
      notificationOverride: notificationOverride || null,
    }),
  stop: (id: number) => tauriInvoke<void>('stop_task', { id }),
  checkNameAvailable: (name: string) => tauriInvoke<boolean>('check_task_name_available', { name }),
};

// Task execution history endpoints
export const taskExecutionsApi = {
  list: (params?: ListExecutionsParams) =>
    tauriInvoke<TaskExecutionResponse[]>('get_executions', { params }),
  get: (id: number) => tauriInvoke<TaskExecutionResponse>('get_execution', { id }),
  getByTaskId: (taskId: number, limit?: number) =>
    tauriInvoke<TaskExecutionResponse[]>('get_task_executions', { taskId, limit }),
  getLatestForTask: (taskId: number) =>
    tauriInvoke<TaskExecutionResponse | null>('get_latest_task_execution', { taskId }),
  count: (params?: ListExecutionsParams) =>
    tauriInvoke<number>('count_executions', { params }),
};

// Task notification config endpoints
export const taskNotificationApi = {
  get: (taskId: number) =>
    tauriInvoke<TaskNotificationConfig | null>('get_task_notification_config', { taskId }),
  upsert: (taskId: number, data: UpsertTaskNotificationConfig) =>
    tauriInvoke<TaskNotificationConfig>('upsert_task_notification_config', { taskId, req: data }),
  resend: (data: ResendNotificationRequest) =>
    tauriInvoke<void>('resend_task_notification', { req: data }),
};

// App endpoints
export const appApi = {
  getDatabasePath: () => tauriInvoke<string>('get_database_path'),
  getLogsPath: () => tauriInvoke<string>('get_logs_path'),
};
