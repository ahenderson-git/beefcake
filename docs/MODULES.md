# Module Documentation

Detailed reference for all Beefcake modules and their responsibilities.

## Rust Backend Modules

### `src/lib.rs`
**Entry Point**: Public library API
**Purpose**: Exposes core functionality for use as a library
**Key Exports**:
- `analyser`: Data analysis subsystem
- `pipeline`: Automation framework
- `error`: Error types
- `utils`: Utility functions

---

### `src/main.rs`
**Entry Point**: Application binary
**Purpose**: Routes to CLI or GUI mode based on arguments
**Dependencies**: `cli`, `tauri_app`, `tokio`

**Example Usage**:
```bash
# GUI mode
beefcake

# CLI mode
beefcake analyze data.csv
```

---

### `src/analyser/`

#### `src/analyser.rs`
**Purpose**: Analysis subsystem entry point
**Sub-modules**:
- `logic`: Core analysis algorithms
- `lifecycle`: Version management system
- `db`: Database integration

#### `src/analyser/logic/`

##### `analysis.rs`
**Purpose**: Orchestrates full dataset analysis
**Key Functions**:
- `analyze_file(path)` - Main entry point for analysis
- `analyze_dataframe(df)` - Analyzes in-memory data

**Process**:
1. Load file into Polars LazyFrame
2. Detect column types
3. Compute statistics per type
4. Generate health score
5. Produce recommendations

##### `profiling.rs`
**Purpose**: Generates statistical profiles for columns
**Key Functions**:
- `profile_numeric(series)` - Mean, median, percentiles, histogram
- `profile_text(series)` - Distinct values, patterns, length stats
- `profile_temporal(series)` - Date ranges, gaps, frequency

##### `types.rs`
**Purpose**: Detects semantic column types
**Key Types**:
- `Numeric`: Integers, floats
- `Temporal`: Dates, times, timestamps
- `Categorical`: Low-cardinality text
- `Boolean`: True/false values
- `Text`: Free-form strings

**Algorithm**:
1. Check data type (i64, f64, str, etc.)
2. Sample values for pattern matching
3. Heuristics (e.g., <50 distinct = categorical)

##### `health.rs`
**Purpose**: Assesses data quality
**Metrics**:
- Completeness (null ratio)
- Consistency (outliers, invalid formats)
- Accuracy (range checks)

**Returns**: Score 0-100 and list of issues

##### `cleaning.rs`
**Purpose**: Applies data transformations
**Operations**:
- Text cleaning (trim, case, regex)
- Type conversion
- Null standardisation
- Rounding/formatting

##### `ml.rs`
**Purpose**: ML preprocessing transformations
**Features**:
- Imputation (mean, median, mode)
- Normalization (z-score, min-max)
- One-hot encoding
- Outlier clipping

##### `interpretation.rs`
**Purpose**: Generates human-readable insights
**Output**:
- Business summaries (e.g., "High null rate may indicate data collection issue")
- ML advice (e.g., "Consider imputation before modeling")
- Data interpretation (e.g., "Strongly left-skewed distribution")

#### `src/analyser/lifecycle/`

##### `mod.rs`
**Purpose**: Dataset version management registry
**Key Types**:
- `DatasetRegistry` - Thread-safe dataset collection
- `Dataset` - Versioned dataset with transformation history

**API**:
```rust
let registry = DatasetRegistry::new(base_path)?;
let dataset_id = registry.create_dataset(name, path)?;
let version_id = registry.apply_transforms(dataset_id, pipeline, stage)?;
```

##### `version.rs`
**Purpose**: Version metadata and tree structure
**Key Types**:
- `DatasetVersion` - Single version with metadata
- `VersionTree` - Tracks version relationships
- `VersionMetadata` - Timestamp, stage, parent reference

##### `storage.rs`
**Purpose**: Persistent file storage for versions
**Key Types**:
- `VersionStore` - Manages parquet file storage
- `DataLocation` - File path or in-memory reference

