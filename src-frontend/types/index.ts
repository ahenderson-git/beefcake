export * from './analysis';
export * from './pipeline';
export * from './dataset';
export * from './config';
export * from './integrity';

import { AnalysisResponse, ColumnInfo } from './analysis';
import {
  ColumnCleanConfig,
  AppConfig,
  WatcherState,
  WatcherActivity,
  StandardPaths,
} from './config';
import { CurrentDataset } from './dataset';

export type View =
  | 'Dashboard'
  | 'Analyser'
  | 'PowerShell'
  | 'Python'
  | 'SQL'
  | 'Settings'
  | 'CLI'
  | 'ActivityLog'
  | 'Reference'
  | 'Lifecycle'
  | 'Pipeline'
  | 'Watcher'
  | 'Dictionary'
  | 'Integrity';

export interface AppState {
  version: string;
  config: AppConfig | null;
  currentView: View;
  analysisResponse: AnalysisResponse | null;
  expandedRows: Set<string>;
  cleaningConfigs: Record<string, ColumnCleanConfig>;
  isAddingConnection: boolean;
  isLoading: boolean;
  isAborting: boolean;
  isCreatingLifecycle: boolean;
  loadingMessage: string;
  pythonScript: string | null;
  sqlScript: string | null;
  pythonSkipCleaning: boolean;
  sqlSkipCleaning: boolean;
  currentDataset: CurrentDataset | null;
  selectedColumns: Set<string>;
  useOriginalColumnNames: boolean;
  cleanAllActive: boolean;
  advancedProcessingEnabled: boolean;
  watcherState: WatcherState | null;
  watcherActivities: WatcherActivity[];
  polarsVersion?: string;
  selectedVersionId: string | null;
  currentIdeColumns: ColumnInfo[] | null;
  previousVersionId: string | null;
  paths?: StandardPaths;
}
