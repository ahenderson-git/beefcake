# Beefcake Limitations

> **Known constraints, edge cases, and unsuitable use cases**

*Last Updated: January 2025*

---

## Overview

Beefcake is an experimental learning project, not a production-ready data platform. This document provides an honest assessment of its current limitations, constraints, and scenarios where it should **not** be used.

For a summary of what Beefcake **can** do, see [FEATURES.md](FEATURES.md).

---

## Performance & Scale

### Large File Handling

**Files >10GB:**
- May cause out-of-memory errors
- Streaming mode incomplete (partially implemented)
- Recommendation: Split files or use external tools for initial processing

**Workarounds:**
- Enable sampling mode in settings (default: 10M rows)
- Use external tools (DuckDB, Polars CLI) for initial filtering
- Consider upgrading RAM if working with large datasets frequently

### Complex Pipelines

**Deep Transformation Chains:**
- Performance degrades with >20 steps in a pipeline
- Each step materialises intermediate results (no fusion optimisation)
- Recommendation: Break complex pipelines into smaller sub-pipelines

**Nested Operations:**
- Avoid deeply nested transformations (e.g., multiple group-bys)
- Polars optimizer has limits

### Concurrent Operations

**Single-Threaded GUI:**
- Tauri frontend is single-threaded
- Long-running operations block UI
- Background processing limited

**Workarounds:**
- Use CLI mode for batch processing
- Run multiple CLI instances in parallel
- Future: Add async task queue

---

## Platform & Integration

### Operating System Support

**Windows:**
- ✅ Fully supported (primary development platform)
- PowerShell script export works

**macOS:**
- ✅ Supported (tested on Apple Silicon)
- Bash script export not implemented (uses PowerShell)
- Some UI rendering differences

**Linux:**
- ✅ Supported (Ubuntu 20.04+, Fedora 35+)
- Bash script export not implemented
- May require additional system dependencies

### Python Dependency

**Manual Installation Required:**
- User must install Python 3.8+ separately
- Must install Polars package via pip
- No bundled Python runtime

**Common Issues:**
- Python not in PATH
- Wrong Python version (2.x vs 3.x)
- Polars version mismatch

**Workarounds:**
- Use Python IDE diagnostics to check setup
- Follow installation guide carefully
- Future: Consider bundling Python with app

### Database Support

**PostgreSQL Only:**
- MySQL not supported
- SQLite not supported
- SQL Server not supported
- Oracle not supported

**Limitations Within PostgreSQL:**
- No stored procedure execution
- No transaction management
- No support for PostGIS or extensions

**Why:**
- SQLx Postgres driver is mature
- Adding other databases requires significant testing
- Future: May add MySQL and SQLite

---

## Stability & Maturity

### Error Handling

**Panic Instead of Graceful Errors:**
- Some edge cases cause application crashes
- Error messages not always user-friendly
- Stack traces exposed to users

**Common Panic Scenarios:**
- Invalid regex patterns
- Type coercion failures
- Out-of-memory during large operations
- Missing files during lifecycle operations

**Mitigation:**
- Backup data before experiments
- Test transformations on small samples first
- Report crashes via GitHub Issues

### Test Coverage

**~60% Code Coverage:**
- Many edge cases untested
- Integration tests incomplete
- UI components lack automated tests

**High-Risk Areas:**
- Lifecycle stage transitions
- Pipeline step execution
- Database import/export
- Python script execution

**Best Practices:**
- Validate pipelines on test data first
- Keep backups of important datasets
- Expect occasional bugs

### Documentation Gaps

**Missing Documentation:**
- Advanced pipeline features
- Python API reference
- SQL dialect differences
- Troubleshooting guides

**Workarounds:**
- Read inline code comments
- Check example pipelines in `examples/`
- Ask questions via GitHub Issues

### Breaking Changes

**No API Stability Guarantee:**
- JSON schema may change between versions
- Tauri command signatures may change
- File formats may evolve

**Migration:**
- No automated migration tools
- May require manual pipeline updates
- Backward compatibility not guaranteed

**Recommendation:**
- Version control your pipelines (Git)
- Test after upgrading Beefcake
- Pin to specific version in production scripts

---

## Feature-Specific Limitations

### Data Analysis

**Sampling Bias:**
- Large files use sampling (not full scan)
- Statistics may not be representative
- First N rows used (not random sampling in some cases)

**Type Inference:**
- Heuristic-based (not always accurate)
- Struggles with mixed-type columns
- Date parsing limited to common formats

**Missing Value Detection:**
- Only detects nulls and empty strings
- Doesn't detect sentinel values (e.g., -999, "N/A")
- No configurable missing value patterns

### Lifecycle Management

**Storage Growth:**
- Each version stored as separate Parquet file
- No automatic cleanup (manual deletion required)
- Can consume significant disk space

**Version Comparison:**
- Slow for large datasets (>1M rows)
- Diff only shows changed rows, not detailed changes
- No merge conflict resolution

**Limitations:**
- No branching (linear version history only)
- No rollback beyond previous version
- No distributed coordination (single-user only)

### Pipeline Builder

**Step Limitations:**
- Only 11 transformation types
- No conditional logic (if/else)
- No loops or recursion
- No custom step plugins (yet)

