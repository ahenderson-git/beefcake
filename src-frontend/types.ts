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

export type NormalizationMethod = "None" | "ZScore" | "MinMax";
export type ImputeMode = "None" | "Mean" | "Median" | "Zero" | "Mode";
export type TextCase = "None" | "Lowercase" | "Uppercase" | "TitleCase";

export interface ColumnCleanConfig {
  new_name: string;
  target_dtype: string | null;
  active: boolean;
  advanced_cleaning: boolean;
  ml_preprocessing: boolean;
  trim_whitespace: boolean;
  remove_special_chars: boolean;
  text_case: TextCase;
  standardize_nulls: boolean;
  remove_non_ascii: boolean;
  regex_find: string;
  regex_replace: string;
  rounding: number | null;
  extract_numbers: boolean;
  clip_outliers: boolean;
  temporal_format: string;
  timezone_utc: boolean;
  freq_threshold: number | null;
  normalization: NormalizationMethod;
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

export type View = "Dashboard" | "Analyser" | "PowerShell" | "Python" | "SQL" | "Settings" | "CLI" | "ActivityLog" | "Reference" | "Lifecycle" | "Pipeline";

// Dataset Lifecycle Types
export type LifecycleStage = "Raw" | "Profiled" | "Cleaned" | "Advanced" | "Validated" | "Published";
export type PublishMode = "View" | "Snapshot";

export interface VersionMetadata {
  description: string;
  tags: string[];
  row_count: number | null;
  column_count: number | null;
  file_size_bytes: number | null;
  created_by: string;
  custom_fields: Record<string, any>;
}

export interface DataLocation {
  ParquetFile?: string;
  OriginalFile?: string;
}

export interface TransformSpec {
  transform_type: string;
  parameters: Record<string, any>;
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
  path?: string;    // Original data path
}

export interface ExportDestination {
  type: 'File' | 'Database';
  target: string;   // File path or Connection ID
  format?: 'csv' | 'json' | 'parquet';
}

export interface ExportOptions {
  source: ExportSource;
  configs: Record<string, ColumnCleanConfig>;
  destination: ExportDestination;
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

export interface AppConfig {
  connections: DbConnection[];
  active_import_id: string | null;
  active_export_id: string | null;
  powershell_font_size: number;
  python_font_size: number;
  sql_font_size: number;
  audit_log: AuditEntry[];
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
    text_case: "None",
    standardize_nulls: true,
    remove_non_ascii: false,
    regex_find: "",
    regex_replace: "",
    rounding: null,
    extract_numbers: false,
    clip_outliers: false,
    temporal_format: "",
    timezone_utc: false,
    freq_threshold: null,
    normalization: "None",
    one_hot_encode: false,
    impute_mode: "None"
  };
}
