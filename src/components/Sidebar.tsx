import { cn } from "@/utils";
import { useDiscoveryStats } from "@hooks/useJobs";
import {
  LayoutDashboard,
  Clock,
  List,
  Calendar,
  Network,
  Activity,
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

export function Sidebar({ currentView, onNavigate }: SidebarProps) {
  const { stats, loading } = useDiscoveryStats();

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

      {/* Status footer */}
      <div className="border-t px-4 py-3 space-y-1">
        <div className="flex items-center gap-2">
          <div
            className={`h-2 w-2 rounded-full ${
              loading
                ? "bg-yellow-500 animate-pulse"
                : "bg-green-500"
            }`}
          />
          <span className="text-xs text-muted-foreground">
            {loading ? "Scanning..." : "Live"}
          </span>
        </div>
        {stats && !loading && (
          <p className="text-xs text-muted-foreground pl-4">
            {stats.totalJobs} jobs across {stats.schedulersDetected} schedulers
          </p>
        )}
      </div>
    </aside>
  );
}