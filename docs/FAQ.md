# Beefcake Frequently Asked Questions (FAQ)

> **Quick answers to common questions**

*Version 0.2.0 | Last Updated: January 2025*

---

## General Questions

### What is Beefcake?

Beefcake is a desktop data analysis and transformation toolkit built with Rust and TypeScript. It helps you profile datasets, clean messy data, and automate repetitive transformation workflows—all without writing code.

Think of it as a middle ground between Excel (too manual) and Python scripts (too code-heavy).

---

### Is Beefcake production-ready?

**No.** Beefcake is an **experimental, learning project**. It's subject to frequent changes, has incomplete test coverage, and may have bugs. Use it for:
- ✅ Learning and experimentation
- ✅ Non-critical data exploration
- ✅ Prototyping data workflows

Do **not** use it for:
- ❌ Mission-critical production systems
- ❌ Regulated data (HIPAA, GDPR, etc.)
- ❌ Data you can't afford to lose

Always keep backups of original data.

---

### What platforms does Beefcake support?

- **Windows**: Windows 10/11 (64-bit)
- **macOS**: macOS 10.15 (Catalina) or later
- **Linux**: Ubuntu 20.04+, Fedora 35+, or equivalent (GLIBC 2.27+)

Both Intel and Apple Silicon (M1/M2) Macs are supported.

---

### Is Beefcake free?

Yes! Beefcake is open-source under the MIT license. You can:
- Use it for free (personal or commercial)
- Modify the source code
- Distribute modified versions

However, there's **no warranty** and **no official support**. See [LICENSE](../LICENSE) for details.

---

## Features & Capabilities

### What file formats are supported?

**Input formats**:
- CSV (`.csv`) - Comma-separated values
- JSON (`.json`) - JSON arrays or objects
- Parquet (`.parquet`) - Apache Parquet columnar format

**Output formats**:
- CSV
- JSON
- Parquet

**Not supported** (as of v0.2.0):
- Excel (`.xlsx`, `.xls`)
- SQL databases (read-only via SQL IDE)
- Google Sheets
- XML, YAML, TOML

To use Excel files, convert them to CSV first (Excel → "Save As" → CSV).

---

### Can Beefcake handle large datasets?

Yes, but with limitations:

- **Small files (<100K rows)**: Instant analysis, full profiling
- **Medium files (100K-1M rows)**: 10-30 seconds, full profiling
- **Large files (1M-5M rows)**: 1-5 minutes, sampling used
- **Very large files (>5M rows)**: Use Parquet format, expect 5-10 minute load times

**Memory**: Beefcake can handle files up to ~50% of your available RAM. For a machine with 8GB RAM, max practical file size is ~4GB.

**Tip**: Use Parquet format for large files—it's faster and more memory-efficient than CSV.

---

### Does Beefcake modify my original data?

**No, never.** Beefcake uses an immutable versioning system:
- Original file is copied to internal storage (Raw stage)
- All transformations create new versions
- Original data is never modified

You can always:
- Return to the original version
- Delete versions you don't need
- Export any version to a new file

---

### Can I undo transformations?

Yes! The Lifecycle system provides full version control:
- Each transformation creates a new version
- Previous versions remain intact
- You can:
  - View any previous version
  - Compare versions with diff engine
  - "Promote" a previous version to active

Think of it like Git for data.

---

### Does Beefcake send my data to the cloud?

**No.** Beefcake is **local-first**:
- All data stays on your machine
- No telemetry or analytics sent to servers
- No cloud sync or backups

**Exception**: The AI Assistant feature sends **summary statistics** (column names, types, null counts) to OpenAI's API if enabled. It does **not** send your actual data rows.

