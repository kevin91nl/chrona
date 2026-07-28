export type View = "dashboard" | "timeline" | "jobs" | "calendar" | "system";

export type JobStatus = "active" | "inactive" | "error" | "running" | "unknown";
export type ExecutionStatus = "success" | "failure" | "running" | "skipped" | "timeout";

export interface Job {
  id: string;
  name: string;
  provider: string;
  source: string;
  schedule: string;
  timezone: string | null;
  command: string;
  workingDirectory: string | null;
  status: JobStatus;
  enabled: boolean;
  discoveredAt: string;
  updatedAt: string;
  nextExecution: string | null;
  previousExecution: string | null;
}

export interface Execution {
  id: string;
  jobId: string;
  startTime: string;
  endTime: string | null;
  duration: number | null;
  status: ExecutionStatus;
  exitCode: number | null;
  logs: string | null;
}

export interface SchedulerInfo {
  id: string;
  name: string;
  available: boolean;
  jobCount: number;
}

export interface DiscoveryStats {
  totalJobs: number;
  activeJobs: number;
  failedJobs: number;
  schedulersDetected: number;
  nextScheduled: { job: Job; time: string } | null;
  recentExecutions: Execution[];
}