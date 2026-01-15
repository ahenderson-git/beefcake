# Beefcake Documentation

Welcome to the Beefcake documentation! This directory contains guides, architecture docs, and learning resources.

## 📚 Documentation Structure

### For New Users / Learners

Start here if you're new to the codebase, Rust, or TypeScript:

1. **[LEARNING_GUIDE.md](LEARNING_GUIDE.md)** - Start here!
   - Project structure overview
   - Quick start guide
   - Learning path for beginners
   - Tips for understanding the codebase

2. **[RUST_CONCEPTS.md](RUST_CONCEPTS.md)** - Rust patterns explained
   - Ownership & borrowing
   - Error handling with Result
   - Traits and generics
   - Async/await
   - Smart pointers (Arc, RwLock, Box)
   - Real examples from the codebase

3. **[TYPESCRIPT_PATTERNS.md](TYPESCRIPT_PATTERNS.md)** - Frontend patterns
   - TypeScript basics
   - Async/await & Promises
   - Tauri bridge pattern
   - Component architecture
   - State management
   - Event handling

### For Understanding the System

4. **[ARCHITECTURE.md](ARCHITECTURE.md)** - System design
   - High-level architecture
   - Data flow diagrams
   - Key design patterns
   - Subsystem descriptions
   - Performance considerations

5. **[MODULES.md](MODULES.md)** - Module reference
   - Detailed module breakdown
   - Responsibilities and APIs
   - Dependency graph
   - Quick reference for adding features

### Generated Documentation

In addition to these guides, you can generate detailed API documentation:

#### Rust API Docs
```bash
# Generate and open
cargo doc --open --document-private-items

# Or use the helper script
make docs-rust         # Unix/Linux/macOS
.\scripts\docs.ps1 rust    # Windows PowerShell
```

Output: `target/doc/beefcake/index.html`

#### TypeScript API Docs
```bash
# Generate
npm run docs:ts

# Or use the helper script
make docs-ts               # Unix/Linux/macOS
.\scripts\docs.ps1 ts      # Windows PowerShell
```

Output: `docs/typescript/index.html`

---

## 🎯 Documentation by Task

### I want to...

#### Learn Rust basics from this codebase
→ Read [RUST_CONCEPTS.md](RUST_CONCEPTS.md)
→ Study `src/error.rs` (well-commented error handling)
→ Look at `src/analyser/lifecycle.rs` (ownership examples)

#### Learn TypeScript/Frontend patterns
→ Read [TYPESCRIPT_PATTERNS.md](TYPESCRIPT_PATTERNS.md)
→ Study `src-frontend/main.ts` (state management)
→ Look at `src-frontend/api.ts` (Tauri bridge)

#### Understand the overall system
→ Read [ARCHITECTURE.md](ARCHITECTURE.md)
→ Trace data flow from user action to backend and back

#### Find what a specific module does
→ Read [MODULES.md](MODULES.md)
→ Use Ctrl+F to find the module name

#### Add a new feature
1. Read [ARCHITECTURE.md](ARCHITECTURE.md) - understand where it fits
2. Read [MODULES.md](MODULES.md) - find relevant modules
3. Read inline code comments - understand implementation details
4. Run `cargo doc --open` - browse API docs

#### Debug an issue
1. Read relevant module in [MODULES.md](MODULES.md)
2. Check inline comments in the source file
3. Look at tests for expected behaviour
4. Use `RUST_LOG=debug` to see detailed logs

#### Contribute to the project
1. Read [LEARNING_GUIDE.md](LEARNING_GUIDE.md) - get oriented
2. Read [ARCHITECTURE.md](ARCHITECTURE.md) - understand design principles
3. Look at existing code with similar functionality
4. Write tests and documentation for your changes

---

## 📖 Documentation Standards

### Rust Documentation

We use rustdoc conventions:

```rust
//! Module-level documentation (top of file)
//! Explains purpose, concepts, examples

/// Function documentation (above items)
///
/// # Arguments
/// * `path` - File path to analyze
///
/// # Returns
/// Analysis results or error
///
/// # Examples
/// ```
/// let result = analyze_file("data.csv")?;
/// ```
pub fn analyze_file(path: &str) -> Result<AnalysisResponse> {
    // Inline comment explaining complex logic
    let df = polars::LazyFrame::scan_csv(path)?;
    Ok(result)
}
```

### TypeScript Documentation

We use JSDoc/TSDoc conventions:

```typescript
/**
 * Analyzes a data file and returns statistics.
 *
 * @param path - Absolute path to the data file
 * @returns Promise resolving to analysis results
 * @throws Error if file not found or invalid format
 *
 * @example
 * ```typescript
 * const response = await api.analyseFile('data.csv');
 * console.log(response.row_count);
 * ```
 */
