export interface NumericStats {
  min: number | null;
  distinct_count: number;
  p05: number | null;
  q1: number | null;
  median: number | null;
  mean: number | null;
  trimmed_mean: number | null;
  q3: number | null;
  p95: number | null;
  max: number | null;
  std_dev: number | null;
  skew: number | null;
  zero_count: number;
  negative_count: number;
  is_integer: boolean;
  is_sorted: boolean;
  is_sorted_rev: boolean;
  bin_width: number;
  histogram: [number, number][] | null;
}

export interface TemporalStats {
  min: string | null;
  max: string | null;
  distinct_count: number;
  p05: number | null;
  p95: number | null;
  is_sorted: boolean;
  is_sorted_rev: boolean;
  bin_width: number;
  histogram: [number, number][] | null;
}

export interface ColumnStats {
  Numeric?: NumericStats;
  Temporal?: TemporalStats;
  Categorical?: Record<string, number>;
  Boolean?: { true_count: number; false_count: number };
  Text?: {
    distinct: number;
    top_value: [string, number] | null;
    min_length: number;
    max_length: number;
    avg_length: number;
  };
}

export interface ColumnSummary {
  name: string;
  standardized_name: string;
  kind: string;
  count: number;
  nulls: number;
  stats: ColumnStats;
  interpretation: string[];
  ml_advice: string[];
  business_summary: string[];
  samples: string[];
}

export interface ColumnInfo {
  name: string;
  dtype: string;
}

export type NormalisationMethod = 'None' | 'ZScore' | 'MinMax';
export type ImputeMode = 'None' | 'Mean' | 'Median' | 'Zero' | 'Mode';
export type TextCase = 'None' | 'Lowercase' | 'Uppercase' | 'TitleCase';

export interface ColumnCleanConfig {
  new_name: string;
  target_dtype: string | null;
  active: boolean;
  advanced_cleaning: boolean;
  ml_preprocessing: boolean;
  trim_whitespace: boolean;
  remove_special_chars: boolean;
  text_case: TextCase;
  standardise_nulls: boolean;
  remove_non_ascii: boolean;
  regex_find: string;
  regex_replace: string;
  rounding: number | null;
  extract_numbers: boolean;
  clip_outliers: boolean;
  temporal_format: string;
  timezone_utc: boolean;
  freq_threshold: number | null;
  normalisation: NormalisationMethod;
  one_hot_encode: boolean;
  impute_mode: ImputeMode;
}

export interface CorrelationMatrix {
  columns: string[];
  data: number[][];
}

export interface FileHealth {
  score: number;
  risks: string[];
}

export interface AnalysisResponse {
  file_name: string;
  path: string;
  file_size: number;
  row_count: number;
  total_row_count: number;
  column_count: number;
  summary: ColumnSummary[];
  health: FileHealth;
  analysis_duration: { secs: number; nanos: number };
  correlation_matrix: CorrelationMatrix | null;
}

export interface AuditEntry {
  timestamp: string;
  action: string;
  details: string;
}

export type View =
  | 'Dashboard'
  | 'Analyser'
  | 'PowerShell'
  | 'Python'
  | 'SQL'
  | 'Settings'
  | 'CLI'
  | 'ActivityLog'
  | 'Reference'
  | 'Lifecycle'
  | 'Pipeline'
  | 'Watcher'
  | 'Dictionary';

// Dataset Lifecycle Types
export type LifecycleStage =
  | 'Raw'
  | 'Profiled'
  | 'Cleaned'
  | 'Advanced'
  | 'Validated'
  | 'Published';
export type PublishMode = 'View' | 'Snapshot';

export interface VersionMetadata {
  description: string;
  tags: string[];
  row_count: number | null;
  column_count: number | null;
  file_size_bytes: number | null;
  created_by: string;
  custom_fields: Record<string, unknown>;
}

export interface DataLocation {
  ParquetFile?: string;
  OriginalFile?: string;
  path?: string; // Added for convenience
}

export interface TransformSpec {
  transform_type: string;
  parameters: Record<string, unknown>;
}

export interface TransformPipeline {
  transforms: TransformSpec[];
}

export interface DatasetVersion {
  id: string;
  dataset_id: string;
  parent_id: string | null;
  stage: LifecycleStage;
  pipeline: TransformPipeline;
  data_location: DataLocation;
  metadata: VersionMetadata;
  created_at: string;
}

export interface SchemaChanges {
  columns_added: string[];
  columns_removed: string[];
  columns_renamed: [string, string][];
  type_changes: TypeChange[];
}

export interface TypeChange {
  column: string;
  old_type: string;
  new_type: string;
}

export interface RowChanges {
  rows_v1: number;
  rows_v2: number;
  rows_added: number | null;
  rows_removed: number | null;
  rows_modified: number | null;
}

export interface StatisticalChange {
  column: string;
  metric: string;
  value_v1: number | null;
  value_v2: number | null;
  change_percent: number | null;
}

export interface SampleChange {
  row_index: number;
  column: string;
  old_value: string;
  new_value: string;
}

export interface DiffSummary {
  version1_id: string;
  version2_id: string;
  schema_changes: SchemaChanges;
  row_changes: RowChanges;
  statistical_changes: StatisticalChange[];
  sample_changes: SampleChange[];
}

export interface CurrentDataset {
  id: string;
  name: string;
  versions: DatasetVersion[];
  activeVersionId: string;
  rawVersionId: string;
}

export interface ExportSource {
  type: 'Analyser' | 'Python' | 'SQL';
  content?: string; // The script or query
  path?: string; // Original data path
}

export interface ExportDestination {
  type: 'File' | 'Database';
  target: string; // File path or Connection ID
  format?: 'csv' | 'json' | 'parquet';
}

