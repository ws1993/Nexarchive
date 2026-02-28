# Changelog

All notable changes to this project will be documented in this file.

## [0.1.6] - 2026-02-28

### Changed
- Adjusted settings page action button layout for improved usability.

## [0.1.5] - 2026-02-27

### Build
- Updated Tauri updater public key to full minisign format.

## [0.1.4] - 2026-02-27

### Build
- Updated Tauri updater public key.

## [0.1.3] - 2026-02-27

### Changed
- Updated `release-changelog` skill to skip git operations; changelog and version bumps only.

## [0.1.2] - 2026-02-27

### Added
- Added `release-changelog` project skill to automate CHANGELOG updates, version bumps, and git tagging.

## [0.1.1] - 2026-02-27

### Changed

- Extracted folder tree data into a shared module (`src/data/folderTree.ts`) to eliminate duplication between RulesPage and InitWizardPage.
- InitWizardPage folder preview now uses static shared data instead of a redundant API call.
- Fixed display level and tag color of `99_历史档案` to match top-level folder styling.

### Build

- Added `publish-release.bat` script to automate the full release flow (prepare, verify, build, commit, tag, push).
- Enhanced Tauri release workflow with signing key pre-validation to fail fast on missing secrets.

## [0.1.0] - 2026-02-27

### Added

- Initial MVP release for local file auto-archiving.
- Tauri + React desktop application shell.
- Initialization wizard and folder structure bootstrap.
- Scheduled/manual jobs, LLM classification, MinerU integration and local parser fallback.
- Task logs, recycle restore, autostart and settings management.
