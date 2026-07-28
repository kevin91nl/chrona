import { useState } from "react";
import { useFilteredJobs as useJobs } from "@hooks/useJobs";
import { Card, CardContent } from "@components/ui/Card";

const HOURS = Array.from({ length: 24 }, (_, i) => i);

function parseCronHour(schedule: string): number | null {
  // "0 2 * * *" -> 2, "0 */6 * * *" -> null (multiple), "*/30 * * * *" -> null
  const parts = schedule.split(" ");
  if (parts.length < 2) return null;
  const hour = parts[1];
  if (hour === "*") return null;
  if (hour.includes("/")) return null;
  const h = parseInt(hour, 10);
  return isNaN(h) ? null : h;
}

function getJobColor(provider: string): string {
  switch (provider) {
    case "cron":
      return "bg-blue-500/20 text-blue-300 border-blue-500/30";
    case "launchd":
      return "bg-green-500/20 text-green-300 border-green-500/30";
    case "codex":
      return "bg-purple-500/20 text-purple-300 border-purple-500/30";
    case "systemd":
      return "bg-orange-500/20 text-orange-300 border-orange-500/30";
    default:
      return "bg-muted text-muted-foreground border-border";
  }
}

export function CalendarView() {
  const { jobs, loading } = useJobs();
  const [selectedHour, setSelectedHour] = useState<number | null>(null);

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center">
        <p className="text-muted-foreground">Loading calendar...</p>
      </div>
    );
  }

  // Group jobs by hour
  const jobsByHour: Record<number, typeof jobs> = {};
  for (const job of jobs ?? []) {
    const hour = parseCronHour(job.schedule);
    if (hour !== null) {
      if (!jobsByHour[hour]) jobsByHour[hour] = [];
      jobsByHour[hour].push(job);
    }
  }

  return (
    <div className="space-y-6 p-6">
      <div>
        <h1 className="text-2xl font-bold">Calendar</h1>
        <p className="text-muted-foreground">
          Visual schedule overview — when does each job run?
        </p>
      </div>

      <div className="grid grid-cols-[80px_1fr] gap-0 rounded-lg border overflow-hidden">
        {/* Header */}
        <div className="bg-muted/50 p-2 text-xs font-medium text-muted-foreground border-b border-r">
          Time
        </div>
        <div className="bg-muted/50 p-2 text-xs font-medium text-muted-foreground border-b">
          Jobs
        </div>

        {/* Hour rows */}
        {HOURS.map((hour) => {
          const hourJobs = jobsByHour[hour] || [];
          const hasJobs = hourJobs.length > 0;
          const isSelected = selectedHour === hour;

          return (
            <div
              key={hour}
              className={`contents ${isSelected ? "[&>*]:bg-accent/50" : ""}`}
            >
              <div
                className={`border-b border-r p-2 text-xs font-mono cursor-pointer hover:bg-accent/30 transition-colors ${
                  hasJobs ? "font-semibold text-foreground" : "text-muted-foreground"
                }`}
                onClick={() => setSelectedHour(isSelected ? null : hour)}
              >
                {String(hour).padStart(2, "0")}:00
              </div>
              <div className="border-b p-2 min-h-[36px] flex flex-wrap gap-1 items-center">
                {hasJobs ? (
                  hourJobs.map((job) => (
                    <span
                      key={job.id}
                      className={`inline-flex items-center rounded border px-2 py-0.5 text-xs ${getJobColor(job.provider)}`}
                    >
                      {job.name}
                    </span>
                  ))
                ) : (
                  <span className="text-xs text-muted-foreground/30">—</span>
                )}
              </div>
            </div>
          );
        })}
      </div>

      {/* Legend */}
      <Card>
        <CardContent className="p-4">
          <div className="flex flex-wrap gap-4 text-xs">
            <div className="flex items-center gap-2">
              <div className="h-3 w-3 rounded bg-blue-500/20 border border-blue-500/30" />
              <span className="text-muted-foreground">cron</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="h-3 w-3 rounded bg-green-500/20 border border-green-500/30" />
              <span className="text-muted-foreground">launchd</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="h-3 w-3 rounded bg-purple-500/20 border border-purple-500/30" />
              <span className="text-muted-foreground">codex</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="h-3 w-3 rounded bg-orange-500/20 border border-orange-500/30" />
              <span className="text-muted-foreground">systemd</span>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}