# Beefcake Pipeline Automation Guide

## Overview

Beefcake's pipeline automation system allows you to capture data transformation workflows as versioned JSON specifications that can be:

- **Executed headlessly** via CLI for batch processing
- **Exported as PowerShell scripts** for Windows Task Scheduler automation
- **Version controlled** in Git alongside your code
- **Edited manually** for fine-grained control
- **Shared across teams** for standardized data processing

This document explains how to create, run, and schedule automated data pipelines.

---

## Quick Start

### 1. Capture a Pipeline from the GUI

1. Load a dataset in Beefcake GUI (Analyser view)
2. Configure transformations (rename columns, clean data, type conversions, etc.)
3. Click "Automation" in the navigation
4. Click "Save Pipeline Spec" and choose a location
5. Click "Generate PowerShell" to create an automation script

### 2. Run the Pipeline via CLI

```powershell
# Run with explicit output path
beefcake run --spec my_pipeline.json --input data/input.csv --output data/output.parquet

# Use spec's path template (with {date} substitution)
beefcake run --spec my_pipeline.json --input data/input.csv

# With logging
beefcake run --spec my_pipeline.json --input data/input.csv --log logs/run.log
```

### 3. Schedule with PowerShell

```powershell
# Test the generated PowerShell script manually
.\run.ps1 -InputPath "C:\data\input.csv" -OutputPath "C:\data\output.parquet"

# Schedule in Windows Task Scheduler (see section below)
```

---

## Pipeline Specification Format

### Minimal Example

```json
{
  "version": "0.1",
  "name": "simple_clean",
  "input": {
    "format": "csv",
    "has_header": true,
    "delimiter": ",",
    "encoding": "utf-8"
  },
  "schema": {
    "match_mode": "tolerant",
    "required_columns": []
  },
  "steps": [
    {
      "op": "trim_whitespace",
      "columns": ["name", "email"]
    }
  ],
  "output": {
    "format": "parquet",
    "path_template": "output/cleaned.parquet",
    "overwrite": true
  }
}
```

### Full Example (Customer Import)

```json
{
  "version": "0.1",
  "name": "daily_customer_import",
  "input": {
    "format": "csv",
    "has_header": true,
    "delimiter": ",",
    "encoding": "utf-8"
  },
  "schema": {
    "match_mode": "strict",
    "required_columns": [
      "customer_id",
      "age",
      "signup_date",
      "state",
      "income"
    ]
  },
  "steps": [
    {
      "op": "drop_columns",
      "columns": ["internal_notes", "debug_flag"]
    },
    {
      "op": "rename_columns",
      "mapping": {
        "cust_id": "customer_id",
        "dob": "date_of_birth"
      }
    },
    {
      "op": "trim_whitespace",
      "columns": ["state"]
    },
    {
      "op": "cast_types",
      "columns": {
        "age": "i64",
        "income": "f64"
      }
    },
    {
      "op": "parse_dates",
      "columns": {
        "signup_date": "yyyy-MM-dd"
      }
    },
    {
      "op": "impute",
      "strategy": "median",
      "columns": ["income", "age"]
    },
    {
      "op": "one_hot_encode",
      "columns": ["state"],
      "drop_original": true
    }
  ],
  "output": {
    "format": "parquet",
    "path_template": "output/cleaned_{date}.parquet",
    "overwrite": true
  }
}
```

---

## Spec Reference

### Top-Level Fields

| Field    | Type           | Description                                      |
|----------|----------------|--------------------------------------------------|
| `version` | string         | Spec version (currently "0.1")                  |
| `name`    | string         | Human-readable pipeline name                     |
| `input`   | InputConfig    | Input file configuration                         |
| `schema`  | SchemaConfig   | Schema validation rules                          |
| `steps`   | Step[]         | Ordered transformation steps                     |
| `output`  | OutputConfig   | Output file configuration                        |

### Input Configuration

```json
{
  "format": "csv",          // csv | json | parquet
  "has_header": true,       // CSV: first row is header
  "delimiter": ",",         // CSV delimiter
  "encoding": "utf-8"       // File encoding
}
```

### Schema Configuration

