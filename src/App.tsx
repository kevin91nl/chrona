import { useState } from "react";
import { FilterProvider } from "@contexts/FilterContext";
import { Dashboard } from "@components/dashboard/Dashboard";
import { Timeline } from "@components/timeline/Timeline";
import { JobExplorer } from "@components/jobs/JobExplorer";
import { CalendarView } from "@components/calendar/CalendarView";
import { SystemMap } from "@components/system/SystemMap";
import { Sidebar } from "@components/Sidebar";
import type { View } from "@models/index";

function App() {
  const [currentView, setCurrentView] = useState<View>("dashboard");

  const renderView = () => {
    switch (currentView) {
      case "dashboard":
        return <Dashboard />;
      case "timeline":
        return <Timeline />;
      case "jobs":
        return <JobExplorer />;
      case "calendar":
        return <CalendarView />;
      case "system":
        return <SystemMap />;
      default:
        return <Dashboard />;
    }
  };

  return (
    <FilterProvider>
      <div className="flex h-screen overflow-hidden">
        <Sidebar currentView={currentView} onNavigate={setCurrentView} />
        <main className="flex-1 overflow-auto">{renderView()}</main>
      </div>
    </FilterProvider>
  );
}

export default App;