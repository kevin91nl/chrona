import { useDiscoveryStats, useSchedulers } from "@hooks/useJobs";
import { Card, CardContent, CardHeader, CardTitle } from "@components/ui/Card";
import { StatCard } from "@components/ui/StatCard";
import { Badge } from "@components/ui/Badge";
import { Activity, Clock, AlertTriangle, Server, Zap } from "lucide-react";

export function Dashboard() {
  const { stats, loading: statsLoading } = useDiscoveryStats();
  const { schedulers, loading: schedulersLoading } = useSchedulers();

  if (statsLoading || schedulersLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="flex items-center gap-2 text-muted-foreground">
          <Activity className="h-5 w-5 animate-spin" />
          <span>Scanning schedulers...</span>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6 p-6">
      <div>
        <h1 className="text-2xl font-bold">Dashboard</h1>
        <p className="text-muted-foreground">
          Automatic scheduling overview for this machine
        </p>
      </div>

      {/* Stats grid */}
      <div className="grid grid-cols-4 gap-4">
        <StatCard
          icon={<Zap className="h-5 w-5" />}
          label="Total Jobs"
          value={stats?.totalJobs ?? 0}
          description="Discovered"
        />
        <StatCard
          icon={<Server className="h-5 w-5" />}
          label="Schedulers"
          value={stats?.schedulersDetected ?? 0}
          description="Detected"
        />
        <StatCard
          icon={<Clock className="h-5 w-5" />}
          label="Running Today"
          value={stats?.activeJobs ?? 0}
          description="Scheduled"
        />
        <StatCard
          icon={<AlertTriangle className="h-5 w-5" />}
          label="Failures"
          value={stats?.failedJobs ?? 0}
          description="Recent"
        />
      </div>

      <div className="grid grid-cols-2 gap-6">
        {/* Detected schedulers */}
        <Card>
          <CardHeader>
            <CardTitle>Detected Schedulers</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {schedulers?.map((s) => (
                <div
                  key={s.id}
                  className="flex items-center justify-between rounded-md border p-3"
                >
                  <div className="flex items-center gap-3">
                    <div
                      className={`h-2 w-2 rounded-full ${
                        s.available ? "bg-green-500" : "bg-muted-foreground"
                      }`}
                    />
                    <span className="font-medium">{s.name}</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="text-sm text-muted-foreground">
                      {s.jobCount} jobs
                    </span>
                    <Badge variant={s.available ? "success" : "muted"}>
                      {s.available ? "Active" : "Unavailable"}
                    </Badge>
                  </div>
                </div>
              ))}
              {!schedulers?.length && (
                <p className="text-sm text-muted-foreground">
                  No schedulers detected
                </p>
              )}
            </div>
          </CardContent>
        </Card>

        {/* Next scheduled */}
        <Card>
          <CardHeader>
            <CardTitle>Next Scheduled</CardTitle>
          </CardHeader>
          <CardContent>
            {stats?.nextScheduled ? (
              <div className="rounded-md border p-4">
                <p className="font-medium">{stats.nextScheduled.job.name}</p>
                <p className="text-sm text-muted-foreground">
                  {stats.nextScheduled.job.command}
                </p>
                <p className="mt-2 text-sm text-primary">
                  {stats.nextScheduled.time}
                </p>
              </div>
            ) : (
              <p className="text-sm text-muted-foreground">
                No upcoming executions
              </p>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}