export interface ExportOptions {
  source: ExportSource;
  configs: Record<string, ColumnCleanConfig>;
  destination: ExportDestination;
  create_dictionary?: boolean;
}

export interface DbConnection {
  id: string;
  name: string;
  settings: {
    db_type: string;
    host: string;
    port: string;
    user: string;
    password: string;
    database: string;
    schema: string;
    table: string;
  };
}

export interface AIConfig {
  enabled: boolean;
  model: string;
  temperature: number;
  max_tokens: number;
}

export interface AppConfig {
  connections: DbConnection[];
  active_import_id: string | null;
  active_export_id: string | null;
  powershell_font_size: number;
  python_font_size: number;
  sql_font_size: number;
  analysis_sample_size?: number;
  sampling_strategy?: string;
  first_run_completed?: boolean;
  trusted_paths?: string[];
  security_warning_acknowledged?: boolean;
  ai_config?: AIConfig;
  audit_log: AuditEntry[];
}

export interface StandardPaths {
  base_dir: string;
  input_dir: string;
  output_dir: string;
  scripts_dir: string;
  logs_dir: string;
  templates_dir: string;
}

export interface AppState {
  version: string;
  config: AppConfig | null;
  currentView: View;
  analysisResponse: AnalysisResponse | null;
  expandedRows: Set<string>;
  cleaningConfigs: Record<string, ColumnCleanConfig>;
  isAddingConnection: boolean;
  isLoading: boolean;
  isAborting: boolean;
  loadingMessage: string;
  pythonScript: string | null;
  sqlScript: string | null;
  pythonSkipCleaning: boolean;
  sqlSkipCleaning: boolean;
  currentDataset: CurrentDataset | null;
  selectedColumns: Set<string>;
  useOriginalColumnNames: boolean;
  cleanAllActive: boolean;
  advancedProcessingEnabled: boolean;
  watcherState: WatcherState | null;
  watcherActivities: WatcherActivity[];
  polarsVersion?: string;
  selectedVersionId: string | null;
  currentIdeColumns: ColumnInfo[] | null;
  previousVersionId: string | null;
}

export function getDefaultColumnCleanConfig(col: ColumnSummary): ColumnCleanConfig {
  return {
    new_name: col.standardized_name || col.name,
    target_dtype: null,
    active: true,
    advanced_cleaning: false,
    ml_preprocessing: false,
    trim_whitespace: true,
    remove_special_chars: false,
    text_case: 'None',
    standardise_nulls: true,
    remove_non_ascii: false,
    regex_find: '',
    regex_replace: '',
    rounding: null,
    extract_numbers: false,
    clip_outliers: false,
    temporal_format: '',
    timezone_utc: false,
    freq_threshold: null,
    normalisation: 'None',
    one_hot_encode: false,
    impute_mode: 'None',
  };
}

// Watcher Types
export interface WatcherState {
  enabled: boolean;
  folder: string;
  state: 'idle' | 'watching' | 'ingesting' | 'error';
  message?: string;
}

export interface WatcherActivity {
  id: string;
  timestamp: string;
  filename: string;
  path: string;
  status: 'detected' | 'ingesting' | 'success' | 'failed';
  message?: string;
  datasetId?: string | undefined;
  rows?: number | undefined;
  cols?: number | undefined;
}

export interface WatcherEventPayload {
  path?: string;
  timestamp?: string;
  status?: string;
  rows?: number;
  cols?: number;
  datasetId?: string;
  message?: string;
}

// Data Dictionary Types
export interface DataDictionary {
  snapshot_id: string;
  dataset_name: string;
  export_timestamp: string;
  dataset_metadata: DatasetMetadata;
  columns: ColumnMetadata[];
  previous_snapshot_id: string | null;
}

export interface DatasetMetadata {
  technical: TechnicalMetadata;
  business: DatasetBusinessMetadata;
}

export interface TechnicalMetadata {
  input_sources: InputSource[];
  pipeline_id: string | null;
  pipeline_json: string | null;
  input_dataset_hash: string | null;
  output_dataset_hash: string;
  row_count: number;
  column_count: number;
  export_format: string;
  quality_summary: QualitySummary;
}

export interface InputSource {
  path: string;
  hash: string | null;
}

export interface QualitySummary {
  avg_null_percentage: number;
  empty_column_count: number;
  constant_column_count: number;
  duplicate_row_count: number | null;
  overall_score: number;
}

export interface DatasetBusinessMetadata {
  description?: string;
  intended_use?: string;
  owner_or_steward?: string;
  refresh_expectation?: string;
  sensitivity_classification?: string;
  known_limitations?: string;
  tags: string[];
}

export interface ColumnMetadata {
  column_id: string;
  current_name: string;
  original_name: string | null;
  technical: ColumnTechnicalMetadata;
  business: ColumnBusinessMetadata;
}

export interface ColumnTechnicalMetadata {
  data_type: string;
  nullable: boolean;
  null_percentage: number;
  distinct_count: number;
  min_value: string | null;
  max_value: string | null;
  sample_values: string[];
  warnings: string[];
  stats_json: string | null;
}

export interface ColumnBusinessMetadata {
  business_definition?: string;
  business_rules?: string;
  sensitivity_tag?: string;
  approved_examples: string[];
  notes?: string;
}

export interface SnapshotMetadata {
  snapshot_id: string;
  dataset_name: string;
  timestamp: string;
  output_hash: string;
  row_count: number;
  column_count: number;
  completeness_pct: number;
}

/**
 * Documentation file metadata returned from the backend
 */
export interface DocFileMetadata {
  /** Relative path from docs/ directory (e.g., "README.md") */
  path: string;
  /** Display name for the UI */
  title: string;
  /** Category for grouping (e.g., "Getting Started", "Reference") */
  category: string;
}
