use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU16, Ordering},
        Arc, RwLock,
    },
};

use anyhow::{Context, Result};
use chrono::{DateTime, FixedOffset, Utc};
use serde_json::json;
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;
use walkdir::WalkDir;

use crate::{
    constants::{top_dir_name, SUPPORTED_EXTENSIONS},
    models::{AppConfig, FileTaskRecord, JobRecord, LogFilters, PagedResult, TriggerType},
    services::{
        config_service::ConfigService, db_service::DbService,
        extractor_service::{ExtractedContent, ExtractorService},
        init_service::InitService, llm_service::LlmService, logging_service::LoggingService,
        mineru_service::MineruService, scheduler_service::SchedulerService,
        system_service::SystemService,
    },
    utils::path_utils::{
        ensure_parent, sanitize_filename_component, sanitize_relative_subpath, unique_path,
    },
};

pub struct AppState {
    config_service: ConfigService,
    db: Arc<DbService>,
    logger: Arc<LoggingService>,
    init_service: InitService,
    extractor: ExtractorService,
    mineru: MineruService,
    llm: LlmService,
    pub scheduler: SchedulerService,
    config: RwLock<AppConfig>,
    run_guard: Mutex<()>,
    recycle_dir: PathBuf,
}

#[derive(Debug, Clone, Copy)]
enum ProcessOutcome {
    Success,
    Review,
    Skipped,
    Failed,
}

static ID_SEQUENCE: AtomicU16 = AtomicU16::new(0);

impl AppState {
    pub fn new() -> Result<Arc<Self>> {
        let app_data_dir = resolve_app_data_dir()?;

        fs::create_dir_all(&app_data_dir)?;
        fs::create_dir_all(app_data_dir.join("logs"))?;
        fs::create_dir_all(app_data_dir.join("recycle"))?;

        let config_service = ConfigService::new(app_data_dir.clone());
        let config = config_service.load_config().unwrap_or_default();

        let db = Arc::new(DbService::new(app_data_dir.join("nexarchive.db"))?);
        let logger = Arc::new(LoggingService::new(
            app_data_dir.join("logs"),
            db.clone(),
            config.retention.clone(),
        )?);

        SystemService::apply_autostart(config.autostart)?;

        let state = Arc::new(Self {
            config_service,
            db,
            logger,
            init_service: InitService,
            extractor: ExtractorService::new(),
            mineru: MineruService::new(),
            llm: LlmService::new(),
            scheduler: SchedulerService::new(),
            config: RwLock::new(config),
            run_guard: Mutex::new(()),
            recycle_dir: app_data_dir.join("recycle"),
        });

        Ok(state)
    }

    pub async fn bootstrap_scheduler(self: &Arc<Self>) {
        let config = self.load_settings().await;
        self.scheduler
            .reschedule(self.clone(), config.schedule_hours)
            .await;
    }

    pub async fn get_init_preview(&self) -> Vec<crate::models::InitPreviewItem> {
        self.init_service.get_preview()
    }

    pub async fn init_system(&self, inbox_path: String, archive_root_path: String) -> Result<bool> {
        let inbox = PathBuf::from(inbox_path.clone());
        let archive = PathBuf::from(archive_root_path.clone());

        self.init_service.init_system(&inbox, &archive)?;

        let mut config = self.current_config();
        config.inbox_path = inbox_path;
        config.archive_root_path = archive_root_path;

        self.config_service.save_config(&config)?;
        if let Ok(mut guard) = self.config.write() {
            *guard = config;
        }

        self.logger.info(
            "初始化",
            "目录结构初始化完成",
            None,
            None,
            Some(json!({"inbox": inbox.display().to_string(), "archive_root": archive.display().to_string()})),
        );

        Ok(true)
    }

