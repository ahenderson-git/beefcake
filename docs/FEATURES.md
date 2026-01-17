# Beefcake Features

> **Comprehensive guide to Beefcake's capabilities**

*Last Updated: January 2025*

---

## Overview

This document provides a detailed breakdown of Beefcake's implemented features. Remember that these are **prototype implementations** subject to refinement and redesign as the project evolves.

For a high-level summary, see the main [README](../README.md).

---

## 1. Data Analysis & Profiling

### Column Statistics

Beefcake automatically calculates basic statistical measures for numeric columns:

- **Central Tendency**: Mean, median, mode
- **Dispersion**: Standard deviation, variance
- **Range**: Min, max values
- **Percentiles**: 25th, 50th (median), 75th percentiles

**Limitations:**
- Large datasets (>5M rows) use sampling for performance
- Missing values are excluded from calculations
- No support for weighted statistics

### Distribution Analysis

**Skewness Detection:**
- Measures asymmetry in data distribution
- Identifies left-skewed, right-skewed, or symmetric distributions
- Uses moment-based calculation method

**Kurtosis Analysis:**
- Detects heavy-tailed or light-tailed distributions
- Helps identify potential outliers or anomalies

**Outlier Detection:**
- Uses Interquartile Range (IQR) method
- Flags values outside 1.5 × IQR from Q1/Q3
- Visual representation in profile view

### Type Detection

Automatic inference of column data types:

- **Numeric**: Integers (i32, i64), floats (f32, f64)
- **Text**: Strings with pattern recognition
- **Temporal**: Date, datetime, timestamp parsing
- **Categorical**: Limited-cardinality string columns
- **Boolean**: True/false values

**Detection Heuristics:**
- Sample-based inference (first 1000 rows by default)
- Regex patterns for dates and special formats
- Cardinality thresholds for categorical classification

### Missing Value Detection

Identifies and reports:
- Null values (explicit nulls in data)
- Empty strings (`""`)
- Whitespace-only strings (configurable)
- Placeholder values (e.g., "N/A", "NULL", "-")

**Recommendations Engine:**
- Suggests imputation strategies based on column type
- Estimates impact of dropping vs. filling missing values
- Warns about high missingness rates (>30%)

### Business Insights

**Experimental Feature**: AI-powered interpretation of statistical patterns

- Generates natural language summaries of data characteristics
- Flags potential quality issues (high missingness, extreme skew)
- Suggests next steps for data cleaning

**Note:** This feature uses heuristic rules, not machine learning models. Output quality varies.

---

## 2. Dataset Lifecycle Management

### Lifecycle Stages

Track datasets through five logical stages:

```
Raw → Profiled → Cleaned → Advanced → Validated → Published
```

**Stage Descriptions:**

1. **Raw**: Initial import, no transformations applied
2. **Profiled**: Statistics calculated, quality assessed
3. **Cleaned**: Basic cleaning applied (trim, drop columns, impute)
4. **Advanced**: ML preprocessing, feature engineering
5. **Validated**: Quality checks passed, ready for use
6. **Published**: Exported for consumption

### Version Control

**Immutable Versions:**
- Each transformation creates a new version
- Original data never modified
- Versions stored as Parquet files on disk
- Copy-on-write semantics ensure data integrity

**Version Tree:**
- Visualize version history as a tree structure
- Navigate between versions with single click
- See parent-child relationships
- View active version indicator

**Version Diff Engine:**
- Compare any two versions side-by-side
- **Schema Changes**: Columns added, removed, or renamed
- **Row Changes**: Count differences with delta calculations
- **Statistical Changes**: Per-column metric comparisons (mean, median, etc.)
- Color-coded diff visualization (green = added, red = removed)

**Version History:**
- View all versions with timestamps
- Compare any two versions (diff view)
- Rollback to previous version by promoting it
- Metadata tracking (creator, timestamp, transform applied)

