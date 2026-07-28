import type { Job } from "@models/index";
import { Card, CardContent, CardHeader, CardTitle } from "@components/ui/Card";
import { Badge } from "@components/ui/Badge";
import { formatSchedule, formatRelativeTime } from "@/utils";
import { ArrowLeft, Clock, Folder, FileText, Terminal } from "lucide-react";

interface JobDetailProps {
  job: Job;
  onBack: () => void;
}

export function JobDetail({ job, onBack }: JobDetailProps) {
  return (
    <div className="space-y-6 p-6">
      <div className="flex items-center gap-3">
        <button
          onClick={onBack}
          className="flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground transition-colors"
        >
          <ArrowLeft className="h-4 w-4" />
          Back
        </button>
      </div>

      <div className="flex items-center gap-3">
        <div
          className={`h-3 w-3 rounded-full ${
            job.status === "error"
              ? "bg-red-500"
              : job.enabled
                ? "bg-green-500"
                : "bg-muted-foreground"
          }`}
        />
        <h1 className="text-2xl font-bold">{job.name}</h1>
        <Badge variant="muted">{job.provider}</Badge>
        <Badge
          variant={
            job.status === "error"
              ? "error"
              : job.enabled
                ? "success"
                : "warning"
          }
        >
          {job.status === "error"
            ? "Error"
            : job.enabled
              ? "Enabled"
              : "Disabled"}
        </Badge>
      </div>

      <div className="grid grid-cols-2 gap-6">
        {/* Details */}
        <Card>
          <CardHeader>
            <CardTitle>Details</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-start gap-3">
              <Terminal className="h-4 w-4 mt-0.5 text-muted-foreground" />
              <div>
                <p className="text-sm text-muted-foreground">Command</p>
                <p className="font-mono text-sm">{job.command}</p>
              </div>
            </div>
            <div className="flex items-start gap-3">
              <Clock className="h-4 w-4 mt-0.5 text-muted-foreground" />
              <div>
                <p className="text-sm text-muted-foreground">Schedule</p>
                <p className="font-mono text-sm">
                  {job.schedule}{" "}
                  <span className="text-muted-foreground">
                    ({formatSchedule(job.schedule)})
                  </span>
                </p>
              </div>
            </div>
            <div className="flex items-start gap-3">
              <FileText className="h-4 w-4 mt-0.5 text-muted-foreground" />
              <div>
                <p className="text-sm text-muted-foreground">Source</p>
                <p className="font-mono text-sm break-all">{job.source}</p>
              </div>
            </div>
            {job.workingDirectory && (
              <div className="flex items-start gap-3">
                <Folder className="h-4 w-4 mt-0.5 text-muted-foreground" />
                <div>
                  <p className="text-sm text-muted-foreground">
                    Working Directory
                  </p>
                  <p className="font-mono text-sm">{job.workingDirectory}</p>
                </div>
              </div>
            )}
            {job.timezone && (
              <div>
                <p className="text-sm text-muted-foreground">Timezone</p>
                <p className="text-sm">{job.timezone}</p>
              </div>
            )}
          </CardContent>
        </Card>

        {/* Execution info */}
        <Card>
          <CardHeader>
            <CardTitle>Execution</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div>
              <p className="text-sm text-muted-foreground">Next Execution</p>
              <p className="text-sm font-medium">
                {job.nextExecution
                  ? formatRelativeTime(job.nextExecution)
                  : "Unknown"}
              </p>
              {job.nextExecution && (
                <p className="text-xs text-muted-foreground mt-0.5">
                  {new Date(job.nextExecution).toLocaleString()}
                </p>
              )}
            </div>
            <div>
              <p className="text-sm text-muted-foreground">
                Previous Execution
              </p>
              <p className="text-sm font-medium">
                {job.previousExecution
                  ? formatRelativeTime(job.previousExecution)
                  : "None"}
              </p>
              {job.previousExecution && (
                <p className="text-xs text-muted-foreground mt-0.5">
                  {new Date(job.previousExecution).toLocaleString()}
                </p>
              )}
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Provider</p>
              <p className="text-sm">{job.provider}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Discovered</p>
              <p className="text-sm">
                {new Date(job.discoveredAt).toLocaleString()}
              </p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Last Updated</p>
              <p className="text-sm">
                {new Date(job.updatedAt).toLocaleString()}
              </p>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}