    pub async fn save_settings(self: &Arc<Self>, config: AppConfig) -> Result<bool> {
        if config.schedule_hours == 0 {
            anyhow::bail!("schedule_hours must be >= 1");
        }

        self.config_service.save_config(&config)?;
        SystemService::apply_autostart(config.autostart)?;

        if let Ok(mut guard) = self.config.write() {
            *guard = config.clone();
        }

        self.logger.set_retention(config.retention.clone());
        self.scheduler
            .reschedule(self.clone(), config.schedule_hours)
            .await;

        self.logger
            .info("设置", "设置已保存", None, None, None);

        Ok(true)
    }

    pub async fn load_settings(&self) -> AppConfig {
        self.current_config()
    }

    pub fn run_in_background_enabled(&self) -> bool {
        self.current_config().run_in_background
    }

    pub async fn test_llm_connection(&self) -> Result<bool> {
        let config = self.current_config();
        self.llm.test_connection(&config).await?;
        self.logger
            .info("模型", "连通性测试成功", None, None, None);
        Ok(true)
    }

    pub async fn test_mineru_connection(&self) -> Result<bool> {
        let config = self.current_config();
        self.mineru.test_connection(&config).await?;
        self.logger
            .info("MinerU", "连通性测试成功", None, None, None);
        Ok(true)
    }

    pub async fn run_job_once(self: &Arc<Self>) -> Result<String> {
        self.run_job(TriggerType::Manual).await
    }

    pub fn get_jobs(
        &self,
        page: usize,
        page_size: usize,
        status: Option<String>,
        date_range: Option<Vec<String>>,
    ) -> Result<PagedResult<JobRecord>> {
        self.db.get_jobs(page, page_size, status, date_range)
    }

    pub fn get_file_tasks(
        &self,
        job_id: String,
        status: Option<String>,
    ) -> Result<Vec<FileTaskRecord>> {
        self.db.get_file_tasks(&job_id, status)
    }

    pub fn get_logs(&self, filters: LogFilters) -> Result<PagedResult<crate::models::LogEvent>> {
        self.db.get_logs(&filters)
    }

    pub fn restore_from_recycle_bin(&self, task_id: String) -> Result<bool> {
        let task = self
            .db
            .get_file_task_by_id(&task_id)?
            .context("task not found")?;
        let recycle_path = task.recycle_path.context("task has no recycle path")?;

        let source = PathBuf::from(recycle_path);
        if !source.exists() {
            anyhow::bail!("recycle source does not exist");
        }

        let target = unique_path(&PathBuf::from(task.src_path));
        ensure_parent(&target)?;
        move_file(&source, &target)?;

        self.logger.info(
            "回收区",
            "文件已从回收区恢复",
            Some(&task.job_id),
            Some(&task.task_id),
            Some(json!({"target": target.display().to_string()})),
        );

        Ok(true)
    }

    pub async fn run_job(self: &Arc<Self>, trigger: TriggerType) -> Result<String> {
        let _job_guard = self.run_guard.lock().await;

        let config = self.current_config();
        if config.inbox_path.trim().is_empty() || config.archive_root_path.trim().is_empty() {
            anyhow::bail!("inbox_path and archive_root_path must be configured");
        }

        let job_id = next_time_id();
        let job = JobRecord {
            job_id: job_id.clone(),
            trigger_type: trigger.as_str().to_string(),
            start_at: beijing_now_rfc3339(),
            end_at: None,
            status: "running".to_string(),
            summary: "执行中".to_string(),
        };

        self.db.insert_job(&job)?;
        self.logger.info(
            "任务",
            "任务开始执行",
            Some(&job_id),
            None,
            Some(json!({"trigger": trigger.as_str()})),
        );

        let mut scanned_files = Vec::new();
        let inbox_path = PathBuf::from(&config.inbox_path);
        fs::create_dir_all(&inbox_path)?;

        for entry in WalkDir::new(&inbox_path)
            .min_depth(1)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path().to_path_buf();
            if path.starts_with(inbox_path.join("_Failed"))
                || path.starts_with(inbox_path.join("_Review"))
            {
                continue;
            }

            let ext = path
                .extension()
                .and_then(|v| v.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();

            if SUPPORTED_EXTENSIONS.contains(&ext.as_str()) {
                scanned_files.push(path);
            }
        }

        let mut success = 0usize;
        let mut review = 0usize;
        let mut skipped = 0usize;
        let mut failed = 0usize;

        for file_path in scanned_files {
            match self.process_one_file(&job_id, &file_path, &config).await {
                Ok(ProcessOutcome::Success) => success += 1,
                Ok(ProcessOutcome::Review) => review += 1,
                Ok(ProcessOutcome::Skipped) => skipped += 1,
                Ok(ProcessOutcome::Failed) => failed += 1,
                Err(err) => {
                    failed += 1;
                    self.logger.error(
                        "流程",
                        "处理流程出现未预期错误",
                        Some(&job_id),
                        None,
                        Some(json!({"file": file_path.display().to_string(), "error": err.to_string()})),
                    );
                }
            }
        }

        let status = if failed > 0 { "partial" } else { "success" };
        let summary = format!("成功={success}，待复核={review}，已跳过={skipped}，失败={failed}");

        self.db.finish_job(&job_id, status, &summary)?;
        let _ = self.logger.cleanup_db_logs();

        self.logger.info(
            "任务",
            "任务执行完成",
            Some(&job_id),
            None,
            Some(json!({"status": status, "summary": summary})),
        );

        Ok(job_id)
    }

