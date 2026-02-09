# Comprehensive File Logging Implementation

This document summarizes the file logging system implemented for Beefcake to capture all errors and diagnostic information for troubleshooting GUI issues.

## ✅ Implementation Complete

### Backend (Rust)

1. **Dependencies Added** (`Cargo.toml`)
   - `tracing = "0.1"` - Structured logging framework
   - `tracing-subscriber = { version = "0.3", features = ["env-filter", "json", "fmt"] }` - Log formatting and filtering
   - `tracing-appender = "0.2"` - File rotation and appending

2. **Logging Module** (`src/logging.rs`)
   - Multi-target logging (console + rotating files)
   - Platform-specific log directories:
     - Windows: `%APPDATA%\beefcake\logs`
     - macOS: `~/Library/Application Support/beefcake/logs`
     - Linux: `~/.local/share/beefcake/logs`
   - Two log files:
     - `beefcake.YYYY-MM-DD.log` - All logs (info, warn, error, debug)
     - `error.YYYY-MM-DD.log` - Errors and warnings only
   - Daily rotation with 10 files retained
   - Helper functions: `get_log_dir()`, `get_current_log_path()`, `get_current_error_log_path()`

3. **Application Initialization** (`src/main.rs`)
   - Logging initialized before any other operations
   - Fallback to `env_logger` if file logging fails
   - Tracing macros used throughout startup sequence

4. **Tauri Commands Enhanced** (`src/commands/`)
   - `analyze_file` - Logs analysis start, completion, and errors
   - `run_powershell` - Logs script execution and results
   - `run_python` - Logs Python execution with data prep errors
   - Replaced `eprintln!` in `tauri_app.rs` with `tracing::error!`

5. **New System Commands** (`src/commands/system.rs`)
   - `log_frontend_error(level, message, context)` - Receives frontend logs
   - `get_log_directory()` - Returns log directory path
   - `get_current_log_file()` - Returns current main log path
   - `get_current_error_log_file()` - Returns current error log path

### Frontend (TypeScript)

1. **API Functions** (`src-frontend/api.ts`)
   ```typescript
   logFrontendError(level, message, context)
   getLogDirectory()
   getCurrentLogFile()
   getCurrentErrorLogFile()
   ```

2. **Global Error Handlers** (`src-frontend/main.ts`)
   - `window.addEventListener('error')` - Catches uncaught errors
   - `window.addEventListener('unhandledrejection')` - Catches promise rejections
   - `console.error` override - Logs all console errors to file
   - `console.warn` override - Logs all warnings to file
   - All errors include stack traces and context

3. **Settings UI** (`src-frontend/renderers/settings.ts`)
   - New "Application Logs" section with:
     - Log directory path display
     - Button to open log folder
     - Buttons to open individual log files
     - Information about log rotation

4. **Settings Component** (`src-frontend/components/SettingsComponent.ts`)
   - `openLogDirectory()` - Opens log folder in file explorer
   - `openLogFile(type)` - Opens specific log file
   - `loadLogPath()` - Displays log directory path on load

## 📋 What Gets Logged

### Backend Logs
- ✅ Application startup and shutdown
- ✅ File analysis operations (path, row count, column count)
- ✅ PowerShell script execution
- ✅ Python script execution (with data preparation)
- ✅ Watcher service initialization
- ✅ Directory creation errors
- ✅ All Tauri command executions
- ✅ Error contexts with full error chains

### Frontend Logs
- ✅ Uncaught JavaScript errors with stack traces
- ✅ Unhandled promise rejections
- ✅ All console.error() calls
- ✅ All console.warn() calls
- ✅ BeefcakeApp initialization
- ✅ Component lifecycle errors
- ✅ Tauri invoke() failures

## 🚀 How to Use

### 1. Build and Run
```bash
# Build the project (this will download new dependencies)
npm run tauri:build

# Or run in dev mode
npm run tauri:dev
```

### 2. Access Logs

**Via Settings UI:**
1. Open Beefcake
2. Navigate to Settings
3. Scroll to "Application Logs" section
4. Click "Open Folder" to view all logs
5. Click individual "Open" buttons for specific log files

**Direct Access:**
- Windows: `C:\Users\<username>\AppData\Roaming\beefcake\logs\`
- macOS: `~/Library/Application Support/beefcake/logs/`
- Linux: `~/.local/share/beefcake/logs/`

### 3. Log Files

**beefcake.YYYY-MM-DD.log** - Complete application log
```
[2026-01-25T12:34:56.789Z] INFO beefcake - Beefcake application starting
[2026-01-25T12:34:57.123Z] INFO beefcake::tauri_app - Tauri setup complete
[2026-01-25T12:35:00.456Z] INFO beefcake::commands::analysis - analyze_file command called with path: C:\data\test.csv
[2026-01-25T12:35:02.789Z] INFO beefcake::commands::analysis - File analysis completed successfully: 1000 rows, 10 columns
```

**error.YYYY-MM-DD.log** - Errors and warnings only
```
[2026-01-25T12:36:00.123Z] ERROR beefcake::commands::analysis - File analysis failed: Failed to parse CSV: invalid UTF-8
[2026-01-25T12:36:05.456Z] WARN beefcake::tauri_app - Failed to initialize watcher service: Permission denied
[2026-01-25T12:36:10.789Z] ERROR beefcake::commands::system - [Frontend] Uncaught error: Cannot read property 'render' of undefined | context: {"filename":"main.ts","lineno":234,"colno":12}
```

## 🐛 Troubleshooting GUI Issues

When you encounter GUI problems:

1. **Open the error log immediately** (Settings → Application Logs → Errors Only)
2. **Look for recent `[Frontend]` errors** - These are from the TypeScript code
3. **Check for Tauri command failures** - Look for command names like `analyze_file`, `run_python`
4. **Find stack traces** - All errors include full stack traces for debugging
5. **Copy the entire error log** and provide it to Claude for analysis

### Example Error Investigation

If the GUI freezes during file analysis:

1. Check `error.log` for:
   ```
   ERROR beefcake::commands::analysis - analyze_file command called with path: ...
   ERROR beefcake::analyser::logic - Failed to read file: ...
   ```

2. Check `beefcake.log` for the full context:
   ```
   INFO beefcake::commands::analysis - Analyzing file: ...
   DEBUG beefcake::analyser::logic - Reading CSV with 1000000 rows
   ERROR beefcake::analyser::logic - Memory allocation failed
   ```

3. Copy the relevant log entries and share with Claude

## 🔄 Environment Variables

You can control log verbosity with the `RUST_LOG` environment variable:

```bash
# Windows PowerShell
$env:RUST_LOG="debug"
.\beefcake.exe

# Linux/macOS
RUST_LOG=debug ./beefcake

# Available levels: error, warn, info, debug, trace
# Module-specific: RUST_LOG=beefcake::analyser=debug
```

## 📊 Log Rotation

- Logs rotate **daily** at midnight
- Maximum **10 old files** retained (10 days of logs)
- Old logs are automatically deleted
- Log directory size is self-managing

## 🎯 Next Steps

1. Run the application and verify logs are being created
2. Trigger some errors intentionally to test logging
3. When you encounter GUI issues, immediately check the error log
4. Provide the log excerpts to Claude for detailed analysis

---

**Implementation Date:** 2026-01-25
**Status:** ✅ Complete and Ready for Testing
