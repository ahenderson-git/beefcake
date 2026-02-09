import { ColumnCleanConfig } from './config';
import { TransformPipeline } from './pipeline';

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
  path?: string;
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

export interface CurrentDataset {
  id: string;
  name: string;
  versions: DatasetVersion[];
  activeVersionId: string;
  rawVersionId: string;
}

export interface ExportSource {
  type: 'Analyser' | 'Python' | 'SQL';
  content?: string;
  path?: string;
}

export interface ExportDestination {
  type: 'File' | 'Database';
  target: string;
  format?: 'csv' | 'json' | 'parquet';
}

export interface ExportOptions {
  source: ExportSource;
  configs: Record<string, ColumnCleanConfig>;
  destination: ExportDestination;
  create_dictionary?: boolean;
  create_receipt?: boolean;
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
