# Beefcake Architecture

High-level system design and architecture documentation.

## Overview

Beefcake is a desktop data analysis application built with:
- **Backend**: Rust (high-performance data processing)
- **Frontend**: TypeScript + HTML/CSS (user interface)
- **Bridge**: Tauri (connects frontend to backend)

## System Architecture

```
┌────────────────────────────────────────────────────────────────┐
│                         Desktop Application                     │
│                              (Tauri)                            │
├────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────┐         ┌────────────────────────┐   │
│  │   Frontend (TS)     │   IPC   │   Backend (Rust)       │   │
│  │                     │ <────> │                        │   │
│  │  - UI Components    │         │  - Data Processing     │   │
│  │  - State Management │         │  - File I/O            │   │
│  │  - Event Handling   │         │  - Analysis Logic      │   │
│  │  - Rendering        │         │  - Database Ops        │   │
│  └─────────────────────┘         └────────────────────────┘   │
│                                                                 │
└────────────────────────────────────────────────────────────────┘
                              │
                              ▼
        ┌──────────────────────────────────────┐
        │         External Systems             │
        ├──────────────────────────────────────┤
        │  - CSV/JSON/Parquet Files            │
        │  - PostgreSQL Databases              │
        │  - Python Runtime (embedded)         │
        │  - PowerShell Runtime                │
        └──────────────────────────────────────┘
```

## Core Subsystems

### 1. Data Analysis Engine (`src/analyser/`)

```
src/analyser/
├── logic/              # Core analysis algorithms
│   ├── analysis.rs     # Main analysis orchestration
│   ├── profiling.rs    # Column statistics & profiling
│   ├── types.rs        # Type detection (numeric, text, etc.)
│   ├── health.rs       # Data quality assessment
│   ├── cleaning.rs     # Data cleaning transformations
│   ├── ml.rs           # ML preprocessing
│   └── interpretation.rs # Business insights generation
├── lifecycle/          # Dataset version management
│   ├── mod.rs          # Lifecycle registry & coordination
│   ├── version.rs      # Version data structures
│   ├── storage.rs      # File persistence layer
│   ├── transforms.rs   # Transformation pipeline
│   ├── diff.rs         # Version comparison
│   ├── query.rs        # Dataset querying
│   └── stages/         # Lifecycle stage implementations
│       ├── profile.rs  # Raw → Profiled
│       ├── clean.rs    # Profiled → Cleaned
│       ├── advanced.rs # Cleaned → Advanced
│       ├── validate.rs # Advanced → Validated
│       └── publish.rs  # Validated → Published
└── db.rs               # Database integration (PostgreSQL)
```

**Purpose**: Analyzes datasets, generates statistics, detects data quality issues, and manages dataset lifecycles.

**Key Concepts**:
- **Lazy Evaluation**: Uses Polars `LazyFrame` to defer computation until needed
- **Immutable Versions**: Never modifies raw data; creates new versions through transformations
- **Pipeline Architecture**: Transformations are composable and serializable

### 2. Pipeline Automation (`src/pipeline/`)

```
src/pipeline/
├── mod.rs           # Public API
├── spec.rs          # JSON pipeline specification format
├── executor.rs      # Pipeline execution engine
├── validation.rs    # Schema & step validation
└── powershell.rs    # PowerShell script generation
```

**Purpose**: Captures UI operations as reusable JSON "pipeline specs" for automation.

**Key Features**:
- **11 Transformation Steps**: Drop columns, rename, trim whitespace, cast types, parse dates, impute, normalize, clip outliers, one-hot encode, extract numbers, regex replace
- **8 Built-in Templates**: Pre-configured pipelines for common workflows (data cleaning, ML preprocessing, etc.)
- **Drag-and-Drop Editor**: Visual pipeline builder with step reordering
- **Validation Engine**: Schema checking against input datasets
- **PowerShell Export**: Generate standalone automation scripts