    async fn process_one_file(
        &self,
        job_id: &str,
        file_path: &Path,
        config: &AppConfig,
    ) -> Result<ProcessOutcome> {
        let task_id = next_time_id();
        let fingerprint = build_fingerprint(file_path)?;

        let mut task = FileTaskRecord {
            task_id: task_id.clone(),
            job_id: job_id.to_string(),
            src_path: file_path.display().to_string(),
            hash: fingerprint.clone(),
            extract_status: "pending".to_string(),
            classify_status: "pending".to_string(),
            rename_status: "pending".to_string(),
            archive_status: "pending".to_string(),
            final_path: None,
            error_code: None,
            error_message: None,
            recycle_path: None,
        };

        self.db.insert_file_task(&task)?;

        if self.db.is_duplicate_success(&fingerprint)? {
            task.extract_status = "skipped".to_string();
            task.classify_status = "skipped".to_string();
            task.rename_status = "skipped".to_string();
            task.archive_status = "skipped".to_string();
            self.db.update_file_task(&task)?;

            self.logger.info(
                "去重",
                "检测到重复文件，已跳过",
                Some(job_id),
                Some(&task_id),
                Some(json!({"file": file_path.display().to_string()})),
            );
            return Ok(ProcessOutcome::Skipped);
        }

        let extracted = match self
            .extract_with_fallback(job_id, &task_id, file_path, config)
            .await
        {
            Ok(v) => {
                task.extract_status = "success".to_string();
                self.db.update_file_task(&task)?;
                v
            }
            Err(err) => {
                return self.fail_task(
                    task,
                    file_path,
                    config,
                    "extract_failed",
                    &err.to_string(),
                    "提取",
                );
            }
        };

        let file_name = file_path
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("unknown");

        let classification = {
            let mut last_error: Option<anyhow::Error> = None;
            let mut result = None;
            for _ in 0..2 {
                match self
                    .llm
                    .classify(
                        config,
                        file_name,
                        &extracted.text,
                        extracted.image_data_url.as_deref(),
                    )
                    .await
                {
                    Ok(v) => {
                        result = Some(v);
                        break;
                    }
                    Err(err) => {
                        last_error = Some(err);
                    }
                }
            }

            if let Some(v) = result {
                v
            } else {
                let err = last_error.context("llm classify failed")?;
                return self.fail_task(
                    task,
                    file_path,
                    config,
                    "classify_failed",
                    &err.to_string(),
                    "分类",
                );
            }
        };

        task.classify_status = "success".to_string();
        self.db.update_file_task(&task)?;

        if classification.confidence < 0.70 {
            let review_dir = PathBuf::from(&config.inbox_path).join("_Review");
            fs::create_dir_all(&review_dir)?;

            let review_target = unique_path(
                &review_dir.join(
                    file_path
                        .file_name()
                        .map(|s| s.to_os_string())
                        .unwrap_or_else(|| "unknown.file".into()),
                ),
            );

            move_file(file_path, &review_target)?;
            task.rename_status = "review".to_string();
            task.archive_status = "review".to_string();
            task.final_path = Some(review_target.display().to_string());
            self.db.update_file_task(&task)?;

            self.logger.warn(
                "分类",
                "置信度较低，已移入复核目录",
                Some(job_id),
                Some(&task.task_id),
                Some(json!({"confidence": classification.confidence})),
            );

            return Ok(ProcessOutcome::Review);
        }

        let ext = file_path
            .extension()
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        let file_date = file_best_date(file_path).format("%Y%m%d").to_string();
        let mut new_name = format!(
            "{}_{}_{}",
            file_date,
            sanitize_filename_component(&classification.doc_type),
            sanitize_filename_component(&classification.core_title)
        );

        for tag in &classification.tags {
            new_name.push('#');
            new_name.push_str(&sanitize_filename_component(tag));
        }
        for p in &classification.people {
            new_name.push('@');
            new_name.push_str(&sanitize_filename_component(p));
        }
        if let Some(note) = &classification.note {
            if !note.trim().is_empty() {
                new_name.push('&');
                new_name.push_str(&sanitize_filename_component(note));
            }
        }

        let final_name = if ext.is_empty() {
            new_name
        } else {
            format!("{new_name}.{ext}")
        };

        let top_dir = top_dir_name(&classification.target_top_dir).context("invalid top dir")?;
        let subpath = sanitize_relative_subpath(&classification.target_subpath)
            .context("invalid target subpath")?;

        let final_dir = PathBuf::from(&config.archive_root_path)
            .join(top_dir)
            .join(subpath);
        fs::create_dir_all(&final_dir)?;

        let final_path = unique_path(&final_dir.join(final_name));
        fs::copy(file_path, &final_path)?;

        let original_name = file_path
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("file");
        let recycle_path = unique_path(&self.recycle_dir.join(format!(
            "{}_{}",
            task.task_id,
            sanitize_filename_component(original_name)
        )));

        match move_file(file_path, &recycle_path) {
            Ok(_) => {
                task.recycle_path = Some(recycle_path.display().to_string());
            }
            Err(err) => {
                self.logger.warn(
                    "回收区",
                    "移动到回收区失败，已保留原文件",
                    Some(job_id),
                    Some(&task.task_id),
                    Some(json!({"error": err.to_string()})),
                );
            }
        }

        task.rename_status = "success".to_string();
        task.archive_status = "success".to_string();
        task.final_path = Some(final_path.display().to_string());
        self.db.update_file_task(&task)?;

        self.logger.info(
            "归档",
            "文件归档完成",
            Some(job_id),
            Some(&task.task_id),
            Some(json!({
              "source": task.src_path,
              "target": final_path.display().to_string(),
              "confidence": classification.confidence
            })),
        );

        Ok(ProcessOutcome::Success)
    }