**Layout**:
```
data/
├── datasets/
│   ├── {dataset-id}/
│   │   ├── {version-id}.parquet
│   │   ├── {version-id}.parquet
│   │   └── metadata.json
```

##### `transforms.rs`
**Purpose**: Serializable transformation pipeline
**Key Types**:
- `Transform` - Single transformation operation
- `TransformPipeline` - Ordered list of transforms

**JSON Format**:
```json
{
  "transforms": [
    {"type": "RenameColumn", "from": "age", "to": "customer_age"},
    {"type": "FillNull", "column": "income", "value": 0}
  ]
}
```

##### `diff.rs`
**Purpose**: Compare two dataset versions
**Key Functions**:
- `compute_version_diff(v1, v2)` - Schema and data comparison

**Returns**:
- Columns added/removed/renamed
- Row count change
- Sample data differences

##### `query.rs`
**Purpose**: Query interface for dataset versions
**Key Types**:
- `VersionQuery` - Fluent API for querying

**Example**:
```rust
let query = VersionQuery::new(dataset_id)
    .filter_by_stage(LifecycleStage::Cleaned)
    .latest();
```

##### `stages/`
**Purpose**: Lifecycle stage implementations

###### `profile.rs`
Stage: **Raw → Profiled**
- Captures analysis metadata
- No data transformation

###### `clean.rs`
Stage: **Profiled → Cleaned**
- Text/type cleaning
- Standardisation
- Deterministic transforms

###### `advanced.rs`
Stage: **Cleaned → Advanced**
- ML preprocessing
- Imputation
- Normalization
- Feature engineering

###### `validate.rs`
Stage: **Advanced → Validated**
- Business rule validation
- QA gates
- Assertion checks

###### `publish.rs`
Stage: **Validated → Published**
- Creates view (lazy) or snapshot (materialized)
- Marks as production-ready

---

### `src/pipeline/`

#### `mod.rs`
**Purpose**: Pipeline automation API
**Key Exports**:
- `PipelineSpec` - JSON specification format
- `run_pipeline()` - Execute pipeline on dataset
- `validate_pipeline()` - Check pipeline validity
- `generate_powershell_script()` - Export as PowerShell automation

**Pipeline Steps (11 Total)**:
1. `drop_columns` - Remove columns by name
2. `rename_columns` - Rename columns with mapping
3. `trim_whitespace` - Trim leading/trailing spaces
4. `cast_types` - Convert column data types
5. `parse_dates` - Parse date strings with format
6. `impute` - Fill missing values (mean/median/mode/zero)
7. `normalize_columns` - Scale numeric values (z-score/min-max)
8. `clip_outliers` - Cap extreme values using quantiles
9. `one_hot_encode` - Convert categorical to binary columns
10. `extract_numbers` - Extract numeric values from text
11. `regex_replace` - Pattern-based text substitution

#### `spec.rs`
**Purpose**: Pipeline specification data structures
**Key Types**:
- `PipelineSpec` - Complete pipeline definition with metadata
- `Step` - Enum representing 11 transformation types
- `InputConfig` / `OutputConfig` - I/O settings
- `ImputeStrategy` - Enum for missing value strategies
- `SchemaMatchMode` - Enum for schema validation strictness

**JSON Schema Example**:
```json
{
  "version": "0.1",
  "name": "ML Preprocessing Pipeline",
  "description": "Prepare data for machine learning",
  "input": {
    "format": "csv"
  },
  "steps": [
    {"op": "trim_whitespace", "columns": ["name", "email"]},
    {"op": "cast_types", "columns": {"age": "i64", "salary": "f64"}},
    {"op": "impute", "strategy": "mean", "columns": ["salary"]},
    {"op": "normalize_columns", "method": "min_max", "columns": ["age", "salary"]},
    {"op": "one_hot_encode", "columns": ["department"], "drop_original": true}
  ],
  "output": {
    "format": "parquet",
    "path": "processed.parquet"
  }
}
```

