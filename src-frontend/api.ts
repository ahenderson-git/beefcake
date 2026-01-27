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
 * | Array&lt;T&gt;         | Vec&lt;T&gt;            |
 * | null/undefined   | Option&lt;T&gt;         |
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
 * @see Architecture Documentation: ../docs/ARCHITECTURE.md
 * @see TypeScript Patterns: ../docs/TYPESCRIPT_PATTERNS.md
 */

import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';

import {
  AnalysisResponse,
  AppConfig,
  ColumnCleanConfig,
  ExportOptions,
  WatcherState,
  DataDictionary,
  DatasetBusinessMetadata,
  ColumnBusinessMetadata,
  SnapshotMetadata,
  DbConnection,
  DiffSummary,
  DocFileMetadata,
  ColumnInfo,
  StandardPaths,
  VerificationResult,
  TransformPipeline,
  TransformSpec,
} from './types';

/**
 * Analyses a data file (CSV, JSON, or Parquet) and returns statistics.
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
 *   console.log(`Analysed ${response.row_count} rows`);
 *   response.summary.forEach(col => {
 *     console.log(`${col.name}: ${col.kind}`);
 *   });
 * } catch (error) {
 *   console.error('Analysis failed:', error);
 * }
 * ```
 */
export async function analyseFile(path: string): Promise<AnalysisResponse> {
  return await invoke('analyze_file', { path });
}

export async function getAppVersion(): Promise<string> {
  return await invoke('get_app_version');
}

export async function loadAppConfig(): Promise<AppConfig> {
  const rawConfig = await invoke('get_config');

  // Optional: Add runtime validation with Zod (commented out by default for performance)
  // Uncomment to enable strict runtime validation that would have caught the audit_log bug
  // import { safeValidateBackendConfig } from './schemas/config';
  // const result = safeValidateBackendConfig(rawConfig);
  // if (!result.success) {
  //   console.error('[loadAppConfig] Validation failed:', result.error);
  //   throw new Error(result.error);
  // }
  // return result.data;

  return rawConfig as AppConfig;
}

export async function saveAppConfig(config: AppConfig): Promise<void> {
  await invoke('save_config', { config });
}

export async function runPowerShell(script: string): Promise<string> {
  return await invoke('run_powershell', { script });
}

export async function runPython(
  script: string,
  dataPath?: string,
  configs?: Record<string, ColumnCleanConfig>
): Promise<string> {
  return await invoke('run_python', { script, dataPath, configs });
}

export async function runSql(
  query: string,
  dataPath?: string,
  configs?: Record<string, ColumnCleanConfig>
): Promise<string> {
  return await invoke('run_sql', { query, dataPath, configs });
}

export async function installPythonPackage(pkg: string): Promise<string> {
  return await invoke('install_python_package', { package: pkg });
}

export async function pushToDb(
  path: string,
  connectionId: string,
  configs: Record<string, ColumnCleanConfig>
): Promise<void> {
  await invoke('push_to_db', { path, connectionId, configs });
}

export async function testConnection(
  settings: DbConnection['settings'],
  connectionId?: string
): Promise<string> {
  return await invoke('test_connection', { settings, connectionId });
}

export async function deleteConnection(id: string): Promise<void> {
  await invoke('delete_connection', { id });
}

export async function exportData(options: ExportOptions): Promise<void> {
  await invoke('export_data', { options });
}

/**
 * Verify the integrity of a file using its receipt.
 *
 * @param receiptPath - Path to .receipt.json file
 * @returns Promise resolving to verification result
 */
export async function verifyReceipt(receiptPath: string): Promise<VerificationResult> {
  return await invoke('verify_receipt', { receiptPath });
}

export async function abortProcessing(): Promise<void> {
  await invoke('abort_processing');
}

export async function resetAbortSignal(): Promise<void> {
  await invoke('reset_abort_signal');
}

export async function getStandardPaths(): Promise<StandardPaths> {
  return await invoke('get_standard_paths');
}

export async function openPath(path: string): Promise<void> {
  await invoke('open_path', { path });
}