    async fn extract_with_fallback(
        &self,
        job_id: &str,
        task_id: &str,
        file_path: &Path,
        config: &AppConfig,
    ) -> Result<ExtractedContent> {
        let ext = file_path
            .extension()
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        if self.mineru.is_configured(config) && MineruService::supports_extension(&ext) {
            match self.mineru.extract(config, file_path).await {
                Ok(extracted) if !extracted.text.trim().is_empty() => {
                    self.logger.info(
                        "提取",
                        "MinerU 提取成功",
                        Some(job_id),
                        Some(task_id),
                        Some(json!({"file": file_path.display().to_string()})),
                    );
                    return Ok(extracted);
                }
                Ok(_) => {
                    self.logger.warn(
                        "提取",
                        "MinerU 返回空文本，已回退本地解析",
                        Some(job_id),
                        Some(task_id),
                        Some(json!({"file": file_path.display().to_string()})),
                    );
                }
                Err(err) => {
                    self.logger.warn(
                        "提取",
                        "MinerU 提取失败，已回退本地解析",
                        Some(job_id),
                        Some(task_id),
                        Some(json!({"file": file_path.display().to_string(), "error": err.to_string()})),
                    );
                }
            }
        }

        if matches!(ext.as_str(), "doc" | "ppt") && !self.mineru.is_configured(config) {
            anyhow::bail!(
                "file extension .{} requires MinerU parser; please configure token in settings",
                ext
            );
        }

        self.extractor.extract(file_path).await
    }

