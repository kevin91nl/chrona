# 🕐 Chrona

**Automatic scheduling observability for your machine.**

Chrona is a cross-platform desktop application that automatically discovers, indexes, monitors and visualizes all scheduled jobs running on your machine.

> "I installed it and immediately understood everything my computer does automatically."

![Chrona Dashboard](docs/screenshot-dashboard.png)

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) (v18+)
- npm (comes with Node.js)

### Install & Run

```bash
git clone https://github.com/kevin91nl/chrona.git
cd chrona
npm install
npx tauri dev
```

Chrona will start, detect your OS, and begin discovering scheduled jobs immediately.

### Production Build

```bash
npx tauri build
```

The binary will be in `src-tauri/target/release/` (Linux/Windows) or `src-tauri/target/release/bundle/` (macOS .app).

### Frontend-only Development

For UI work without the Rust backend:

```bash
npm run dev
```

Opens at `http://localhost:1420` with demo data.

## Features

- **Zero-config discovery** — detects cron, launchd, systemd timers, Windows Task Scheduler, Codex automations automatically
- **Dashboard** — stats overview, detected schedulers, next scheduled events, failures
- **Timeline** — chronological view with visual timeline, error highlighting
- **Job Explorer** — search, filter, click-through to detailed job view
- **Calendar** — 24-hour grid showing when each job runs
- **System Map** — visualize all schedulers and their jobs by provider
- **Live monitoring** — continuous background scanning, auto-refresh every 30s
- **Dark theme** — professional developer-tool aesthetic

## Supported Schedulers

### Built-in

| Scheduler | Platform | Status |
|-----------|----------|--------|
| cron | macOS, Linux | ✅ Active |
| launchd | macOS | ✅ Active |
| Codex Scheduled Tasks | All | ✅ Active |
| systemd timers | Linux | 🔜 Ready (inactive) |
| Windows Task Scheduler | Windows | 🔜 Ready (inactive) |

### Planned

- GitHub Actions
- Docker scheduled jobs
- Kubernetes CronJobs
- n8n workflows
- Temporal workflows

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop runtime | Tauri v2 |
| Backend | Rust |
| Frontend | React + TypeScript |
| Bundler | Vite |
| Styling | Tailwind CSS |
| Database | SQLite |

## Architecture

```
                 Chrona UI (React + TypeScript)
                            ↓
                    Chrona Core Engine (Rust)
                            ↓
                     Provider Interface
                            ↓
  ┌──────────┬──────────┬──────────┬──────────┐
  cron     launchd   codex     systemd
  provider  provider  provider  provider
                            ↓
                   Operating System APIs
```

Chrona uses a **provider-based architecture**. The core engine has no scheduler-specific knowledge. Each provider implements:

```rust
pub trait SchedulerProvider: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn is_available(&self) -> bool;
    fn discover(&self) -> Result<Vec<DiscoveredJob>>;
    fn refresh(&self) -> Result<Vec<DiscoveredJob>>;
}
```

## Project Structure

```
chrona/
├── src/                          # React frontend
│   ├── components/
│   │   ├── dashboard/            # Stats & overview
│   │   ├── timeline/             # Chronological job view
│   │   ├── jobs/                 # Explorer + detail view
│   │   ├── calendar/             # 24h schedule grid
│   │   ├── system/               # Scheduler map
│   │   └── ui/                   # Shared components
│   ├── hooks/                    # React hooks (data fetching)
│   ├── lib/                      # Utilities & Tauri bridge
│   └── types/                    # TypeScript types
├── src-tauri/                    # Rust backend
│   └── src/
│       ├── providers/            # Scheduler providers
│       ├── db/                   # SQLite schema & queries
│       ├── discovery/            # Discovery engine
│       ├── models/               # Data models
│       └── commands/             # Tauri IPC commands
└── docs/                         # Documentation & screenshots
```

## Roadmap

- [x] Provider system (cron, launchd, codex)
- [x] SQLite persistence
- [x] Discovery engine with auto-refresh
- [x] Dashboard, Timeline, Jobs, Calendar, System Map views
- [ ] Job execution history & logs
- [ ] Intelligence layer (duplicates, conflicts, anomalies)
- [ ] systemd provider activation
- [ ] Windows Task Scheduler provider activation
- [ ] GitHub Actions provider
- [ ] Docker provider
- [ ] Remote machine monitoring via SSH
- [ ] Notifications
- [ ] Job editing & creation

## License

MIT