**Workflow**:
1. User creates pipeline in visual editor (or loads template)
2. Configure each step with parameters
3. Validate against input dataset schema
4. Save as JSON specification file
5. Execute via GUI or CLI: `beefcake pipeline execute spec.json`
6. Or export as PowerShell script for scheduling

**Example Pipeline Spec**:
```json
{
  "version": "0.1",
  "name": "Data Cleaning",
  "steps": [
    {"op": "trim_whitespace", "columns": ["name", "email"]},
    {"op": "drop_columns", "columns": ["temp_id"]},
    {"op": "impute", "strategy": "mean", "columns": ["age"]}
  ],
  "input": {"format": "csv"},
  "output": {"format": "parquet", "path": "cleaned.parquet"}
}
```

### 3. Filesystem Watcher (`src/watcher/`)

```
src/watcher/
├── mod.rs           # Public API & global service
├── config.rs        # Configuration persistence
├── events.rs        # Event types & payloads
└── service.rs       # Background watcher service
```

**Purpose**: Monitors a folder for new data files and automatically ingests them into the lifecycle system.

**Architecture**:
- **Background Thread**: Runs independently from main application
- **notify Crate**: OS-level filesystem event notifications (inotify/FSEvents/ReadDirectoryChangesW)
- **Stability Checker**: Waits for file writes to complete before ingestion
- **Event Emission**: Sends real-time updates to frontend via Tauri events

**Event Flow**:
```
Filesystem Change
      ↓
notify::Watcher detects event
      ↓
WatcherService filters & validates
      ↓
StabilityChecker waits for completion
      ↓
Ingest file → Create dataset (Raw stage)
      ↓
Emit success/failure event to UI
      ↓
Update activity feed
```

**Key Features**:
- Non-recursive folder monitoring (single directory)
- File stability detection (prevents incomplete reads)
- Supported formats: CSV, JSON, Parquet
- Persistent configuration (enabled state, folder path)
- Auto-start on app launch
- Real-time activity feed with status indicators

**Limitations**:
- Single folder only (no multi-folder support)
- No file filtering or pattern matching
- No deduplication (same file ingested multiple times)

### 4. Frontend UI (`src-frontend/`)

```
src-frontend/
├── main.ts              # Application entry & state management
├── api.ts               # Tauri bridge to Rust backend
├── api-pipeline.ts      # Pipeline-specific API bindings
├── types.ts             # TypeScript type definitions
├── components/          # UI component classes
│   ├── Component.ts           # Abstract base component
│   ├── DashboardComponent.ts
│   ├── AnalyserComponent.ts   # Data profiling & cleaning
│   ├── LifecycleComponent.ts  # Version tree & publishing
│   ├── LifecycleRailComponent.ts  # Stage navigation sidebar
│   ├── PipelineComponent.ts   # Pipeline manager wrapper
│   ├── PipelineLibrary.ts     # Pipeline browser & templates
│   ├── PipelineEditor.ts      # Visual drag-and-drop editor
│   ├── PipelineExecutor.ts    # Execution modal with progress
│   ├── StepPalette.ts         # 11 transformation step types
│   ├── StepConfigPanel.ts     # Dynamic step configuration forms
│   ├── WatcherComponent.ts    # Filesystem watcher UI
│   ├── PowerShellComponent.ts
│   ├── PythonComponent.ts
│   ├── SQLComponent.ts
│   └── SettingsComponent.ts
├── renderers/           # HTML generation functions
│   ├── analyser.ts
│   ├── lifecycle.ts
│   ├── watcher.ts
│   └── layout.ts
└── utils.ts             # Utility functions
```

**Component Architecture**:
- **Abstract Base**: `Component` class with `render()` and `bindEvents()` methods
- **Composition**: Components can contain sub-components (e.g., `PipelineComponent` → `PipelineEditor` → `StepPalette`)
- **State-Driven**: All components re-render on state changes
- **Event Emission**: Custom events for inter-component communication