export async function analyseFile(path: string): Promise<AnalysisResponse> {
  return await invoke("analyze_file", { path });
}
```

---

## 🛠️ Generating Documentation

### All Documentation
```bash
# Unix/Linux/macOS
make docs

# Windows PowerShell
.\scripts\docs.ps1 all

# npm script (both platforms)
npm run docs
```

### Rust Documentation Only
```bash
# Unix/Linux/macOS
make docs-rust

# Windows PowerShell
.\scripts\docs.ps1 rust

# Direct cargo command
cargo doc --document-private-items --open
```

### TypeScript Documentation Only
```bash
# Unix/Linux/macOS
make docs-ts

# Windows PowerShell
.\scripts\docs.ps1 ts

# Direct npm command
npm run docs:ts
```

---

## 🔍 Tips for Reading Documentation

### Rust Docs (cargo doc)

1. **Search is your friend**: Use the search bar at the top
2. **Browse by module**: Left sidebar shows module hierarchy
3. **Look at examples**: Most functions have usage examples
4. **Check trait implementations**: See what traits a type implements
5. **Follow links**: Click type names to see their definitions

### TypeScript Docs (typedoc)

1. **Use navigation**: Left sidebar organized by module
2. **Filter by kind**: Classes, Interfaces, Functions, etc.
3. **See inheritance**: TypeDoc shows class hierarchies
4. **Read descriptions**: Module and function-level docs explain purpose

### Markdown Guides (this directory)

1. **Start with LEARNING_GUIDE.md**: Best entry point
2. **Use the table of contents**: Jump to relevant sections
3. **Follow links**: Cross-references between docs
4. **Read examples**: Code snippets show real usage

---

## 📝 Contributing to Documentation

When adding new code, please document it:

### For Rust:
- Add module docs (`//!`) at top of file
- Add function/struct docs (`///`) above items
- Include examples in doc comments
- Add inline comments for complex logic

### For TypeScript:
- Add file-level JSDoc comment at top
- Add JSDoc comments for exported functions
- Include `@param`, `@returns`, `@throws` annotations
- Add usage examples

### For Architecture Changes:
- Update [ARCHITECTURE.md](ARCHITECTURE.md)
- Update [MODULES.md](MODULES.md) if modules change
- Add to learning guides if introducing new concepts

---

## 🎓 Learning Resources

### Rust
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rustlings](https://github.com/rust-lang/rustlings)

### TypeScript
- [TypeScript Handbook](https://www.typescriptlang.org/docs/handbook/intro.html)
- [TypeScript Deep Dive](https://basarat.gitbook.io/typescript/)

### Tauri
- [Tauri Documentation](https://tauri.app/v1/guides/)
- [Tauri Examples](https://github.com/tauri-apps/tauri/tree/dev/examples)

### Polars (Data Processing)
- [Polars User Guide](https://pola-rs.github.io/polars-book/)
- [Polars API Docs](https://docs.rs/polars/latest/polars/)

---

## ❓ Getting Help

### Documentation Issues
- If documentation is unclear, open an issue
- Suggest improvements via pull request
- Ask questions in discussions

### Code Questions
1. Check [LEARNING_GUIDE.md](LEARNING_GUIDE.md)
2. Search generated docs (`cargo doc --open`)
3. Read inline code comments
4. Look at tests for examples
5. Ask in discussions or issues

---

## 📦 Documentation Files

```
docs/
├── README.md                    (this file)
├── LEARNING_GUIDE.md           (start here!)
├── RUST_CONCEPTS.md            (Rust patterns explained)
├── TYPESCRIPT_PATTERNS.md      (Frontend patterns)
├── ARCHITECTURE.md             (system design)
├── MODULES.md                  (module reference)
├── AUTOMATION.md               (existing automation guide)
└── typescript/                 (generated TypeDoc output)

target/doc/                     (generated rustdoc output)
└── beefcake/
    └── index.html              (Rust API docs)
```

---

## 🚀 Quick Links

- **Project Root**: [../README.md](../README.md)
- **Learning Guide**: [LEARNING_GUIDE.md](LEARNING_GUIDE.md)
- **Architecture**: [ARCHITECTURE.md](ARCHITECTURE.md)
- **Rust Concepts**: [RUST_CONCEPTS.md](RUST_CONCEPTS.md)
- **TypeScript Patterns**: [TYPESCRIPT_PATTERNS.md](TYPESCRIPT_PATTERNS.md)
- **Module Reference**: [MODULES.md](MODULES.md)

---

Happy learning! 🎉

For questions or improvements, please open an issue or pull request.