**Publish Modes:**
- **View**: Lazy reference to transformation pipeline (doesn't materialize data)
- **Snapshot**: Materialized copy of data at point in time

**Storage:**
- Versions stored in `data/lifecycle/{dataset_id}/`
- Automatic cleanup of old versions (configurable retention)
- Efficient Parquet format for disk space optimisation

### Audit Trail

**Transformation Logs:**
- Timestamp of each operation
- User who performed operation (if applicable)
- Parameters used for transformation
- Row/column counts before and after

**Log Format:**
```json
{
  "timestamp": "2025-01-13T10:30:00Z",
  "operation": "Clean",
  "details": {
    "columns_dropped": ["temp_col"],
    "rows_before": 1000,
    "rows_after": 950
  }
}
```

### Query Engine

**SQL-like Queries:**
- Query any version using Polars SQL syntax
- Filter, aggregate, join across versions
- Export query results to new dataset

**Example:**
```text
SELECT column1, AVG(column2)
FROM dataset_v3
WHERE column1 > 100
GROUP BY column1;
```

**Limitations:**
- Polars SQL dialect (not full ANSI SQL)
- No support for complex window functions
- Joins limited to same lifecycle instance

---

## 3. Visual Pipeline Builder

### Overview

Create reusable data transformation workflows through a drag-and-drop interface.

### Step Types (11 Total)

#### Column Management
1. **Drop Columns**: Remove unwanted columns by name
2. **Rename Columns**: Change column names with mapping

#### Text Processing
3. **Trim Whitespace**: Remove leading/trailing whitespace
4. **Regex Replace**: Pattern-based text substitution

#### Type Conversion
5. **Cast Types**: Convert column data types (int, float, string, date)
6. **Parse Dates**: Parse date strings with format specification

#### Missing Values
7. **Impute**: Fill missing values with:
   - Mean (numeric columns)
   - Median (numeric columns)
   - Mode (categorical columns)
   - Zero (numeric columns)

#### Machine Learning
8. **One-Hot Encoding**: Convert categorical to binary columns
9. **Normalize Columns**: Scale numeric values:
   - Z-score normalisation (standardisation)
   - Min-max scaling (0-1 range)

#### Outlier Handling
10. **Clip Outliers**: Cap values using quantile thresholds
11. **Extract Numbers**: Extract numeric values from text using regex

### Drag-and-Drop Interface

**Features:**
- Reorder steps by dragging handles (⋮⋮)
- Visual feedback during drag operations
- Selection follows dragged step
- Alternative: Up/Down buttons for keyboard navigation

### Built-In Templates (8 Total)

Pre-configured pipelines for common workflows:

1. **Data Cleaning** 🧹
   - Trim whitespace
   - Drop unwanted columns
   - Impute missing values (mean)

2. **ML Preprocessing** 🤖
   - Cast types to numeric
   - Impute missing (mean)
   - Normalise columns (min-max)
   - One-hot encode categoricals

3. **Date Normalisation** 📅
   - Parse dates with common formats
   - Cast to datetime type

4. **Text Processing** 📝
   - Trim whitespace
   - Rename columns to lowercase
   - Cast types for consistency

5. **Outlier Handling** 📊
   - Clip outliers (1st-99th percentile)
   - Normalise with z-score

6. **Column Selection** 🗂️
   - Drop unwanted columns
   - Rename for clarity

7. **Missing Data Handling** 🔧
   - Drop columns with >50% missing
   - Impute numeric (mean)
   - Impute categorical (mode)

8. **Type Conversion** 🔄
   - Cast types
   - Parse dates

### Step Configuration

Each step has a dedicated configuration panel:
- **Column Selection**: Multi-select dropdown
- **Parameters**: Type-specific inputs (thresholds, formats, etc.)
- **Validation**: Real-time feedback on invalid configs
- **Preview**: See affected columns before execution

### Save/Load/Share

**Pipeline Storage:**
- Pipelines saved as JSON files
- Version control friendly (plain text)
- Shareable across teams

**Example Pipeline Spec:**
```json
{
  "version": "0.1",
  "name": "my_pipeline",
  "steps": [
    {"TrimWhitespace": {"columns": ["name"]}},
    {"Impute": {"strategy": "mean", "columns": ["age"]}}
  ]
}
```

### Pipeline UI Components

**PipelineLibrary:**
- Grid view of saved pipelines with metadata
- Search and filter by pipeline name
- Two-tab interface: "My Pipelines" and "Templates"
- Quick actions: Edit, Execute, Delete
- Empty state with "Create Pipeline" CTA

**PipelineEditor:**
- Three-panel layout: Palette (left), Canvas (center), Config (right)
- Visual drag-and-drop with HTML5 API
- Step reordering with drag handles (⋮⋮) or Up/Down buttons
- Real-time step configuration validation
- Save/Execute directly from editor
- Dirty state tracking (warns before closing with unsaved changes)

**StepPalette:**
- 11 transformation types in 5 categories
- Click to add step to pipeline
- Category icons and descriptions
- Collapsible category groups

**StepConfigPanel:**
- Dynamic forms based on selected step type
- Column multi-select dropdowns
- Strategy/method selection (dropdowns)
- Parameter validation with error messages
- Help text and examples for each parameter

**PipelineExecutor:**
- Modal overlay for execution
- Input/output file selection
- Progress tracking with step-by-step feedback
- Execution metrics (duration, rows processed)
- Success/error result display
- Close and retry capabilities

**Supported Interactions:**
- Drag steps from palette to canvas
- Drag steps within canvas to reorder
- Click step to select and configure
- Delete step via trash icon
- Keyboard navigation (Tab, Enter, Arrow keys)
- Form auto-save (parameters saved as you type)

---

## 4. Embedded Development Environments

### SQL IDE

**Features:**
- Monaco Editor with SQL syntax highlighting
- Auto-complete for table and column names
- Execute queries against loaded datasets
- Configurable preview limit (default: 100 rows)
- Export results as CSV, JSON, or Parquet

**Supported SQL:**
- Polars SQL dialect (subset of ANSI SQL)
- SELECT, WHERE, GROUP BY, ORDER BY
- Basic aggregations (SUM, AVG, COUNT, etc.)
- Limited JOIN support

**Limitations:**
- No data modification (INSERT, UPDATE, DELETE)
- No CTEs (Common Table Expressions)
- No window functions (yet)

### Python IDE

**Features:**
- Monaco Editor with Python syntax highlighting
- Execute Python scripts with Polars DataFrame API
- Environment diagnostics (check Python/Polars installation)
- Script templates for common operations
- Security warnings for arbitrary code execution

**Available Libraries:**
- Polars (required)
- NumPy, Pandas (if installed)
- Standard library modules

**Execution Model:**
- Scripts run in subprocess (isolated from main process)
- Output captured and displayed in IDE
- Timeout protection (configurable, default: 60s)

**Security:**
- Warning modal on first Python execution
- No automatic execution on load
- User must explicitly run scripts

**Limitations:**
- No interactive input (stdin)
- Limited to single-threaded execution
- No GPU support

---

## 5. Machine Learning Preprocessing

### Train/Test Split

**Implementation:**
- Basic 80/20 split for model evaluation
- Random sampling (not stratified)
- Reproducible with seed parameter

**Usage:**
```rust
fn example(df: DataFrame) {
    // In ML workflow
    let (train, test) = split_train_test(df, 0.8);
}
```

### Model Types

**1. Linear Regression**
- Ordinary Least Squares (OLS) implementation
- Supports multiple features
- Evaluation: R² score on test set

**2. Logistic Regression**
- Binary classification
- Sigmoid activation
- Evaluation: Accuracy, confusion matrix

**3. Decision Trees**
- Basic CART algorithm
- Configurable max depth
- Evaluation: Accuracy on test set

**Limitations:**
- No hyperparameter tuning
- No cross-validation
- No ensemble methods (yet)
- Results may not match scikit-learn due to different implementations

### Feature Engineering

**Type Casting:**
- Convert strings to numeric
- Handle errors gracefully (coercion to null)

**Normalisation:**
- Z-score: `(x - μ) / σ`
- Min-max: `(x - min) / (max - min)`

**Categorical Encoding:**
- One-hot encoding only
- No label encoding or target encoding (yet)

### Evaluation Metrics

**Regression:**
- R² (coefficient of determination)
- Mean Squared Error (MSE)
- Root Mean Squared Error (RMSE)

**Classification:**
- Accuracy
- Confusion matrix (2x2 for binary)
- Precision, Recall (coming soon)

---

## 6. Database Integration

### PostgreSQL Support

**Connection:**
- Standard PostgreSQL connection strings
- Connection pooling via SQLx
- SSL/TLS support

**Schema Inspection:**
- Browse all tables in database
- View column names and data types
- Preview table contents (limited rows)

### Import/Export

**Import from Database:**
- SELECT entire table or custom query
- Load into Beefcake for analysis
- Automatic type mapping (Postgres → Polars)

**Export to Database:**
- Write DataFrame back to Postgres
- Create new table or replace existing
- Batch insert for performance

### Connection Management

**Storage:**
- Connection strings stored in app settings
- Optional encryption (platform keychain)
- Remember last used connection

**Limitations:**
- PostgreSQL only (no MySQL, SQLite, etc.)
- No support for stored procedures
- No transaction management

---

## 7. Automation & Export

### PowerShell Script Generation

**Features:**
- Convert pipelines to standalone `.ps1` scripts
- Scripts are self-contained (no Beefcake dependency)
- Call Beefcake CLI in headless mode

**Generated Script Structure:**
```powershell
param(
    [string]$InputPath,
    [string]$OutputPath
)

# Validate inputs
# Execute pipeline via CLI
beefcake run --spec "pipeline.json" --input $InputPath --output $OutputPath
```

**Scheduling:**
- Compatible with Windows Task Scheduler
- Can be run from cron (if on WSL)

### CLI Mode

**Headless Execution:**
```bash
beefcake run --spec pipeline.json --input data.csv --output result.parquet
```

**Options:**
- `--spec`: Path to pipeline JSON
- `--input`: Input dataset path
- `--output`: Output path (optional, uses spec default)
- `--log`: Log file path for debugging

**Exit Codes:**
- `0`: Success
- `1`: Pipeline error (transformation failed)
- `2`: Input/output error (file not found, etc.)

### Date Templating

**Dynamic Paths:**
- Use `{date}` placeholder in output paths
- Replaced with current date at runtime

**Example:**
```json
{
  "output": {
    "path": "output_{date}.parquet"
  }
}
```

**Result:** `output_2025-01-13.parquet`

### Logging

**Detailed Execution Logs:**
- Timestamps for each step
- Row/column counts before/after each transformation
- Error messages with stack traces
- Performance metrics (execution time per step)

**Log Levels:**
- `DEBUG`: All operations
- `INFO`: Major milestones
- `WARN`: Non-fatal issues
- `ERROR`: Failures

---

## 8. Filesystem Watcher

### Overview

Automatically monitor a folder for new data files and ingest them into the lifecycle system without manual intervention.

### Features

**Folder Monitoring:**
- Watch a single folder for new files (non-recursive)
- Real-time detection using OS-level filesystem events
- Cross-platform support (Windows, macOS, Linux)

**File Stability Detection:**
- Wait for file writes to complete before ingestion
- Prevents reading incomplete or locked files
- Configurable stability window (default: 2 seconds)

**Supported Formats:**
- CSV files (`.csv`)
- JSON files (`.json`)
- Parquet files (`.parquet`)

**Auto-Ingestion:**
- Automatically create new dataset in Raw stage
- Profile dataset immediately after ingestion
- Generate unique dataset ID
- Emit events to UI for real-time feedback

### Configuration

**Watch Folder:**
- Select folder via system dialog
- Path persisted across app restarts
- Can change watched folder without stopping watcher

**Enable/Disable:**
- Toggle watcher on/off from UI
- State persisted in config file
- Auto-start on app launch (if previously enabled)

**Activity Feed:**
- Real-time log of detected files
- Status indicators (detected, ingesting, success, failed)
- File details (name, size, timestamp)
- Dataset link for successful ingestions
- Retry button for failed ingestions

### Event Flow

```
1. File appears in watched folder
   ↓
2. Watcher detects filesystem event
   ↓
3. Stability checker waits for writes to complete
   ↓
4. File validated (format, accessibility)
   ↓
5. Dataset created in lifecycle (Raw stage)
   ↓
6. Automatic profiling triggered
   ↓
7. UI updated with new dataset
   ↓
8. Activity feed shows success status
```

### Use Cases

**1. Batch Processing:**
- Drop multiple CSV files into watched folder
- Each file ingested automatically
- Process later in bulk using pipelines

**2. Data Pipeline Integration:**
- Upstream system writes files to shared folder
- Beefcake picks up files immediately
- Downstream analysis begins automatically

**3. Scheduled Imports:**
- ETL job outputs daily reports to watched folder
- Beefcake ingests at regular intervals
- Historical versions maintained in lifecycle

### Limitations

- **Single Folder**: Cannot watch multiple folders simultaneously
- **No Recursion**: Subdirectories not monitored
- **No Filtering**: All supported files are ingested (no pattern matching)
- **No Deduplication**: Same filename ingested multiple times creates duplicate datasets
- **Platform-Specific**: File notification timing varies by OS

### Error Handling

**Failed Ingestions:**
- Malformed data (invalid CSV, corrupt JSON)
- Permission errors (locked files, access denied)
- Insufficient disk space for Parquet conversion
- Schema conflicts (unsupported data types)

**Recovery:**
- Failed files logged in activity feed
- Manual retry button available
- Original file preserved for debugging
- Error messages displayed to user

### Configuration Storage

**Location:**
- Config stored in `config/watcher.json`
- Contains: `enabled`, `folder`, `auto_start`

**Example:**
```json
{
  "enabled": true,
  "folder": "C:\\Users\\data\\incoming",
  "auto_start": true
}
```

---

## Feature Comparison Matrix

| Feature | Status | Quality | Notes |
|---------|--------|---------|-------|
| Column Statistics | ✅ Implemented | Good | Sample-based for large files |
| Type Detection | ✅ Implemented | Good | Heuristic-based |
| Missing Value Analysis | ✅ Implemented | Fair | Basic detection only |
| Lifecycle Management | ✅ Implemented | Good | 6-stage immutable versioning |
| Version Diff Engine | ✅ Implemented | Good | Schema + statistical comparison |
| Publish Modes | ✅ Implemented | Good | View (lazy) vs Snapshot (materialized) |
| Visual Pipeline Builder | ✅ Implemented | Good | 11 step types, 8 templates |
| Drag-and-Drop Editor | ✅ Implemented | Good | Reorder steps, visual feedback |
| Pipeline Templates | ✅ Implemented | Good | 8 pre-configured workflows |
| Pipeline Executor | ✅ Implemented | Good | Modal with progress tracking |
| Step Palette | ✅ Implemented | Good | 11 steps in 5 categories |
| Step Config Panel | ✅ Implemented | Good | Dynamic parameter forms |
| Filesystem Watcher | ✅ Implemented | Good | Auto-ingest with stability detection |
| Watcher Activity Feed | ✅ Implemented | Good | Real-time ingestion status |
| SQL IDE | ✅ Implemented | Fair | Limited SQL dialect |
| Python IDE | ✅ Implemented | Fair | Requires manual Python install |
| ML Preprocessing | ✅ Implemented | Fair | Basic models only |
| PostgreSQL Support | ✅ Implemented | Good | Import/export works well |
| PowerShell Export | ✅ Implemented | Good | Windows only |
| CLI Mode | ✅ Implemented | Good | Stable interface |

---

## Future Enhancements

See [ROADMAP.md](ROADMAP.md) for planned features and exploratory directions.

---

## Questions or Issues?

- Check [LIMITATIONS.md](LIMITATIONS.md) for known constraints
- See [ARCHITECTURE.md](ARCHITECTURE.md) for technical details
- Open an issue on GitHub for bugs or feature requests
