# NexArchive

NexArchive is an intelligent Windows desktop application designed for automated local file archiving and organization. It leverages the power of Local LLMs and advanced file extraction to seamlessly categorize, rename, and file your documents based on a controlled vocabulary and structured archive hierarchy.

---

## 🌟 Core Features

- **Automated Processing Pipeline**: Periodically scans your `Inbox` folder, performing type-based extraction, LLM-powered classification, and automated archiving without manual intervention.
- **Intelligent LLM Classification**: Uses OpenAI-compatible Chat Completions API to understand document semantics and assign appropriate tags, people involved, and context.
- **Advanced File Extraction**: Built-in support for processing a wide variety of formats (`pdf`, `docx`, `pptx`, `xlsx`, `md`, `png`, `jpg`, etc.) with both an optional high-accuracy MinerU API and a robust built-in Rust fallback.
- **Standardized Retitling & Routing**: Automatically renames files using the template `YYYYMMDD_文档类型_核心标题[#标签][@人物][&备注].扩展名` and routes them into standard top-level Chinese taxonomy folders (e.g., `10_身份基石`, `20_责任领域`, `30_行动项目`, `40_知识金库`, `50_数字资产`, `99_历史档案`).
- **Safety First**: Implements a dedicated in-app recycle bin (`%APPDATA%/NexArchive/recycle`) for source files, ensuring original files can be safely restored if needed. Unconfident classifications (`< 0.70`) or failures are routed to designated review folders, avoiding misfiling.
- **Local & Private**: No external runtime dependencies like Python or Tesseract needed. Operations run efficiently on your local system, with API keys safely encrypted.

---

## 📸 Highlights & Previews

> *Note: Please replace the placeholder image paths below with your actual screenshots in the `docs/images/` directory.*

### 1. Dashboard Overview
View system metrics, recent processing jobs, and real-time archiving status.

![Dashboard Preview](./docs/images/dashboard.png)

### 2. Rule & Vocabulary Management
View and manage the directory structure and vocabulary rules used by the LLM.

![Rules Preview](./docs/images/rules.png)

### 3. Settings & Configuration
Easily configure LLM connections, set up your Inbox/Archive root paths, and adjust extraction priorities.

![Settings Preview](./docs/images/settings.png)

---

## 🛠️ Development Guide

### Start in Development Mode
Run the React frontend and Tauri backend together with live-reload:
```bash
npm install
npm run tauri dev
```

### Build the Installer (Windows)
Create the final production `.msi` and `.exe` installers:
```bash
npm run tauri build
```
*Build outputs will be available at `src-tauri/target/release/bundle/msi/` and `src-tauri/target/release/bundle/nsis/`.*

---

## 🚀 Release Workflow

The project uses a Tag-Driven release workflow. To trigger a new release via GitHub Actions (building and publishing release assets and updater metadata), simply create and push a version tag:

```bash
# Example: Releasing version v0.2.0
git tag v0.2.0
git push origin v0.2.0
```

*(Note: Ensure your `package.json` version and `tauri.conf.json` versions are bumped before tagging. Internal scripts like `npm run release:prepare` can assist with version and changelog prep).*

---

## 🔐 Updater Signing Setup

The built-in updater requires signed artifacts to ensure security. The GitHub release workflow requires the following repository secrets:
- `TAURI_SIGNING_PRIVATE_KEY`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`

**Generate keys locally (One-time setup):**
```bash
npm run tauri signer generate -w ~/.tauri/nexarchive.key
```
After generation, copy the output public key and replace the value in `src-tauri/tauri.conf.json` under `plugins.updater.pubkey`.