**Pipeline Editor Architecture**:
```
PipelineComponent (manager)
    │
    ├─> PipelineLibrary (list view)
    │     ├─> My Pipelines tab
    │     │     ├─> Search/filter
    │     │     ├─> Pipeline cards (edit/execute/delete actions)
    │     │     └─> Empty state with "Create Pipeline" button
    │     │
    │     └─> Templates tab
    │           ├─> 8 pre-built templates
    │           ├─> Template categories (cleaning, ML, dates, etc.)
    │           └─> "Use Template" action
    │
    ├─> PipelineEditor (edit view)
    │     ├─> StepPalette (left sidebar - 250px)
    │     │     ├─> Column Management (drop, rename)
    │     │     ├─> Text Processing (trim, regex)
    │     │     ├─> Type Conversion (cast, parse dates)
    │     │     ├─> Missing Values (impute)
    │     │     └─> Machine Learning (normalize, one-hot, clip, extract)
    │     │
    │     ├─> Pipeline Canvas (center - flexible)
    │     │     ├─> Step cards with drag handles (⋮⋮)
    │     │     ├─> HTML5 Drag-and-Drop API
    │     │     ├─> Visual feedback (opacity, borders)
    │     │     ├─> Selection tracking
    │     │     └─> Up/Down reorder buttons (fallback)
    │     │
    │     ├─> StepConfigPanel (right sidebar - 350px)
    │     │     ├─> Step-specific forms
    │     │     ├─> Column multi-select dropdowns
    │     │     ├─> Parameter validation
    │     │     └─> Help text and examples
    │     │
    │     └─> Toolbar
    │           ├─> Save button
    │           ├─> Execute button
    │           ├─> Back to library
    │           └─> Pipeline name/description
    │
    └─> PipelineExecutor (modal overlay)
          ├─> File selection (input/output)
          ├─> Progress tracking
          ├─> Step-by-step feedback
          └─> Success/error results
```

**Pipeline Data Flow**:
```
User clicks "New Pipeline" in Library
    ↓
PipelineComponent.showEditor() with empty spec
    ↓
PipelineEditor renders with empty canvas
    ↓
User drags step from StepPalette to canvas
    ↓
PipelineEditor.addStep(stepType)
    ↓
Canvas re-renders with new step card
    ↓
User clicks step card
    ↓
StepConfigPanel renders step-specific form
    ↓
User configures parameters (columns, strategy, etc.)
    ↓
StepConfigPanel.updateStepParams()
    ↓
Pipeline spec updated in editor state
    ↓
User clicks "Save"
    ↓
api-pipeline.savePipeline(spec, path)
    ↓
Tauri invokes Rust "save_pipeline" command
    ↓
JSON written to disk
    ↓
PipelineLibrary refreshed
```

**Drag-and-Drop Implementation**:
```typescript
// PipelineEditor.ts
class PipelineEditor {
  private draggedStepIndex: number | null = null;

  // Attach drag event handlers
  private attachDragHandlers(card: HTMLElement, index: number) {
    card.addEventListener('dragstart', (e) => {
      this.draggedStepIndex = index;
      card.classList.add('dragging');
      e.dataTransfer.effectAllowed = 'move';
    });

    card.addEventListener('dragover', (e) => {
      e.preventDefault();
      e.dataTransfer.dropEffect = 'move';
      card.classList.add('drag-over');
    });

    card.addEventListener('drop', (e) => {
      e.preventDefault();
      const dropIndex = parseInt(card.dataset.index);
      this.moveStepToIndex(this.draggedStepIndex, dropIndex);
      card.classList.remove('drag-over');
    });

    card.addEventListener('dragend', () => {
      card.classList.remove('dragging');
      this.draggedStepIndex = null;
    });
  }

  // Reorder steps and update selection
  private moveStepToIndex(from: number, to: number) {
    const steps = [...this.state.pipeline.steps];
    const [movedStep] = steps.splice(from, 1);
    steps.splice(to, 0, movedStep);

    this.state.pipeline.steps = steps;

    // Update selected index if necessary
    if (this.state.selectedStepIndex === from) {
      this.state.selectedStepIndex = to;
    }

    this.render();
  }
}
```

