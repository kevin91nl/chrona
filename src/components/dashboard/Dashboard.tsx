import { useMemo, useState } from "react";
import { useSchedulers, useFilteredJobs } from "@hooks/useJobs";
import { useFilters } from "@contexts/FilterContext";
import { triggerDiscovery } from "@/tauri";
import { Card, CardContent, CardHeader, CardTitle } from "@components/ui/Card";
import { StatCard } from "@components/ui/StatCard";
import { Badge } from "@components/ui/Badge";
import { Activity, Clock, AlertTriangle, Server, Zap, RefreshCw } from "lucide-react";
import type { View } from "@models/index";

interface DashboardProps {
  onNavigate?: (view: View) => void;
}

export function Dashboard({ onNavigate }: DashboardProps) {
  const { isProviderEnabled } = useFilters();
  const { schedulers, loading: schedulersLoading } = useSchedulers();
  const { jobs: filteredJobs, loading: statsLoading } = useFilteredJobs();
  const [scanning, setScanning] = useState(false);

  const handleScan = async () => {
    setScanning(true);
    try {
      await triggerDiscovery();
      // Reload the page data by waiting a tick
      setTimeout(() => window.location.reload(), 500);
    } catch (e) {
      alert(`Scan failed: ${e}`);
    } finally {
      setTimeout(() => setScanning(false), 1000);
    }
  };

  const filteredStats = useMemo(() => {
    if (!filteredJobs) return null;
    const total = filteredJobs.length;
    const active = filteredJobs.filter((j) => j.status === "active").length;
    const failed = filteredJobs.filter((j) => j.status === "error").length;
    const nextScheduled = filteredJobs
      .filter((j) => j.nextExecution)
      .sort((a, b) => a.nextExecution!.localeCompare(b.nextExecution!))[0];
    return {
      totalJobs: total,
      activeJobs: active,
      failedJobs: failed,
      nextScheduled: nextScheduled
        ? { job: nextScheduled, time: new Date(nextScheduled.nextExecution!).toLocaleString() }
        : null,
    };
  }, [filteredJobs]);

  const filteredSchedulers = useMemo(() => {
    if (!schedulers) return schedulers;
    const providerCounts = new Map<string, number>();
    for (const j of filteredJobs ?? []) {
      providerCounts.set(j.provider, (providerCounts.get(j.provider) ?? 0) + 1);
    }
    // Enabled first, filtered-out last (greyed out)
    const enabled = schedulers
      .filter((s) => isProviderEnabled(s.id))
      .map((s) => ({ ...s, jobCount: providerCounts.get(s.id) ?? 0, filtered: false }));
    const disabled = schedulers
      .filter((s) => !isProviderEnabled(s.id))
      .map((s) => ({ ...s, jobCount: providerCounts.get(s.id) ?? 0, filtered: true }));
    return [...enabled, ...disabled];
  }, [schedulers, filteredJobs, isProviderEnabled]);

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

  const displayStats = filteredStats;

  return (
    <div className="space-y-6 p-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Dashboard</h1>
          <p className="text-muted-foreground">
            Automatic scheduling overview for this machine
          </p>
        </div>
        <button
          onClick={handleScan}
          disabled={scanning}
          className="flex items-center gap-2 rounded-md border px-3 py-2 text-sm font-medium text-muted-foreground hover:text-foreground hover:bg-accent transition-colors disabled:opacity-50"
        >
          <RefreshCw className={`h-4 w-4 ${scanning ? "animate-spin" : ""}`} />
          {scanning ? "Scanning..." : "Scan now"}
        </button>
      </div>

      {/* Stats grid */}
      <div className="grid grid-cols-4 gap-4">
        <StatCard
          icon={<Zap className="h-5 w-5" />}
          label="Total Jobs"
          value={displayStats?.totalJobs ?? 0}
          description="Discovered"
        />
        <StatCard
          icon={<Server className="h-5 w-5" />}
          label="Schedulers"
          value={filteredSchedulers?.filter((s) => s.available).length ?? 0}
          description="Detected"
        />
        <StatCard
          icon={<Clock className="h-5 w-5" />}
          label="Active Jobs"
          value={displayStats?.activeJobs ?? 0}
          description="Enabled"
        />
        <StatCard
          icon={<AlertTriangle className="h-5 w-5" />}
          label="Failures"
          value={displayStats?.failedJobs ?? 0}
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
              {filteredSchedulers?.map((s) => (
                <div
                  key={s.id}
                  onClick={() => onNavigate?.("system")}
                  className={`flex items-center justify-between rounded-md border p-3 transition-colors ${
                    s.filtered
                      ? "opacity-40 cursor-default"
                      : "cursor-pointer hover:bg-accent"
                  }`}
                >
                  <div className="flex items-center gap-3">
                    <div
                      className={`h-2 w-2 rounded-full ${
                        !s.available
                          ? "bg-muted-foreground"
                          : s.filtered
                            ? "bg-muted-foreground"
                            : "bg-green-500"
                      }`}
                    />
                    <span className={s.filtered ? "text-muted-foreground" : "font-medium"}>
                      {s.name}
                    </span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="text-sm text-muted-foreground">
                      {s.jobCount} jobs
                    </span>
                    <Badge variant={s.filtered ? "muted" : s.available ? "success" : "muted"}>
                      {s.filtered ? "Filtered" : s.available ? "Active" : "Unavailable"}
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
            {displayStats?.nextScheduled ? (
              <div
                onClick={() => onNavigate?.("jobs")}
                className="rounded-md border p-4 cursor-pointer hover:bg-accent transition-colors"
              >
                <p className="font-medium">{displayStats.nextScheduled.job.name}</p>
                <p className="text-sm text-muted-foreground">
                  {displayStats.nextScheduled.job.command}
                </p>
                <p className="mt-2 text-sm text-primary">
                  {displayStats.nextScheduled.time}
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