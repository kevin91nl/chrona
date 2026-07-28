use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(path: &Path) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.initialize()?;
        Ok(db)
    }

    pub fn new_in_memory() -> Result<Self, rusqlite::Error> {
        let conn = Connection::open_in_memory()?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.initialize()?;
        Ok(db)
    }

    fn initialize(&self) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS jobs (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                provider TEXT NOT NULL,
                source TEXT NOT NULL,
                schedule TEXT NOT NULL,
                timezone TEXT,
                command TEXT NOT NULL,
                working_directory TEXT,
                status TEXT NOT NULL DEFAULT 'unknown',
                enabled INTEGER NOT NULL DEFAULT 1,
                discovered_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                next_execution TEXT,
                previous_execution TEXT
            );

            CREATE TABLE IF NOT EXISTS executions (
                id TEXT PRIMARY KEY,
                job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
                start_time TEXT NOT NULL,
                end_time TEXT,
                duration_ms INTEGER,
                status TEXT NOT NULL DEFAULT 'running',
                exit_code INTEGER,
                logs TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_jobs_provider ON jobs(provider);
            CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status);
            CREATE INDEX IF NOT EXISTS idx_jobs_enabled ON jobs(enabled);
            CREATE INDEX IF NOT EXISTS idx_executions_job_id ON executions(job_id);
            CREATE INDEX IF NOT EXISTS idx_executions_start_time ON executions(start_time);

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            ",
        )?;
        Ok(())
    }

    pub fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap()
    }
}