```json
{
  "match_mode": "tolerant", // strict | tolerant
  "required_columns": ["id", "name"]
}
```

**Match Modes:**
- `tolerant`: Required columns must exist; extra columns allowed
- `strict`: Only required columns allowed; no extras

### Transformation Steps

All steps have an `op` field and operation-specific parameters.

#### Drop Columns

```json
{
  "op": "drop_columns",
  "columns": ["col1", "col2"]
}
```

#### Rename Columns

```json
{
  "op": "rename_columns",
  "mapping": {
    "old_name": "new_name",
    "another_old": "another_new"
  }
}
```

#### Trim Whitespace

```json
{
  "op": "trim_whitespace",
  "columns": ["name", "email"]
}
```

#### Cast Types

```json
{
  "op": "cast_types",
  "columns": {
    "age": "i64",
    "price": "f64",
    "description": "String",
    "is_active": "Boolean"
  }
}
```

**Supported types:** `i64`, `f64`, `String`, `Boolean`, `Numeric`, `Text`, `Categorical`, `Temporal`

#### Parse Dates

```json
{
  "op": "parse_dates",
  "columns": {
    "signup_date": "yyyy-MM-dd",
    "timestamp": "yyyy-MM-dd HH:mm:ss"
  }
}
```

#### Impute Missing Values

```json
{
  "op": "impute",
  "strategy": "median",  // mean | median | mode | zero
  "columns": ["age", "income"]
}
```

#### One-Hot Encode

```json
{
  "op": "one_hot_encode",
  "columns": ["category", "region"],
  "drop_original": true
}
```

#### Normalize Columns

```json
{
  "op": "normalize_columns",
  "method": "zscore",  // zscore | minmax
  "columns": ["price", "quantity"]
}
```

#### Clip Outliers

```json
{
  "op": "clip_outliers",
  "columns": ["price"],
  "lower_quantile": 0.05,
  "upper_quantile": 0.95
}
```

#### Extract Numbers

```json
{
  "op": "extract_numbers",
  "columns": ["mixed_field"]
}
```

Extracts numeric values from text using regex `(\d+\.?\d*)`.

#### Regex Replace

```json
{
  "op": "regex_replace",
  "columns": ["phone"],
  "pattern": "[^0-9]",
  "replacement": ""
}
```

### Output Configuration

```json
{
  "format": "parquet",                    // csv | json | parquet
  "path_template": "output/file_{date}.parquet",
  "overwrite": true
}
```

**Path Template Variables:**
- `{date}`: Current date in YYYY-MM-DD format (or --date CLI arg)

---

## CLI Reference

### `beefcake run`

Execute a pipeline specification.

```bash
beefcake run --spec <PATH> --input <PATH> [OPTIONS]
```

**Required Arguments:**

- `--spec <PATH>`: Path to pipeline spec JSON file
- `--input <PATH>`: Path to input data file

**Optional Arguments:**

