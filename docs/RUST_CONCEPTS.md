# Rust Concepts in Beefcake

This guide explains Rust concepts and patterns used throughout the Beefcake codebase.

## Table of Contents
- [Ownership & Borrowing](#ownership--borrowing)
- [Result & Error Handling](#result--error-handling)
- [Traits](#traits)
- [Generics](#generics)
- [Smart Pointers](#smart-pointers)
- [Async/Await](#asyncawait)
- [Macros](#macros)
- [Module System](#module-system)

---

## Ownership & Borrowing

### Core Concept
Rust's memory safety comes from these rules:
1. Each value has one owner
2. When owner goes out of scope, value is dropped (freed)
3. You can borrow references (&T immutable, &mut T mutable)

### Examples from Beefcake

#### Moving Ownership (src/analyser/lifecycle.rs:56)
```rust
pub fn create_dataset(&self, name: String, raw_data_path: PathBuf) -> Result<Uuid> {
    // `name` and `raw_data_path` are moved into Dataset::new()
    // After this call, we can't use them again in this function
    let dataset = Dataset::new(name, raw_data_path, Arc::clone(&self.store))?;
    // ...
}
```

#### Immutable Borrowing (src/analyser/lifecycle.rs:67)
```rust
pub fn get_dataset(&self, id: &Uuid) -> Result<Dataset> {
    // &Uuid borrows the ID without taking ownership
    // Caller still owns the UUID after this function returns
    let datasets = self.datasets.read()?;
    datasets.get(id).cloned()
}
```

#### Mutable Borrowing (src/analyser/lifecycle.rs:82)
```rust
pub fn apply_transforms(&self, dataset_id: &Uuid, /*...*/) -> Result<Uuid> {
    let mut datasets = self.datasets.write()?;
    // get_mut returns &mut Dataset - we can modify it
    let dataset = datasets.get_mut(dataset_id)?;
    dataset.apply_pipeline(pipeline, stage)  // Can modify dataset
}
```

### When to Use What?

| Pattern | When to Use | Example |
|---------|-------------|---------|
| `T` (move) | Transfer ownership, caller won't need it | `String`, `Vec<T>` parameters |
| `&T` (borrow) | Read-only access, caller keeps ownership | `&str`, `&[u8]` parameters |
| `&mut T` (mutable borrow) | Need to modify, caller keeps ownership | Modifying collections |
| `Clone` | Need a copy, performance OK | Small types, infrequent calls |

---

## Result & Error Handling

### Core Concept
Rust has no exceptions. Functions that can fail return `Result<T, E>`:
- `Ok(value)` - Success
- `Err(error)` - Failure

### Examples from Beefcake

#### Returning Results (throughout codebase)
```rust
pub fn create_dataset(&self, name: String, path: PathBuf) -> Result<Uuid> {
    // Result is from anyhow crate: Result<T, anyhow::Error>
    if !path.exists() {
        return Err(anyhow::anyhow!("Path does not exist: {}", path.display()));
    }
    Ok(uuid)
}
```

#### The ? Operator (propagating errors)
```rust
pub fn load_data(&self) -> Result<DataFrame> {
    // If read_csv fails, ? returns the error immediately
    let df = polars::io::csv::CsvReader::read_csv(path)?;

    // If validate fails, its error is returned
    self.validate(&df)?;

    // If we get here, both succeeded
    Ok(df)
}
```

#### Pattern Matching Results
```rust
match analyze_file(path) {
    Ok(data) => println!("Success: {}", data),
    Err(e) => eprintln!("Failed: {}", e),
}
```

#### anyhow vs thiserror

**anyhow** (used in application code):
```rust
use anyhow::{Result, Context};

fn process() -> Result<()> {
    let data = load_file()
        .context("Failed to load configuration")?;  // Add context
    Ok(())
}
```

**thiserror** (for custom error types in src/error.rs):
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("File not found: {0}")]
    NotFound(String),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}
```

---

## Traits

### Core Concept
Traits define shared behaviour (like interfaces in other languages).

### Examples from Beefcake

#### Defining Traits (src/analyser/lifecycle/stages.rs)
```rust
pub trait StageExecutor {
    /// Execute this stage's transformations on a lazy dataframe
    fn execute(&self, df: LazyFrame) -> Result<LazyFrame>;

    /// Validate that this stage can be applied
    fn validate(&self, df: &LazyFrame) -> Result<()> {
        // Default implementation
        Ok(())
    }
}
```

#### Implementing Traits
```rust
pub struct CleanStage {
    pub pipeline: TransformPipeline,
}

impl StageExecutor for CleanStage {
    fn execute(&self, df: LazyFrame) -> Result<LazyFrame> {
        self.pipeline.apply(df)
    }
}
```

#### Trait Bounds (generic constraints)
```rust
// T must implement both Debug and Clone traits
fn process<T: Debug + Clone>(item: T) {
    println!("Processing: {:?}", item);
    let copy = item.clone();
}

// Alternative "where" syntax (more readable for complex bounds)
fn process<T>(item: T)
where
    T: Debug + Clone,
{
    // ...
}
```

#### Commonly Used Traits in Beefcake

| Trait | Purpose | Example |
|-------|---------|---------|
| `Clone` | Create deep copy | `dataset.clone()` |
| `Debug` | Format with `{:?}` | `println!("{:?}", value)` |
| `Serialize/Deserialize` | JSON conversion (serde) | `serde_json::to_string(&data)` |
| `From/Into` | Type conversion | `let uuid: Uuid = string.parse()?` |

---

## Generics

### Core Concept
Write code that works with multiple types.

### Examples from Beefcake

#### Generic Functions
```rust
// Works with any type T that implements Serialize
fn to_json<T: Serialize>(value: &T) -> Result<String> {
    serde_json::to_string(value)
        .map_err(|e| anyhow::anyhow!("Serialization failed: {}", e))
}
```

#### Generic Structs
```rust
pub struct VersionStore<T> {
    data: HashMap<Uuid, T>,
}

impl<T> VersionStore<T> {
    pub fn new() -> Self {
        Self { data: HashMap::new() }
    }

    pub fn store(&mut self, id: Uuid, value: T) {
        self.data.insert(id, value);
    }
}
```

---

## Smart Pointers

### Core Concept
Types that act like pointers but have additional metadata and capabilities.

### Examples from Beefcake

#### `Box<T>` - Heap Allocation
```rust
// Put large data on heap instead of stack
let large_data: Box<DataFrame> = Box::new(df);
```

#### `Arc<T>` - Atomic Reference Counting (src/analyser/lifecycle.rs:42)
```rust
use std::sync::Arc;

pub struct DatasetRegistry {
    // Arc allows multiple owners, thread-safe
    store: Arc<VersionStore>,
}

impl DatasetRegistry {
    pub fn new(base_path: PathBuf) -> Result<Self> {
        let store = Arc::new(VersionStore::new(base_path)?);
        Ok(Self {
            // Clone creates new pointer, not new data
            store: Arc::clone(&store),
        })
    }
}
```

#### `RwLock<T>` - Read-Write Lock (src/analyser/lifecycle.rs:41)
```rust
use std::sync::RwLock;

pub struct DatasetRegistry {
    // Multiple readers OR one writer
    datasets: Arc<RwLock<HashMap<Uuid, Dataset>>>,
}

impl DatasetRegistry {
    pub fn read_dataset(&self, id: &Uuid) -> Result<Dataset> {
        // Multiple threads can read simultaneously
        let datasets = self.datasets.read()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        datasets.get(id).cloned()
    }

    pub fn modify_dataset(&self, id: &Uuid) -> Result<()> {
        // Only one thread can write
        let mut datasets = self.datasets.write()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        datasets.get_mut(id);
        Ok(())
    }
}
```

#### Common Patterns

| Pattern | Use Case | Thread-Safe? |
|---------|----------|--------------|
| `Box<T>` | Heap allocation, trait objects | No |
| `Rc<T>` | Shared ownership (single thread) | No |
| `Arc<T>` | Shared ownership (multi-thread) | Yes |
| `Mutex<T>` | Exclusive access | Yes |
| `RwLock<T>` | Many readers, one writer | Yes |

---

## Async/Await

### Core Concept
Non-blocking asynchronous code using `async`/`await` syntax with the Tokio runtime.

### Examples from Beefcake

#### Async Functions (src/cli.rs)
```rust
use tokio::runtime::Runtime;

#[tokio::main]  // Macro sets up async runtime
async fn main() -> Result<()> {
    let result = analyze_data().await;  // Wait for async operation
    Ok(())
}

// Returns a Future
async fn analyze_data() -> Result<DataFrame> {
    // Can await other async functions
    let data = load_from_db().await?;
    Ok(data)
}
```

#### Tokio Runtime (src/main.rs:20)
```rust
// Create runtime manually
let runtime = tokio::runtime::Runtime::new()?;
runtime.block_on(async_function())?;  // Run async code to completion
```

#### Common Async Patterns

```rust
// Running multiple operations concurrently
use tokio::try_join;

async fn load_multiple() -> Result<()> {
    let (data1, data2, data3) = try_join!(
        load_file("a.csv"),
        load_file("b.csv"),
        load_file("c.csv"),
    )?;
    Ok(())
}

// Spawning background tasks
use tokio::spawn;

async fn background_work() {
    let handle = spawn(async {
        // This runs in the background
        expensive_computation().await
    });

    // Do other work...

    let result = handle.await.unwrap();
}
```

---

## Macros

### Core Concept
Macros generate code at compile time.

### Examples from Beefcake

#### Derive Macros (throughout codebase)
```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineSpec {
    pub version: String,
    pub steps: Vec<Step>,
}
// Compiler generates Debug, Clone, Serialize, Deserialize implementations
```

#### Function-like Macros
```rust
// println! and format! are macros
println!("Dataset: {} has {} rows", name, count);

// vec! macro creates vectors
let items = vec![1, 2, 3, 4, 5];

// anyhow! creates errors
return Err(anyhow::anyhow!("Something went wrong: {}", msg));
```

#### Attribute Macros (Tauri commands in src/tauri_app.rs)
```rust
#[tauri::command]
async fn analyze_file(path: String) -> Result<AnalysisResponse, String> {
    // This macro makes the function callable from TypeScript
    analyser::analyze(&path)
        .map_err(|e| e.to_string())
}
```

---

## Module System

### Core Concept
Organize code into hierarchical modules.

### Examples from Beefcake

#### Module Declaration (src/lib.rs)
```rust
// Declare modules (looks for mod.rs or <name>.rs)
pub mod analyser;  // -> analyser/mod.rs
pub mod error;     // -> error.rs
pub mod pipeline;  // -> pipeline.rs

// Re-export items for convenience
pub use analyser::logic::analyze_file;
pub use error::DataError;
```

#### Module Structure
```
src/
├── lib.rs                  (pub mod analyser; pub mod error;)
├── analyser/
│   ├── mod.rs             (pub mod logic; pub mod lifecycle;)
│   ├── logic/
│   │   ├── mod.rs         (pub mod analysis; pub use analysis::*;)
│   │   └── analysis.rs    (pub fn analyze_file() {})
│   └── lifecycle/
│       └── mod.rs
```

#### Visibility

```rust
pub fn public_fn() {}          // Available to everyone
pub(crate) fn crate_fn() {}    // Available within crate only
pub(super) fn parent_fn() {}   // Available to parent module only
fn private_fn() {}             // Only in this module
```

#### Using Modules
```rust
// Absolute path from crate root
use crate::analyser::logic::analysis;

// Relative path
use super::utils;      // Parent module
use self::helpers;     // Current module

// Multiple imports
use crate::analyser::{
    logic::analyze_file,
    lifecycle::Dataset,
};

// Wildcard (use sparingly)
use crate::analyser::logic::*;
```

---

## Common Patterns in Beefcake

### Builder Pattern
```rust
let spec = PipelineSpec::builder()
    .name("my-pipeline")
    .add_step(Step::Clean { .. })
    .add_step(Step::Transform { .. })
    .build()?;
```

### Newtype Pattern
```rust
// Wrapper for type safety
pub struct DatasetId(Uuid);
pub struct VersionId(Uuid);

// Can't accidentally mix them up!
fn get_dataset(id: DatasetId) -> Dataset { }
```

### Option & Result Combinators
```rust
// Instead of nested if/match
let value = optional_value
    .ok_or_else(|| anyhow::anyhow!("Missing value"))?
    .trim()
    .parse::<i32>()?;

// Map over Option
let doubled = maybe_number.map(|n| n * 2);

// Chain operations
dataset
    .get_version(&id)?
    .get_data()?
    .collect()?
```

---

## Resources for Learning More

### Official Resources
- [The Rust Book](https://doc.rust-lang.org/book/) - Start here!
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rustlings](https://github.com/rust-lang/rustlings) - Interactive exercises

### Beefcake-Specific
- Run `cargo doc --open` to browse generated documentation
- Read module docs (`//!` comments) at the top of files
- Look at test files to see usage examples

### Advanced Topics
- [Async Book](https://rust-lang.github.io/async-book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Polars User Guide](https://pola-rs.github.io/polars-book/)

---

## Quick Reference Card

### Memory
- `T` = move ownership
- `&T` = immutable borrow
- `&mut T` = mutable borrow
- `Box<T>` = heap allocation
- `Arc<T>` = shared ownership (thread-safe)
- `Rc<T>` = shared ownership (single thread)

### Errors
- `Result<T, E>` = either `Ok(T)` or `Err(E)`
- `?` = propagate error
- `unwrap()` = panic if error (avoid in production)
- `expect("msg")` = panic with message (use for unreachable cases)

### Collections
- `Vec<T>` = growable array
- `HashMap<K, V>` = key-value map
- `HashSet<T>` = unique values
- `[T; N]` = fixed-size array

### Traits (Derivable)
- `Debug` = `{:?}` formatting
- `Clone` = `.clone()` method
- `Copy` = automatic copying (stack only)
- `PartialEq`/`Eq` = equality comparison
- `PartialOrd`/`Ord` = ordering

---

Happy coding! 🦀
