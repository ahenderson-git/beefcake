import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { AnalysisResponse, AppConfig, ColumnCleanConfig, ExportOptions } from "./types";

export async function analyseFile(path: string, trimPct: number): Promise<AnalysisResponse> {
  return await invoke("analyze_file", { path, trimPct });
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
