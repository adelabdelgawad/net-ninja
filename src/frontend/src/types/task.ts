// Task-specific types for the Task System feature

export interface Schedule {
  days: number[];   // 0-6 for Sunday-Saturday
  times: string[];  // "HH:MM" format
}

export interface Task {
  id: number;
  name: string;
  taskTypes: TaskType[];  // CHANGED: was taskType
  runMode: RunMode;
  schedule: Schedule | null;
  status: TaskStatus;
  isActive: boolean;  // NEW
  showBrowser: boolean;  // Show browser window during quota check (false = invisible/headless)
  lineIds: number[];
  lines: Array<{
    id: number;
    name: string;
    lineNumber: string;
    ipAddress: string;
  }>;
  lastRunAt: string | null;
  nextRunAt: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface CreateTaskRequest {
  name: string;
  taskTypes: TaskType[];  // CHANGED: was taskType
  runMode: RunMode;
  schedule?: Schedule;
  lineIds: number[];
  showBrowser?: boolean;  // Show browser window during quota check (default: false)
}

export interface UpdateTaskRequest {
  name?: string;
  taskTypes?: TaskType[];
  runMode?: RunMode;
  schedule?: Schedule;
  lineIds?: number[];
  showBrowser?: boolean;  // Show browser window during quota check
}

// Execution result types
export interface LineExecutionResult {
  lineId: number;
  lineName: string;
  taskType: TaskType;
  status: 'success' | 'failed';
  errorMessage: string | null;
  durationMs: number;
  startedAt: string;
  completedAt: string;
}

export interface TaskTypeResults {
  speedTest?: LineExecutionResult[];
  quotaCheck?: LineExecutionResult[];
}

export interface TaskExecutionResult {
  taskId: number;
  taskName: string;
  status: TaskStatus;
  results: TaskTypeResults;
  startedAt: string;
  finishedAt: string | null;
}

export type TaskType = 'speed_test' | 'quota_check';
export type RunMode = 'one_time' | 'scheduled';
export type TaskStatus = 'pending' | 'running' | 'completed' | 'failed';  // CHANGED: was 'active', now 'running'
export type ExecutionTrigger = 'manual' | 'scheduler';

// Task execution history types
export interface ExecutionResultSummary {
  totalLines: number;
  successCount: number;
  failureCount: number;
  speedTestCount?: number;
  quotaCheckCount?: number;
}

export interface TaskExecutionLineResultResponse {
  id: number;
  executionId: string;
  lineId: number;
  lineName: string;
  taskType: TaskType;
  status: 'success' | 'failed';
  errorMessage: string | null;
  durationMs: number | null;
  startedAt: string | null;
  completedAt: string | null;
}

export interface TaskExecutionResponse {
  id: number;
  taskId: number;
  taskName: string;
  executionId: string;
  triggeredBy: ExecutionTrigger;
  scheduledFor: string | null;
  startedAt: string;
  completedAt: string | null;
  status: 'running' | 'completed' | 'failed';
  errorMessage: string | null;
  durationMs: number | null;
  resultSummary: ExecutionResultSummary | null;
  lineResults: TaskExecutionLineResultResponse[];
}

export interface ListExecutionsParams {
  taskId?: number;
  status?: 'running' | 'completed' | 'failed';
  triggeredBy?: ExecutionTrigger;
  limit?: number;
  offset?: number;
}

// Task notification config types
export interface TaskNotificationConfig {
  id: number;
  taskId: number;
  isEnabled: boolean;
  smtpConfigId: number | null;
  emailSubject: string | null;
  toRecipientIds: number[];
  ccRecipientIds: number[];
  createdAt: string;
  updatedAt: string;
}

export interface UpsertTaskNotificationConfig {
  isEnabled: boolean;
  smtpConfigId?: number;
  emailSubject?: string;
  toRecipientIds: number[];
  ccRecipientIds: number[];
}

export interface RuntimeNotificationConfig {
  isEnabled: boolean;
  smtpConfigId?: number;
  emailSubject?: string;
  toRecipientIds: number[];
  ccRecipientIds: number[];
}

export interface ResendNotificationRequest {
  executionId: string;
  smtpConfigId: number;
  emailSubject: string;
  toRecipientIds: number[];
  ccRecipientIds: number[];
}