To be extra safe:
- Disable AI Assistant when working with sensitive data
- Use airplane mode
- Inspect source code (it's open-source!)

---

## Data Processing

### What transformations can Beefcake perform?

**Basic cleaning**:
- Trim whitespace
- Drop columns
- Rename columns
- Remove duplicates (via SQL IDE)

**Type handling**:
- Cast types (int, float, string, date)
- Parse dates with custom formats

**Missing values**:
- Impute with mean, median, mode, or zero
- Drop rows/columns with nulls (via SQL IDE)

**ML preprocessing**:
- One-hot encoding (categorical → binary)
- Normalization (z-score, min-max)
- Clip outliers (quantile-based)

**Text processing**:
- Regex find/replace
- Extract numbers from text

**Advanced** (via SQL or Python IDE):
- Joins, aggregations, window functions
- Custom formulas and calculations

---

### Can I write custom Python code?

Yes! Beefcake has an embedded Python IDE:
- Execute Python scripts directly on loaded data
- Uses Polars DataFrame API (faster than Pandas)
- Scripts run in isolated subprocess (safe)

**Requirements**:
- Python 3.8+ must be installed on your system
- Polars library must be installed: `pip install polars`

**Limitations**:
- No interactive input (stdin)
- 60-second timeout by default
- Stdout/stderr captured in IDE

---

### How do I share my transformation workflow?

**Option 1: Pipeline JSON**
- Build pipeline in Pipeline Editor
- Save as JSON file (e.g., `my_cleaning.json`)
- Share file via email, Git, Slack, etc.
- Recipients can load and execute the pipeline

**Option 2: PowerShell Script**
- Export pipeline as `.ps1` script
- Script includes embedded Beefcake CLI commands
- Can be run on any machine with Beefcake installed
- Schedulable with Task Scheduler

**Option 3: Python Script**
- Write Python code in Python IDE
- Save/copy script to `.py` file
- Share via Git, Jupyter notebook, etc.

---

## AI Assistant

### What can the AI Assistant do?

The AI Assistant can:
- ✅ Answer questions about your data
- ✅ Explain statistics (mean, skewness, etc.)
- ✅ Identify quality issues
- ✅ Suggest cleaning strategies
- ✅ Provide links to documentation

The AI Assistant **cannot**:
- ❌ Modify your data
- ❌ Execute pipelines
- ❌ Write files
- ❌ Run SQL/Python code

Think of it as a **read-only advisor**, not an automation tool.

---

### Do I need an OpenAI API key?

Yes, if you want to use the AI Assistant. Here's how:
1. Go to https://platform.openai.com/api-keys
2. Create a new API key
3. Open Beefcake → Settings → AI Assistant
4. Paste key into "API Key" field
5. Enable "Enable AI Assistant" toggle
6. Save

**Cost**: OpenAI charges per request (~$0.01-0.05 per query with GPT-4). Set spending limits in OpenAI dashboard.

**Alternative**: Disable AI Assistant if you don't want to use it (it's optional).

---

### Will the AI Assistant make mistakes?

**Yes.** AI models can:
- Provide incorrect information
- Misinterpret your data
- Give overly generic advice
- Hallucinate facts

**Always verify** AI suggestions before applying them. The AI sees only summary statistics, not your actual data, so it can't know the full context.

---

## Automation & Integration

### Can I schedule Beefcake pipelines?

Yes, using PowerShell scripts:

1. **Export pipeline** as PowerShell script
2. **Test script** manually to ensure it works
3. **Schedule with Windows Task Scheduler**:
   - Open Task Scheduler
   - Create Basic Task
   - Trigger: Daily, Weekly, etc.
   - Action: "Start a program"
   - Program: `powershell.exe`
   - Arguments: `-File "C:\path\to\script.ps1"`

Linux/macOS: Use `cron` to schedule scripts.

---

### Can I call Beefcake from my Python scripts?

Not directly, but you can use the CLI mode:

```python
import subprocess

result = subprocess.run([
    'beefcake', 'run',
    '--spec', 'pipeline.json',
    '--input', 'data.csv',
    '--output', 'cleaned.csv'
], capture_output=True, text=True)

print(result.stdout)
```

**Note**: CLI mode is experimental and may change between versions.

---

### Can I use Beefcake as a library?

**Rust**: Yes, Beefcake is published as a Rust crate (eventually). Add to `Cargo.toml`:
```toml
beefcake = "0.2.0"
```

**Python/JavaScript/Other**: No direct bindings yet. Use CLI mode as a workaround.

---

## Troubleshooting

### Why is my file not loading?

Common reasons:
1. **Unsupported format**: Only CSV, JSON, Parquet work
2. **Malformed data**: Check for invalid characters, missing quotes
3. **File locked**: Close Excel/other programs using the file
4. **Permission denied**: Ensure file is readable
5. **Out of memory**: File too large for available RAM

See [TROUBLESHOOTING.md](TROUBLESHOOTING.md) for detailed solutions.

---

### Why is analysis taking so long?

