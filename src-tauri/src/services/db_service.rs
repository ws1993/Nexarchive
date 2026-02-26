use std::{path::PathBuf, sync::Mutex};

use anyhow::Result;
use chrono::{Duration, Utc};
use rusqlite::{params, Connection};

use crate::models::{FileTaskRecord, JobRecord, LogEvent, LogFilters, PagedResult};

pub struct DbService {
    conn: Mutex<Connection>,
}

impl DbService {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            r#"
      PRAGMA journal_mode=WAL;
      PRAGMA synchronous=NORMAL;

      CREATE TABLE IF NOT EXISTS jobs (
        job_id TEXT PRIMARY KEY,
        trigger_type TEXT NOT NULL,
        start_at TEXT NOT NULL,
        end_at TEXT,
        status TEXT NOT NULL,
        summary TEXT NOT NULL
      );

      CREATE TABLE IF NOT EXISTS file_tasks (
        task_id TEXT PRIMARY KEY,
        job_id TEXT NOT NULL,
        src_path TEXT NOT NULL,
        hash TEXT NOT NULL,
        extract_status TEXT NOT NULL,
        classify_status TEXT NOT NULL,
        rename_status TEXT NOT NULL,
        archive_status TEXT NOT NULL,
        final_path TEXT,
        error_code TEXT,
        error_message TEXT,
        recycle_path TEXT,
        created_at TEXT NOT NULL,
        FOREIGN KEY(job_id) REFERENCES jobs(job_id)
      );

      CREATE TABLE IF NOT EXISTS logs (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        timestamp TEXT NOT NULL,
        level TEXT NOT NULL,
        job_id TEXT,
        task_id TEXT,
        stage TEXT NOT NULL,
        message TEXT NOT NULL,
        payload_json TEXT
      );