    fn fail_task(
        &self,
        mut task: FileTaskRecord,
        file_path: &Path,
        config: &AppConfig,
        error_code: &str,
        error_message: &str,
        stage: &str,
    ) -> Result<ProcessOutcome> {
        task.archive_status = "failed".to_string();
        task.error_code = Some(error_code.to_string());
        task.error_message = Some(error_message.to_string());

        let failed_dir = PathBuf::from(&config.inbox_path).join("_Failed");
        fs::create_dir_all(&failed_dir)?;

        if file_path.exists() && !file_path.starts_with(&failed_dir) {
            let failed_target = unique_path(
                &failed_dir.join(
                    file_path
                        .file_name()
                        .map(|v| v.to_os_string())
                        .unwrap_or_else(|| "failed.file".into()),
                ),
            );
            let _ = move_file(file_path, &failed_target);
            task.final_path = Some(failed_target.display().to_string());
        }

        self.db.update_file_task(&task)?;
        self.logger.error(
            stage,
            error_message,
            Some(&task.job_id),
            Some(&task.task_id),
            Some(json!({"error_code": error_code})),
        );

        Ok(ProcessOutcome::Failed)
    }

    fn current_config(&self) -> AppConfig {
        self.config
            .read()
            .map(|v| v.clone())
            .unwrap_or_else(|_| AppConfig::default())
    }
}

fn next_time_id() -> String {
    let now = beijing_now();
    let sequence = ID_SEQUENCE.fetch_add(1, Ordering::Relaxed) % 1000;
    format!(
        "{}{:03}{:03}",
        now.format("%Y%j%H%M%S"),
        now.timestamp_subsec_millis(),
        sequence
    )
}

fn beijing_now_rfc3339() -> String {
    beijing_now().to_rfc3339()
}

fn beijing_now() -> DateTime<FixedOffset> {
    let offset = FixedOffset::east_opt(8 * 60 * 60).expect("valid UTC+8 offset");
    Utc::now().with_timezone(&offset)
}

fn resolve_app_data_dir() -> Result<PathBuf> {
    if let Some(base) = dirs::data_dir() {
        return Ok(base.join("NexArchive"));
    }
    Ok(std::env::current_dir()?.join(".nexarchive-data"))
}

fn build_fingerprint(path: &Path) -> Result<String> {
    let metadata = fs::metadata(path)?;
    let file_len = metadata.len();
    let modified = metadata
        .modified()
        .ok()
        .and_then(|v| v.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|v| v.as_secs())
        .unwrap_or(0);

    let mut hasher = Sha256::new();
    let mut file = fs::File::open(path)?;
    let mut buffer = [0_u8; 8192];

    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    let digest = format!("{:x}", hasher.finalize());
    Ok(format!("{}:{}:{}", digest, file_len, modified))
}

fn file_best_date(path: &Path) -> DateTime<Utc> {
    let now = Utc::now();
    if let Ok(meta) = fs::metadata(path) {
        if let Ok(created) = meta.created() {
            return DateTime::<Utc>::from(created);
        }
        if let Ok(modified) = meta.modified() {
            return DateTime::<Utc>::from(modified);
        }
    }
    now
}

fn move_file(source: &Path, target: &Path) -> Result<()> {
    ensure_parent(target)?;

    match fs::rename(source, target) {
        Ok(_) => Ok(()),
        Err(_) => {
            fs::copy(source, target)?;
            fs::remove_file(source)?;
            Ok(())
        }
    }
}
