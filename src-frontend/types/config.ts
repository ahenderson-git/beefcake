import { ColumnSummary } from './analysis';

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

export interface AuditEntry {
  timestamp: string;
  action: string;
  details: string;
}

export interface AppSettings {
  connections: DbConnection[];
  active_import_id: string | null;
  active_export_id: string | null;
  powershell_font_size: number;
  python_font_size: number;
  sql_font_size: number;
  first_run_completed: boolean;
  trusted_paths: string[];
  preview_row_limit: number;
  security_warning_acknowledged: boolean;
  skip_full_row_count: boolean;
  analysis_sample_size: number;
  sampling_strategy: string;
  ai_config: AIConfig;
}

export interface AppConfig {
  settings: AppSettings;
  audit_log: {
    entries: AuditEntry[];
  };
}

export function getDefaultAppConfig(): AppConfig {
  return {
    settings: {
      connections: [],
      active_import_id: null,
      active_export_id: null,
      powershell_font_size: 14,
      python_font_size: 14,
      sql_font_size: 14,
      first_run_completed: false,
      trusted_paths: [],
      preview_row_limit: 100,
      security_warning_acknowledged: false,
      skip_full_row_count: false,
      analysis_sample_size: 10000,
      sampling_strategy: 'balanced',
      ai_config: {
        enabled: true,
        model: 'gpt-4o',
        temperature: 0.7,
        max_tokens: 2000,
      },
    },
    audit_log: {
      entries: [],
    },
  };
}

export interface StandardPaths {
  base_dir: string;
  input_dir: string;
  output_dir: string;
  scripts_dir: string;
  logs_dir: string;
  templates_dir: string;
}

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

export interface DocFileMetadata {
  path: string;
  title: string;
  category: string;
}