**Expected times**:
- Small files (<10K rows): 1-5 seconds
- Medium files (10K-1M rows): 5-30 seconds
- Large files (1M-5M rows): 30-120 seconds

**If slower**:
- Close other CPU-heavy apps
- Use Parquet format instead of CSV
- Work with a sample of data first

---

### Why did the app crash?

Possible reasons:
1. **Out of memory**: File too large
2. **Corrupted data**: Malformed CSV/JSON
3. **Bug**: Unexpected edge case

**What to do**:
1. Check logs in `%LOCALAPPDATA%\beefcake\logs` (Windows) or `~/.local/share/beefcake/logs` (Linux/macOS)
2. Report bug at https://github.com/yourusername/beefcake/issues
3. Include log excerpt and steps to reproduce

---

## Security & Privacy

### Is my data secure?

Beefcake does **not**:
- Send data to external servers (except AI Assistant metadata)
- Store passwords in plain text (uses OS keychain)
- Transmit telemetry or analytics

Beefcake **does**:
- Store data locally in `%LOCALAPPDATA%\beefcake` (Windows) or `~/.local/share/beefcake` (Linux/macOS)
- Log errors to local log files

**Best practices**:
- Don't use Beefcake on shared computers
- Keep backups of important data
- Encrypt sensitive files before loading

---

### Can I use Beefcake with regulated data?

**Not recommended.** Beefcake:
- Is not certified for HIPAA, GDPR, SOC 2, etc.
- Has no audit logging
- Has no role-based access control
- Is experimental software

If you must use it:
- Anonymize data first
- Work with test datasets only
- Consult your compliance team

---

### Can I review the source code?

**Yes!** Beefcake is open-source:
- GitHub: https://github.com/yourusername/beefcake
- License: MIT (see [LICENSE](../LICENSE))
- Feel free to audit, fork, or contribute

---

## Contributing

### Can I contribute to Beefcake?

**Yes!** Contributions are welcome:
- Bug reports: https://github.com/yourusername/beefcake/issues
- Feature requests: https://github.com/yourusername/beefcake/discussions
- Pull requests: Follow [CONTRIBUTING.md](CONTRIBUTING.md) (if it exists)

**Areas needing help**:
- Documentation improvements
- Test coverage
- Bug fixes
- Performance optimizations

---

### How do I report a bug?

1. Go to https://github.com/yourusername/beefcake/issues
2. Click "New Issue"
3. Use bug report template
4. Include:
   - Beefcake version (Help → About)
   - Operating system
   - Steps to reproduce
   - Expected vs actual behavior
   - Log file excerpt (if applicable)

---

### How do I request a feature?

1. Go to https://github.com/yourusername/beefcake/discussions
2. Check if feature already requested
3. Create new discussion with:
   - Clear description of feature
   - Use case (why you need it)
   - Examples or mockups (if applicable)

**Note**: Feature requests may not be implemented. Beefcake is a learning project, not a product with a roadmap.

---

## Learning Resources

### How do I learn to use Beefcake?

1. **Start here**:
   - [USER_GUIDE.md](USER_GUIDE.md) - Getting started guide
   - [FEATURES.md](FEATURES.md) - Full feature documentation

2. **Deep dives**:
   - [PIPELINE_IMPLEMENTATION_GUIDE.md](PIPELINE_IMPLEMENTATION_GUIDE.md) - Pipeline system internals
   - [ARCHITECTURE.md](ARCHITECTURE.md) - Technical architecture

3. **Troubleshooting**:
   - [TROUBLESHOOTING.md](TROUBLESHOOTING.md) - Common issues
   - [LIMITATIONS.md](LIMITATIONS.md) - Known limitations

4. **Examples**:
   - Load `testdata/` files in the installation directory
   - Follow workflows in USER_GUIDE.md

---

### Where can I ask questions?

- **GitHub Discussions**: https://github.com/yourusername/beefcake/discussions
- **GitHub Issues**: https://github.com/yourusername/beefcake/issues (for bugs)
- **Documentation**: `docs/` folder

**Note**: There's no official support channel. Help is community-driven and best-effort.

---

## Still have questions?

- **Search documentation**: `docs/` folder in your installation
- **Search GitHub Issues**: Someone may have asked already
- **Ask on GitHub Discussions**: Community may help
- **Read the source code**: It's open-source!

---

*Remember: Beefcake is experimental software developed as a learning project. Expect rough edges, bugs, and evolving features.*
