---
name: release-changelog
description: 更新 CHANGELOG.md 并同步版本号。触发场景：用户说"把更新维护到 CHANGELOG，tag X.X.X"、"更新 changelog X.X.X"、"发布 X.X.X" 等要求完成版本记录的指令。只负责文件更新，不执行 git 操作。
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

### 5. 完成

文件更新后告知用户已完成，提示运行：

```bash
npm run release:publish -- X.X.X
```

**不执行任何 git add / git commit / git tag 操作。**