**Validation:**
- Real-time validation incomplete
- Some errors only caught at execution
- No dry-run mode

**Template Library:**
- Only 8 templates
- No user-contributed templates
- No template versioning

### SQL IDE

**Polars SQL Dialect:**
- Subset of ANSI SQL
- No CTEs (Common Table Expressions)
- No window functions
- Limited JOIN support (no FULL OUTER JOIN)

**Execution:**
- No result pagination (loads all rows)
- No query cancellation
- No query history

### Python IDE

**Execution Model:**
- Single-threaded (no parallelism)
- No interactive input (stdin)
- No GPU support (CUDA/ROCm)

**Library Support:**
- Only libraries installed in user's Python environment
- No automatic package installation
- No virtual environment management

### Machine Learning

**Model Quality:**
- Basic implementations (not production-ready)
- No hyperparameter tuning
- No cross-validation
- Results may differ from scikit-learn

**Feature Engineering:**
- Limited encoding options (one-hot only)
- No feature selection
- No automated feature engineering

**Evaluation:**
- No stratified splitting
- No k-fold cross-validation
- No learning curves

---

## Not Recommended For

### ❌ Production Data Pipelines

**Why:**
- No SLA guarantees
- Limited error handling
- Breaking changes between versions
- Single-user design

**Better Alternatives:**
- Apache Airflow
- Prefect
- dbt
- AWS Glue

### ❌ Mission-Critical Workflows

**Why:**
- Insufficient testing
- Possible data loss bugs
- No high-availability mode
- No monitoring/alerting

**Better Alternatives:**
- Enterprise ETL platforms
- Cloud data pipelines (AWS, GCP, Azure)

### ❌ Multi-User Environments

**Why:**
- No concurrent editing
- No access control
- No conflict resolution
- Single-writer model

**Better Alternatives:**
- Shared database systems
- Cloud-based data platforms
- Snowflake, Databricks, etc.

### ❌ Compliance-Critical Workflows

**Why:**
- Audit trail incomplete
- No data encryption at rest
- No GDPR/HIPAA compliance
- No role-based access control

**Better Alternatives:**
- Compliance-certified platforms
- Enterprise data governance tools

### ❌ Real-Time Data Processing

**Why:**
- Batch-oriented design
- No streaming support
- High latency (seconds to minutes)

**Better Alternatives:**
- Apache Kafka + Kafka Streams
- Apache Flink
- AWS Kinesis

### ❌ Distributed Data Processing

**Why:**
- Single-machine design
- No cluster support
- No distributed file system integration

**Better Alternatives:**
- Apache Spark
- Dask
- Ray

---

## When Beefcake IS Appropriate

### ✅ Learning & Experimentation

**Good For:**
- Exploring data engineering concepts
- Testing transformation ideas
- Prototyping pipelines
- Learning Rust and Polars

### ✅ Personal Projects

**Good For:**
- Ad-hoc data analysis
- Data cleaning for personal use
- Small-scale ETL workflows
- Local data exploration

### ✅ Non-Critical Automation

**Good For:**
- Scheduled data exports
- Report generation
- Data format conversions
- Simple transformations

### ✅ Development & Testing

**Good For:**
- Generating test datasets
- Validating data quality
- Pipeline prototyping
- Schema exploration

---

## Mitigation Strategies

### For Known Limitations

**Large Files:**
1. Use external pre-filtering (DuckDB, Polars CLI)
2. Enable sampling mode
3. Split files into manageable chunks

**Stability Issues:**
1. Test on small datasets first
2. Keep backups of important data
3. Version control pipelines
4. Report bugs promptly

**Platform Differences:**
1. Test on target platform before deployment
2. Use CLI mode for cross-platform scripts
3. Avoid platform-specific features

**Missing Features:**
1. Use Python IDE for advanced transformations
2. Combine with external tools (pandas, DuckDB)
3. Write custom scripts for unique needs

---

## Future Improvements

See [ROADMAP.md](ROADMAP.md) for potential enhancements that may address some of these limitations.

**High Priority:**
- Improve error handling
- Expand test coverage
- Add streaming support for large files
- Implement proper async task execution

**Medium Priority:**
- Add MySQL and SQLite support
- Improve SQL dialect coverage
- Bundle Python runtime
- Add more pipeline step types

**Low Priority:**
- Multi-user support
- Real-time collaboration
- Distributed processing
- Enterprise features

---

## Reporting Issues

If you encounter a limitation not listed here:

1. **Check GitHub Issues**: Someone may have already reported it
2. **Open a New Issue**: Describe the limitation and use case
3. **Include Details**: Version, OS, dataset size, error messages
4. **Be Patient**: This is a personal project with no guaranteed response time

---

## Acknowledgment

Beefcake's limitations are by design - it's a learning project exploring modern data engineering patterns, not a commercial product. If you need production-ready tools, consider the "Better Alternatives" listed above.

That said, limitations are also opportunities for improvement. Contributions and feedback are welcome!

---

## Summary

**Beefcake is best for:**
- Learning and experimentation
- Personal projects and non-critical automation
- Prototyping and development

**Beefcake is NOT for:**
- Production pipelines with strict SLAs
- Mission-critical workflows
- Multi-user environments
- Compliance-critical data
- Real-time or distributed processing

**When in doubt:** Test on non-critical data first, and have backup plans.
