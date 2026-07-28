import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatRelativeTime(date: string | Date): string {
  const now = new Date();
  const target = new Date(date);
  const diffMs = target.getTime() - now.getTime();
  const absDiff = Math.abs(diffMs);
  const isPast = diffMs < 0;

  const minutes = Math.floor(absDiff / 60000);
  const hours = Math.floor(absDiff / 3600000);
  const days = Math.floor(absDiff / 86400000);

  if (minutes < 1) return isPast ? "just now" : "in <1 min";
  if (minutes < 60) return isPast ? `${minutes}m ago` : `in ${minutes}m`;
  if (hours < 24) return isPast ? `${hours}h ago` : `in ${hours}h`;
  return isPast ? `${days}d ago` : `in ${days}d`;
}

export function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
  const minutes = Math.floor(ms / 60000);
  const seconds = Math.floor((ms % 60000) / 1000);
  return `${minutes}m ${seconds}s`;
}

export function formatSchedule(schedule: string): string {
  // Human-readable cron description (simplified)
  if (schedule === "* * * * *") return "Every minute";
  if (schedule === "0 * * * *") return "Every hour";
  if (schedule === "0 0 * * *") return "Daily at midnight";
  if (schedule.startsWith("*/")) {
    const interval = schedule.split(" ")[0].slice(2);
    return `Every ${interval} minutes`;
  }
  return schedule;
}