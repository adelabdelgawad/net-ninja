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
  /** "running" (a fresh scheduler lock is held), "idle", or "unknown" (fallback mode). */
  status: string;
  isRunning: boolean;
  /** Who holds the scheduler lock: "service", "desktop", or null. */
  lockHolder: string | null;
  /** ISO timestamp of the last scheduler heartbeat. */
  lastHeartbeat: string | null;
  /** Map of job name ("quota_check" | "speed_test" | "cleanup") -> ISO timestamp of last success. */
  lastRuns: Record<string, string>;
}

export interface ServiceStatusResponse {
  /** Whether the Windows service is registered with the SCM. */
  installed: boolean;
  /** Whether the Windows service is currently running. */
  running: boolean;
  /** Reported scheduler/service version, if known. */
  version: string | null;
  /** ISO timestamp of the last scheduler heartbeat. */
  lastHeartbeat: string | null;
  /** Who holds the scheduler lock: "service", "desktop", or null. */
  lockHolder: string | null;
}
