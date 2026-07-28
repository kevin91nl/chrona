import { useFilteredJobs as useJobs } from "@hooks/useJobs";
import { Card, CardContent, CardHeader, CardTitle } from "@components/ui/Card";
import { Badge } from "@components/ui/Badge";
import { formatSchedule, formatRelativeTime } from "@/utils";
import { Clock, AlertTriangle } from "lucide-react";

function parseCronHour(schedule: string): number | null {
  const parts = schedule.split(" ");
  if (parts.length < 2) return null;
  const hour = parts[1];
  if (hour === "*") return null;
  if (hour.includes("/")) return null;
  const h = parseInt(hour, 10);
  return isNaN(h) ? null : h;
}

function providerDotColor(provider: string): string {
  switch (provider) {
    case "cron":
      return "bg-blue-500";
    case "launchd":
      return "bg-green-500";
    case "codex":
      return "bg-purple-500";
    case "systemd":
      return "bg-orange-500";
    default:
      return "bg-muted-foreground";
  }
}

export function Timeline() {
  const { jobs, loading } = useJobs();

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center">
        <Clock className="h-5 w-5 animate-spin text-muted-foreground" />
      </div>
    );
  }

  // Sort by next execution time, then by schedule
  const sortedJobs = [...(jobs ?? [])].sort((a, b) => {
    // Errors first
    if (a.status === "error" && b.status !== "error") return -1;
    if (b.status === "error" && a.status !== "error") return 1;
    // Then by next execution
    if (a.nextExecution && b.nextExecution) {
      return a.nextExecution.localeCompare(b.nextExecution);
    }
    if (a.nextExecution) return -1;
    if (b.nextExecution) return 1;
    return a.name.localeCompare(b.name);
  });

  const errorJobs = sortedJobs.filter((j) => j.status === "error");
  const normalJobs = sortedJobs.filter((j) => j.status !== "error");

  return (
    <div className="space-y-6 p-6">
      <div>
        <h1 className="text-2xl font-bold">Timeline</h1>
        <p className="text-muted-foreground">
          Chronological view of all scheduled jobs
        </p>
      </div>

      {/* Errors banner */}
      {errorJobs.length > 0 && (
        <Card className="border-red-500/30 bg-red-500/5">
          <CardContent className="p-4">
            <div className="flex items-center gap-2 mb-2">
              <AlertTriangle className="h-4 w-4 text-red-400" />
              <span className="text-sm font-medium text-red-400">
                {errorJobs.length} job{errorJobs.length > 1 ? "s" : ""} with
                errors
              </span>
            </div>
            <div className="space-y-2">
              {errorJobs.map((job) => (
                <div
                  key={job.id}
                  className="flex items-center gap-3 rounded border border-red-500/20 bg-red-500/10 p-2"
                >
                  <div className="h-2 w-2 rounded-full bg-red-500" />
                  <span className="font-medium text-sm">{job.name}</span>
                  <span className="text-xs text-muted-foreground font-mono">
                    {job.source}
                  </span>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Timeline */}
      <Card>
        <CardHeader>
          <CardTitle>Schedule</CardTitle>
        </CardHeader>
        <CardContent>
          {normalJobs.length ? (
            <div className="relative">
              {/* Timeline line */}
              <div className="absolute left-[19px] top-0 bottom-0 w-px bg-border" />

              <div className="space-y-0">
                {normalJobs.map((job) => {
                  const hour = parseCronHour(job.schedule);
                  const timeLabel =
                    hour !== null
                      ? `${String(hour).padStart(2, "0")}:00`
                      : formatSchedule(job.schedule);

                  return (
                    <div
                      key={job.id}
                      className="relative flex items-start gap-4 py-3 pl-0 transition-colors hover:bg-accent/30 rounded-md -mx-2 px-2"
                    >
                      {/* Timeline dot */}
                      <div className="relative z-10 flex h-10 w-10 shrink-0 items-center justify-center">
                        <div
                          className={`h-3 w-3 rounded-full border-2 border-background ${providerDotColor(job.provider)}`}
                        />
                      </div>

                      {/* Time */}
                      <div className="w-16 shrink-0 pt-2">
                        <span className="text-sm font-mono font-semibold text-foreground">
                          {timeLabel}
                        </span>
                      </div>

                      {/* Content */}
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2">
                          <p className="font-medium truncate">{job.name}</p>
                          <Badge variant="muted">{job.provider}</Badge>
                          {!job.enabled && (
                            <Badge variant="warning">Disabled</Badge>
                          )}
                        </div>
                        <p className="text-xs text-muted-foreground font-mono truncate mt-0.5">
                          {job.command}
                        </p>
                        {job.source && (
                          <p className="text-xs text-muted-foreground mt-1">
                            {job.source}
                          </p>
                        )}
                      </div>

                      {/* Next run */}
                      <div className="shrink-0 text-right pt-2">
                        {job.nextExecution ? (
                          <span className="text-xs text-primary">
                            {formatRelativeTime(job.nextExecution)}
                          </span>
                        ) : (
                          <span className="text-xs text-muted-foreground">
                            {formatSchedule(job.schedule)}
                          </span>
                        )}
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          ) : (
            <p className="text-sm text-muted-foreground">No jobs discovered</p>
          )}
        </CardContent>
      </Card>
    </div>
  );
}