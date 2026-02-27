---
name: release-changelog
description: 自动化版本发布工作流：更新 CHANGELOG.md、同步版本号、提交并打 tag。触发场景：用户说"把更新维护到 CHANGELOG，tag X.X.X"、"更新 changelog，打 tag X.X.X"、"发布 X.X.X"、"release X.X.X" 等要求完成版本记录和 git tag 的指令。
---

# Release Changelog

## 工作流

### 1. 确定版本号

从用户指令中提取目标版本号（如 `0.1.1`）。

### 2. 收集变更

```bash
# 找到上一个 tag
git describe --tags --abbrev=0

# 获取自上个 tag 以来的所有提交
git log --oneline <prev-tag>..HEAD
```

### 3. 更新版本文件

根据项目类型更新版本号，常见文件：

| 文件 | 方式 |
|------|------|
| `package.json` | `sed -i 's/"version": "OLD"/"version": "NEW"/'` |
| `package-lock.json` | 同上（若存在） |
| `src-tauri/tauri.conf.json` | `sed -i 's/"version": "OLD"/"version": "NEW"/'` |
| `src-tauri/Cargo.toml` | `sed -i 's/^version = "OLD"/version = "NEW"/'` |

只更新存在的文件，跳过不存在的。

### 4. 写入 CHANGELOG.md

在已有最新版本条目之前插入新版本块，格式：

```markdown
## [X.X.X] - YYYY-MM-DD

### Added
- 新功能描述

### Changed
- 变更描述

### Fixed
- 修复描述

### Build
- 构建/CI 相关变更
```

规则：
- 日期使用当前实际日期
- 根据提交内容归类到合适的 section，空 section 省略
- 提交前缀参考：`功能:` / `feat` → Added；`优化:` / `fix` → Changed 或 Fixed；`构建:` / `build` / `chore` → Build；`重构:` → Changed
- 用简洁的英文或与 CHANGELOG 现有语言风格一致的语言书写条目

### 5. 提交并打 tag

```bash
# 暂存版本相关文件（不要 git add .）
git add CHANGELOG.md package.json package-lock.json src-tauri/Cargo.toml src-tauri/tauri.conf.json

# 提交
git commit -m "chore(release): vX.X.X"

# 打带注释的 tag
git tag -a vX.X.X -m "release: vX.X.X"
```

完成后告知用户 tag 已创建，并提示推送命令：`git push origin main vX.X.X`。
