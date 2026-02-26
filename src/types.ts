export type TriggerType = "manual" | "schedule";

export interface LLMConfig {
  base_uri: string;
  api_key_encrypted: string;
  model: string;
  timeout_sec: number;
}

export interface MineruConfig {
  enabled: boolean;
  base_uri: string;
  api_token_encrypted: string;
  model_version: string;
  language: string;
  is_ocr: boolean;
  timeout_sec: number;
  max_wait_sec: number;
}

export interface RetentionConfig {
  max_log_file_mb: number;
  max_log_files: number;
  max_db_logs: number;
  db_log_retention_days: number;
}

export interface AppConfig {
  inbox_path: string;
  archive_root_path: string;
  autostart: boolean;
  run_in_background: boolean;
  schedule_hours: number;
  llm: LLMConfig;
  mineru: MineruConfig;
  retention: RetentionConfig;
}

export interface InitPreviewItem {
  code: string;
  folder: string;
  children?: InitPreviewItem[];
}

export interface JobRecord {
  job_id: string;
  trigger_type: TriggerType;
  start_at: string;
  end_at?: string;
  status: string;
  summary: string;
}

export interface FileTaskRecord {
  task_id: string;
  job_id: string;
  src_path: string;
  hash: string;
  extract_status: string;
  classify_status: string;
  rename_status: string;
  archive_status: string;
  final_path?: string;
  error_code?: string;
  error_message?: string;
  recycle_path?: string;
}

export interface LogFilters {
  level?: string;
  stage?: string;
  job_id?: string;
  status?: string;
  query?: string;
  page: number;
  page_size: number;
}

export interface LogEvent {
  timestamp: string;
  level: string;
  job_id?: string;
  task_id?: string;
  stage: string;
  message: string;
  payload_json?: string;
}

export interface PagedResult<T> {
  total: number;
  items: T[];
}

export const defaultConfig: AppConfig = {
  inbox_path: "",
  archive_root_path: "",
  autostart: false,
  run_in_background: true,
  schedule_hours: 24,
  llm: {
    base_uri: "https://api.openai.com/v1",
    api_key_encrypted: "",
    model: "gpt-4o-mini",
    timeout_sec: 30
  },
  mineru: {
    enabled: false,
    base_uri: "https://mineru.net/api/v4",
    api_token_encrypted: "",
    model_version: "vlm",
    language: "ch",
    is_ocr: true,
    timeout_sec: 60,
    max_wait_sec: 300
  },
  retention: {
    max_log_file_mb: 10,
    max_log_files: 5,
    max_db_logs: 10000,
    db_log_retention_days: 30
  }
};
