use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::core::Job;

/// Database for job persistence
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Get the database file path
    pub fn db_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "nanobanan", "banana-cli")
            .context("Failed to determine data directory")?;
        let data_dir = proj_dirs.data_dir();
        std::fs::create_dir_all(data_dir)?;
        Ok(data_dir.join("jobs.db"))
    }

    /// Open or create the database
    pub fn open() -> Result<Self> {
        let path = Self::db_path()?;
        let conn = Connection::open(&path)?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        db.init_schema()?;
        Ok(db)
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS jobs (
                id TEXT PRIMARY KEY,
                action_json TEXT NOT NULL,
                params_json TEXT NOT NULL,
                status_json TEXT NOT NULL,
                images_json TEXT NOT NULL,
                model TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                parent_id TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_jobs_created_at ON jobs(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status_json);
            "#,
        )?;
        Ok(())
    }

    /// Insert a new job
    pub fn insert_job(&self, job: &Job) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT INTO jobs (id, action_json, params_json, status_json, images_json, model, created_at, updated_at, parent_id)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                job.id,
                serde_json::to_string(&job.action)?,
                serde_json::to_string(&job.params)?,
                serde_json::to_string(&job.status)?,
                serde_json::to_string(&job.images)?,
                job.model,
                job.created_at.to_rfc3339(),
                job.updated_at.to_rfc3339(),
                job.parent_id,
            ],
        )?;
        Ok(())
    }

    /// Update an existing job
    pub fn update_job(&self, job: &Job) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            UPDATE jobs SET
                action_json = ?2,
                params_json = ?3,
                status_json = ?4,
                images_json = ?5,
                model = ?6,
                updated_at = ?7,
                parent_id = ?8
            WHERE id = ?1
            "#,
            params![
                job.id,
                serde_json::to_string(&job.action)?,
                serde_json::to_string(&job.params)?,
                serde_json::to_string(&job.status)?,
                serde_json::to_string(&job.images)?,
                job.model,
                job.updated_at.to_rfc3339(),
                job.parent_id,
            ],
        )?;
        Ok(())
    }

    /// Get a job by ID
    pub fn get_job(&self, id: &str) -> Result<Option<Job>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, action_json, params_json, status_json, images_json, model, created_at, updated_at, parent_id FROM jobs WHERE id = ?1"
        )?;

        stmt.query_row(params![id], |row| {
            Ok(self.row_to_job(row))
        })
        .optional()?
        .transpose()
    }

    /// List jobs with optional filters
    pub fn list_jobs(&self, limit: u32, status_filter: Option<&str>) -> Result<Vec<Job>> {
        let conn = self.conn.lock().unwrap();

        let mut jobs = Vec::new();

        if let Some(status) = status_filter {
            let query = "SELECT id, action_json, params_json, status_json, images_json, model, created_at, updated_at, parent_id FROM jobs WHERE status_json LIKE ?1 ORDER BY created_at DESC LIMIT ?2";
            let mut stmt = conn.prepare(query)?;
            let pattern = format!("%\"status\":\"{}%", status);
            let rows = stmt.query_map(params![pattern, limit], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, Option<String>>(8)?,
                ))
            })?;

            for row in rows.flatten() {
                if let Ok(job) = self.tuple_to_job(row) {
                    jobs.push(job);
                }
            }
        } else {
            let query = "SELECT id, action_json, params_json, status_json, images_json, model, created_at, updated_at, parent_id FROM jobs ORDER BY created_at DESC LIMIT ?1";
            let mut stmt = conn.prepare(query)?;
            let rows = stmt.query_map(params![limit], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, Option<String>>(8)?,
                ))
            })?;

            for row in rows.flatten() {
                if let Ok(job) = self.tuple_to_job(row) {
                    jobs.push(job);
                }
            }
        }

        Ok(jobs)
    }

    /// Delete a job
    pub fn delete_job(&self, id: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let deleted = conn.execute("DELETE FROM jobs WHERE id = ?1", params![id])?;
        Ok(deleted > 0)
    }

    /// Get job count
    pub fn count_jobs(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM jobs", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Convert a database row to a Job
    fn row_to_job(&self, row: &rusqlite::Row) -> Result<Job> {
        let action_json: String = row.get(1)?;
        let params_json: String = row.get(2)?;
        let status_json: String = row.get(3)?;
        let images_json: String = row.get(4)?;
        let created_at_str: String = row.get(6)?;
        let updated_at_str: String = row.get(7)?;

        Ok(Job {
            id: row.get(0)?,
            action: serde_json::from_str(&action_json)?,
            params: serde_json::from_str(&params_json)?,
            status: serde_json::from_str(&status_json)?,
            images: serde_json::from_str(&images_json)?,
            model: row.get(5)?,
            created_at: DateTime::parse_from_rfc3339(&created_at_str)?.with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&updated_at_str)?.with_timezone(&Utc),
            parent_id: row.get(8)?,
        })
    }

    /// Convert a tuple to a Job
    fn tuple_to_job(&self, row: (String, String, String, String, String, String, String, String, Option<String>)) -> Result<Job> {
        Ok(Job {
            id: row.0,
            action: serde_json::from_str(&row.1)?,
            params: serde_json::from_str(&row.2)?,
            status: serde_json::from_str(&row.3)?,
            images: serde_json::from_str(&row.4)?,
            model: row.5,
            created_at: DateTime::parse_from_rfc3339(&row.6)?.with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.7)?.with_timezone(&Utc),
            parent_id: row.8,
        })
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self {
            conn: Arc::clone(&self.conn),
        }
    }
}
