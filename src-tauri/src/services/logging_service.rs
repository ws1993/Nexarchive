use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
};

use anyhow::Result;
use chrono::{FixedOffset, Utc};

use crate::{
    models::{LogEvent, RetentionConfig},
    services::db_service::DbService,
};

pub struct LoggingService {
    db: Arc<DbService>,
    current_log_path: PathBuf,
    retention: RwLock<RetentionConfig>,
    guard: Mutex<()>,
}

impl LoggingService {
    pub fn new(log_dir: PathBuf, db: Arc<DbService>, retention: RetentionConfig) -> Result<Self> {
        fs::create_dir_all(&log_dir)?;

        Ok(Self {
            db,
            current_log_path: log_dir.join("app.log"),
            retention: RwLock::new(retention),
            guard: Mutex::new(()),
        })
    }

    pub fn set_retention(&self, retention: RetentionConfig) {
        if let Ok(mut r) = self.retention.write() {
            *r = retention;
        }
    }

    pub fn info(
        &self,
        stage: &str,
        message: &str,
        job_id: Option<&str>,
        task_id: Option<&str>,
        payload: Option<serde_json::Value>,
    ) {
        let _ = self.log("INFO", stage, message, job_id, task_id, payload);
    }

    pub fn warn(
        &self,
        stage: &str,
        message: &str,
        job_id: Option<&str>,
        task_id: Option<&str>,
        payload: Option<serde_json::Value>,
    ) {
        let _ = self.log("WARN", stage, message, job_id, task_id, payload);
    }

    pub fn error(
        &self,
        stage: &str,
        message: &str,
        job_id: Option<&str>,
        task_id: Option<&str>,
        payload: Option<serde_json::Value>,
    ) {
        let _ = self.log("ERROR", stage, message, job_id, task_id, payload);
    }

    pub fn log(
        &self,
        level: &str,
        stage: &str,
        message: &str,
        job_id: Option<&str>,
        task_id: Option<&str>,
        payload: Option<serde_json::Value>,
    ) -> Result<()> {
        let event = LogEvent {
            timestamp: beijing_now_rfc3339(),
            level: level.to_string(),
            job_id: job_id.map(|s| s.to_string()),
            task_id: task_id.map(|s| s.to_string()),
            stage: stage.to_string(),
            message: message.to_string(),
            payload_json: payload.map(|p| p.to_string()),
        };

        self.db.insert_log(&event)?;
        self.append_file_log(&event)?;
        Ok(())
    }

    pub fn cleanup_db_logs(&self) -> Result<()> {
        let retention = self.retention.read().expect("retention poisoned").clone();
        self.db
            .cleanup_logs(retention.db_log_retention_days, retention.max_db_logs)
    }

    fn append_file_log(&self, event: &LogEvent) -> Result<()> {
        let _guard = self.guard.lock().expect("log guard poisoned");

        self.rotate_if_needed()?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.current_log_path)?;

        let line = format!(
            "{} [{}] [{}] job={} task={} {} {}\n",
            event.timestamp,
            event.level,
            event.stage,
            event.job_id.clone().unwrap_or_else(|| "-".to_string()),
            event.task_id.clone().unwrap_or_else(|| "-".to_string()),
            event.message,
            event.payload_json.clone().unwrap_or_else(|| "".to_string()),
        );

        file.write_all(line.as_bytes())?;
        Ok(())
    }

    fn rotate_if_needed(&self) -> Result<()> {
        let retention = self.retention.read().expect("retention poisoned").clone();
        let max_bytes = (retention.max_log_file_mb.max(1) as u64) * 1024 * 1024;

        if let Ok(metadata) = fs::metadata(&self.current_log_path) {
            if metadata.len() < max_bytes {
                return Ok(());
            }

            let max_files = retention.max_log_files.max(1);
            let parent = self
                .current_log_path
                .parent()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."));

            let oldest = parent.join(format!("app.{max_files}.log"));
            if oldest.exists() {
                let _ = fs::remove_file(oldest);
            }

            for idx in (1..max_files).rev() {
                let from = parent.join(format!("app.{idx}.log"));
                let to = parent.join(format!("app.{}.log", idx + 1));
                if from.exists() {
                    let _ = fs::rename(from, to);
                }
            }

            let first = parent.join("app.1.log");
            if self.current_log_path.exists() {
                let _ = fs::rename(&self.current_log_path, first);
            }
        }

        Ok(())
    }
}

fn beijing_now_rfc3339() -> String {
    // Force log timestamps to China Standard Time (UTC+8), regardless of host timezone.
    let offset = FixedOffset::east_opt(8 * 60 * 60).expect("valid UTC+8 offset");
    Utc::now().with_timezone(&offset).to_rfc3339()
}
