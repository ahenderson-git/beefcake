import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { AnalysisResponse, AppConfig } from "./types";

export async function analyseFile(path: string, trimPct: number): Promise<AnalysisResponse> {
  return await invoke("analyze_file", { path, trim_pct: trimPct });
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

export async function pushToDb(path: string, connectionId: string): Promise<void> {
  await invoke("push_to_db", { path, connectionId });
}

export async function readTextFile(path: string): Promise<string> {
  return await invoke("read_text_file", { path });
}

export async function writeTextFile(path: string, contents: string): Promise<void> {
  await invoke("write_text_file", { path, contents });
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
