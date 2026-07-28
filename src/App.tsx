import { useState } from "react";
import { Dashboard } from "@components/dashboard/Dashboard";
import { Timeline } from "@components/timeline/Timeline";
import { JobExplorer } from "@components/jobs/JobExplorer";
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
      default:
        return <Dashboard />;
    }
  };

  return (
    <div className="flex h-screen overflow-hidden">
      <Sidebar currentView={currentView} onNavigate={setCurrentView} />
      <main className="flex-1 overflow-auto">{renderView()}</main>
    </div>
  );
}

export default App;