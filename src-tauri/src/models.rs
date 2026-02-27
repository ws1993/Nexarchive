use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LLMConfig {
    pub base_uri: String,
    pub api_key_encrypted: String,
    pub model: String,
    pub timeout_sec: u64,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            base_uri: "https://api.openai.com/v1".to_string(),
            api_key_encrypted: String::new(),
            model: "gpt-4o-mini".to_string(),
            timeout_sec: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MineruConfig {
    pub enabled: bool,
    pub base_uri: String,
    pub api_token_encrypted: String,
    pub model_version: String,
    pub language: String,
    pub is_ocr: bool,
    pub timeout_sec: u64,
    pub max_wait_sec: u64,
}

impl Default for MineruConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            base_uri: "https://mineru.net/api/v4".to_string(),
            api_token_encrypted: String::new(),
            model_version: "vlm".to_string(),
            language: "ch".to_string(),
            is_ocr: true,
            timeout_sec: 60,
            max_wait_sec: 300,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RetentionConfig {
    pub max_log_file_mb: usize,
    pub max_log_files: usize,
    pub max_db_logs: usize,
    pub db_log_retention_days: i64,
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self {
            max_log_file_mb: 10,
            max_log_files: 5,
            max_db_logs: 10_000,
            db_log_retention_days: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UpdaterConfig {
    pub auto_check_on_startup: bool,
    pub proxy_enabled: bool,
    pub proxy_url_encrypted: String,
}

impl Default for UpdaterConfig {
    fn default() -> Self {
        Self {
            auto_check_on_startup: true,
            proxy_enabled: false,
            proxy_url_encrypted: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub inbox_path: String,
    pub archive_root_path: String,
    pub autostart: bool,
    pub run_in_background: bool,
    pub schedule_hours: u64,
    pub llm: LLMConfig,
    pub mineru: MineruConfig,
    pub retention: RetentionConfig,
    pub updater: UpdaterConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            inbox_path: String::new(),
            archive_root_path: String::new(),
            autostart: false,
            run_in_background: true,
            schedule_hours: 24,
            llm: LLMConfig::default(),
            mineru: MineruConfig::default(),
            retention: RetentionConfig::default(),
            updater: UpdaterConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitPreviewItem {
    pub code: String,
    pub folder: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<InitPreviewItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TriggerType {
    Manual,
    Schedule,
}

impl TriggerType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Schedule => "schedule",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRecord {
    pub job_id: String,
    pub trigger_type: String,
    pub start_at: String,
    pub end_at: Option<String>,
    pub status: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTaskRecord {
    pub task_id: String,
    pub job_id: String,
    pub src_path: String,
    pub hash: String,
    pub extract_status: String,
    pub classify_status: String,
    pub rename_status: String,
    pub archive_status: String,
    pub final_path: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub recycle_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    pub timestamp: String,
    pub level: String,
    pub job_id: Option<String>,
    pub task_id: Option<String>,
    pub stage: String,
    pub message: String,
    pub payload_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagedResult<T> {
    pub total: usize,
    pub items: Vec<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFilters {
    pub level: Option<String>,
    pub stage: Option<String>,
    pub job_id: Option<String>,
    pub status: Option<String>,
    pub query: Option<String>,
    pub page: usize,
    pub page_size: usize,
}

impl Default for LogFilters {
    fn default() -> Self {
        Self {
            level: None,
            stage: None,
            job_id: None,
            status: None,
            query: None,
            page: 1,
            page_size: 50,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmClassification {
    pub doc_type: String,
    pub core_title: String,
    pub tags: Vec<String>,
    pub people: Vec<String>,
    pub note: Option<String>,
    pub target_top_dir: String,
    pub target_subpath: String,
    pub confidence: f32,
}
