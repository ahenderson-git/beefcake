# Beefcake Learning Guide

Welcome! This guide helps you understand the Beefcake codebase, especially if you're new to Rust and TypeScript.

## 🎯 What is Beefcake?

Beefcake is a data analysis and transformation tool with:
- **Backend (Rust)**: High-performance data processing using Polars (similar to pandas)
- **Frontend (TypeScript)**: Desktop UI built with Tauri (like Electron, but faster)
- **Features**: CSV analysis, data cleaning, pipeline automation, SQL/Python integration

## 📁 Project Structure

```
beefcake/
├── src/                          # Rust backend code
│   ├── main.rs                   # Application entry point
│   ├── lib.rs                    # Library root (public API)
│   ├── analyser/                 # Data analysis engine
│   │   ├── logic/                # Core analysis algorithms
│   │   └── lifecycle/            # Dataset version management
│   ├── pipeline/                 # Automation pipeline system
│   ├── cli.rs                    # Command-line interface
│   ├── tauri_app.rs              # Desktop app bridge
│   └── error.rs                  # Error handling types
├── src-frontend/                 # TypeScript frontend code
│   ├── main.ts                   # Frontend entry point
│   ├── api.ts                    # Rust backend communication
│   ├── types.ts                  # TypeScript type definitions
│   ├── components/               # UI components
│   └── renderers/                # HTML rendering logic
├── docs/                         # Documentation (you are here!)
└── Cargo.toml                    # Rust dependencies
```

## 🦀 Rust Basics You'll See

### 1. **Result Type** - Error Handling
```rust
fn analyze_file(path: &str) -> Result<DataFrame> {
    // Result&lt;T&gt; means: returns T on success, Error on failure
    // No exceptions in Rust!
    let df = read_csv(path)?;  // ? propagates errors up
    Ok(df)
}
```

### 2. **Ownership & Borrowing**
```rust
fn process(data: DataFrame) { }      // Takes ownership (moves data)
fn analyze(data: &DataFrame) { }     // Borrows immutably (read-only)
fn modify(data: &mut DataFrame) { }  // Borrows mutably (can change)
```

### 3. **Traits** - Like Interfaces
```rust
pub trait StageExecutor {
    fn execute(&self, df: LazyFrame) -> Result<LazyFrame>;
}
// Any type implementing this can be used as a stage
```

### 4. **Match Expressions** - Pattern Matching
```rust
fn handle_type(column_type: ColumnType) {
    match column_type {
        ColumnType::Numeric => process_numbers(),
        ColumnType::Text => process_text(),
        _ => default_handler(),
    }
}
```

## 💻 TypeScript Basics You'll See

### 1. **Interfaces** - Type Definitions
```typescript
interface AnalysisResponse {
    file_name: string;
    summary: ColumnSummary[];
    health: FileHealth;
}
```

### 2. **Async/Await** - Handling Promises
```typescript
async function loadData() {
    const data = await api.analyseFile(path);  // Wait for Rust backend
    console.log(data);
}
```

### 3. **Tauri invoke()** - Call Rust from TypeScript
```typescript
import { invoke } from "@tauri-apps/api/core";

// This calls a #[tauri::command] function in Rust
const result = await invoke("analyze_file", { path: "/data.csv" });
```

## 🗺️ Key Modules Explained

### Backend (Rust)

#### `src/analyser/`
**Purpose**: Analyzes CSV/JSON files and generates statistics
**Key Files**:
- `logic/analysis.rs` - Column type detection, statistics
- `logic/profiling.rs` - Data quality metrics
- `lifecycle.rs` - Version control for datasets

#### `src/pipeline/`
**Purpose**: Automates data transformations
**Key Files**:
- `spec.rs` - JSON pipeline definition format
- `executor.rs` - Runs pipeline steps
- `powershell.rs` - Generates automation scripts

### Frontend (TypeScript)

