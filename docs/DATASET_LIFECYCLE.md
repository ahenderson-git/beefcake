# Dataset Lifecycle System

## Overview

The Beefcake Dataset Lifecycle system implements an immutable, versioned approach to data transformation with the following key principles:

✅ **Never replace raw DataFrame** - Original data remains untouched
✅ **Serializable transforms** - All operations are parameterized and can be saved as JSON
✅ **Version control** - Git-like system for datasets
✅ **Diff summaries** - Track changes between versions
✅ **Active version pointer** - Select which version to use for queries
✅ **View vs Snapshot** - Choose between lazy (computed) and materialized (stored) data

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     DatasetRegistry                          │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ Dataset                                               │  │
│  │  raw_version ───────► VersionTree                    │  │
│  │  active_version ──┐     │                            │  │
│  │                   │     ├── Raw Version (immutable)  │  │
│  │                   │     ├── Profiled Version         │  │
│  │                   └────►├── Cleaned Version          │  │
│  │                         ├── Advanced Version         │  │
│  │                         ├── Validated Version        │  │
│  │                         └── Published Version        │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Lifecycle Stages

### 1. Raw
- **Purpose**: Immutable original data ingestion
- **Transform**: None (just copy to version storage)
- **Output**: Unchanged data in parquet format

### 2. Profiled
- **Purpose**: Statistical analysis, issue detection, recommendations
- **Transform**: None (metadata only)
- **Output**: ColumnSummary with stats, health metrics, ML advice

### 3. Cleaned
- **Purpose**: Deterministic text and type transformations
- **Transforms**:
  - Trim whitespace
  - Case normalisation
  - Special character removal
  - Type casting
  - Column renaming
  - Null standardisation
- **Restricted Mode**: `true` (deterministic only)

### 4. Advanced
- **Purpose**: ML preprocessing
- **Transforms**:
  - Imputation (mean, median, mode)
  - Normalization (z-score, min-max)
  - Outlier clipping
  - One-hot encoding
  - Feature engineering
- **Restricted Mode**: `false` (allows statistical operations)

### 5. Validated
- **Purpose**: QA gates and schema validation
- **Rules**:
  - Null percentage checks
  - Value range validation
  - Column existence checks
  - Row count boundaries
  - Duplicate detection
  - Custom conditions
- **Output**: ValidationResults (pass/fail)

### 6. Published
- **Purpose**: Finalized dataset for consumption
- **Modes**:
  - **View**: Logical view (pipeline computed on access)
  - **Snapshot**: Physical copy (materialized data)

## Core Components

### DatasetRegistry
Central registry tracking all datasets and versions.

```rust
let registry = DatasetRegistry::new(base_path)?;
let dataset_id = registry.create_dataset("my_data".to_string(), raw_path)?;
```

### Dataset
Container for a single dataset with all its versions.

```rust
pub struct Dataset {
    id: Uuid,
    name: String,
    raw_version_id: Uuid,      // Immutable
    active_version_id: Uuid,    // Current working version
    versions: VersionTree,
    created_at: DateTime<Utc>,
}
```

### DatasetVersion
A specific version in the lifecycle.

```rust
pub struct DatasetVersion {
    id: Uuid,
    dataset_id: Uuid,
    parent_id: Option<Uuid>,    // Lineage
    stage: LifecycleStage,
    pipeline: TransformPipeline, // Serializable transforms
    data_location: DataLocation, // Parquet path
    metadata: VersionMetadata,
    created_at: DateTime<Utc>,
}
```

### TransformPipeline
Ordered sequence of serializable transformations.

```rust
pub struct TransformPipeline {
    transforms: Vec<TransformSpec>,
}

pub struct TransformSpec {
    transform_type: String,
    parameters: HashMap<String, serde_json::Value>,
}
```

### Transform Trait
All transforms implement this trait for consistency.

```rust
pub trait Transform {
    fn apply(&self, lf: LazyFrame) -> Result<LazyFrame>;
    fn name(&self) -> &str;
    fn parameters(&self) -> HashMap<String, serde_json::Value>;
    fn description(&self) -> String;
    fn to_json(&self) -> Result<String>;
}
```

## Usage Examples

### Creating a Dataset

```rust
use beefcake::analyser::lifecycle::DatasetRegistry;

let registry = DatasetRegistry::new(PathBuf::from("./data"))?;
let dataset_id = registry.create_dataset(
    "sales_data".to_string(),
    PathBuf::from("./raw/sales.csv")
)?;
```

### Applying Transforms

```rust
use beefcake::analyser::lifecycle::stages::{CleanStageExecutor, StageExecutor};

// Build cleaning configs
let mut configs = HashMap::new();
let mut config = ColumnCleanConfig::default();
config.active = true;
config.trim_whitespace = true;
config.standardise_nulls = true;
configs.insert("customer_name".to_string(), config);

// Create executor and generate pipeline
let executor = CleanStageExecutor::new(configs);
let pipeline = executor.execute(LazyFrame::default())?;

// Apply to dataset
let version_id = registry.apply_transforms(
    &dataset_id,
    pipeline,
    LifecycleStage::Cleaned
)?;
```

