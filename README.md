# 🕐 Chrona

**Automatic scheduling observability for your machine.**

Chrona is a cross-platform desktop application that automatically discovers, indexes, monitors and visualizes all scheduled jobs running on your machine.

> "I installed it and immediately understood everything my computer does automatically."

## Features

- **Zero-config discovery** — detects cron, launchd, systemd timers, Windows Task Scheduler automatically
- **Timeline view** — see what runs when, at a glance
- **Live monitoring** — continuous background scanning for new, changed, or removed jobs
- **Intelligence layer** — detects duplicate jobs, conflicts, and suspicious activity
- **Cross-platform** — macOS, Linux, Windows
- **Read-only & safe** — never modifies scheduler config without explicit user action

## Supported Schedulers

### Built-in (MVP)

| Scheduler | Platform |
|-----------|----------|
| cron | macOS, Linux |
| launchd | macOS |
| systemd timers | Linux |
| Windows Task Scheduler | Windows |

### Planned

- GitHub Actions
- Codex Scheduled Tasks
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
| UI components | shadcn/ui |
| Database | SQLite |

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) (v18+)
- [pnpm](https://pnpm.io/) (recommended) or npm

### Install

```bash
git clone https://github.com/kevin91nl/chrona.git
cd chrona
pnpm install
```

### Development

```bash
pnpm tauri dev
```

Chrona will start, detect your OS, and begin discovering scheduled jobs immediately.

### Build

```bash
pnpm tauri build
```

## Architecture

```
                 Chrona UI (React + TypeScript)
                            ↓
                    Chrona Core Engine (Rust)
                            ↓
                     Provider Interface
                            ↓
  ┌──────────┬──────────┬──────────┬──────────┐
  cron     launchd   systemd    Windows
  provider  provider  provider   provider
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
    fn list_jobs(&self) -> Result<Vec<Job>>;
    fn get_job(&self, id: &str) -> Result<Option<Job>>;
    fn get_execution_history(&self, id: &str) -> Result<Vec<Execution>>;
    fn get_logs(&self, id: &str) -> Result<String>;
}
```

## Data Model

All discovered jobs are normalized into a local SQLite database:

**Jobs** — id, name, provider, source, schedule, timezone, command, working directory, status, enabled state, timestamps

**Executions** — job id, start time, end time, duration, status, exit code, logs

## Roadmap

- [x] Project scaffolding & architecture
- [ ] Provider system (cron, launchd, systemd, Windows Task Scheduler)
- [ ] SQLite schema & persistence
- [ ] Discovery engine
- [ ] Dashboard UI
- [ ] Timeline view
- [ ] Job explorer & detail view
- [ ] Calendar view
- [ ] System map
- [ ] Intelligence layer (duplicates, conflicts, anomalies)
- [ ] Background monitoring & refresh
- [ ] GitHub Actions provider
- [ ] Docker provider
- [ ] Kubernetes CronJob provider
- [ ] Remote machine monitoring via SSH
- [ ] Notifications
- [ ] Job editing & creation
- [ ] AI-assisted explanations

## License

MIT