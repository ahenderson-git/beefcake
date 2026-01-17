# Beefcake

> A desktop data analysis and transformation toolkit built with Rust and TypeScript

[![Version](https://img.shields.io/badge/version-0.2.0-blue.svg)](https://github.com/yourusername/beefcake)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![TypeScript](https://img.shields.io/badge/typescript-5.0+-blue.svg)](https://www.typescriptlang.org)

---

## Overview

Beefcake is a **high-performance desktop application** for data analysis, quality assessment, and automated transformation workflows. It combines the computational power of Rust with an intuitive TypeScript-based user interface to help data engineers, analysts, and scientists work with datasets ranging from small CSVs to multi-million row tables.

> **⚠️ Project Status**: Beefcake is an experimental, evolving project developed primarily as a learning and exploration platform for high-performance data tooling in Rust and TypeScript. While many core features are functional, this is a personal project subject to ongoing redesign, refactoring, and architectural changes. It is **not production-ready** and should be considered a prototype for exploring data engineering patterns.

### Key Features

- **📊 Interactive Data Analysis**: Profile datasets with statistics, detect data quality issues, and explore column distributions
- **🔄 Dataset Lifecycle Management**: Immutable version control through 6 stages (Raw → Profiled → Cleaned → Advanced → Validated → Published) with diff engine
- **⚙️ Visual Pipeline Builder**: Create data transformation pipelines with drag-and-drop interface, 11 step types, and 8 built-in templates
- **👁️ Filesystem Watcher**: Automatically detect and ingest new CSV/JSON files from monitored folders
- **🗃️ Multi-Format Support**: Work with CSV, JSON, Parquet, and PostgreSQL databases
- **🤖 Machine Learning Prep**: Basic preprocessing workflows including scaling, encoding, and train/test splits
- **💻 Embedded IDEs**: Execute SQL queries and Python scripts directly on datasets with syntax highlighting
- **📦 Automation Ready**: Export pipelines as PowerShell scripts for scheduling

📖 **[Full Feature Documentation →](docs/FEATURES.md)**

---

## Motivation

Modern data work involves repetitive, error-prone tasks: cleaning messy CSVs, standardizing column names, handling missing values, and running the same transformations every time new data arrives. Existing tools are either too heavyweight (enterprise ETL platforms), too code-centric (scripting everything manually), or too limited (spreadsheet functions).

**Beefcake bridges this gap** by providing:

1. **Interactive Exploration** - Understand your data before transforming it
2. **Reusable Pipelines** - Capture transformations as version-controlled specifications
3. **Local-First Architecture** - No cloud dependencies, your data stays on your machine
4. **Performance at Scale** - Rust backend handles millions of rows efficiently
5. **Developer-Friendly** - Export to PowerShell/Python for integration with existing workflows

Whether you're a **data engineer** building ETL workflows, an **analyst** preparing datasets for modeling, or a **scientist** standardizing research data, Beefcake accelerates the tedious parts of data work.

---

## What Beefcake Is NOT

To set clear expectations, Beefcake is explicitly **not**:

- ❌ **A polished commercial ETL product** - This is a personal learning project with rough edges
- ❌ **A replacement for enterprise data platforms** - Not intended for mission-critical production workloads
- ❌ **API-stable or production-ready** - Interfaces and data formats may change without notice
- ❌ **Fully tested or documented** - Test coverage is incomplete, documentation is evolving
- ❌ **Optimized for all datasets** - Performance characteristics vary, especially for edge cases
- ❌ **Supported or maintained on a schedule** - Development happens in bursts as time and interest permit

Beefcake is best understood as an **experimental toolkit** for exploring modern data engineering patterns in Rust, rather than a finished product. Use it for learning, experimentation, and non-critical data tasks.

📖 **[Known Limitations →](docs/LIMITATIONS.md)**

---

## Architecture

### Technology Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| **Backend** | Rust 1.75+ | High-performance data processing |
| **Data Engine** | Polars 0.45.0 | Lazy evaluation, multi-threading |
| **Database** | SQLx (Postgres) | Database connectivity |
| **Frontend** | TypeScript 5.0+ | Interactive UI components |
| **Desktop Framework** | Tauri 2.1.1 | Native app packaging |
| **Styling** | CSS3 | Responsive, modern UI |

### High-Level Design

```
┌─────────────────────────────────────────────────────────────┐
│                    Desktop Application (Tauri)              │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌────────────────────┐         ┌─────────────────────┐    │
│  │  Frontend (TS)     │   IPC   │   Backend (Rust)    │    │
│  │                    │ <────>  │                     │    │
│  │  • UI Components   │         │  • Data Processing  │    │
│  │  • State Mgmt      │         │  • Lifecycle Mgmt   │    │
│  │  • Event Handling  │         │  • Pipeline Engine  │    │
│  │  • Visualization   │         │  • Watcher Service  │    │
│  │  • Pipeline Editor │         │  • ML Algorithms    │    │
│  └────────────────────┘         └─────────────────────┘    │
│                                                              │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
        ┌───────────────────────────────────────┐
        │       External Systems                │
        ├───────────────────────────────────────┤
        │  • CSV/JSON/Parquet Files             │
        │  • PostgreSQL Databases               │
        │  • Python Runtime (optional)          │
        │  • PowerShell Scripts (export)        │
        │  • Monitored Folders (watcher)        │
        └───────────────────────────────────────┘
```

📖 **[Detailed Architecture Documentation →](docs/ARCHITECTURE.md)**

---

## Roadmap

> **Note**: "Implemented" phases represent initial working versions that may be revised, refactored, or redesigned as the project evolves. This is an iterative learning project, not a fixed roadmap.

### Completed Phases

- ✅ **Phase 1: Core Data Analysis** - File loading, profiling, type detection, quality assessment
- ✅ **Phase 2: Lifecycle Management** - Immutable version control, six logical stages, diff engine, audit logging
- ✅ **Phase 3: Database Integration** - PostgreSQL connectivity, SQL IDE, import/export
- ✅ **Phase 4: Embedded Python** - Python script execution, Polars integration, diagnostics
- ✅ **Phase 5: Pipeline Automation** - Visual builder with 11 step types, 8 templates, CLI mode, PowerShell export
- ✅ **Phase 6: Filesystem Watcher** - Auto-detection and ingestion of new data files with real-time monitoring

### Future Directions (Exploratory)

- 🔮 **Phase 7: Advanced Features** - Real-time collaboration, cloud connectors, ML models, advanced visualizations
- 🔮 **Phase 8: Enterprise Features** - RBAC, audit dashboards, Docker support, CI/CD integration

*Future phases represent potential exploration areas, not committed deliverables.*

📖 **[Full Roadmap with Timelines →](docs/ROADMAP.md)**

---

## Project Status

**Current Version**: `0.2.0` (January 2025)

### Recent Milestones

- ✅ **v0.2.0** (Jan 2025): Pipeline Builder with 11 step types, 8 templates, drag-and-drop editor, filesystem watcher, and pipeline executor
- ✅ **v0.1.5** (Dec 2024): Lifecycle management with immutable version control and diff engine
- ✅ **v0.1.0** (Nov 2024): Initial release with data profiling and SQL IDE

### Build Status

- **Rust Backend**: ✅ Compiles without errors
- **TypeScript Frontend**: ✅ Type-checks successfully
- **Production Build**: ✅ Vite builds in ~13s
- **Tests**: 🚧 Coverage at ~60% (expanding)

### Quick Limitations Summary

- Files >10GB may cause memory issues
- PowerShell export Windows-only (macOS/Linux coming)
- Requires manual Python 3.8+ installation
- PostgreSQL only (no MySQL/SQLite yet)
- ~60% test coverage, some edge cases untested
- APIs may change between versions

📖 **[Comprehensive Limitations List →](docs/LIMITATIONS.md)**

---

## Getting Started

### Prerequisites

- **Rust** 1.75+: [Install Rust](https://www.rust-lang.org/tools/install)
- **Node.js** 18+: [Install Node.js](https://nodejs.org/)
- **Python** 3.8+ (optional, for Python IDE): [Install Python](https://www.python.org/)

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/beefcake.git
cd beefcake

# Install dependencies
npm install

# Build the application
npm run tauri build

# Run in development mode
npm run tauri dev
```

### Quick Start

1. **Load a Dataset**: File → Open → Select CSV/JSON/Parquet
2. **Explore Data**: View statistics, distributions, and quality metrics
3. **Create Pipeline**: Pipeline → New → Add transformation steps
4. **Execute**: Run pipeline on current dataset
5. **Export**: Save pipeline as JSON or PowerShell script

---

## Documentation

### User Guides

- **[Learning Guide](docs/LEARNING_GUIDE.md)** - Start here for a tour of the codebase
- **[Features](docs/FEATURES.md)** - Detailed capability breakdown
- **[Limitations](docs/LIMITATIONS.md)** - Known constraints and unsuitable use cases
- **[Automation](docs/AUTOMATION.md)** - Pipeline automation workflows

### Technical Documentation

- **[Architecture](docs/ARCHITECTURE.md)** - System design and patterns
- **[Modules](docs/MODULES.md)** - Module reference and responsibilities
- **[Roadmap](docs/ROADMAP.md)** - Development phases and future directions
- **[Rust Concepts](docs/RUST_CONCEPTS.md)** - Rust patterns explained
- **[TypeScript Patterns](docs/TYPESCRIPT_PATTERNS.md)** - Frontend architecture

### API Documentation

Generate comprehensive API docs:

```bash
# Rust API documentation
cargo doc --open --document-private-items

# TypeScript API documentation
npm run docs:ts
```

---

## Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) (coming soon) for details on:

- Code style and conventions
- How to submit pull requests
- Running tests and linters
- Documentation requirements

### Development Setup

```bash
# Install development tools
cargo install cargo-clippy cargo-fmt

# Run tests
cargo test
npm test

# Format code
cargo fmt
npm run format

# Lint
cargo clippy
npm run lint
```

---

## Platform Support

Beefcake runs as a native desktop application on:

- ✅ **Windows 10/11**
- ✅ **macOS 11+** (Apple Silicon & Intel)
- ✅ **Linux** (Ubuntu 20.04+, Fedora 35+)

---

## License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

---

## Acknowledgments

- **[Polars](https://pola.rs/)** - Lightning-fast DataFrame library
- **[Tauri](https://tauri.app/)** - Rust-powered desktop framework
- **[smartcore](https://smartcorelib.org/)** - Machine learning algorithms
- **[Monaco Editor](https://microsoft.github.io/monaco-editor/)** - Code editor for SQL/Python IDEs

---

## Contact

- **Author**: Anthony Henderson
- **Email**: anthony.s.henderson@gmail.com
- **Project Issues**: [GitHub Issues](https://github.com/yourusername/beefcake/issues)

---

**Built with ❤️ using Rust and TypeScript**
