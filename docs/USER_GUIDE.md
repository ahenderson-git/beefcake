# Beefcake User Guide

> **Getting started with Beefcake data analysis and transformation**

*Version 0.3.1 | Last Updated: February 2026*

---

## Table of Contents

1. [Installation](#installation)
2. [First Steps](#first-steps)
3. [Common Workflows](#common-workflows)
4. [Features Guide](#features-guide)
5. [Tips & Best Practices](#tips--best-practices)

---

## Installation

### Prerequisites

- **Windows**: Windows 10 or later (64-bit)
- **macOS**: macOS 10.15 (Catalina) or later
- **Linux**: Ubuntu 20.04+ or equivalent

### Download

1. Go to the [Releases page](https://github.com/yourusername/beefcake/releases)
2. Download the appropriate installer for your platform:
   - Windows: `beefcake-setup.exe`
   - macOS: `beefcake.dmg`
   - Linux: `beefcake.AppImage`
3. Run the installer and follow the prompts

### First Launch

On first launch, Beefcake will:
- Create a data directory at `~/.local/share/beefcake/` (Linux/macOS) or `%APPDATA%\beefcake\` (Windows)
- Initialize configuration files
- Display the Dashboard

---

## First Steps

### Opening Your First File

1. **Launch Beefcake** - You'll see the Dashboard with a welcome screen
2. **Click "Open File"** - Opens a file picker dialog
3. **Select a data file**:
   - Supported formats: CSV, JSON, Parquet
   - Example: Try `testdata/sample.csv` if available
4. **Wait for analysis** - Beefcake will automatically profile your data (5-30 seconds for most files)
5. **View results** - The Analyser view shows column statistics, data quality, and insights

### Understanding the Analysis

After loading a file, you'll see:

- **Summary Cards**: Row count, column count, file size, health score
- **Column List**: Each column with its type, null count, and statistics
- **Health Score**: Overall data quality (0-100)
  - 🟢 **90-100**: Excellent quality
  - 🟡 **70-89**: Good quality, minor issues
  - 🔴 **<70**: Poor quality, needs attention

### Expanding Column Details

Click any column row to see:
- **Statistics**: Mean, median, min, max, percentiles
- **Distribution**: Histogram visualization
- **Quality Indicators**: Outliers, skewness, missing values
- **Recommendations**: Suggested cleaning strategies

---

## Common Workflows

### 1. Quick Data Quality Check

**Goal**: Quickly assess if a dataset is ready for use

**Steps**:
1. Open file → Analyser automatically runs
2. Check **Health Score** in summary banner
3. Scan **column list** for red flags:
   - High null percentages (>10%)
   - "❗" warning icons
4. Expand suspicious columns to view details
5. Read **Business Summary** for quick insights

**Time**: 30 seconds - 2 minutes

---

### 2. Clean and Export Dataset

**Goal**: Remove issues and export a clean version

**Steps**:
1. **Open file** → Analysis completes
2. **Switch to Lifecycle view** (left sidebar)
3. **Navigate to "Clean" stage**:
   - Click "Clean" in the stage rail
4. **Apply cleaning**:
   - Check boxes for operations:
     - ✅ Trim whitespace
     - ✅ Drop columns with >50% nulls
     - ✅ Impute missing values (mean/median)
   - Click "Apply Transform"
5. **Export cleaned data**:
   - Click "Export" button in top bar
   - Choose format (CSV, Parquet, JSON)
   - Select save location
6. **Done!** Your cleaned dataset is ready

**Time**: 2-5 minutes

---

### 3. Build a Reusable Pipeline

**Goal**: Create a transformation workflow you can run repeatedly

**Steps**:
1. **Open Pipeline Builder** (left sidebar → Pipeline)
2. **Add steps from palette** (left panel):
   - Drag or click to add:
     - Trim Whitespace
     - Drop Columns (`temp_col`, `debug_info`)
     - Impute (strategy: mean, columns: `age`, `income`)
     - Cast Types (`age` → i64, `price` → f64)
3. **Configure each step**:
   - Click step in canvas
   - Set parameters in right panel
   - Validate (green checkmark = valid)
4. **Save pipeline**:
   - Click "Save" button
   - Name it: `monthly_data_clean`
   - Choose save location
5. **Execute pipeline**:
   - Click "Execute"
   - Select input file
   - Choose output path
   - Click "Run"
6. **Re-use anytime**:
   - Pipeline Library → Load saved pipeline
   - Execute on new files instantly

**Time**: 5-10 minutes (first time), 30 seconds (reuse)

---

### 4. Monitor a Folder for New Files

**Goal**: Automatically ingest files as they arrive

**Steps**:
1. **Open Watcher view** (left sidebar → Watcher)
2. **Enable watcher**:
   - Toggle "Enable Watcher" switch
3. **Choose folder**:
   - Click "Select Folder"
   - Pick folder to monitor (e.g., `C:\data\incoming`)
4. **Configure (optional)**:
   - Auto-ingest: On (files ingested immediately)
   - File types: CSV, JSON, Parquet
5. **Watch activity feed**:
   - New files appear in real-time
   - Status: Detected → Stable → Ingesting → Success
6. **Access ingested datasets**:
   - Click dataset link in activity feed
   - Opens in Lifecycle view

**Time**: 1 minute setup, runs continuously

---

### 5. Run SQL Queries on Data

**Goal**: Explore data using SQL

**Steps**:
1. **Load file** → Analysis completes
2. **Switch to SQL IDE** (left sidebar → SQL)
3. **Write query**:
   <!-- noinspection SqlNoDataSourceInspection -->
   ```sql
   SELECT
     category,
     AVG(price) as avg_price,
     COUNT(*) as count
   FROM data
   WHERE price > 100
   GROUP BY category
   ORDER BY avg_price DESC
   LIMIT 10;
   ```
4. **Execute**:
   - Click "Run" button (or Ctrl+Enter)
   - Results appear in bottom panel
5. **Export results** (optional):
   - Click "Export Results"
   - Choose CSV, JSON, or Parquet

**Time**: 1-3 minutes per query

---

## Features Guide

### Dashboard
- **Quick Start**: Open files, create pipelines, access recent datasets
- **Recent Activity**: Last 10 files analyzed
- **System Status**: Memory usage, dataset count

### Analyser
- **Column Profiling**: Automatic type detection, statistics, quality checks
- **Health Scoring**: 0-100 quality metric based on nulls, outliers, and completeness
- **Visualization**: Histograms for numeric/categorical columns
- **Recommendations**: AI-powered suggestions for cleaning

### Lifecycle Management
- **6 Stages**: Raw → Profiled → Cleaned → Advanced → Validated → Published
- **Version Control**: Immutable versions, never lose original data
- **Diff Engine**: Compare any two versions (schema, row counts, statistics)
- **Rollback**: Promote any previous version to active

### Pipeline Builder
- **Visual Editor**: Drag-and-drop interface for transformations
- **11 Step Types**:
  - Column Management: Drop, Rename
  - Text Processing: Trim, Regex Replace
  - Type Conversion: Cast, Parse Dates
  - Missing Values: Impute
  - ML Preprocessing: One-Hot Encode, Normalize
  - Outlier Handling: Clip Outliers, Extract Numbers
- **8 Templates**: Pre-built pipelines for common tasks
- **Save & Share**: Export as JSON for version control

### Watcher
- **Auto-Ingestion**: Detect new files and ingest automatically
- **Stability Detection**: Wait for file writes to complete
- **Activity Feed**: Real-time status of detected files
- **Manual Trigger**: Force ingestion of specific files

### AI Assistant
- **Context-Aware**: Sees your loaded dataset's metadata
- **Q&A**: Ask about statistics, quality issues, cleaning strategies
- **Documentation Links**: Direct links to relevant docs
- **Limitations**: Advisory only—cannot modify data or execute actions

---

## Tips & Best Practices

### Performance

- **Large Files (>1GB)**: Use Parquet format for faster loading
- **Sampling**: Analysis automatically samples large datasets (>5M rows)
- **Lazy Evaluation**: Polars engine only loads what's needed

### Data Quality

- **Always Check Health Score First**: Saves time by identifying issues upfront
- **Expand Suspicious Columns**: Don't just rely on summary—view distributions
- **Test Pipelines on Small Files**: Validate logic before running on production data

### Organization

- **Name Datasets Descriptively**: Use dates and versions (`sales_2025-01-15_v2`)
- **Save Pipelines Often**: Easier to iterate and revert
- **Use Lifecycle Stages**: Track progress from raw to validated

### Safety

- **Original Data Never Modified**: All transformations create new versions
- **Export Before Publishing**: Keep local copies of important datasets
- **Test SQL Queries**: Start with `LIMIT 10` to verify logic

### Automation

- **PowerShell Export**: Convert pipelines to scripts for scheduling
- **Watcher for Batch Jobs**: Monitor ETL output folders
- **CLI Mode**: Run pipelines headlessly in automated workflows

---

## Next Steps

- **Learn More**: See [FEATURES.md](FEATURES.md) for detailed feature documentation
- **Get Help**: Check [TROUBLESHOOTING.md](TROUBLESHOOTING.md) for common issues
- **Advanced Topics**: Explore [PIPELINE_IMPLEMENTATION_GUIDE.md](PIPELINE_IMPLEMENTATION_GUIDE.md)

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+O` | Open File |
| `Ctrl+S` | Save Current Work |
| `Ctrl+E` | Export Data |
| `Ctrl+Enter` | Execute Query/Pipeline |
| `Ctrl+/` | Toggle AI Assistant |
| `F5` | Refresh Current View |

---

## Support

- **Documentation**: `docs/` folder in installation directory
- **Issues**: [GitHub Issues](https://github.com/yourusername/beefcake/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/beefcake/discussions)

---

*For technical details, see [ARCHITECTURE.md](ARCHITECTURE.md)*
