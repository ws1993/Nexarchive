import {
  check,
  type CheckOptions,
  type DownloadEvent,
  type Update
} from "@tauri-apps/plugin-updater";

const DEFAULT_CHECK_TIMEOUT_MS = 45_000;

export interface UpdaterCheckInput {
  proxyEnabled: boolean;
  proxyUrl: string;
}

export interface UpdateSummary {
  currentVersion: string;
  version: string;
  date?: string;
  body?: string;
}

export interface UpdaterProgress {
  phase: "downloading" | "installing";
  downloadedBytes: number;
  totalBytes?: number;
}

export function resolveUpdaterProxy(proxyEnabled: boolean, proxyUrl: string): string | undefined {
  if (!proxyEnabled) return undefined;

  const value = proxyUrl.trim();
  if (!value) return undefined;

  let parsed: URL;
  try {
    parsed = new URL(value);
  } catch {
    throw new Error("代理地址格式无效，请填写完整的 http:// 地址");
  }

  if (parsed.protocol !== "http:") {
    throw new Error("更新代理仅支持 http:// 地址");
  }

  return value;
}

export async function checkForUpdate(input: UpdaterCheckInput): Promise<Update | null> {
  const options: CheckOptions = {
    timeout: DEFAULT_CHECK_TIMEOUT_MS
  };

  const proxy = resolveUpdaterProxy(input.proxyEnabled, input.proxyUrl);
  if (proxy) {
    options.proxy = proxy;
  }

  return check(options);
}

export function summarizeUpdate(update: Update): UpdateSummary {
  return {
    currentVersion: update.currentVersion,
    version: update.version,
    date: update.date,
    body: update.body
  };
}

export async function downloadAndInstallUpdate(
  update: Update,
  onProgress?: (progress: UpdaterProgress) => void
): Promise<void> {
  let downloadedBytes = 0;
  let totalBytes: number | undefined;

  await update.downloadAndInstall((event: DownloadEvent) => {
    if (event.event === "Started") {
      downloadedBytes = 0;
      totalBytes = event.data.contentLength;
      onProgress?.({
        phase: "downloading",
        downloadedBytes,
        totalBytes
      });
      return;
    }

    if (event.event === "Progress") {
      downloadedBytes += event.data.chunkLength;
      onProgress?.({
        phase: "downloading",
        downloadedBytes,
        totalBytes
      });
      return;
    }

    onProgress?.({
      phase: "installing",
      downloadedBytes,
      totalBytes
    });
  });
}

export async function disposeUpdate(update: Update | null | undefined): Promise<void> {
  if (!update) return;

  try {
    await update.close();
  } catch {
    // Ignore close errors because updater handle may already be released after install.
  }
}

export function formatUpdaterError(error: unknown): string {
  const message =
    error instanceof Error
      ? error.message
      : typeof error === "string"
        ? error
        : "未知错误";

  return `更新失败：${message}`;
}
