import { invoke } from "@tauri-apps/api/core";
import type {
  AppConfig,
  FileTaskRecord,
  InitPreviewItem,
  JobRecord,
  LogEvent,
  LogFilters,
  PagedResult
} from "./types";

export const api = {
  initSystem: (inboxPath: string, archiveRootPath: string) =>
    invoke<boolean>("init_system", {
      inbox_path: inboxPath,
      archive_root_path: archiveRootPath
    }),

  getInitPreview: () => invoke<InitPreviewItem[]>("get_init_preview"),

  saveSettings: (config: AppConfig) =>
    invoke<boolean>("save_settings", { config }),

  loadSettings: () => invoke<AppConfig>("load_settings"),

  testLlmConnection: () => invoke<boolean>("test_llm_connection"),
  testMineruConnection: () => invoke<boolean>("test_mineru_connection"),

  runJobOnce: () => invoke<string>("run_job_once"),

  getJobs: (
    page: number,
    pageSize: number,
    status?: string,
    dateRange?: [string, string]
  ) =>
    invoke<PagedResult<JobRecord>>("get_jobs", {
      page,
      page_size: pageSize,
      status,
      date_range: dateRange
    }),

  getFileTasks: (jobId: string, status?: string) =>
    invoke<FileTaskRecord[]>("get_file_tasks", { job_id: jobId, status }),

  getLogs: (filters: LogFilters) => invoke<PagedResult<LogEvent>>("get_logs", { filters }),

  restoreFromRecycleBin: (taskId: string) =>
    invoke<boolean>("restore_from_recycle_bin", { task_id: taskId })
};