**Architecture**: Component-based with centralized state management.

**Data Flow**:
```
User Event → Component → Update State → Render → Update DOM
                ↓
           Tauri invoke()
                ↓
           Rust Backend
```

### 4. CLI Interface (`src/cli.rs`)

```text
Commands:
- analyze [file]              # Analyze CSV/JSON/Parquet
- clean [file] [config]       # Apply cleaning config
- pipeline execute [spec]     # Run pipeline
- pipeline validate [spec]    # Check pipeline validity
- db push [file] [conn-id]    # Upload to database
```

**Use Cases**:
- Headless processing on servers
- Integration with CI/CD pipelines
- Batch processing scripts
- Automation without GUI

### 5. Embedded Runtimes

#### Python Runner (`src/python_runner.rs`)
- Embeds Python interpreter
- Provides `df` variable (dataset as pandas DataFrame)
- Captures stdout/stderr with ANSI color support
- Package management via `pip`

#### PowerShell Runner (`src/pipeline/powershell.rs`)
- Executes PowerShell scripts
- Passes dataset paths as parameters
- Captures output streams

## Data Flow

### Analysis Flow

```text
1. User selects file
   ↓
2. Frontend: api.analyseFile(path)
   ↓
3. Tauri IPC: invoke("analyze_file", { path })
   ↓
4. Backend: analyze_file(path) -> Result<AnalysisResponse>
   ↓
5. Polars: Read file → LazyFrame
   ↓
6. Analyser: Detect types, compute stats, assess health
   ↓
7. Response: JSON with stats, issues, recommendations
   ↓
8. Frontend: Update state, render results
```

### Lifecycle Flow

```text
1. Create Dataset (Raw stage)
   ↓
2. Profile (analyze, generate recommendations)
   ↓
3. Clean (apply text/type transformations)
   ↓
4. Advanced (ML preprocessing: imputation, normalization)
   ↓
5. Validate (QA gates, business rules)
   ↓
6. Publish (create view or snapshot)
```

Each stage creates a new immutable version with:
- Transformation pipeline (JSON)
- Data location (file path)
- Metadata (timestamp, stats)
- Parent version reference

### Pipeline Execution Flow

```text
1. Load PipelineSpec from JSON
   ↓
2. Validate spec (schema, column existence)
   ↓
3. Load input data
   ↓
4. For each step in spec:
   - Apply transformation
   - Validate result
   ↓
5. Write output file
   ↓
6. Generate report
```

## Key Design Patterns

### 1. Trait-Based Extensibility

```rust
pub trait StageExecutor {
    fn execute(&self, df: LazyFrame) -> Result<LazyFrame>;
}

// Each lifecycle stage implements this trait
impl StageExecutor for CleanStage { /* ... */ }
impl StageExecutor for AdvancedStage { /* ... */ }
```

Benefits:
- Easy to add new stages
- Polymorphic execution
- Testable in isolation

### 2. Immutable Versioning

```rust
impl Dataset {
    // Never mutates data, always creates new version
    pub fn apply_transforms(&mut self, pipeline: TransformPipeline) -> Result<Uuid> {
        let new_version = self.create_version(pipeline)?;
        self.versions.add(new_version);
        Ok(new_version.id)
    }
}
```

Benefits:
- Reproducibility (can recreate any version)
- Audit trail (know what changed when)
- Safe experimentation (can always revert)

### 3. Lazy Evaluation

```rust
fn example() -> Result<()> {
    // Build query plan (no data loaded yet)
    let lf = LazyFrame::scan_csv(path)?
        .select([col("age"), col("name")])
        .filter(col("age").gt(18))
        .groupby([col("country")])
        .agg([col("age").mean()]);

    // Execute only when needed
    let df = lf.collect()?;  // Now data is processed
    Ok(())
}
```

