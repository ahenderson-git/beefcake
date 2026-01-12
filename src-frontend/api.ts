/**
 * # Tauri Backend API
 *
 * This module provides TypeScript wrappers for Rust backend functions.
 * All communication with the Rust backend happens through Tauri's `invoke()` function.
 *
 * ## How It Works
 *
 * ```
 * TypeScript (Frontend)           Rust (Backend)
 * ────────────────────────────────────────────────
 * api.analyseFile(path)
 *   ↓
 * invoke("analyze_file", { path })
 *   ↓ (IPC/JSON serialization)
 *   ↓
 *                              #[tauri::command]
 *                              fn analyze_file(path: String)
 *   ↓
 * Promise<AnalysisResponse>
 * ```
 *
 * ## TypeScript-Rust Type Mapping
 *
 * | TypeScript       | Rust              |
 * |------------------|-------------------|
 * | string           | String            |
 * | number           | i32, u32, f64     |
 * | boolean          | bool              |
 * | object           | struct (Serialize)|
 * | Array<T>         | Vec<T>            |
 * | null/undefined   | Option<T>         |
 *
 * ## Error Handling
 *
 * All async functions can throw errors (returned as Promise rejection):
 *
 * ```typescript
 * try {
 *   const data = await api.analyseFile(path);
 * } catch (error) {
 *   console.error('Backend error:', error);  // error is a string
 * }
 * ```
 *
 * ## Naming Convention
 *
 * - TypeScript: camelCase (analyseFile)
 * - Rust: snake_case (analyze_file)
 * - invoke() uses Rust name: "analyze_file"
 *
 * @module api
 * @see {@link docs/ARCHITECTURE.md} for system architecture
 * @see {@link docs/TYPESCRIPT_PATTERNS.md} for Tauri bridge pattern
 */

import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { AnalysisResponse, AppConfig, ColumnCleanConfig, ExportOptions } from "./types";

/**
 * Analyzes a data file (CSV, JSON, or Parquet) and returns statistics.
 *
 * **Backend**: Calls `analyze_file` in `src/tauri_app.rs`
 *
 * The analysis includes:
 * - Column type detection (numeric, text, temporal, etc.)
 * - Statistical summaries (mean, median, percentiles)
 * - Data quality assessment (nulls, outliers, patterns)
 * - Business insights and recommendations
 *
 * @param path - Absolute path to the data file
 * @returns Promise resolving to analysis results
 * @throws Error string if file not found, invalid format, or analysis fails
 *
 * @example
 * ```typescript
 * try {
 *   const response = await api.analyseFile('C:/data/customers.csv');
 *   console.log(`Analyzed ${response.row_count} rows`);
 *   response.summary.forEach(col => {
 *     console.log(`${col.name}: ${col.kind}`);
 *   });
 * } catch (error) {
 *   console.error('Analysis failed:', error);
 * }
 * ```
 */
export async function analyseFile(path: string): Promise<AnalysisResponse> {
  return await invoke("analyze_file", { path });
}

export async function getAppVersion(): Promise<string> {
  return await invoke("get_app_version");
}

export async function loadAppConfig(): Promise<AppConfig> {
  return await invoke("get_config");
}

export async function saveAppConfig(config: AppConfig): Promise<void> {
  await invoke("save_config", { config });
}

export async function runPowerShell(script: string): Promise<string> {
  return await invoke("run_powershell", { script });
}

export async function runPython(script: string, dataPath?: string, configs?: Record<string, ColumnCleanConfig>): Promise<string> {
  return await invoke("run_python", { script, dataPath, configs });
}

export async function runSql(query: string, dataPath?: string, configs?: Record<string, ColumnCleanConfig>): Promise<string> {
  return await invoke("run_sql", { query, dataPath, configs });
}

export async function installPythonPackage(pkg: string): Promise<string> {
  return await invoke("install_python_package", { package: pkg });
}