export async function logFrontendEvent(
  level: 'info' | 'warn' | 'error' | 'debug',
  action: string,
  details: string,
  context?: Record<string, unknown>
): Promise<void> {
  await invoke('log_frontend_event', { level, action, details, context });
}

export async function readTextFile(path: string): Promise<string> {
  return await invoke('read_text_file', { path });
}

export async function writeTextFile(path: string, contents: string): Promise<void> {
  await invoke('write_text_file', { path, contents });
}

export async function sanitizeHeaders(names: string[]): Promise<string[]> {
  return await invoke('sanitize_headers', { names });
}

export async function openFileDialog(
  filters?: { name: string; extensions: string[] }[]
): Promise<string | null> {
  const selected = await open({
    multiple: false,
    filters: filters ?? [
      {
        name: 'Data Files',
        extensions: ['csv', 'json', 'parquet'],
      },
    ],
  });
  return typeof selected === 'string' ? selected : null;
}

export async function saveFileDialog(
  filters?: { name: string; extensions: string[] }[]
): Promise<string | null> {
  return await save({
    filters: filters ?? [
      {
        name: 'Data Files',
        extensions: ['csv', 'json', 'parquet'],
      },
    ],
  });
}

export async function openFolderDialog(): Promise<string | null> {
  const selected = await open({
    multiple: false,
    directory: true,
  });
  return typeof selected === 'string' ? selected : null;
}

export async function listTrustedPaths(): Promise<string[]> {
  return await invoke('list_trusted_paths');
}

export async function addTrustedPath(path: string): Promise<string[]> {
  return await invoke('add_trusted_path', { path });
}

export async function removeTrustedPath(path: string): Promise<string[]> {
  return await invoke('remove_trusted_path', { path });
}

// ============================================================================
// Dataset Lifecycle API
// ============================================================================

export async function createDataset(name: string, path: string): Promise<string> {
  return await invoke('lifecycle_create_dataset', {
    request: { name, source_path: path },
  });
}

export async function applyTransforms(
  datasetId: string,
  pipelineJson: string,
  stage: string
): Promise<string> {
  let transforms: TransformSpec[] = [];
  try {
    const parsed = JSON.parse(pipelineJson) as TransformPipeline;
    if (Array.isArray(parsed?.transforms)) {
      transforms = parsed.transforms;
    }
  } catch {
    // Ignore parse errors, use empty array
  }

  return await invoke('lifecycle_apply_transforms', {
    request: { dataset_id: datasetId, transforms, next_stage: stage },
  });
}

export async function setActiveVersion(datasetId: string, versionId: string): Promise<void> {
  await invoke('lifecycle_set_active_version', {
    request: { dataset_id: datasetId, version_id: versionId },
  });
}

export async function publishVersion(
  datasetId: string,
  versionId: string,
  mode: 'view' | 'snapshot'
): Promise<string> {
  return await invoke('lifecycle_publish_version', {
    request: { dataset_id: datasetId, version_id: versionId, mode },
  });
}

export async function getVersionDiff(
  datasetId: string,
  version1Id: string,
  version2Id: string
): Promise<DiffSummary> {
  return await invoke('lifecycle_get_version_diff', {
    request: { dataset_id: datasetId, from_version_id: version1Id, to_version_id: version2Id },
  });
}

export async function listVersions(datasetId: string): Promise<string> {
  return await invoke('lifecycle_list_versions', {
    request: { dataset_id: datasetId },
  });
}

export async function getVersionSchema(
  datasetId: string,
  versionId: string
): Promise<ColumnInfo[]> {
  return await invoke('lifecycle_get_version_schema', {
    request: { dataset_id: datasetId, version_id: versionId },
  });
}

// ============================================================================
// Pipeline Automation API
// ============================================================================

export async function savePipelineSpec(specJson: string, path: string): Promise<void> {
  await invoke('save_pipeline_spec', { specJson, path });
}

export async function loadPipelineSpec(path: string): Promise<string> {
  return await invoke('load_pipeline_spec', { path });
}

export async function validatePipelineSpec(specJson: string, inputPath: string): Promise<string[]> {
  return await invoke('validate_pipeline_spec', { specJson, inputPath });
}

