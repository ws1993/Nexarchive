# NexArchive 开发计划（Windows / Tauri + React / 云端 LLM）

## 一、计划摘要

* 目标：交付一个可在 Windows 10/11 本地运行的文件自动归档桌面程序，支持初始化目录、定时处理 Inbox、自动理解文件内容并重命名归档、可视化日志、后台与开机自启。
* 输入约束：以 [demang.md](https://file+.vscode-resource.vscode-cdn.net/c%3A/Users/wangs/.vscode/extensions/openai.chatgpt-0.4.76-win32-x64/webview/#) 为需求基线，以 [reference.md](https://file+.vscode-resource.vscode-cdn.net/c%3A/Users/wangs/.vscode/extensions/openai.chatgpt-0.4.76-win32-x64/webview/#) 的目录结构、命名模板、控制词表为规则源。
* 你已确认的关键决策：技术栈 **Tauri + React**，模型策略 **云端 LLM API**，MVP 文件类型 **办公文档+图片+PDF**，Inbox 清理策略 **先移动到回收站**。

## 二、范围定义（MVP）

1. 包含功能
   * 初始化向导：设置 Inbox 根目录与归档根目录。
   * 一键初始化：按 [reference.md](https://file+.vscode-resource.vscode-cdn.net/c%3A/Users/wangs/.vscode/extensions/openai.chatgpt-0.4.76-win32-x64/webview/#) 创建多级目录结构。
   * 定时任务：按小时配置（如每 2 小时 / 每 24 小时）。
   * 文件处理：扫描 Inbox，抽取内容，LLM 分类，按规则重命名并移动到目标目录。
   * 设置中心：Base URI / API Key / Model 配置与连通性测试。
   * 系统能力：托盘后台运行、开机自启。
   * 日志中心：按状态展示成功/重命名/失败，支持筛选与搜索。
   * 日志治理：日志滚动与数据库清理策略，防止无限增长。
2. 不包含功能（延期到 v1.1+）

* 音视频语义理解。
* 多用户协作与云同步。
* 自动从外部知识库动态学习目录规则。

## 三、技术架构与模块划分

1. 客户端层（Tauri + React + TypeScript）
   * 页面：Dashboard、任务日志、规则预览、设置、初始化向导。
   * 状态管理：Zustand。
   * UI 组件：Ant Design（加自定义主题，保证层级清晰与可读性）。
2. 桌面后端（Rust，作为 Tauri Core）
   * **config_service**：配置读写、敏感信息加密。
   * **scheduler_service**：定时触发、手动触发、任务互斥。
   * **pipeline_service**：扫描、抽取、LLM 分类、重命名、归档。
   * **logging_service**：结构化日志、状态流转、日志清理。
   * **system_service**：开机自启、托盘、后台运行。
3. 文件内容抽取（Rust 内置）
   * 由 Tauri Rust 后端内置完成多格式抽取，不依赖 Python Sidecar。
   * 支持：**txt/md/pdf/docx/xlsx/pptx/jpg/png/jpeg**。
   * 图片走多模态 LLM 直传（data URL），PDF 与 Office 走本地解析。
4. 本地数据层

* SQLite：配置镜像、任务记录、文件处理记录、日志索引。
* 文件日志：按大小滚动（10MB * 5）。
* 应用数据目录：**%APPDATA%/NexArchive/**。

## 四、关键接口与类型（实现时必须固定）

1. 核心配置类型
   * **AppConfig**：**inbox_path**, **archive_root_path**, **autostart**, **run_in_background**, **schedule_hours**, **llm**, **retention**.
   * **LLMConfig**：**base_uri**, **api_key_encrypted**, **model**, **timeout_sec**.
   * **RetentionConfig**：**max_log_file_mb**, **max_log_files**, **max_db_logs**, **db_log_retention_days**.
2. 任务与日志类型
   * **JobRecord**：**job_id**, **trigger_type(manual|schedule)**, **start_at**, **end_at**, **status**, **summary**.
   * **FileTaskRecord**：**task_id**, **job_id**, **src_path**, **hash**, **extract_status**, **classify_status**, **rename_status**, **archive_status**, **final_path**, **error_code**, **error_message**.
   * **LogEvent**：**timestamp**, **level**, **job_id**, **task_id**, **stage**, **message**, **payload_json**.
3. Tauri Command API（前后端契约）
   * **init_system(inboxPath, archiveRootPath)**
   * **get_init_preview()**
   * **save_settings(AppConfig)**
   * **load_settings()**
   * **test_llm_connection()**
   * **run_job_once()**
   * **get_jobs(page, pageSize, status, dateRange)**
   * **get_file_tasks(jobId, status)**
   * **get_logs(filters)**
   * **restore_from_recycle_bin(taskId)**（只针对本应用触发的删除）
4. LLM 输出 JSON Schema（强校验）

* **doc_type**（必须来自控制词表）
* **core_title**（必填）
* **tags**（可空数组）
* **people**（可空数组）
* **note**（可空）
* **target_top_dir**（限定为 **10/20/30/40/50/99**）
* **target_subpath**（相对路径，禁止越界）
* **confidence**（0~1）

## 五、处理流程（端到端）

1. 任务启动
   * 调度器触发后创建 **JobRecord**，全局互斥锁防止并发重复跑。
2. 扫描与去重
   * 递归扫描 Inbox。
   * 计算 **sha256 + size + mtime**，跳过已成功处理文件。
3. 内容抽取
   * 文本类直接读。
   * Office/PDF 走 Rust 本地解析，图片转 data URL 提供给多模态 LLM。
   * 抽取失败时标记失败并写入日志，不中断整批任务。
4. LLM 分类与命名
   * 将抽取文本摘要 + 控制词表 + 目录结构发送 LLM。
   * 校验返回 JSON，不合法则重试 1 次。
   * 文件名按模板生成：[YYYYMMDD_文档类型_核心标题_可选字段.扩展名](https://file+.vscode-resource.vscode-cdn.net/c%3A/Users/wangs/.vscode/extensions/openai.chatgpt-0.4.76-win32-x64/webview/#)。
   * 日期优先级：文件元数据创建日 > 修改日 > 当前日期。
5. 归档与清理
   * 自动创建目标子目录。
   * 同名冲突追加 **_dupN**。
   * 归档成功后从 Inbox 移除；失败文件进入 **Inbox/_Failed**。
   * 清理动作走回收站策略（不直接永久删除）。
6. 低置信度兜底

* **confidence < 0.70** 的文件进入 **Inbox/_Review**，等待人工确认。

## 六、迭代里程碑（5 个阶段）

1. Phase 1（基础骨架）
   * Tauri + React 工程、SQLite、配置读写、初始化目录创建、基础 UI。
2. Phase 2（处理引擎）
   * 扫描、去重、重命名、归档、任务日志状态流。
3. Phase 3（内容理解）
   * Rust 内置抽取、多格式支持、LLM 调用、连通性测试。
4. Phase 4（调度与系统集成）
   * 定时任务、托盘后台、开机自启、日志滚动与清理。
5. Phase 5（质量与发布）

* 自动化测试、异常恢复、安装包（MSI/EXE）、用户文档。

## 七、测试与验收标准

1. 单元测试
   * 命名模板生成（必填/可选字段组合）。
   * 路径安全校验（禁止越界、非法字符过滤）。
   * 词表校验（仅允许控制词表项）。
2. 集成测试
   * 200 份混合样本文件跑批。
   * LLM 超时、空响应、错误 JSON、网络异常。
   * 文件占用/权限不足/同名冲突。
3. E2E 场景
   * 首次安装到首次归档全流程。
   * 修改定时周期并生效。
   * 开机自启与托盘后台行为。
   * 日志检索与失败任务复查。
4. MVP 验收门槛

* 可稳定连续运行 72 小时无崩溃。
* 任务失败可追踪到文件级原因。
* 日志与数据库增长受控。
* 低置信度文件不会被直接误归档。

## 八、默认假设与已锁定选择

* 需求文件按 [demang.md](https://file+.vscode-resource.vscode-cdn.net/c%3A/Users/wangs/.vscode/extensions/openai.chatgpt-0.4.76-win32-x64/webview/#) 视作 [demand.md](https://file+.vscode-resource.vscode-cdn.net/c%3A/Users/wangs/.vscode/extensions/openai.chatgpt-0.4.76-win32-x64/webview/#)。
* 平台仅 Windows 10/11。
* 云端 LLM 接口采用兼容 OpenAI Chat Completions 风格。
* MVP 文件类型固定为：**docx/xlsx/pptx/pdf/txt/md/jpg/png/jpeg**。
* Inbox 清理默认通过回收站，不做不可逆删除。
* 旧文件遵循截止线策略，不做全量历史重命名迁移。