export async function pushToDb(path: string, connectionId: string, configs: Record<string, ColumnCleanConfig>): Promise<void> {
  await invoke("push_to_db", { path, connectionId, configs });
}

export async function testConnection(settings: any, connectionId?: string): Promise<string> {
  return await invoke("test_connection", { settings, connectionId });
}

export async function deleteConnection(id: string): Promise<void> {
  await invoke("delete_connection", { id });
}

export async function exportData(options: ExportOptions): Promise<void> {
  await invoke("export_data", { options });
}

export async function abortProcessing(): Promise<void> {
  await invoke("abort_processing");
}

export async function resetAbortSignal(): Promise<void> {
  await invoke("reset_abort_signal");
}

export async function readTextFile(path: string): Promise<string> {
  return await invoke("read_text_file", { path });
}

export async function writeTextFile(path: string, contents: string): Promise<void> {
  await invoke("write_text_file", { path, contents });
}

export async function sanitizeHeaders(names: string[]): Promise<string[]> {
  return await invoke("sanitize_headers", { names });
}

export async function openFileDialog(filters?: { name: string, extensions: string[] }[]): Promise<string | null> {
  const selected = await open({
    multiple: false,
    filters: filters || [{
      name: 'Data Files',
      extensions: ['csv', 'json', 'parquet']
    }]
  });
  return typeof selected === 'string' ? selected : null;
}

export async function saveFileDialog(filters?: { name: string, extensions: string[] }[]): Promise<string | null> {
  return await save({
    filters: filters || [{
      name: 'Data Files',
      extensions: ['csv', 'json', 'parquet']
    }]
  });
}

// ============================================================================
// Dataset Lifecycle API
// ============================================================================

export async function createDataset(name: string, path: string): Promise<string> {
  return await invoke("lifecycle_create_dataset", {
    request: { name, path }
  });
}

export async function applyTransforms(
  datasetId: string,
  pipelineJson: string,
  stage: string
): Promise<string> {
  return await invoke("lifecycle_apply_transforms", {
    request: { dataset_id: datasetId, pipeline_json: pipelineJson, stage }
  });
}

export async function setActiveVersion(datasetId: string, versionId: string): Promise<void> {
  await invoke("lifecycle_set_active_version", {
    request: { dataset_id: datasetId, version_id: versionId }
  });
}

export async function publishVersion(
  datasetId: string,
  versionId: string,
  mode: 'view' | 'snapshot'
): Promise<string> {
  return await invoke("lifecycle_publish_version", {
    request: { dataset_id: datasetId, version_id: versionId, mode }
  });
}

export async function getVersionDiff(
  datasetId: string,
  version1Id: string,
  version2Id: string
): Promise<any> {
  return await invoke("lifecycle_get_version_diff", {
    request: { dataset_id: datasetId, version1_id: version1Id, version2_id: version2Id }
  });
}

export async function listVersions(datasetId: string): Promise<string> {
  return await invoke("lifecycle_list_versions", {
    request: { dataset_id: datasetId }
  });
}

// ============================================================================
// Pipeline Automation API
// ============================================================================

export async function savePipelineSpec(specJson: string, path: string): Promise<void> {
  await invoke("save_pipeline_spec", { specJson, path });
}

export async function loadPipelineSpec(path: string): Promise<string> {
  return await invoke("load_pipeline_spec", { path });
}

export async function validatePipelineSpec(specJson: string, inputPath: string): Promise<string[]> {
  return await invoke("validate_pipeline_spec", { specJson, inputPath });
}

export async function generatePowerShell(specJson: string, outputPath: string): Promise<string> {
  return await invoke("generate_powershell", { specJson, outputPath });
}

export async function pipelineFromConfigs(
  name: string,
  configsJson: string,
  inputFormat: string,
  outputPath: string
): Promise<string> {
  return await invoke("pipeline_from_configs", { name, configsJson, inputFormat, outputPath });
}