#### `executor.rs`
**Purpose**: Executes pipeline steps sequentially on Polars DataFrame
**Key Functions**:
- `run_pipeline(spec, input_path, output_path)` - Main execution entry
- `apply_step(df, step)` - Apply single transformation
- `validate_step_columns(df, step)` - Pre-flight column checks

**Returns**: `RunReport` with:
- Execution status (success/failure)
- Row/column counts before & after
- Warnings generated
- Execution duration
- Steps applied count

**Error Handling**:
- Column not found errors
- Type conversion failures
- Invalid parameter errors
- I/O errors (file not found, permission denied)

#### `validation.rs`
**Purpose**: Validates pipeline before execution
**Key Functions**:
- `validate_pipeline(spec, input_path)` - Full validation
- `validate_step(step, schema)` - Single step validation
- `check_column_exists(name, schema)` - Column reference check

**Validation Checks**:
- Input file exists and is readable
- Required columns exist in input schema
- Parameter types are correct (e.g., quantiles 0-1)
- Step configurations are complete
- No duplicate column operations

**Returns**: `Vec<ValidationError>` with detailed error messages

#### `powershell.rs`
**Purpose**: Generates standalone PowerShell automation scripts
**Key Functions**:
- `generate_powershell_script(spec, output_path)` - Generate .ps1 file
- `escape_powershell_string(s)` - Safe string escaping

**Generated Script Features**:
- Parameter support for input/output paths
- Validation of inputs before execution
- Error handling with exit codes
- Logging to console
- Calls Beefcake CLI in headless mode

**Example Generated Script**:
```powershell
# Auto-generated by Beefcake Pipeline Builder
# Pipeline: ML Preprocessing Pipeline
param(
    [string]$InputPath = "data.csv",
    [string]$OutputPath = "processed.parquet"
)

if (!(Test-Path $InputPath)) {
    Write-Error "Input file not found: $InputPath"
    exit 1
}

Write-Host "Executing pipeline..."
& beefcake pipeline execute "ml_pipeline.json" --input $InputPath --output $OutputPath

if ($LASTEXITCODE -eq 0) {
    Write-Host "Pipeline completed successfully"
} else {
    Write-Error "Pipeline failed with exit code $LASTEXITCODE"
    exit $LASTEXITCODE
}
```

---

### `src/watcher/`

#### `mod.rs`
**Purpose**: Filesystem watcher module entry point
**Key Exports**:
- `WatcherService` - Background watcher thread
- `WatcherMessage` - Command enum for service control
- `WatcherEvent` - Event types emitted to UI
- `init(app)` - Initialize watcher on app startup
- `start(folder)` / `stop()` - Control functions

**Global State**:
- `WATCHER_SERVICE` - LazyLock singleton for global access
- Thread-safe with `Arc<Mutex<Option<WatcherService>>>`

#### `config.rs`
**Purpose**: Persistent watcher configuration
**Key Type**:
- `WatcherConfig` - Serializable config struct

**Fields**:
- `enabled: bool` - Whether watcher auto-starts
- `folder: PathBuf` - Watched folder path
- `stability_window_secs: u64` - File stability timeout

**Storage**:
- Location: `config/watcher.json`
- Format: JSON
- Auto-save on changes

**Functions**:
- `load()` - Load from disk (or create default)
- `save()` - Persist to disk

#### `service.rs`
**Purpose**: Background watcher service implementation
**Key Type**:
- `WatcherService` - Main service struct with background thread

**Architecture**:
```
WatcherService
    │
    ├─> Command Channel (mpsc) - Receive Start/Stop/IngestNow commands
    ├─> notify::Watcher - OS-level filesystem event listener
    ├─> StabilityChecker - HashMap tracking file modification times
    └─> Tauri AppHandle - Emit events to frontend
```

**Message Types**:
- `Start(PathBuf)` - Begin watching folder
- `Stop` - Stop watching
- `IngestNow(PathBuf)` - Manually trigger ingestion