      CREATE INDEX IF NOT EXISTS idx_jobs_start_at ON jobs(start_at DESC);
      CREATE INDEX IF NOT EXISTS idx_tasks_job_id ON file_tasks(job_id);
      CREATE INDEX IF NOT EXISTS idx_tasks_hash ON file_tasks(hash);
      CREATE INDEX IF NOT EXISTS idx_logs_time ON logs(timestamp DESC);
      CREATE INDEX IF NOT EXISTS idx_logs_job ON logs(job_id);
      "#,
        )?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn insert_job(&self, job: &JobRecord) -> Result<()> {
        let conn = self.conn.lock().expect("db poisoned");
        conn.execute(
      "INSERT INTO jobs(job_id, trigger_type, start_at, end_at, status, summary) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
      params![
        job.job_id,
        job.trigger_type,
        job.start_at,
        job.end_at,
        job.status,
        job.summary
      ],
    )?;
        Ok(())
    }

    pub fn finish_job(&self, job_id: &str, status: &str, summary: &str) -> Result<()> {
        let conn = self.conn.lock().expect("db poisoned");
        conn.execute(
            "UPDATE jobs SET end_at = ?1, status = ?2, summary = ?3 WHERE job_id = ?4",
            params![Utc::now().to_rfc3339(), status, summary, job_id],
        )?;
        Ok(())
    }

    pub fn insert_file_task(&self, task: &FileTaskRecord) -> Result<()> {
        let conn = self.conn.lock().expect("db poisoned");
        conn.execute(
            r#"
      INSERT INTO file_tasks(
        task_id, job_id, src_path, hash, extract_status, classify_status, rename_status,
        archive_status, final_path, error_code, error_message, recycle_path, created_at
      ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
      "#,
            params![
                task.task_id,
                task.job_id,
                task.src_path,
                task.hash,
                task.extract_status,
                task.classify_status,
                task.rename_status,
                task.archive_status,
                task.final_path,
                task.error_code,
                task.error_message,
                task.recycle_path,
                Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn update_file_task(&self, task: &FileTaskRecord) -> Result<()> {
        let conn = self.conn.lock().expect("db poisoned");
        conn.execute(
            r#"
      UPDATE file_tasks
      SET extract_status = ?1,
          classify_status = ?2,
          rename_status = ?3,
          archive_status = ?4,
          final_path = ?5,
          error_code = ?6,
          error_message = ?7,
          recycle_path = ?8
      WHERE task_id = ?9
      "#,
            params![
                task.extract_status,
                task.classify_status,
                task.rename_status,
                task.archive_status,
                task.final_path,
                task.error_code,
                task.error_message,
                task.recycle_path,
                task.task_id,
            ],
        )?;
        Ok(())
    }

    pub fn is_duplicate_success(&self, fingerprint: &str) -> Result<bool> {
        let conn = self.conn.lock().expect("db poisoned");
        let mut stmt = conn.prepare(
            "SELECT 1 FROM file_tasks WHERE hash = ?1 AND archive_status = 'success' LIMIT 1",
        )?;

        let mut rows = stmt.query(params![fingerprint])?;
        Ok(rows.next()?.is_some())
    }

    pub fn insert_log(&self, log: &LogEvent) -> Result<()> {
        let conn = self.conn.lock().expect("db poisoned");
        conn.execute(
            r#"
      INSERT INTO logs(timestamp, level, job_id, task_id, stage, message, payload_json)
      VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
      "#,
            params![
                log.timestamp,
                log.level,
                log.job_id,
                log.task_id,
                log.stage,
                log.message,
                log.payload_json,
            ],
        )?;
        Ok(())
    }

    pub fn get_jobs(
        &self,
        page: usize,
        page_size: usize,
        status: Option<String>,
        date_range: Option<Vec<String>>,
    ) -> Result<PagedResult<JobRecord>> {
        let page = page.max(1);
        let page_size = page_size.max(1).min(500);
        let offset = (page - 1) * page_size;

        let (from, to) = normalize_date_range(date_range);

        let conn = self.conn.lock().expect("db poisoned");

        let total: usize = conn.query_row(
      "SELECT COUNT(1) FROM jobs WHERE (?1 IS NULL OR status = ?1) AND (?2 IS NULL OR start_at >= ?2) AND (?3 IS NULL OR start_at <= ?3)",
      params![status, from, to],
      |row| row.get(0),
    )?;

        let mut stmt = conn.prepare(
      "SELECT job_id, trigger_type, start_at, end_at, status, summary FROM jobs WHERE (?1 IS NULL OR status = ?1) AND (?2 IS NULL OR start_at >= ?2) AND (?3 IS NULL OR start_at <= ?3) ORDER BY start_at DESC LIMIT ?4 OFFSET ?5",
    )?;

        let items = stmt
            .query_map(
                params![status, from, to, page_size as i64, offset as i64],
                |row| {
                    Ok(JobRecord {
                        job_id: row.get(0)?,
                        trigger_type: row.get(1)?,
                        start_at: row.get(2)?,
                        end_at: row.get(3)?,
                        status: row.get(4)?,
                        summary: row.get(5)?,
                    })
                },
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(PagedResult { total, items })
    }

    pub fn get_file_tasks(
        &self,
        job_id: &str,
        status: Option<String>,
    ) -> Result<Vec<FileTaskRecord>> {
        let conn = self.conn.lock().expect("db poisoned");
        let mut stmt = conn.prepare(
            r#"
      SELECT task_id, job_id, src_path, hash, extract_status, classify_status, rename_status,
             archive_status, final_path, error_code, error_message, recycle_path
      FROM file_tasks
      WHERE job_id = ?1
        AND (
          ?2 IS NULL
          OR archive_status = ?2
          OR classify_status = ?2
          OR extract_status = ?2
        )
      ORDER BY created_at DESC
      "#,
        )?;

        let rows = stmt
            .query_map(params![job_id, status], |row| {
                Ok(FileTaskRecord {
                    task_id: row.get(0)?,
                    job_id: row.get(1)?,
                    src_path: row.get(2)?,
                    hash: row.get(3)?,
                    extract_status: row.get(4)?,
                    classify_status: row.get(5)?,
                    rename_status: row.get(6)?,
                    archive_status: row.get(7)?,
                    final_path: row.get(8)?,
                    error_code: row.get(9)?,
                    error_message: row.get(10)?,
                    recycle_path: row.get(11)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    pub fn get_file_task_by_id(&self, task_id: &str) -> Result<Option<FileTaskRecord>> {
        let conn = self.conn.lock().expect("db poisoned");
        let mut stmt = conn.prepare(
            r#"
      SELECT task_id, job_id, src_path, hash, extract_status, classify_status, rename_status,
             archive_status, final_path, error_code, error_message, recycle_path
      FROM file_tasks
      WHERE task_id = ?1
      LIMIT 1
      "#,
        )?;

        let mut rows = stmt.query(params![task_id])?;
        if let Some(row) = rows.next()? {
            return Ok(Some(FileTaskRecord {
                task_id: row.get(0)?,
                job_id: row.get(1)?,
                src_path: row.get(2)?,
                hash: row.get(3)?,
                extract_status: row.get(4)?,
                classify_status: row.get(5)?,
                rename_status: row.get(6)?,
                archive_status: row.get(7)?,
                final_path: row.get(8)?,
                error_code: row.get(9)?,
                error_message: row.get(10)?,
                recycle_path: row.get(11)?,
            }));
        }

        Ok(None)
    }

    pub fn get_logs(&self, filters: &LogFilters) -> Result<PagedResult<LogEvent>> {
        let page = filters.page.max(1);
        let page_size = filters.page_size.max(1).min(500);
        let offset = (page - 1) * page_size;
        let q = filters.query.as_ref().map(|s| format!("%{}%", s));

        let conn = self.conn.lock().expect("db poisoned");

        let total: usize = conn.query_row(
      "SELECT COUNT(1) FROM logs WHERE (?1 IS NULL OR level = ?1) AND (?2 IS NULL OR stage = ?2) AND (?3 IS NULL OR job_id = ?3) AND (?4 IS NULL OR message LIKE ?4)",
      params![filters.level, filters.stage, filters.job_id, q],
      |row| row.get(0),
    )?;

        let mut stmt = conn.prepare(
      "SELECT timestamp, level, job_id, task_id, stage, message, payload_json FROM logs WHERE (?1 IS NULL OR level = ?1) AND (?2 IS NULL OR stage = ?2) AND (?3 IS NULL OR job_id = ?3) AND (?4 IS NULL OR message LIKE ?4) ORDER BY timestamp DESC LIMIT ?5 OFFSET ?6",
    )?;

        let items = stmt
            .query_map(
                params![
                    filters.level,
                    filters.stage,
                    filters.job_id,
                    q,
                    page_size as i64,
                    offset as i64
                ],
                |row| {
                    Ok(LogEvent {
                        timestamp: row.get(0)?,
                        level: row.get(1)?,
                        job_id: row.get(2)?,
                        task_id: row.get(3)?,
                        stage: row.get(4)?,
                        message: row.get(5)?,
                        payload_json: row.get(6)?,
                    })
                },
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(PagedResult { total, items })
    }

    pub fn cleanup_logs(&self, retention_days: i64, max_logs: usize) -> Result<()> {
        let conn = self.conn.lock().expect("db poisoned");

        if retention_days > 0 {
            let threshold = (Utc::now() - Duration::days(retention_days)).to_rfc3339();
            conn.execute("DELETE FROM logs WHERE timestamp < ?1", params![threshold])?;
        }

        if max_logs > 0 {
            conn.execute(
        "DELETE FROM logs WHERE id NOT IN (SELECT id FROM logs ORDER BY timestamp DESC LIMIT ?1)",
        params![max_logs as i64],
      )?;
        }

        Ok(())
    }
}

fn normalize_date_range(date_range: Option<Vec<String>>) -> (Option<String>, Option<String>) {
    if let Some(range) = date_range {
        if range.len() >= 2 {
            return (Some(range[0].clone()), Some(range[1].clone()));
        }
    }
    (None, None)
}