Benefits:
- Memory efficiency
- Query optimisation
- Large dataset support

### 4. Result-Based Error Handling

```rust
pub fn analyze(path: &str) -> Result<AnalysisResponse> {
    let df = read_file(path)?;  // Propagate errors
    let stats = compute_stats(&df)?;
    Ok(AnalysisResponse { stats })
}
```

Benefits:
- Explicit error handling
- No hidden exceptions
- Composable with `?` operator

### 5. Type-Safe IPC Bridge

```rust
// Rust
#[derive(Serialize, Deserialize)]
pub struct AnalysisResponse {
    pub file_name: String,
    pub summary: Vec<ColumnSummary>,
}

#[tauri::command]
fn analyze_file(path: String) -> Result<AnalysisResponse, String> {
    // ...
}
```

```typescript
// TypeScript (matching types)
export interface AnalysisResponse {
  file_name: string;
  summary: ColumnSummary[];
}

export async function analyseFile(path: string): Promise<AnalysisResponse> {
  return await invoke("analyze_file", { path });
}
```

Benefits:
- Compile-time type checking
- Auto-serialization/deserialization
- Refactoring safety

## Performance Considerations

### Memory Management

1. **Streaming**: Large files processed in chunks
2. **Lazy Evaluation**: Only compute what's needed
3. **Arc/RwLock**: Shared state without copying

### Concurrency

1. **Tokio Runtime**: Async I/O operations
2. **Polars**: Multi-threaded DataFrame operations
3. **RwLock**: Multiple readers, single writer

### Optimisation

1. **Release Builds**: Optimised with `-C opt-level=2`
2. **Dependency Optimisation**: Even debug builds optimise dependencies
3. **LazyFrame**: Query optimisation before execution

## Security Model

### Sandboxing

Tauri provides process isolation:
- Frontend runs in webview (limited privileges)
- Backend runs as native process (file system access)
- IPC whitelist: Only exposed commands are callable

### Data Safety

1. **Immutable Original**: Raw data never modified
2. **Type Safety**: Rust prevents memory errors
3. **Input Validation**: CLI/Tauri commands validate inputs

### Credentials

1. **System Keyring**: Database passwords stored securely
2. **No Plaintext**: Passwords never in config files
3. **Environment Variables**: For CLI usage

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_type_detection() {
        // Test individual functions
    }
}
```

### Integration Tests
```rust
#[test]
fn test_full_analysis_workflow() {
    // Test complete analysis pipeline
}
```

### Manual Testing
- GUI testing via Tauri dev mode
- CLI testing with sample datasets
- Memory profiling with large files

## Deployment

### Desktop Application

```bash
# Development
npm run tauri dev

# Production build
npm run tauri build
```

Output: Platform-specific installer
- Windows: `.msi` or `.exe`
- macOS: `.dmg` or `.app`
- Linux: `.deb`, `.rpm`, or `.AppImage`

### CLI Tool

```bash
# Install from source
cargo install --path .

# Run
beefcake analyze data.csv
```

## Future Architecture Considerations

### Scalability
- Add server mode for web deployment
- Distributed processing for multi-GB files
- Cloud storage integration (S3, Azure)

### Extensibility
- Plugin system for custom transformations
- External script support (R, Julia)
- Custom visualization components

### Collaboration
- Multi-user dataset access
- Version control integration (Git-like)
- Shared pipeline library

## Related Documentation

- [LEARNING_GUIDE.md](LEARNING_GUIDE.md) - Getting started
- [MODULES.md](MODULES.md) - Detailed module documentation
- [RUST_CONCEPTS.md](RUST_CONCEPTS.md) - Rust patterns used
- [TYPESCRIPT_PATTERNS.md](TYPESCRIPT_PATTERNS.md) - Frontend patterns

---

For implementation details, see inline code documentation via `cargo doc --open`.
