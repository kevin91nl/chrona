import { useState } from "react";
import { cn } from "@/utils";
import { useSchedulers, useFilteredJobs } from "@hooks/useJobs";
import { useFilters } from "@contexts/FilterContext";
import {
  LayoutDashboard,
  Clock,
  List,
  Calendar,
  Network,
  Activity,
  Filter,
  RotateCcw,
  ChevronDown,
} from "lucide-react";
import type { View } from "@models/index";

interface SidebarProps {
  currentView: View;
  onNavigate: (view: View) => void;
}

const navItems: { view: View; label: string; icon: typeof LayoutDashboard }[] = [
  { view: "dashboard", label: "Dashboard", icon: LayoutDashboard },
  { view: "timeline", label: "Timeline", icon: Clock },
  { view: "jobs", label: "Jobs", icon: List },
  { view: "calendar", label: "Calendar", icon: Calendar },
  { view: "system", label: "System Map", icon: Network },
];

const PROVIDER_COLORS: Record<string, string> = {
  cron: "bg-blue-500",
  launchd: "bg-green-500",
  codex: "bg-purple-500",
  systemd: "bg-orange-500",
  "windows-task": "bg-cyan-500",
};

export function Sidebar({ currentView, onNavigate }: SidebarProps) {
  const { schedulers } = useSchedulers();
  const { jobs: filteredJobs, loading } = useFilteredJobs();
  const {
    enabledProviders,
    toggleProvider,
    isProviderEnabled,
    resetFilters,
    filtersActive,
  } = useFilters();
  const [filtersOpen, setFiltersOpen] = useState(false);

  // Build provider list: discovered schedulers + any known but undiscovered
  const allProviderIds = [
    ...new Set([
      ...(schedulers ?? []).map((s) => s.id),
      ...Object.keys(PROVIDER_COLORS),
    ]),
  ];

  return (
    <aside className="flex w-56 flex-col border-r bg-card">
      {/* Logo */}
      <div className="flex items-center gap-2 border-b px-4 py-4">
        <Activity className="h-6 w-6 text-primary" />
        <span className="text-lg font-bold tracking-tight">Chrona</span>
      </div>

      {/* Navigation */}
      <nav className="flex-1 space-y-1 p-2">
        {navItems.map(({ view, label, icon: Icon }) => (
          <button
            key={view}
            onClick={() => onNavigate(view)}
            className={cn(
              "flex w-full items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-colors",
              currentView === view
                ? "bg-primary/10 text-primary"
                : "text-muted-foreground hover:bg-accent hover:text-foreground",
            )}
          >
            <Icon className="h-4 w-4" />
            {label}
          </button>
        ))}
      </nav>

      {/* Filters */}
      <div className="border-t px-3 py-2">
        <button
          onClick={() => setFiltersOpen(!filtersOpen)}
          className="flex w-full items-center justify-between rounded-md px-2 py-1.5 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
        >
          <span className="flex items-center gap-2">
            <Filter className="h-4 w-4" />
            Filters
            {filtersActive && (
              <span className="flex h-4 min-w-4 items-center justify-center rounded-full bg-primary px-1 text-[10px] font-bold text-primary-foreground">
                {enabledProviders.size}
              </span>
            )}
          </span>
          <ChevronDown
            className={cn(
              "h-3.5 w-3.5 transition-transform",
              filtersOpen && "rotate-180",
            )}
          />
        </button>

        {filtersOpen && (
          <div className="mt-2 space-y-1">
            {allProviderIds.map((id) => {
              const scheduler = schedulers?.find((s) => s.id === id);
              const label = scheduler?.name ?? id;
              const checked = isProviderEnabled(id);
              return (
                <label
                  key={id}
                  className={cn(
                    "flex items-center gap-2 rounded-md px-2 py-1.5 text-xs cursor-pointer transition-colors",
                    checked
                      ? "text-foreground hover:bg-accent"
                      : "text-muted-foreground/50",
                  )}
                >
                  <input
                    type="checkbox"
                    checked={checked}
                    onChange={() => toggleProvider(id)}
                    className="h-3.5 w-3.5 rounded border-muted-foreground accent-primary"
                  />
                  <div
                    className={cn(
                      "h-2 w-2 rounded-full",
                      PROVIDER_COLORS[id] ?? "bg-muted-foreground",
                      !checked && "opacity-30",
                    )}
                  />
                  <span className="truncate">{label}</span>
                </label>
              );
            })}

            {filtersActive && (
              <button
                onClick={resetFilters}
                className="flex w-full items-center gap-1.5 rounded-md px-2 py-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors mt-1"
              >
                <RotateCcw className="h-3 w-3" />
                Show all
              </button>
            )}
          </div>
        )}
      </div>

      {/* Status footer */}
      <div className="border-t px-4 py-3 space-y-1">
        <div className="flex items-center gap-2">
          <div
            className={`h-2 w-2 rounded-full ${
              loading ? "bg-yellow-500 animate-pulse" : "bg-green-500"
            }`}
          />
          <span className="text-xs text-muted-foreground">
            {loading ? "Scanning..." : "Live"}
          </span>
          {filtersActive && (
            <span className="ml-auto text-[10px] text-yellow-400/80">filtered</span>
          )}
        </div>
        {!loading && filteredJobs.length > 0 && (
          <p className="text-xs text-muted-foreground pl-4">
            {filteredJobs.length} jobs across{" "}
            {new Set(filteredJobs.map((j) => j.provider)).size} schedulers
          </p>
        )}
      </div>
    </aside>
  );
}