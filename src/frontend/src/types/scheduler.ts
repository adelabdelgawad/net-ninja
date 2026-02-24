// Job and Scheduler types

export type JobType = 'speed_test' | 'quota_check' | 'full_check' | 'retry_failed';
export type JobStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';

export interface Job {
  id: string;
  jobType: JobType;
  status: JobStatus;
  description: string | null;
  progress: number | null;
  createdAt: string;
  startedAt: string | null;
  completedAt: string | null;
}

export interface JobDetail extends Job {
  lineIds: number[] | null;
  result: string | null;
  error: string | null;
}

export interface SchedulerStatusResponse {
  status: string;
  isRunning: boolean;
}