- `--output <PATH>`: Output file path (overrides spec's path_template)
- `--date <YYYY-MM-DD>`: Date for path template substitution (default: today)
- `--log <PATH>`: Write execution log to file
- `--fail-on-warnings`: Exit with code 3 if warnings are generated

**Exit Codes:**

- `0`: Success
- `1`: General error
- `2`: Validation error (spec invalid or input doesn't match schema)
- `3`: Runtime error (or warnings with --fail-on-warnings)

**Examples:**

```powershell
# Basic usage
beefcake run --spec pipeline.json --input data.csv --output result.parquet

# Use spec's path template
beefcake run --spec pipeline.json --input data.csv

# With logging
beefcake run --spec pipeline.json --input data.csv --log logs/$(date +%Y%m%d).log

# Fail fast on warnings
beefcake run --spec pipeline.json --input data.csv --fail-on-warnings
```

---

## PowerShell Automation

### Generated Script Features

The generated PowerShell script (`run.ps1`) includes:

- **Parameter validation**: Checks input/spec files exist
- **Error handling**: Stops on any error, captures exit codes
- **Colored output**: Info/Success/Error messages
- **Logging**: Automatic log file generation
- **Exit code handling**: Proper error propagation for schedulers

### Running the Script

```powershell
.\run.ps1 -InputPath "C:\data\input.csv" -OutputPath "C:\data\output.parquet"
```

**Parameters:**

- `-InputPath` (required): Input data file
- `-OutputPath` (optional): Output file (uses spec template if omitted)
- `-SpecPath` (optional): Pipeline spec file (default: adjacent JSON file)
- `-Date` (optional): Date for template substitution
- `-LogPath` (optional): Custom log file path
- `-FailOnWarnings`: Exit on warnings

---

## Windows Task Scheduler Setup

### Manual Setup

1. Open Task Scheduler (`taskschd.msc`)
2. Click "Create Task" (not Basic Task)

**General Tab:**
- Name: `Beefcake - Daily Customer Import`
- Description: `Automated data processing pipeline`
- Run whether user is logged on or not: ☑️
- Run with highest privileges: ☐

**Triggers Tab:**
- New Trigger
- Daily at desired time (e.g., 6:00 AM)
- Optionally: Repeat every X hours

**Actions Tab:**
- New Action
- Action: Start a program
- Program/script: `powershell.exe`
- Arguments: `-ExecutionPolicy Bypass -File "C:\path\to\run.ps1" -InputPath "C:\data\input.csv" -OutputPath "C:\data\output.parquet"`
- Start in: (leave blank or set to script directory)

**Conditions Tab:**
- Start only if computer is on AC power: ☐
- Wake computer to run: ☐

**Settings Tab:**
- Allow task to be run on demand: ☑️
- Stop task if it runs longer than: 3 hours
- If task fails, restart every: 10 minutes
- Attempt to restart up to: 3 times

### Automated Setup (PowerShell)

```powershell
# Create scheduled task programmatically
$action = New-ScheduledTaskAction `
  -Execute "powershell.exe" `
  -Argument '-ExecutionPolicy Bypass -File "C:\automation\run.ps1" -InputPath "C:\data\input.csv" -OutputPath "C:\data\output.parquet"'

$trigger = New-ScheduledTaskTrigger -Daily -At 6:00AM

$principal = New-ScheduledTaskPrincipal -UserId "DOMAIN\User" -LogonType S4U

$settings = New-ScheduledTaskSettingsSet `
  -ExecutionTimeLimit (New-TimeSpan -Hours 3) `
  -RestartCount 3 `
  -RestartInterval (New-TimeSpan -Minutes 10)

Register-ScheduledTask `
  -TaskName "Beefcake - Daily Customer Import" `
  -Action $action `
  -Trigger $trigger `
  -Principal $principal `
  -Settings $settings
```

---

## Best Practices

### Pipeline Design

1. **Keep steps atomic**: Each step should do one thing well
2. **Validate early**: Use `match_mode: strict` and `required_columns`
3. **Log everything**: Always use `--log` in production
4. **Version your specs**: Commit pipeline JSON to Git
5. **Test locally first**: Run manually before scheduling

### Error Handling

1. **Enable restarts in Task Scheduler**: Transient failures shouldn't stop automation
2. **Monitor logs**: Set up log rotation and alerting
3. **Use --fail-on-warnings**: Catch potential data quality issues early
4. **Validate inputs**: Check file sizes, dates, formats before processing

### Performance

1. **Use Parquet for large datasets**: Faster and smaller than CSV
2. **Avoid collecting operations**: Pipeline uses streaming where possible
3. **Batch similar operations**: Multiple `trim_whitespace` on different columns = one step
4. **Profile first**: Run with --log to see execution time

### Maintenance

1. **Document custom logic**: Add comments in spec JSON (not standard but readable)
2. **Keep example data**: Store sample input for testing
3. **Semantic versioning**: Increment spec version for breaking changes
4. **Test migrations**: Validate old specs still work after Beefcake upgrades

---

## Troubleshooting

### Common Errors

**"Required column 'X' not found in input"**
- Input file doesn't have expected columns
- Check CSV header, case sensitivity
- Use `match_mode: tolerant` if column names vary

**"Cannot rename 'X' to 'Y': target already exists"**
- Trying to rename a column to a name that already exists
- Drop or rename the conflicting column first

**"Invalid type string 'XYZ'"**
- Typo in `cast_types` step
- Use: `i64`, `f64`, `String`, `Boolean`, `Categorical`, `Temporal`

**"Failed to sink to parquet"**
- Output directory doesn't exist
- Create parent directories manually or fix path_template

**"Task failed with exit code 1 (Task Scheduler)"**
- Check log file for detailed error
- Ensure paths are absolute, not relative
- Verify Beefcake is in system PATH

### Debugging Tips

1. **Run CLI manually first**: Reproduce the exact Task Scheduler command
2. **Check file permissions**: Ensure task user can read/write paths
3. **Enable verbose logging**: Add `--log` to capture full output
4. **Validate spec separately**: Use `beefcake run --help` to check syntax
5. **Test with small data**: Use subset of production data for faster iteration

### Getting Help

- CLI help: `beefcake run --help`
- Example specs: `examples/pipelines/` directory
- Issues: https://github.com/your-org/beefcake/issues

---

## Examples

### Example 1: Daily ETL Pipeline

**Scenario:** Import customer data from CSV every morning at 6 AM, clean it, and save to data warehouse folder.

**pipeline.json:**
```json
{
  "version": "0.1",
  "name": "daily_customer_etl",
  "input": {
    "format": "csv",
    "has_header": true,
    "delimiter": ",",
    "encoding": "utf-8"
  },
  "schema": {
    "match_mode": "tolerant",
    "required_columns": ["customer_id", "name", "email"]
  },
  "steps": [
    {"op": "trim_whitespace", "columns": ["name", "email"]},
    {"op": "drop_columns", "columns": ["temp_field"]},
    {"op": "cast_types", "columns": {"customer_id": "i64"}}
  ],
  "output": {
    "format": "parquet",
    "path_template": "warehouse/customers_{date}.parquet",
    "overwrite": false
  }
}
```

**Task Scheduler:**
- Daily at 6:00 AM
- Arguments: `-ExecutionPolicy Bypass -File "C:\etl\run.ps1" -InputPath "\\fileshare\exports\customers_latest.csv"`

### Example 2: ML Preprocessing

**Scenario:** Prepare training data for machine learning model weekly.

**ml_prep.json:**
```json
{
  "version": "0.1",
  "name": "ml_feature_engineering",
  "input": {"format": "parquet", "has_header": true, "delimiter": "", "encoding": "utf-8"},
  "schema": {"match_mode": "tolerant", "required_columns": ["feature1", "feature2", "target"]},
  "steps": [
    {"op": "impute", "strategy": "mean", "columns": ["feature1", "feature2"]},
    {"op": "clip_outliers", "columns": ["feature1"], "lower_quantile": 0.01, "upper_quantile": 0.99},
    {"op": "normalize_columns", "method": "zscore", "columns": ["feature1", "feature2"]},
    {"op": "one_hot_encode", "columns": ["category_col"], "drop_original": true}
  ],
  "output": {"format": "parquet", "path_template": "ml/train_{date}.parquet", "overwrite": true}
}
```

**Manual run:**
```powershell
beefcake run --spec ml_prep.json --input data/raw.parquet --output ml/train.parquet
```

---

## Migration Guide

### Upgrading from Manual Workflows

If you're currently running manual data cleaning workflows in Beefcake GUI:

1. **Record your workflow once**: Go through your usual clicks in GUI
2. **Save the pipeline spec**: Automation → Save Pipeline Spec
3. **Test the CLI**: Run `beefcake run` with the saved spec
4. **Compare outputs**: Verify CLI output matches GUI output
5. **Generate PowerShell**: Automation → Generate PowerShell
6. **Schedule it**: Set up Task Scheduler

### Version 0.1 → Future Versions

When spec version changes (e.g., 0.1 → 0.2):

1. Beefcake will attempt to migrate old specs automatically
2. Migration warnings will appear in logs
3. Review changes and resave specs in new format
4. Test thoroughly before updating production schedules

---

## Appendix: Complete Spec Schema

See `examples/pipelines/` for real-world examples:
- `simple_clean.json`: Basic text cleaning
- `daily_customer_import.json`: Full ETL pipeline
- `ml_preprocessing.json`: ML feature engineering

---

**Version:** 0.1.0
**Last Updated:** 2026-01-12
