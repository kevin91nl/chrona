import { useJobs } from "@hooks/useJobs";
import { Card, CardContent, CardHeader, CardTitle } from "@components/ui/Card";
import { Badge } from "@components/ui/Badge";
import { formatSchedule } from "@/utils";
import { Clock } from "lucide-react";

export function Timeline() {
  const { jobs, loading } = useJobs();

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center">
        <Clock className="h-5 w-5 animate-spin text-muted-foreground" />
      </div>
    );
  }

  // Group jobs by hour (simplified)
  const sortedJobs = [...(jobs ?? [])].sort((a, b) =>
    (a.nextExecution ?? "").localeCompare(b.nextExecution ?? ""),
  );

  return (
    <div className="space-y-6 p-6">
      <div>
        <h1 className="text-2xl font-bold">Timeline</h1>
        <p className="text-muted-foreground">
          Chronological view of all scheduled jobs
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Schedule</CardTitle>
        </CardHeader>
        <CardContent>
          {sortedJobs.length ? (
            <div className="space-y-1">
              {sortedJobs.map((job) => (
                <div
                  key={job.id}
                  className="flex items-center justify-between rounded-md border p-3 transition-colors hover:bg-accent"
                >
                  <div className="flex items-center gap-4">
                    <div className="w-32 text-sm font-mono text-muted-foreground">
                      {formatSchedule(job.schedule)}
                    </div>
                    <div>
                      <p className="font-medium">{job.name}</p>
                      <p className="text-xs text-muted-foreground font-mono">
                        {job.command}
                      </p>
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <Badge variant="muted">{job.provider}</Badge>
                    <Badge
                      variant={
                        job.status === "error"
                          ? "error"
                          : job.enabled
                            ? "success"
                            : "muted"
                      }
                    >
                      {job.status === "error"
                        ? "Error"
                        : job.enabled
                          ? "Enabled"
                          : "Disabled"}
                    </Badge>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-sm text-muted-foreground">No jobs discovered</p>
          )}
        </CardContent>
      </Card>
    </div>
  );
}