import { invoke } from "@tauri-apps/api/core";
import type { Job, Execution, SchedulerInfo, DiscoveryStats } from "@models/index";

export async function getJobs(): Promise<Job[]> {
  return invoke("get_jobs");
}

export async function getJob(id: string): Promise<Job | null> {
  return invoke("get_job", { id });
}

export async function getExecutions(jobId: string): Promise<Execution[]> {
  return invoke("get_executions", { jobId });
}

export async function getSchedulers(): Promise<SchedulerInfo[]> {
  return invoke("get_schedulers");
}

export async function getDiscoveryStats(): Promise<DiscoveryStats> {
  return invoke("get_discovery_stats");
}

export async function triggerDiscovery(): Promise<void> {
  return invoke("trigger_discovery");
}