import { useState } from "react";
import { useJobs } from "@hooks/useJobs";
import { Card, CardContent, CardHeader, CardTitle } from "@components/ui/Card";
import { Badge } from "@components/ui/Badge";
import { formatSchedule, formatRelativeTime } from "@/utils";
import { Search } from "lucide-react";

export function JobExplorer() {
  const { jobs, loading } = useJobs();
  const [search, setSearch] = useState("");

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
                  className="flex items-center justify-between rounded-md border p-3 transition-colors hover:bg-accent cursor-pointer"
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
                  <div className="flex items-center gap-3 text-sm">
                    <span className="text-muted-foreground">
                      {formatSchedule(job.schedule)}
                    </span>
                    <Badge variant="muted">{job.provider}</Badge>
                    {job.nextExecution && (
                      <span className="text-xs text-muted-foreground">
                        {formatRelativeTime(job.nextExecution)}
                      </span>
                    )}
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