**Event Loop**:
1. Wait for filesystem event from notify
2. Filter for Create/Modify events
3. Check file extension (csv/json/parquet)
4. Add to stability checker
5. Poll stability checker for completed files
6. Ingest completed files
7. Emit success/failure events to UI

**Stability Detection**:
- Track last modification time per file
- Wait for configurable window (default 2s) with no changes
- Prevents reading incomplete/locked files

**Error Handling**:
- Permission errors (log and continue)
- Malformed files (emit failed event)
- Watch errors (stop service, emit error event)

#### `events.rs`
**Purpose**: Event types and payloads
**Key Types**:
- `WatcherStatusPayload` - Current watcher state
- `FileDetectedPayload` - File detected event
- `IngestStartedPayload` - Ingestion began
- `IngestSucceededPayload` - Ingestion completed
- `IngestFailedPayload` - Ingestion error
- `WatcherServiceState` - Enum (Idle/Watching/Ingesting/Error)

**Event Names** (Tauri events):
- `watcher:status`
- `watcher:file_detected`
- `watcher:file_ready`
- `watcher:ingest_started`
- `watcher:ingest_succeeded`
- `watcher:ingest_failed`

**Serialization**:
- All payloads implement `Serialize` for JSON emission
- Frontend receives events via Tauri event system

---

### `src/cli.rs`
**Purpose**: Command-line interface using `clap`
**Commands**:
- `analyze <file>` - Analyze dataset
- `clean <file> <config>` - Apply cleaning
- `pipeline execute <spec>` - Run pipeline
- `pipeline validate <spec>` - Check pipeline
- `db push <file> <conn-id>` - Upload to database

---

### `src/tauri_app.rs`
**Purpose**: Tauri command handlers (IPC bridge)
**Key Functions**: All `#[tauri::command]` annotated functions
**Examples**:
- `analyze_file(path)` → `analyser::logic::analyze_file()`
- `run_python(script)` → `python_runner::run_python()`
- `lifecycle_create_dataset()` → `lifecycle::DatasetRegistry::create_dataset()`

---

### `src/error.rs`
**Purpose**: Centralized error types
**Key Types**:
- `BeefcakeError` - Main error enum
- `ResultExt` - Extension trait for adding context

**Usage**:
```rust
use crate::error::{BeefcakeError, ResultExt};

fn load() -> Result<Data, BeefcakeError> {
    let data = read_file()
        .context("Failed to read file")?;
    Ok(data)
}
```

---

### `src/python_runner.rs`
**Purpose**: Embedded Python runtime
**Key Functions**:
- `run_python(script, data_path)` - Execute Python with dataset
- `install_package(name)` - Install via pip

**Features**:
- Provides `df` variable (pandas DataFrame)
- Captures stdout/stderr with ANSI colors
- Error handling and stack traces

---

### `src/system.rs`
**Purpose**: System-level utilities
**Key Functions**:
- Process monitoring
- Memory usage tracking
- Environment detection

---

### `src/export.rs`
**Purpose**: Data export functionality
**Supported Formats**:
- CSV
- JSON
- Parquet
- Excel (via Polars)

---

## TypeScript Frontend Modules

### `src-frontend/main.ts`
**Purpose**: Application entry point and controller
**Key Class**: `BeefcakeApp`
**Responsibilities**:
- State management
- Component lifecycle
- Event coordination
- Backend communication

---

### `src-frontend/api.ts`
**Purpose**: Tauri bridge to Rust backend
**Pattern**: All functions use `invoke("command_name", args)`
**Key Functions**:
- `analyseFile()` - Analyze data
- `runPython()` / `runSql()` - Execute scripts
- `createDataset()` - Lifecycle API
- `loadAppConfig()` / `saveAppConfig()` - Config management

---

