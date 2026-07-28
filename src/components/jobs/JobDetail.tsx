import { useState } from "react";
import type { Job } from "@models/index";
import { Card, CardContent, CardHeader, CardTitle } from "@components/ui/Card";
import { Badge } from "@components/ui/Badge";
import { formatSchedule, formatRelativeTime } from "@/utils";
import { toggleJobEnabled, removeJob } from "@/tauri";
import {
  ArrowLeft,
  Clock,
  Folder,
  FileText,
  Terminal,
  Pause,
  Play,
  Trash2,
  Loader2,
} from "lucide-react";

interface JobDetailProps {
  job: Job;
  onBack: () => void;
  onJobChanged?: () => void;
}

export function JobDetail({ job, onBack, onJobChanged }: JobDetailProps) {
  const [currentJob, setCurrentJob] = useState(job);
  const [busy, setBusy] = useState<string | null>(null);

  const canToggle = currentJob.provider === "cron" || currentJob.provider === "codex";

  const handleToggle = async () => {
    setBusy("toggle");
    try {
      const updated = await toggleJobEnabled(currentJob.id);
      setCurrentJob(updated);
      onJobChanged?.();
    } catch (e) {
      alert(`Toggle failed: ${e}`);
    } finally {
      setBusy(null);
    }
  };

  const handleDelete = async () => {
    if (!confirm(`Delete "${currentJob.name}"?`)) return;
    setBusy("delete");
    try {
      await removeJob(currentJob.id);
      onJobChanged?.();
      onBack();
    } catch (e) {
      console.error("Delete failed:", e);
      setBusy(null);
    }
  };

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

      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div
            className={`h-3 w-3 rounded-full ${
              currentJob.status === "error"
                ? "bg-red-500"
                : currentJob.enabled
                  ? "bg-green-500"
                  : "bg-muted-foreground"
            }`}
          />
          <h1 className="text-2xl font-bold">{currentJob.name}</h1>
          <Badge variant="muted">{currentJob.provider}</Badge>
          <Badge
            variant={
              currentJob.status === "error"
                ? "error"
                : currentJob.enabled
                  ? "success"
                  : "warning"
            }
          >
            {currentJob.status === "error"
              ? "Error"
              : currentJob.enabled
                ? "Enabled"
                : "Disabled"}
          </Badge>
        </div>

        {/* Action buttons */}
        <div className="flex items-center gap-2">
          <button
            onClick={handleToggle}
            disabled={busy !== null || !canToggle}
            title={canToggle ? undefined : `${currentJob.provider} does not support pause/resume`}
            className={`flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors disabled:opacity-50 ${
              !canToggle
                ? "bg-muted text-muted-foreground border border-border cursor-not-allowed"
                : currentJob.enabled
                  ? "bg-yellow-500/10 text-yellow-400 hover:bg-yellow-500/20 border border-yellow-500/30"
                  : "bg-green-500/10 text-green-400 hover:bg-green-500/20 border border-green-500/30"
            }`}
          >
            {busy === "toggle" ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : currentJob.enabled ? (
              <Pause className="h-4 w-4" />
            ) : (
              <Play className="h-4 w-4" />
            )}
            {currentJob.enabled ? "Disable" : "Enable"}
          </button>
          <button
            onClick={handleDelete}
            disabled={busy !== null}
            className="flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium bg-red-500/10 text-red-400 hover:bg-red-500/20 border border-red-500/30 transition-colors disabled:opacity-50"
          >
            {busy === "delete" ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <Trash2 className="h-4 w-4" />
            )}
            Delete
          </button>
        </div>
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
                <p className="font-mono text-sm">{currentJob.command}</p>
              </div>
            </div>
            <div className="flex items-start gap-3">
              <Clock className="h-4 w-4 mt-0.5 text-muted-foreground" />
              <div>
                <p className="text-sm text-muted-foreground">Schedule</p>
                <p className="font-mono text-sm">
                  {currentJob.schedule}{" "}
                  <span className="text-muted-foreground">
                    ({formatSchedule(currentJob.schedule)})
                  </span>
                </p>
              </div>
            </div>
            <div className="flex items-start gap-3">
              <FileText className="h-4 w-4 mt-0.5 text-muted-foreground" />
              <div>
                <p className="text-sm text-muted-foreground">Source</p>
                <p className="font-mono text-sm break-all">
                  {currentJob.source}
                </p>
              </div>
            </div>
            {currentJob.workingDirectory && (
              <div className="flex items-start gap-3">
                <Folder className="h-4 w-4 mt-0.5 text-muted-foreground" />
                <div>
                  <p className="text-sm text-muted-foreground">
                    Working Directory
                  </p>
                  <p className="font-mono text-sm">
                    {currentJob.workingDirectory}
                  </p>
                </div>
              </div>
            )}
            {currentJob.timezone && (
              <div>
                <p className="text-sm text-muted-foreground">Timezone</p>
                <p className="text-sm">{currentJob.timezone}</p>
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
                {currentJob.nextExecution
                  ? formatRelativeTime(currentJob.nextExecution)
                  : "Unknown"}
              </p>
              {currentJob.nextExecution && (
                <p className="text-xs text-muted-foreground mt-0.5">
                  {new Date(currentJob.nextExecution).toLocaleString()}
                </p>
              )}
            </div>
            <div>
              <p className="text-sm text-muted-foreground">
                Previous Execution
              </p>
              <p className="text-sm font-medium">
                {currentJob.previousExecution
                  ? formatRelativeTime(currentJob.previousExecution)
                  : "None"}
              </p>
              {currentJob.previousExecution && (
                <p className="text-xs text-muted-foreground mt-0.5">
                  {new Date(currentJob.previousExecution).toLocaleString()}
                </p>
              )}
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Provider</p>
              <p className="text-sm">{currentJob.provider}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Discovered</p>
              <p className="text-sm">
                {new Date(currentJob.discoveredAt).toLocaleString()}
              </p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Last Updated</p>
              <p className="text-sm">
                {new Date(currentJob.updatedAt).toLocaleString()}
              </p>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}