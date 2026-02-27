# NexArchive

NexArchive is a Windows desktop app for local file auto-archiving.

## Implemented Scope (MVP)

- Tauri + React desktop UI.
- Initialization wizard for Inbox path and Archive Root path.
- One-click folder structure bootstrap from `reference.md`.
- Inbox is an independent input path and is not duplicated under Archive Root.
- Scheduled processing by hours and manual run.
- LLM classification (OpenAI-compatible Chat Completions API).
- File extraction pipeline for:
  - MinerU API (optional, primary): `pdf`, `doc`, `docx`, `ppt`, `pptx`, `jpg`, `jpeg`, `png`
  - Built-in Rust fallback: `txt`, `md`, `markdown`, `html`, `htm`, `pdf`, `docx`, `xlsx`, `pptx`, `jpg`, `jpeg`, `png`
- Rename template: `YYYYMMDD_文档类型_核心标题[#标签][@人物][&备注].扩展名`.
- Controlled vocabulary + top-level folder constraints.
- Archive top-level folders use Chinese names: `10_身份基石`, `20_责任领域`, `30_行动项目`, `40_知识金库`, `50_数字资产`, `99_历史档案`.
- Low-confidence fallback (`confidence < 0.70`) to `Inbox/_Review`.
- Failure fallback to `Inbox/_Failed`.
- Recycle strategy for processed source files via app recycle folder (`%APPDATA%/NexArchive/recycle`).
- SQLite-based jobs/tasks/logs with query APIs.
- Rotating file log and DB log retention cleanup.
- Windows autostart setting via registry.

## Commands Implemented

- `init_system(inbox_path, archive_root_path)`
- `get_init_preview()`
- `save_settings(config)`
- `load_settings()`
- `test_llm_connection()`
- `test_mineru_connection()`
- `run_job_once()`
- `get_jobs(page, page_size, status, date_range)`
- `get_file_tasks(job_id, status)`
- `get_logs(filters)`
- `restore_from_recycle_bin(task_id)`

## Local Development

### 1. Frontend

```bash
npm install
npm run build
```

### 2. Rust backend

```bash
cd src-tauri
cargo check
```

## Run in dev mode

```bash
npm run tauri dev
```

## Build Installer (Windows)

```bash
npm run tauri build
```

Build outputs:

- `src-tauri/target/release/bundle/msi/NexArchive_<version>_x64_en-US.msi`
- `src-tauri/target/release/bundle/nsis/NexArchive_<version>_x64-setup.exe`

## Release Workflow (Tag Driven)

Prepare a new release version:

```bash
npm run release:prepare -- 0.2.0
npm run release:verify -- 0.2.0
```

Extract the changelog section for a version:

```bash
npm run release:extract-notes -- v0.2.0
```

After committing version changes and changelog updates, push the release tag:

```bash
git tag v0.2.0
git push origin v0.2.0
```

The GitHub workflow `.github/workflows/release.yml` will build and publish release assets and updater metadata.

## Updater Signing Setup

The updater uses signed artifacts and requires these GitHub secrets:

- `TAURI_SIGNING_PRIVATE_KEY`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`

Generate keys once locally:

```bash
npm run tauri signer generate -w ~/.tauri/nexarchive.key
```

Copy the generated public key into `src-tauri/tauri.conf.json` at `plugins.updater.pubkey`.

## Notes

- No external runtime dependencies are required for end users (no Python/Tesseract).
- Current recycle strategy uses app-managed recycle storage for reliable restore API.
- API key is encrypted before being persisted to `%APPDATA%/NexArchive/config.json`.
- The project currently targets Windows (`win10/win11`) only.