export async function generatePowerShell(specJson: string, outputPath: string): Promise<string> {
  return await invoke('generate_powershell', { specJson, outputPath });
}

export async function pipelineFromConfigs(
  name: string,
  configsJson: string,
  inputFormat: string,
  outputPath: string
): Promise<string> {
  return await invoke('pipeline_from_configs', { name, configsJson, inputFormat, outputPath });
}

// Watcher API
export async function watcherGetState(): Promise<WatcherState> {
  return await invoke('watcher_get_state');
}

export async function watcherStart(folder: string): Promise<WatcherState> {
  return await invoke('watcher_start', { folder });
}

export async function watcherStop(): Promise<WatcherState> {
  return await invoke('watcher_stop');
}

export async function watcherSetFolder(folder: string): Promise<WatcherState> {
  return await invoke('watcher_set_folder', { folder });
}

export async function watcherIngestNow(path: string): Promise<void> {
  return await invoke('watcher_ingest_now', { path });
}

// Data Dictionary API
export async function dictionaryLoadSnapshot(snapshotId: string): Promise<DataDictionary> {
  return await invoke('dictionary_load_snapshot', { snapshotId });
}

export async function dictionaryListSnapshots(datasetHash?: string): Promise<SnapshotMetadata[]> {
  return await invoke('dictionary_list_snapshots', { datasetHash });
}

export async function dictionaryUpdateBusinessMetadata(
  snapshotId: string,
  datasetBusiness?: DatasetBusinessMetadata,
  columnBusinessUpdates?: Record<string, ColumnBusinessMetadata>
): Promise<string> {
  return await invoke('dictionary_update_business_metadata', {
    request: {
      snapshot_id: snapshotId,
      dataset_business: datasetBusiness,
      column_business_updates: columnBusinessUpdates,
    },
  });
}

export async function dictionaryExportMarkdown(
  snapshotId: string,
  outputPath: string
): Promise<void> {
  return await invoke('dictionary_export_markdown', { snapshotId, outputPath });
}

/**
 * List all available documentation files with metadata.
 *
 * @returns Array of documentation file metadata
 * @throws Error if documentation directory cannot be accessed
 */
export async function listDocumentationFiles(): Promise<DocFileMetadata[]> {
  return await invoke('list_documentation_files');
}

/**
 * Read the content of a documentation file.
 *
 * @param docPath - Relative path from docs/ directory (e.g., "README.md")
 * @returns Markdown content of the documentation file
 * @throws Error if file cannot be read or path is invalid
 */
export async function readDocumentationFile(docPath: string): Promise<string> {
  return await invoke('read_documentation_file', { docPath });
}

/**
 * Check Python environment and Polars installation status.
 *
 * @returns String describing Python version and Polars installation status
 * @throws Error if Python is not available
 */
export async function checkPythonEnvironment(): Promise<string> {
  return await invoke('check_python_environment');
}

/**
 * Log a frontend error to the backend log files.
 *
 * @param level - Log level (error, warn, info, debug)
 * @param message - The error message
 * @param context - Optional context object with additional details
 */
export async function logFrontendError(
  level: 'error' | 'warn' | 'info' | 'debug',
  message: string,
  context?: Record<string, unknown>
): Promise<void> {
  try {
    await invoke('log_frontend_error', { level, message, context });
  } catch (err) {
    // Silently fail - don't want logging errors to crash the app
    console.error('Failed to log to backend:', err);
  }
}

/**
 * Get the path to the log directory.
 *
 * @returns Absolute path to the log directory
 */
export async function getLogDirectory(): Promise<string> {
  return await invoke('get_log_directory');
}

/**
 * Get the path to the current log file.
 *
 * @returns Absolute path to the current log file
 */
export async function getCurrentLogFile(): Promise<string> {
  return await invoke('get_current_log_file');
}

/**
 * Get the path to the current error log file.
 *
 * @returns Absolute path to the current error log file
 */
export async function getCurrentErrorLogFile(): Promise<string> {
  return await invoke('get_current_error_log_file');
}
