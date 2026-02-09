export interface TransformSpec {
  transform_type: string;
  parameters: Record<string, unknown>;
}

export interface TransformPipeline {
  transforms: TransformSpec[];
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

export interface DiffSummary {
  version1_id: string;
  version2_id: string;
  schema_changes: SchemaChanges;
  row_changes: RowChanges;
  statistical_changes: StatisticalChange[];
  sample_changes: SampleChange[];
}