#### `src-frontend/main.ts`
**Purpose**: Main application controller
**Pattern**: Component-based architecture with centralized state

#### `src-frontend/api.ts`
**Purpose**: Bridge to Rust backend
**Pattern**: All backend calls go through Tauri's `invoke()` function

#### `src-frontend/components/`
**Purpose**: UI components (Dashboard, Analyser, Settings, etc.)
**Pattern**: Each component manages its own rendering and events

## 🔍 How Data Flows

```
User Action (UI)
    ↓
TypeScript Event Handler
    ↓
api.ts invoke() call
    ↓
Tauri IPC Bridge
    ↓
Rust #[tauri::command]
    ↓
Analyser Logic (Polars DataFrame)
    ↓
JSON Result
    ↓
TypeScript receives response
    ↓
Update UI State
    ↓
Re-render Component
```

## 📚 Learning Path

### For Complete Beginners

1. **Start with docs**:
   - Read `ARCHITECTURE.md` for the big picture
   - Read `MODULES.md` for module details

2. **Trace a simple flow**:
   - Open `src/main.rs` - see how app starts
   - Follow to `src/cli.rs` - see command handling
   - Look at `src-frontend/main.ts` - see UI initialization

3. **Read commented code**:
   - Files have inline comments explaining complex logic
   - Module docs (`//!` in Rust) explain purpose and architecture

### For Rust Learners

1. Read `RUST_CONCEPTS.md` for patterns used in this project
2. Study `src/analyser/lifecycle.rs` - great example of:
   - Trait-based design
   - Arc/RwLock for shared state
   - Error handling with Result
3. Look at `src/error.rs` - custom error types with `anyhow`

### For TypeScript Learners

1. Read `TYPESCRIPT_PATTERNS.md` for frontend patterns
2. Study `src-frontend/main.ts` - example of:
   - Class-based state management
   - Async/await patterns
   - Event-driven architecture
3. Look at `src-frontend/components/Component.ts` - base component pattern

## 🛠️ Generating Documentation

### Rust Documentation
```bash
# Generate and open HTML docs
cargo doc --open

# Include private items
cargo doc --document-private-items --open
```

### TypeScript Documentation
```bash
# Install TypeDoc
npm install -g typedoc

# Generate docs
npx typedoc --out docs/typescript src-frontend
```

## 🤔 Common Questions

### "What's the difference between LazyFrame and DataFrame?"
- **DataFrame**: All data loaded in memory (eager evaluation)
- **LazyFrame**: Query plan, only executes when needed (lazy evaluation)
- Beefcake uses LazyFrame for memory efficiency with large files

### "Why use Rust for the backend?"
- Speed: 10-100x faster than Python for data processing
- Safety: No null pointer exceptions, no data races
- Memory: Efficient handling of large datasets

### "What's Tauri?"
- Like Electron, but uses system webview instead of bundling Chrome
- Rust backend + web frontend
- Much smaller binaries (~5MB vs 100MB+)

### "Where do I start making changes?"
1. For new analysis features: `src/analyser/logic/`
2. For UI changes: `src-frontend/components/`
3. For new CLI commands: `src/cli.rs`
4. For pipeline features: `src/pipeline/`

## 📖 Next Steps

- Read `ARCHITECTURE.md` for system design
- Read `RUST_CONCEPTS.md` for Rust patterns
- Read `TYPESCRIPT_PATTERNS.md` for frontend patterns
- Read `MODULES.md` for detailed module documentation
- Look at inline code comments for implementation details

## 💡 Tips for Learning

1. **Use the docs**: `cargo doc --open` generates excellent documentation
2. **Read tests**: Test files show how to use APIs
3. **Follow compiler errors**: Rust compiler is very helpful
4. **Use rust-analyzer**: IDE extension provides inline type hints
5. **Ask questions**: Comment files with "Why?" not just "What?"

Happy learning! 🚀
