@echo off
setlocal EnableExtensions

set "VERSION=%~1"
if "%VERSION%"=="" (
  echo Usage: npm run release:publish -- ^<version^> [remote]
  echo Example: npm run release:publish -- 0.2.0
  exit /b 1
)

set "REMOTE=%~2"
if "%REMOTE%"=="" set "REMOTE=origin"

where git >nul 2>nul
if errorlevel 1 (
  echo [ERROR] git is not available in PATH.
  exit /b 1
)

where npm >nul 2>nul
if errorlevel 1 (
  echo [ERROR] npm is not available in PATH.
  exit /b 1
)

for /f "delims=" %%i in ('git rev-parse --abbrev-ref HEAD 2^>nul') do set "BRANCH=%%i"
if not defined BRANCH (
  echo [ERROR] Unable to detect current git branch.
  exit /b 1
)
if "%BRANCH%"=="HEAD" (
  echo [ERROR] Detached HEAD is not supported for release publish.
  exit /b 1
)

git rev-parse -q --verify "refs/tags/v%VERSION%" >nul 2>nul
if not errorlevel 1 (
  echo [ERROR] Tag v%VERSION% already exists locally.
  exit /b 1
)

echo [1/8] Prepare version files...
call npm run release:prepare -- %VERSION%
if errorlevel 1 exit /b 1

echo [2/8] Verify version consistency...
call npm run release:verify -- %VERSION%
if errorlevel 1 exit /b 1

echo [3/8] Validate changelog section...
call npm run -s release:extract-notes -- v%VERSION% >nul
if errorlevel 1 (
  echo [ERROR] Missing CHANGELOG entry for %VERSION%.
  exit /b 1
)

echo [4/8] Build frontend...
call npm run build
if errorlevel 1 exit /b 1

echo [5/8] Commit release files...
git add CHANGELOG.md package.json package-lock.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
if errorlevel 1 exit /b 1

git commit -m "chore(release): v%VERSION%"
if errorlevel 1 (
  echo [ERROR] git commit failed. Check staged changes and retry.
  exit /b 1
)

echo [6/8] Create annotated tag...
git tag -a v%VERSION% -m "release: v%VERSION%"
if errorlevel 1 exit /b 1

echo [7/8] Push branch %BRANCH% to %REMOTE%...
git push %REMOTE% %BRANCH%
if errorlevel 1 exit /b 1

echo [8/8] Push tag v%VERSION% to %REMOTE%...
git push %REMOTE% v%VERSION%
if errorlevel 1 exit /b 1

echo Release v%VERSION% published. GitHub Actions will run workflow Release.
exit /b 0
