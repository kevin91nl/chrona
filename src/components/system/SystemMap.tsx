import { useSchedulers, useFilteredJobs as useJobs } from "@hooks/useJobs";
import { useFilters } from "@contexts/FilterContext";
import { Card, CardContent, CardHeader, CardTitle } from "@components/ui/Card";
import { Badge } from "@components/ui/Badge";
import { formatRelativeTime } from "@/utils";

function providerColor(id: string): string {
  switch (id) {
    case "cron":
      return "border-blue-500 bg-blue-500/10";
    case "launchd":
      return "border-green-500 bg-green-500/10";
    case "codex":
      return "border-purple-500 bg-purple-500/10";
    case "systemd":
      return "border-orange-500 bg-orange-500/10";
    case "windows-task":
      return "border-cyan-500 bg-cyan-500/10";
    default:
      return "border-muted bg-muted/10";
  }
}

function providerIcon(id: string): string {
  switch (id) {
    case "cron":
      return "⏱";
    case "launchd":
      return "🍎";
    case "codex":
      return "🤖";
    case "systemd":
      return "🐧";
    case "windows-task":
      return "🪟";
    default:
      return "⚙️";
  }
}

export function SystemMap() {
  const { isProviderEnabled } = useFilters();
  const { schedulers, loading: schedulersLoading } = useSchedulers();
  const { jobs, loading: jobsLoading } = useJobs();

  const filteredSchedulers = (schedulers ?? []).filter((s) => isProviderEnabled(s.id));

  if (schedulersLoading || jobsLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <p className="text-muted-foreground">Loading system map...</p>
      </div>
    );
  }

  return (
    <div className="space-y-6 p-6">
      <div>
        <h1 className="text-2xl font-bold">System Map</h1>
        <p className="text-muted-foreground">
          Visualize schedulers, jobs, and their relationships
        </p>
      </div>

      <div className="grid gap-6">
        {filteredSchedulers.map((scheduler) => {
          const schedulerJobs = (jobs ?? []).filter(
            (j) => j.provider === scheduler.id,
          );

          return (
            <Card key={scheduler.id} className={providerColor(scheduler.id)}>
              <CardHeader className="pb-3">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <span className="text-2xl">
                      {providerIcon(scheduler.id)}
                    </span>
                    <div>
                      <CardTitle className="text-lg">
                        {scheduler.name}
                      </CardTitle>
                      <p className="text-xs text-muted-foreground font-mono">
                        {scheduler.id}
                      </p>
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <Badge variant={scheduler.available ? "success" : "muted"}>
                      {scheduler.available ? "Active" : "Unavailable"}
                    </Badge>
                    <span className="text-sm text-muted-foreground">
                      {schedulerJobs.length} jobs
                    </span>
                  </div>
                </div>
              </CardHeader>
              <CardContent>
                {schedulerJobs.length > 0 ? (
                  <div className="grid grid-cols-2 gap-2 sm:grid-cols-3 lg:grid-cols-4">
                    {schedulerJobs.map((job) => (
                      <div
                        key={job.id}
                        className="rounded border bg-card/50 p-3 space-y-1"
                      >
                        <div className="flex items-center gap-2">
                          <div
                            className={`h-2 w-2 rounded-full ${
                              job.status === "error"
                                ? "bg-red-500"
                                : job.enabled
                                  ? "bg-green-500"
                                  : "bg-muted-foreground"
                            }`}
                          />
                          <p className="text-sm font-medium truncate">
                            {job.name}
                          </p>
                        </div>
                        <p className="text-xs font-mono text-muted-foreground truncate">
                          {job.command}
                        </p>
                        <p className="text-xs text-muted-foreground">
                          {job.schedule}
                        </p>
                        {job.nextExecution && (
                          <p className="text-xs text-primary">
                            Next: {formatRelativeTime(job.nextExecution)}
                          </p>
                        )}
                        {!job.enabled && (
                          <p className="text-xs text-yellow-400">Paused</p>
                        )}
                      </div>
                    ))}
                  </div>
                ) : (
                  <p className="text-sm text-muted-foreground">
                    {scheduler.available
                      ? "No jobs discovered"
                      : "Scheduler not available on this OS"}
                  </p>
                )}
              </CardContent>
            </Card>
          );
        })}
      </div>
    </div>
  );
}