### `src-frontend/types.ts`
**Purpose**: TypeScript type definitions
**Key Interfaces**:
- `AppState` - Application state
- `AnalysisResponse` - Analysis results
- `ColumnSummary` - Column statistics
- `ColumnCleanConfig` - Cleaning configuration
- `AppConfig` - Application settings

---

### `src-frontend/components/`

#### `Component.ts`
**Purpose**: Abstract base class for all components
**Pattern**: Template method pattern
**Key Methods**:
- `render(state)` - Update DOM (abstract)
- `getContainer()` - Find DOM element

#### Individual Components
- `DashboardComponent.ts` - Home screen
- `AnalyserComponent.ts` - Analysis view
- `LifecycleComponent.ts` - Dataset lifecycle UI
- `LifecycleRailComponent.ts` - Sidebar lifecycle tracker
- `PowerShellComponent.ts` - PowerShell IDE
- `PythonComponent.ts` - Python IDE
- `SQLComponent.ts` - SQL IDE
- `SettingsComponent.ts` - Configuration
- `ActivityLogComponent.ts` - Audit log
- `ReferenceComponent.ts` - Help/documentation
- `CliHelpComponent.ts` - CLI reference

---

### `src-frontend/renderers/`
**Purpose**: HTML generation functions
**Pattern**: Pure functions returning HTML strings

#### `renderers.ts`
Main rendering functions

#### `lifecycle.ts`
Lifecycle-specific renderers

---

### `src-frontend/utils.ts`
**Purpose**: Utility functions
**Functions**:
- String formatting
- Date formatting
- Number formatting
- DOM helpers

---

## Module Dependency Graph

```
main.rs
  ├─> cli.rs
  │     └─> analyser::logic
  │     └─> pipeline::executor
  └─> tauri_app.rs
        └─> analyser::*
        └─> pipeline::*
        └─> python_runner

analyser/
  ├─> logic/
  │     ├─> analysis.rs (orchestrator)
  │     ├─> profiling.rs
  │     ├─> types.rs
  │     ├─> health.rs
  │     ├─> cleaning.rs
  │     ├─> ml.rs
  │     └─> interpretation.rs
  └─> lifecycle/
        ├─> mod.rs (registry)
        ├─> version.rs
        ├─> storage.rs
        ├─> transforms.rs
        ├─> diff.rs
        ├─> query.rs
        └─> stages/

pipeline/
  ├─> spec.rs
  ├─> executor.rs
  ├─> validation.rs
  └─> powershell.rs

Frontend:
main.ts
  ├─> api.ts (Tauri bridge)
  ├─> components/ (UI)
  ├─> renderers/ (HTML)
  ├─> types.ts
  └─> utils.ts
```

---

## Quick Reference

### Adding a New Lifecycle Stage

1. Create file in `src/analyser/lifecycle/stages/`
2. Implement `StageExecutor` trait
3. Add stage to `LifecycleStage` enum
4. Register in stage factory

### Adding a New CLI Command

1. Add command to `cli::Cli` enum
2. Implement handler in `cli::run_command()`
3. Update help text

### Adding a New Tauri Command

1. Write function in `src/tauri_app.rs` with `#[tauri::command]`
2. Add to `generate_handler![]` macro
3. Create TypeScript wrapper in `src-frontend/api.ts`
4. Update types in `src-frontend/types.ts`

### Adding a New Frontend Component

1. Create class extending `Component` in `src-frontend/components/`
2. Implement `render(state)` method
3. Register in `main.ts::initComponents()`
4. Add navigation item in HTML

---

## Further Reading

- [ARCHITECTURE.md](ARCHITECTURE.md) - System design
- [LEARNING_GUIDE.md](LEARNING_GUIDE.md) - Getting started
- [RUST_CONCEPTS.md](RUST_CONCEPTS.md) - Rust patterns
- [TYPESCRIPT_PATTERNS.md](TYPESCRIPT_PATTERNS.md) - Frontend patterns

For implementation details, run `cargo doc --open` or `npm run docs:ts`.
