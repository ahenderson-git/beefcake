# Beefcake Troubleshooting Guide

> **Solutions to common issues and error messages**

*Version 0.2.0 | Last Updated: January 2025*

---

## Table of Contents

1. [Installation Issues](#installation-issues)
2. [File Loading Errors](#file-loading-errors)
3. [Analysis Failures](#analysis-failures)
4. [Pipeline Execution Errors](#pipeline-execution-errors)
5. [Export Problems](#export-problems)
6. [Watcher Issues](#watcher-issues)
7. [AI Assistant Not Working](#ai-assistant-not-working)
8. [Performance Problems](#performance-problems)
9. [Database Connection Errors](#database-connection-errors)
10. [General Errors](#general-errors)

---

## Installation Issues

### Error: "Application failed to start"

**Symptom**: Beefcake crashes immediately on launch or shows white screen

**Solutions**:
1. **Check system requirements**:
   - Windows 10/11 64-bit
   - macOS 10.15+
   - Linux with GLIBC 2.27+
2. **Run as administrator** (Windows only):
   - Right-click → "Run as administrator"
3. **Check antivirus**:
   - Some antivirus software blocks Tauri apps
   - Add Beefcake to whitelist
4. **Reinstall**:
   - Completely uninstall
   - Delete `%APPDATA%\beefcake` (Windows) or `~/.local/share/beefcake` (Linux/macOS)
   - Reinstall fresh

---

### Error: "Cannot create data directory"

**Symptom**: "Failed to initialize storage: Permission denied"

**Cause**: Insufficient permissions to create application data folder

**Solutions**:
1. **Check folder permissions**:
   - Windows: `%LOCALAPPDATA%\beefcake` should be writable
   - Linux/macOS: `~/.local/share/beefcake` should be writable
2. **Manual creation**:
   ```bash
   # Linux/macOS
   mkdir -p ~/.local/share/beefcake/datasets
   chmod 755 ~/.local/share/beefcake

   # Windows PowerShell
   New-Item -Path "$env:LOCALAPPDATA\beefcake\datasets" -ItemType Directory -Force
   ```
3. **Run with elevated privileges** (last resort)

---

## File Loading Errors

### Error: "File not found"

**Symptom**: "Failed to load file: No such file or directory"

**Solutions**:
1. **Check file path**:
   - Ensure file exists at specified path
   - Use absolute paths (e.g., `C:\data\file.csv`, not `.\file.csv`)
2. **Check file name encoding**:
   - Avoid special characters in filenames
   - Use ASCII characters when possible
3. **Verify file permissions**:
   - Ensure file is readable (not locked by another program)

---

### Error: "Unsupported file format"

**Symptom**: "Failed to analyze: Unsupported format"

**Cause**: File is not CSV, JSON, or Parquet, or file extension is incorrect

**Solutions**:
1. **Check file extension**:
   - Supported: `.csv`, `.json`, `.parquet`
   - Rename file if extension is wrong
2. **Verify file contents**:
   - Open in text editor to confirm format
   - CSV should have comma-separated values
   - JSON should be valid JSON array/object
3. **Convert file**:
   - Use Excel or Python to convert to CSV
   - Example Python:
     ```python
     import pandas as pd
     df = pd.read_excel('file.xlsx')
     df.to_csv('file.csv', index=False)
     ```

---

### Error: "Invalid CSV format"

**Symptom**: "Failed to parse CSV: Unexpected token"

**Cause**: Malformed CSV (unescaped quotes, wrong delimiter, etc.)

**Solutions**:
1. **Check delimiter**:
   - Beefcake expects comma `,` as delimiter
   - If using semicolon `;` or tab, convert first
2. **Fix quoted fields**:
   - Ensure quotes are properly escaped: `"He said ""Hello"""`
   - Use text editor with CSV validation
3. **Check line endings**:
   - Unix (`\n`) vs Windows (`\r\n`) vs Mac (`\r`)
   - Convert to Unix line endings if possible
4. **Pre-process with cleanup tool**:
   ```python
   import pandas as pd
   df = pd.read_csv('broken.csv', on_bad_lines='skip')
   df.to_csv('fixed.csv', index=False)
   ```

---

## Analysis Failures

### Error: "Analysis failed: Out of memory"

**Symptom**: "Failed to analyze: Cannot allocate memory"

**Cause**: File too large for available RAM

**Solutions**:
1. **Free up memory**:
   - Close other applications
   - Restart computer
2. **Use sampling**:
   - Beefcake auto-samples files >5M rows
   - If still too large, manually sample first:
     ```python
     import pandas as pd
     df = pd.read_csv('large.csv', nrows=100000)
     df.to_csv('sample.csv', index=False)
     ```
3. **Increase system resources**:
   - Add more RAM
   - Use 64-bit OS
4. **Convert to Parquet**:
   - More memory-efficient format
   - Use Python to convert:
     ```python
     import pandas as pd
     df = pd.read_csv('large.csv', chunksize=50000)
     pd.concat(df).to_parquet('large.parquet')
     ```

---

### Error: "Column type detection failed"

**Symptom**: "Failed to infer schema: Ambiguous types"

**Cause**: Mixed data types in columns (e.g., `"abc"` and `123` in same column)

**Solutions**:
1. **Accept as Text type**:
   - Beefcake will default to String type
   - Use "Cast Types" pipeline step to convert later
2. **Clean source data**:
   - Ensure consistent types per column
   - Remove or convert invalid entries
3. **Manual type specification**:
   - Use SQL IDE to force cast:
     ```sql
     SELECT CAST(column AS INT64) FROM data
     ```

---

## Pipeline Execution Errors

### Error: "Pipeline validation failed"

**Symptom**: "Column 'X' does not exist in input schema"

**Cause**: Pipeline references columns not present in input data

**Solutions**:
1. **Check column names**:
   - Ensure exact match (case-sensitive)
   - Check for typos
2. **Regenerate pipeline**:
   - Create new pipeline from current dataset
   - Use "Pipeline from Configs" feature
3. **Update pipeline spec**:
   - Edit pipeline JSON file
   - Replace old column names with new ones

---

### Error: "Transformation failed: Invalid operation"

**Symptom**: "Cannot apply step 'Impute': No numeric columns"

**Cause**: Operation requires specific column types (e.g., impute mean needs numeric)

**Solutions**:
1. **Check column types**:
   - View analysis to see detected types
   - Cast columns to required type first
2. **Reorder pipeline steps**:
   - Put "Cast Types" before "Impute"
3. **Use correct strategy**:
   - Mean/Median: numeric columns only
   - Mode: works on any type

---

### Error: "Output file already exists"

**Symptom**: "Failed to write output: File exists and overwrite=false"

**Cause**: Pipeline output path already has a file

**Solutions**:
1. **Enable overwrite** in pipeline spec:
   ```json
   {
     "output": {
       "path": "output.csv",
       "overwrite": true
     }
   }
   ```
2. **Use date templating**:
   ```json
   {
     "output": {
       "path": "output_{date}.csv"
     }
   }
   ```
3. **Delete existing file** before running

---

## Export Problems

### Error: "Export failed: Permission denied"

**Symptom**: "Failed to write file: Access denied"

**Cause**: Output directory not writable or file is open in another program

**Solutions**:
1. **Check file locks**:
   - Close Excel, text editors using the file
   - Use Task Manager to find processes
2. **Choose different location**:
   - Export to `Documents` or `Desktop`
   - Avoid system directories like `C:\Windows`
3. **Check disk space**:
   - Ensure enough free space for output file

---

### Error: "Export failed: Invalid format"

**Symptom**: "Unsupported export format: xlsx"

**Cause**: Beefcake only exports CSV, JSON, Parquet

**Solutions**:
1. **Use supported format**:
   - Export as CSV
   - Open in Excel and save as XLSX
2. **Use Python bridge**:
   - Export as CSV
   - Convert with Python script:
     ```python
     import pandas as pd
     df = pd.read_csv('export.csv')
     df.to_excel('export.xlsx', index=False)
     ```

---

## Watcher Issues

### Error: "Watcher failed to start"

**Symptom**: "Failed to watch folder: Permission denied"

**Cause**: No read permissions on target folder

**Solutions**:
1. **Check folder permissions**:
   - Ensure folder exists and is readable
   - On Windows, avoid system folders
2. **Use user directories**:
   - `C:\Users\username\data` (Windows)
   - `~/data` (Linux/macOS)
3. **Run as administrator** (Windows, last resort)

---

### Problem: "Files not being detected"

**Symptom**: New files added to folder but watcher doesn't react

**Causes & Solutions**:
1. **File type not supported**:
   - Only CSV, JSON, Parquet are detected
   - Check file extensions
2. **File in subdirectory**:
   - Watcher is **non-recursive**
   - Files must be in root of watched folder
3. **File still being written**:
   - Watcher waits for stability (3 consecutive unchanged checks)
   - Large files may take 5-10 seconds to stabilize
4. **Watcher not enabled**:
   - Check "Enable Watcher" toggle is ON
   - Verify correct folder is set

---

### Error: "Ingestion failed: Malformed data"

**Symptom**: File detected but ingestion fails

**Cause**: File is corrupt or improperly formatted

**Solutions**:
1. **Validate file format**:
   - Try opening in Beefcake manually (Open File)
   - Check for detailed error message
2. **Fix file before re-ingestion**:
   - Open in text editor
   - Fix formatting issues
   - Save and retry

---

## AI Assistant Not Working

### Error: "AI Assistant unavailable"

**Symptom**: AI sidebar shows "Disabled" or red indicator

**Cause**: AI Assistant not configured or API key missing

**Solutions**:
1. **Check Settings**:
   - Go to Settings view
   - Scroll to "AI Assistant" section
   - Ensure "Enable AI Assistant" is checked
2. **Add API key**:
   - Get OpenAI API key from https://platform.openai.com/api-keys
   - Paste into "API Key" field
   - Click "Save Configuration"
3. **Verify key validity**:
   - Test key in OpenAI Playground
   - Check for spending limits or expired keys

---

### Error: "AI request failed: Rate limit exceeded"

**Symptom**: "Failed to get response: 429 Too Many Requests"

**Cause**: OpenAI API rate limit hit

**Solutions**:
1. **Wait and retry**:
   - Rate limits reset after a few seconds
   - Try again in 10-30 seconds
2. **Check API usage**:
   - Log in to OpenAI dashboard
   - View usage and limits
3. **Upgrade plan**:
   - Free tier has stricter limits
   - Consider paid plan for higher limits

---

### Error: "AI response timeout"

**Symptom**: Request hangs for 60 seconds then fails

**Cause**: Network issues or OpenAI service slow

**Solutions**:
1. **Check internet connection**:
   - Verify you're online
   - Test other websites
2. **Retry request**:
   - May have been temporary issue
3. **Check OpenAI status**:
   - Visit https://status.openai.com/
   - See if service is down

---

## Performance Problems

### Problem: "Application is slow/laggy"

**Symptoms**: UI freezes, long loading times, choppy animations

**Solutions**:
1. **Close unused views**:
   - Each open view consumes memory
   - Navigate away from heavy views (Lifecycle)
2. **Reduce dataset size**:
   - Work with samples for exploration
   - Only load full data for final export
3. **Restart application**:
   - Memory may accumulate over long sessions
   - Restart every few hours of heavy use
4. **Check system resources**:
   - Task Manager (Windows) or Activity Monitor (macOS)
   - Beefcake should use <2GB RAM normally
   - Close other memory-intensive apps

---

### Problem: "Analysis takes too long"

**Symptoms**: File loaded but profiling runs for >5 minutes

**Expected Times**:
- Small files (<10K rows): 1-5 seconds
- Medium files (10K-1M rows): 5-30 seconds
- Large files (1M-5M rows): 30-120 seconds
- Very large files (>5M rows): 2-5 minutes (with sampling)

**Solutions if slower**:
1. **Check file size**:
   - Files >1GB will be slow
   - Consider sampling first
2. **Check CPU usage**:
   - Analysis is CPU-intensive
   - Close other CPU-heavy apps
3. **Use Parquet format**:
   - Faster to parse than CSV
   - Convert large CSVs to Parquet

---

## Database Connection Errors

### Error: "Database connection failed"

**Symptom**: "Failed to connect: Connection refused"

**Cause**: PostgreSQL not running or connection details incorrect

**Solutions**:
1. **Verify PostgreSQL is running**:
   ```bash
   # Linux/macOS
   sudo systemctl status postgresql

   # Windows (Services app)
   # Check "postgresql-x64-XX" service is Running
   ```
2. **Check connection details**:
   - Host: Usually `localhost` or `127.0.0.1`
   - Port: Default is `5432`
   - Database: Must exist
   - User/Password: Must be correct
3. **Test connection externally**:
   ```bash
   psql -h localhost -U postgres -d mydb
   ```
   If this fails, connection details are wrong

---

### Error: "Authentication failed"

**Symptom**: "Failed to connect: Password authentication failed for user 'X'"

**Cause**: Wrong username or password

**Solutions**:
1. **Reset password**:
   ```bash
   # As postgres user
   psql -U postgres
   ALTER USER username WITH PASSWORD 'newpassword';
   ```
2. **Check pg_hba.conf**:
   - Ensure `md5` or `password` authentication is enabled
   - Restart PostgreSQL after changes

---

## General Errors

### Error: "Unexpected error: {technical message}"

**Symptom**: Generic error with stack trace

**Cause**: Unknown/unhandled error

**Solutions**:
1. **Check logs**:
   - Windows: `%LOCALAPPDATA%\beefcake\logs`
   - Linux/macOS: `~/.local/share/beefcake/logs`
   - Look for `beefcake-{date}.log`
2. **Restart application**:
   - May resolve transient issues
3. **Report issue**:
   - Go to https://github.com/yourusername/beefcake/issues
   - Include:
     - Error message (full text)
     - Steps to reproduce
     - Log file excerpt
     - OS and Beefcake version

---

### Error: "Operation aborted by user"

**Symptom**: "Processing cancelled"

**Cause**: User clicked "Abort" button or closed dialog

**Solution**:
- This is expected behavior, not an error
- Restart operation if aborted accidentally

---

## Getting More Help

If your issue isn't listed here:

1. **Search documentation**:
   - [USER_GUIDE.md](USER_GUIDE.md) - Full feature guide
   - [FEATURES.md](FEATURES.md) - Detailed feature documentation
   - [LIMITATIONS.md](LIMITATIONS.md) - Known limitations

2. **Search GitHub Issues**:
   - https://github.com/yourusername/beefcake/issues
   - Check if someone else reported same issue

3. **Ask for help**:
   - GitHub Discussions: https://github.com/yourusername/beefcake/discussions
   - Include:
     - Beefcake version
     - Operating system
     - Error message (exact text)
     - Steps to reproduce

4. **Report a bug**:
   - GitHub Issues: https://github.com/yourusername/beefcake/issues/new
   - Use bug report template

---

*Remember: Beefcake is experimental software. Some issues may not have immediate solutions.*
