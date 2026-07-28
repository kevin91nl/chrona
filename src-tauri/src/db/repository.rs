use crate::db::schema::Database;
use crate::models::{Execution, ExecutionStatus, Job, JobStatus};
use chrono::{DateTime, Utc};
use rusqlite::params;
use std::sync::Arc;

pub struct JobRepository {
    db: Arc<Database>,
}

impl JobRepository {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub fn upsert_job(&self, job: &Job) -> Result<(), rusqlite::Error> {
        let conn = self.db.conn();
        conn.execute(
            "INSERT INTO jobs (id, name, provider, source, schedule, timezone,
               command, working_directory, status, enabled, discovered_at,
               updated_at, next_execution, previous_execution)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
             ON CONFLICT(id) DO UPDATE SET
               name=excluded.name, source=excluded.source,
               schedule=excluded.schedule, timezone=excluded.timezone,
               command=excluded.command, working_directory=excluded.working_directory,
               status=excluded.status, enabled=excluded.enabled,
               updated_at=excluded.updated_at, next_execution=excluded.next_execution,
               previous_execution=excluded.previous_execution",
            params![
                job.id,
                job.name,
                job.provider,
                job.source,
                job.schedule,
                job.timezone,
                job.command,
                job.working_directory,
                job.status.to_string(),
                job.enabled as i32,
                job.discovered_at.to_rfc3339(),
                job.updated_at.to_rfc3339(),
                job.next_execution.map(|d| d.to_rfc3339()),
                job.previous_execution.map(|d| d.to_rfc3339()),
            ],
        )?;
        Ok(())
    }

    pub fn get_all_jobs(&self) -> Result<Vec<Job>, rusqlite::Error> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT id, name, provider, source, schedule, timezone, command,
                    working_directory, status, enabled, discovered_at, updated_at,
                    next_execution, previous_execution
             FROM jobs ORDER BY name",
        )?;

        let jobs = stmt
            .query_map([], |row| {
                Ok(Job {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    provider: row.get(2)?,
                    source: row.get(3)?,
                    schedule: row.get(4)?,
                    timezone: row.get(5)?,
                    command: row.get(6)?,
                    working_directory: row.get(7)?,
                    status: JobStatus::from(row.get::<_, String>(8)?.as_str()),
                    enabled: row.get::<_, i32>(9)? != 0,
                    discovered_at: parse_timestamp(row.get::<_, String>(10)?),
                    updated_at: parse_timestamp(row.get::<_, String>(11)?),
                    next_execution: row
                        .get::<_, Option<String>>(12)?
                        .map(|s| parse_timestamp(s)),
                    previous_execution: row
                        .get::<_, Option<String>>(13)?
                        .map(|s| parse_timestamp(s)),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(jobs)
    }

    pub fn get_job(&self, id: &str) -> Result<Option<Job>, rusqlite::Error> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT id, name, provider, source, schedule, timezone, command,
                    working_directory, status, enabled, discovered_at, updated_at,
                    next_execution, previous_execution
             FROM jobs WHERE id = ?1",
        )?;

        let mut rows = stmt.query_map(params![id], |row| {
            Ok(Job {
                id: row.get(0)?,
                name: row.get(1)?,
                provider: row.get(2)?,
                source: row.get(3)?,
                schedule: row.get(4)?,
                timezone: row.get(5)?,
                command: row.get(6)?,
                working_directory: row.get(7)?,
                status: JobStatus::from(row.get::<_, String>(8)?.as_str()),
                enabled: row.get::<_, i32>(9)? != 0,
                discovered_at: parse_timestamp(row.get::<_, String>(10)?),
                updated_at: parse_timestamp(row.get::<_, String>(11)?),
                next_execution: row
                    .get::<_, Option<String>>(12)?
                    .map(|s| parse_timestamp(s)),
                previous_execution: row
                    .get::<_, Option<String>>(13)?
                    .map(|s| parse_timestamp(s)),
            })
        })?;

        match rows.next() {
            Some(job) => Ok(Some(job?)),
            None => Ok(None),
        }
    }

    pub fn delete_job(&self, id: &str) -> Result<(), rusqlite::Error> {
        let conn = self.db.conn();
        conn.execute("DELETE FROM jobs WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn insert_execution(&self, exec: &Execution) -> Result<(), rusqlite::Error> {
        let conn = self.db.conn();
        conn.execute(
            "INSERT INTO executions (id, job_id, start_time, end_time, duration_ms, status, exit_code, logs)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                exec.id,
                exec.job_id,
                exec.start_time.to_rfc3339(),
                exec.end_time.map(|d| d.to_rfc3339()),
                exec.duration_ms,
                exec.status.to_string(),
                exec.exit_code,
                exec.logs,
            ],
        )?;
        Ok(())
    }

    pub fn get_executions(&self, job_id: &str) -> Result<Vec<Execution>, rusqlite::Error> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT id, job_id, start_time, end_time, duration_ms, status, exit_code, logs
             FROM executions WHERE job_id = ?1 ORDER BY start_time DESC",
        )?;

        let execs = stmt
            .query_map(params![job_id], |row| {
                Ok(Execution {
                    id: row.get(0)?,
                    job_id: row.get(1)?,
                    start_time: parse_timestamp(row.get::<_, String>(2)?),
                    end_time: row
                        .get::<_, Option<String>>(3)?
                        .map(|s| parse_timestamp(s)),
                    duration_ms: row.get(4)?,
                    status: ExecutionStatus::from(row.get::<_, String>(5)?.as_str()),
                    exit_code: row.get(6)?,
                    logs: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(execs)
    }

    pub fn remove_stale_jobs(
        &self,
        provider: &str,
        active_ids: &[String],
    ) -> Result<usize, rusqlite::Error> {
        let conn = self.db.conn();
        // Remove jobs from this provider that are no longer discovered
        let placeholders: Vec<String> = active_ids
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", i + 2))
            .collect();
        let query = if placeholders.is_empty() {
            "DELETE FROM jobs WHERE provider = ?1".to_string()
        } else {
            format!(
                "DELETE FROM jobs WHERE provider = ?1 AND id NOT IN ({})",
                placeholders.join(",")
            )
        };

        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> =
            vec![Box::new(provider.to_string())];
        for id in active_ids {
            param_values.push(Box::new(id.clone()));
        }

        let params_ref: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        conn.execute(&query, params_ref.as_slice())?;
        Ok(0)
    }

    pub fn get_job_count_by_provider(&self) -> Result<Vec<(String, i64)>, rusqlite::Error> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT provider, COUNT(*) FROM jobs GROUP BY provider ORDER BY provider",
        )?;

        let counts = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(counts)
    }
}

fn parse_timestamp(s: String) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(&s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

pub struct SettingsRepository {
    db: Arc<Database>,
}

impl SettingsRepository {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub fn get(&self, key: &str) -> Result<Option<String>, rusqlite::Error> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query_map(params![key], |row| row.get::<_, String>(0))?;
        match rows.next() {
            Some(val) => Ok(Some(val?)),
            None => Ok(None),
        }
    }

    pub fn set(&self, key: &str, value: &str) -> Result<(), rusqlite::Error> {
        let conn = self.db.conn();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value=excluded.value, updated_at=excluded.updated_at",
            params![key, value, now],
        )?;
        Ok(())
    }

    pub fn delete(&self, key: &str) -> Result<(), rusqlite::Error> {
        let conn = self.db.conn();
        conn.execute("DELETE FROM settings WHERE key = ?1", params![key])?;
        Ok(())
    }
}