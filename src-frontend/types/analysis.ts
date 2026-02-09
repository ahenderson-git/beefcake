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
  histogram: [number, number][] | null; // [bin_centre, count] from Rust Vec<(f64, usize)>
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
  histogram: [number, number][] | null; // [timestamp_ms, count] from Rust Vec<(f64, usize)>
}

export interface ColumnStats {
  Numeric?: NumericStats;
  Temporal?: TemporalStats;
  Categorical?: Record<string, number>; // HashMap<String, usize> from Rust
  Boolean?: { true_count: number; false_count: number };
  Text?: {
    distinct: number; // Note: Rust uses 'distinct', not 'distinct_count'
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

export interface CorrelationMatrix {
  columns: string[];
  data: number[][];
}

export interface FileHealth {
  score: number;
  risks: string[];
  notes: string[];
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