### Querying Versions

```rust
use beefcake::analyser::lifecycle::query::VersionQuery;

// Get specific version
let query = VersionQuery::new().version_id(version_id);
let version = query.execute(&dataset)?;

// Get active version
let query = VersionQuery::new().active();
let lf = query.execute_and_load(&dataset)?;

// Get latest cleaned version
let query = VersionQuery::new()
    .stage(LifecycleStage::Cleaned)
    .latest();
let lf = query.execute_and_load(&dataset)?;
```

### Computing Diffs

```rust
let diff = registry.compute_diff(&dataset_id, &v1_id, &v2_id)?;

println!("Changes: {}", diff.summary_text());
println!("Columns added: {:?}", diff.schema_changes.columns_added);
println!("Rows changed: {:?}", diff.row_changes);
println!("Statistical changes: {}", diff.statistical_changes.len());
```

### Publishing

```rust
use beefcake::analyser::lifecycle::stages::PublishMode;

// Publish as view (lazy computation)
let published_id = registry.publish_version(
    &dataset_id,
    &version_id,
    PublishMode::View
)?;

// Publish as snapshot (materialized)
let published_id = registry.publish_version(
    &dataset_id,
    &version_id,
    PublishMode::Snapshot
)?;
```

## Tauri API Commands

### Create Dataset
```typescript
const datasetId = await invoke('lifecycle_create_dataset', {
  request: {
    name: 'my_dataset',
    path: '/path/to/data.csv'
  }
});
```

### Apply Transforms
```typescript
const pipeline = {
  transforms: [
    {
      transform_type: 'clean',
      parameters: {
        configs: { /* column configs */ },
        restricted: true
      }
    }
  ]
};

const versionId = await invoke('lifecycle_apply_transforms', {
  request: {
    dataset_id: datasetId,
    pipeline_json: JSON.stringify(pipeline),
    stage: 'Cleaned'
  }
});
```

### Set Active Version
```typescript
await invoke('lifecycle_set_active_version', {
  request: {
    dataset_id: datasetId,
    version_id: versionId
  }
});
```

### Publish Version
```typescript
const publishedId = await invoke('lifecycle_publish_version', {
  request: {
    dataset_id: datasetId,
    version_id: versionId,
    mode: 'snapshot' // or 'view'
  }
});
```

### Get Diff
```typescript
const diff = await invoke('lifecycle_get_version_diff', {
  request: {
    dataset_id: datasetId,
    version1_id: v1Id,
    version2_id: v2Id
  }
});

console.log('Summary:', diff.summary_text());
```

### List Versions
```typescript
const versions = JSON.parse(await invoke('lifecycle_list_versions', {
  request: { dataset_id: datasetId }
}));

versions.forEach(v => {
  console.log(`${v.stage} - ${v.id} - ${v.created_at}`);
});
```

## Storage Structure

```
data/
└── datasets/
    └── {dataset-id}/
        ├── {version-id-1}.parquet
        ├── {version-id-1}.meta.json
        ├── {version-id-2}.parquet
        ├── {version-id-2}.meta.json
        └── ...
```

### Metadata Format
```json
{
  "id": "uuid",
  "dataset_id": "uuid",
  "parent_id": "uuid",
  "stage": "Cleaned",
  "pipeline": {
    "transforms": [...]
  },
  "data_location": {
    "ParquetFile": "/path/to/version.parquet"
  },
  "metadata": {
    "description": "Stage: Cleaned",
    "tags": [],
    "row_count": 10000,
    "column_count": 25,
    "created_by": "system"
  },
  "created_at": "2026-01-11T10:30:00Z"
}
```

## Benefits

1. **Reproducibility**: Every transform is serialized, allowing exact reproduction
2. **Auditability**: Full lineage from raw to published
3. **Efficiency**: Streaming operations and parquet compression
4. **Flexibility**: Choose view (lazy) or snapshot (materialized)
5. **Safety**: Raw data is never modified
6. **Versioning**: Git-like branching and merging capabilities
7. **Diff Tracking**: Know exactly what changed between versions

## Best Practices

1. **Start with Profile**: Always profile before cleaning
2. **Incremental Stages**: Progress through stages sequentially
3. **Validate Often**: Add validation rules at each critical stage
4. **Use Views for Experimentation**: Snapshots for production
5. **Tag Versions**: Use metadata tags to mark important versions
6. **Keep Raw Safe**: Never modify the raw version
7. **Document Transforms**: Add descriptions to pipeline steps

## Future Enhancements

- [ ] Version branching and merging
- [ ] Automated pipeline suggestions based on profiling
- [ ] Integration with Git for pipeline versioning
- [ ] Web UI for version tree visualization
- [ ] Export pipeline as Python/SQL scripts
- [ ] Collaborative dataset annotations
- [ ] Time-travel queries across versions
