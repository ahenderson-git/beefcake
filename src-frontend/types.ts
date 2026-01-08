export interface ColumnSummary {
  name: string;
  kind: string;
  count: number;
  nulls: number;
  stats: any;
  interpretation: string[];
  ml_advice: string[];
  business_summary: string[];
  samples: string[];
}

export interface AnalysisResponse {
  file_name: string;
  file_path: string;
  file_size: number;
  row_count: number;
  column_count: number;
  summary: ColumnSummary[];
  analysis_duration: { secs: number; nanos: number };
}

export type View = "Dashboard" | "Analyser" | "PowerShell" | "Settings" | "CLI";

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
}
