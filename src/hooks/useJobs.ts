import { useState, useEffect, useMemo } from "react";
import { getJobs, getSchedulers, getDiscoveryStats } from "@/tauri";
import { useFilters } from "@contexts/FilterContext";
import type { Job, SchedulerInfo, DiscoveryStats } from "@models/index";

export function useJobs(refreshKey?: number) {
  const [jobs, setJobs] = useState<Job[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const load = async () => {
      try {
        const data = await getJobs();
        setJobs(data);
      } catch {
        setJobs(DEMO_JOBS);
      } finally {
        setLoading(false);
      }
    };

    load();
    const interval = setInterval(load, 30000);
    return () => clearInterval(interval);
  }, [refreshKey]);

  return { jobs, loading };
}

export function useFilteredJobs(refreshKey?: number) {
  const { jobs, loading } = useJobs(refreshKey);
  const { isProviderEnabled } = useFilters();

  const filtered = useMemo(
    () => (jobs ?? []).filter((j) => isProviderEnabled(j.provider)),
    [jobs, isProviderEnabled],
  );

  return { jobs: filtered, loading };
}

export function useSchedulers() {
  const [schedulers, setSchedulers] = useState<SchedulerInfo[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const load = async () => {
      try {
        const data = await getSchedulers();
        setSchedulers(data);
      } catch {
        setSchedulers(DEMO_SCHEDULERS);
      } finally {
        setLoading(false);
      }
    };

    load();
  }, []);

  return { schedulers, loading };
}

export function useDiscoveryStats() {
  const [stats, setStats] = useState<DiscoveryStats | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const load = async () => {
      try {
        const data = await getDiscoveryStats();
        setStats(data);
      } catch {
        setStats(DEMO_STATS);
      } finally {
        setLoading(false);
      }
    };

    load();
    const interval = setInterval(load, 30000);
    return () => clearInterval(interval);
  }, []);

  return { stats, loading };
}

// Demo data for development without Tauri backend
const DEMO_JOBS: Job[] = [
  {
    id: "cron-daily-backup",
    name: "Daily Backup",
    provider: "cron",
    source: "/etc/crontab",
    schedule: "0 2 * * *",
    timezone: "UTC",
    command: "/usr/local/bin/backup.sh",
    workingDirectory: "/home/user",
    status: "active",
    enabled: true,
    discoveredAt: "2026-07-28T08:00:00Z",
    updatedAt: "2026-07-28T08:00:00Z",
    nextExecution: "2026-07-29T02:00:00Z",
    previousExecution: "2026-07-28T02:00:00Z",
  },
  {
    id: "launchd-git-sync",
    name: "Git Sync",
    provider: "launchd",
    source: "~/Library/LaunchAgents/com.user.gitsync.plist",
    schedule: "0 */6 * * *",
    timezone: null,
    command: "/usr/local/bin/git-sync.sh",
    workingDirectory: "/Users/user/projects",
    status: "active",
    enabled: true,
    discoveredAt: "2026-07-28T08:00:00Z",
    updatedAt: "2026-07-28T08:00:00Z",
    nextExecution: "2026-07-28T12:00:00Z",
    previousExecution: "2026-07-28T06:00:00Z",
  },
  {
    id: "cron-log-rotation",
    name: "Log Rotation",
    provider: "cron",
    source: "/etc/cron.daily/logrotate",
    schedule: "0 0 * * *",
    timezone: "UTC",
    command: "/usr/sbin/logrotate /etc/logrotate.conf",
    workingDirectory: null,
    status: "active",
    enabled: true,
    discoveredAt: "2026-07-28T08:00:00Z",
    updatedAt: "2026-07-28T08:00:00Z",
    nextExecution: "2026-07-29T00:00:00Z",
    previousExecution: "2026-07-28T00:00:00Z",
  },
  {
    id: "cron-cleanup",
    name: "Temp Cleanup",
    provider: "cron",
    source: "/var/spool/cron/crontabs/user",
    schedule: "*/30 * * * *",
    timezone: "UTC",
    command: "find /tmp -type f -mtime +1 -delete",
    workingDirectory: null,
    status: "active",
    enabled: true,
    discoveredAt: "2026-07-28T08:00:00Z",
    updatedAt: "2026-07-28T08:00:00Z",
    nextExecution: "2026-07-28T09:00:00Z",
    previousExecution: "2026-07-28T08:30:00Z",
  },
  {
    id: "codex-hands-off-data-quality-repair",
    name: "Hands-off data-quality repair",
    provider: "codex",
    source: "~/.codex/automations/hands-off-data-quality-repair/automation.toml",
    schedule: "Every 15 minutes",
    timezone: null,
    command: "codex automation: hands-off-data-quality-repair [gpt-5.6-sol]",
    workingDirectory: "/Users/kevin/projects/riskstudio/riskstudio-worker",
    status: "active",
    enabled: true,
    discoveredAt: "2026-07-28T08:00:00Z",
    updatedAt: "2026-07-28T12:03:00Z",
    nextExecution: "2026-07-28T13:15:00Z",
    previousExecution: "2026-07-28T13:00:00Z",
  },
  {
    id: "cron-failing-job",
    name: "DB Health Check",
    provider: "cron",
    source: "/var/spool/cron/crontabs/user",
    schedule: "0 * * * *",
    timezone: "UTC",
    command: "/opt/scripts/db-health.sh",
    workingDirectory: "/opt/scripts",
    status: "error",
    enabled: true,
    discoveredAt: "2026-07-28T08:00:00Z",
    updatedAt: "2026-07-28T08:00:00Z",
    nextExecution: "2026-07-28T10:00:00Z",
    previousExecution: "2026-07-28T09:00:00Z",
  },
];

const DEMO_SCHEDULERS: SchedulerInfo[] = [
  { id: "cron", name: "cron", available: true, jobCount: 4 },
  { id: "launchd", name: "launchd", available: true, jobCount: 1 },
  { id: "codex", name: "Codex Scheduled Tasks", available: true, jobCount: 1 },
  { id: "systemd", name: "systemd timers", available: false, jobCount: 0 },
  { id: "windows-task", name: "Windows Task Scheduler", available: false, jobCount: 0 },
];

const DEMO_STATS: DiscoveryStats = {
  totalJobs: 6,
  activeJobs: 5,
  failedJobs: 1,
  schedulersDetected: 3,
  nextScheduled: {
    job: DEMO_JOBS[3],
    time: "Today at 09:00",
  },
  recentExecutions: [],
};