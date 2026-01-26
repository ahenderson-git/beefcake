export function fmtBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

export function fmtDuration(duration: { secs: number; nanos: number }): string {
  const ms = duration.secs * 1000 + duration.nanos / 1000000;
  if (ms < 1000) return `${ms.toFixed(2)}ms`;
  return `${(ms / 1000).toFixed(2)}s`;
}

export function escapeHtml(unsafe: string | null | undefined): string {
  if (unsafe === null || unsafe === undefined) return '';
  return unsafe
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

/**
 * Gets the appropriate data file path for execution based on the current app state.
 *
 * Priority order:
 * 1. If dataset versions exist, use the selected (or active) version's data location
 * 2. Otherwise, fall back to the analysis response path (original file)
 *
 * @param state - The application state
 * @returns The path to the data file to use for execution, or undefined if none available
 */
export function getDataPathForExecution(state: {
  currentDataset: {
    versions: Array<{
      id: string;
      data_location: { path?: string; ParquetFile?: string; OriginalFile?: string };
    }>;
    activeVersionId: string;
  } | null;
  selectedVersionId: string | null;
  analysisResponse: { path: string } | null;
}): string | undefined {
  // If we have a dataset with versions, use the selected version's path
  if (state.currentDataset && (state.currentDataset.versions?.length ?? 0) > 0) {
    const selectedVersionId = state.selectedVersionId ?? state.currentDataset.activeVersionId;
    const selectedVersion = state.currentDataset.versions.find(v => v.id === selectedVersionId);
    if (selectedVersion) {
      const loc = selectedVersion.data_location;
      return loc.path ?? loc.ParquetFile ?? loc.OriginalFile;
    }
  }
  // Fall back to the analysis response path
  return state.analysisResponse?.path;
}

/**
 * Gets a visual badge/icon for a lifecycle stage.
 *
 * @param stage - The lifecycle stage name
 * @returns An emoji or icon representing the stage
 */
export function getStageIcon(stage: string): string {
  const stageMap: Record<string, string> = {
    Raw: '📥',
    Profiled: '🔍',
    Cleaned: '✨',
    Advanced: '⚙️',
    Validated: '✅',
    Published: '🚀',
  };
  return stageMap[stage] ?? '📊';
}

/**
 * Gets the sort order for a lifecycle stage (for proper ordering in dropdowns).
 *
 * @param stage - The lifecycle stage name
 * @returns A numeric sort order (lower = earlier stage)
 */
export function getStageOrder(stage: string): number {
  const orderMap: Record<string, number> = {
    Raw: 0,
    Profiled: 1,
    Cleaned: 2,
    Advanced: 3,
    Validated: 4,
    Published: 5,
  };
  return orderMap[stage] ?? 999; // Unknown stages go to end
}
