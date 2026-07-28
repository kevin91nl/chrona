import { useState, useCallback } from "react";
import { useFilteredJobs as useJobs } from "@hooks/useJobs";
import { Card, CardContent, CardHeader, CardTitle } from "@components/ui/Card";
import { Badge } from "@components/ui/Badge";
import { formatSchedule, formatRelativeTime } from "@/utils";
import { toggleJobEnabled, removeJob } from "@/tauri";
import { Search, Pause, Play, Trash2 } from "lucide-react";
import { JobDetail } from "./JobDetail";
import type { Job } from "@models/index";

export function JobExplorer() {
  const [refreshKey, setRefreshKey] = useState(0);
  const { jobs, loading } = useJobs(refreshKey);
  const [search, setSearch] = useState("");
  const [selectedJob, setSelectedJob] = useState<Job | null>(null);

  const refresh = useCallback(() => {
    setRefreshKey((k) => k + 1);
  }, []);

  // If a job is selected, show detail view
  if (selectedJob) {
    return (
      <JobDetail
        job={selectedJob}
        onBack={() => setSelectedJob(null)}
        onJobChanged={refresh}
      />
    );
  }

  const handleQuickToggle = async (e: React.MouseEvent, job: Job) => {
    e.stopPropagation();
    if (job.provider !== "cron" && job.provider !== "codex") {
      alert(`${job.provider} does not support pause/resume`);
      return;
    }
    try {
      await toggleJobEnabled(job.id);
      refresh();
    } catch (err) {
      alert(`Toggle failed: ${err}`);
    }
  };

  const handleQuickDelete = async (e: React.MouseEvent, job: Job) => {
    e.stopPropagation();
    if (!confirm(`Delete "${job.name}"?`)) return;
    try {
      await removeJob(job.id);
      refresh();
    } catch (err) {
      console.error("Delete failed:", err);
    }
  };

  const filtered = jobs?.filter(
    (j) =>
      j.name.toLowerCase().includes(search.toLowerCase()) ||
      j.command.toLowerCase().includes(search.toLowerCase()) ||
      j.provider.toLowerCase().includes(search.toLowerCase()),
  );

  return (
    <div className="space-y-6 p-6">
      <div>
        <h1 className="text-2xl font-bold">Jobs</h1>
        <p className="text-muted-foreground">
          Search and filter all discovered jobs
        </p>
      </div>

      {/* Search */}
      <div className="relative">
        <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
        <input
          type="text"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="Filter by name, command, or provider..."
          className="w-full rounded-md border bg-input py-2 pl-10 pr-4 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
        />
      </div>

      <Card>
        <CardHeader>
          <CardTitle>
            {filtered?.length ?? 0} Jobs
            {search && ` matching "${search}"`}
          </CardTitle>
        </CardHeader>
        <CardContent>
          {loading ? (
            <p className="text-sm text-muted-foreground">Loading...</p>
          ) : filtered?.length ? (
            <div className="space-y-2">
              {filtered.map((job) => (
                <div
                  key={job.id}
                  onClick={() => setSelectedJob(job)}
                  className="group flex items-center justify-between rounded-md border p-3 transition-colors hover:bg-accent cursor-pointer"
                >
                  <div className="flex items-center gap-3">
                    <div
                      className={`h-2 w-2 rounded-full ${
                        job.status === "error"
                          ? "bg-red-500"
                          : job.enabled
                            ? "bg-green-500"
                            : "bg-muted-foreground"
                      }`}
                    />
                    <div>
                      <p className="font-medium">{job.name}</p>
                      <p className="text-xs font-mono text-muted-foreground">
                        {job.command}
                      </p>
                    </div>
                  </div>
                  <div className="flex items-center gap-2 text-sm">
                    <span className="text-muted-foreground">
                      {formatSchedule(job.schedule)}
                    </span>
                    <Badge variant="muted">{job.provider}</Badge>
                    {job.nextExecution && (
                      <span className="text-xs text-muted-foreground">
                        {formatRelativeTime(job.nextExecution)}
                      </span>
                    )}
                    {/* Quick actions — visible on hover */}
                    <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity ml-2">
                      <button
                        onClick={(e) => handleQuickToggle(e, job)}
                        title={job.enabled ? "Disable" : "Enable"}
                        className={`rounded p-1 transition-colors ${
                          job.enabled
                            ? "text-yellow-400 hover:bg-yellow-500/20"
                            : "text-green-400 hover:bg-green-500/20"
                        }`}
                      >
                        {job.enabled ? (
                          <Pause className="h-3.5 w-3.5" />
                        ) : (
                          <Play className="h-3.5 w-3.5" />
                        )}
                      </button>
                      <button
                        onClick={(e) => handleQuickDelete(e, job)}
                        title="Delete"
                        className="rounded p-1 text-red-400 hover:bg-red-500/20 transition-colors"
                      >
                        <Trash2 className="h-3.5 w-3.5" />
                      </button>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-sm text-muted-foreground">No jobs found</p>
          )}
        </CardContent>
      </Card>
    </div>
  );
}