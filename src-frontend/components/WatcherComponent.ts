import * as api from '../api';
import { renderWatcherPanel } from '../renderers/watcher';
import { AppState, WatcherActivity, WatcherState, WatcherEventPayload } from '../types';

import { Component, ComponentActions } from './Component';

export class WatcherComponent extends Component {
  constructor(containerId: string, actions: ComponentActions) {
    super(containerId, actions);
  }

  public override render(state: AppState): void {
    const container = this.getContainer();

    container.innerHTML = renderWatcherPanel(state.watcherState, state.watcherActivities);

    this.bindEvents(state);
  }

  public override bindEvents(state: AppState): void {
    // Toggle watcher on/off
    document.getElementById('btn-toggle-watcher')?.addEventListener('click', () => {
      void this.handleToggleWatcher(state);
    });

    // Open folder picker dialog
    document.getElementById('btn-select-folder')?.addEventListener('click', () => {
      void this.handleSelectFolder(state);
    });

    // Toggle auto-ingest setting
    document.getElementById('chk-auto-ingest')?.addEventListener('change', e => {
      const target = e.target as HTMLInputElement;
      void this.handleToggleAutoIngest(state, target.checked);
    });

    // Retry failed ingestion
    document.querySelectorAll('.btn-retry-ingest').forEach(btn => {
      btn.addEventListener('click', e => {
        const target = e.target as HTMLElement;
        const activityId = target.dataset.activityId;
        if (activityId) {
          void this.handleRetryIngest(state, activityId);
        }
      });
    });

    // View dataset from successful ingestion
    document.querySelectorAll('.btn-view-dataset').forEach(btn => {
      btn.addEventListener('click', e => {
        const target = e.target as HTMLElement;
        const datasetId = target.dataset.datasetId;
        if (datasetId) {
          this.handleViewDataset(datasetId);
        }
      });
    });

    // Clear activity feed
    document.getElementById('btn-clear-activities')?.addEventListener('click', () => {
      this.handleClearActivities(state);
    });
  }

  private async handleToggleWatcher(state: AppState): Promise<void> {
    try {
      const currentlyEnabled = state.watcherState?.enabled ?? false;

      if (currentlyEnabled) {
        // Stop watcher
        state.watcherState = await api.watcherStop();
        this.actions.showToast('Watcher stopped', 'info');
      } else {
        // Start watcher - need folder
        if (!state.watcherState?.folder) {
          this.actions.showToast('Please select a folder first', 'error');
          return;
        }
        state.watcherState = await api.watcherStart(state.watcherState.folder);
        this.actions.showToast('Watcher started', 'success');
      }

      this.actions.onStateChange();
    } catch (err) {
      this.actions.showToast(`Failed to toggle watcher: ${String(err)}`, 'error');
    }
  }

  private async handleSelectFolder(state: AppState): Promise<void> {
    try {
      // Use Tauri dialog plugin
      const { open } = await import('@tauri-apps/plugin-dialog');

      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select Folder to Watch',
      });

      if (!selected) {
        return;
      }

      // Ensure selected is a string
      const folderPath = (Array.isArray(selected) ? selected[0] : selected) as unknown as string;
      if (!folderPath) return;

      state.watcherState = await api.watcherSetFolder(folderPath);

      this.actions.showToast(`Watch folder set: ${folderPath}`, 'success');
      this.actions.onStateChange();
    } catch (err) {
      this.actions.showToast(`Failed to select folder: ${String(err)}`, 'error');
    }
  }

  private async handleToggleAutoIngest(_state: AppState, enabled: boolean): Promise<void> {
    try {
      // This will be implemented when we add auto-ingest config to backend
      // For now, just show a message
      this.actions.showToast(`Auto-ingest ${enabled ? 'enabled' : 'disabled'}`, 'info');
      await Promise.resolve(); // satisfy require-await if needed
    } catch (err) {
      this.actions.showToast(`Failed to update auto-ingest: ${String(err)}`, 'error');
    }
  }

  private async handleRetryIngest(state: AppState, activityId: string): Promise<void> {
    const activity = state.watcherActivities.find(a => a.id === activityId);
    if (!activity) return;

    try {
      await api.watcherIngestNow(activity.path);
      this.actions.showToast(`Retrying ingestion: ${activity.filename}`, 'info');
    } catch (err) {
      this.actions.showToast(`Failed to retry: ${String(err)}`, 'error');
    }
  }

  private handleViewDataset(datasetId: string): void {
    if (this.actions.navigateTo) {
      void this.actions.navigateTo('analyser', datasetId);
    }
  }

  private handleClearActivities(state: AppState): void {
    state.watcherActivities = [];
    this.actions.onStateChange();
    this.actions.showToast('Activity feed cleared', 'info');
  }

  /**
   * Handle watcher event from backend
   */
  public handleWatcherEvent(state: AppState, eventType: string, payload: unknown): void {
    if (eventType === 'watcher:status') {
      state.watcherState = payload as WatcherState;
      this.actions.onStateChange();
      return;
    }

    const p = payload as WatcherEventPayload;
    if (!p) return;

    switch (eventType) {
      case 'watcher:file_detected':
        this.addActivity(state, {
          id: crypto.randomUUID(),
          timestamp: p.timestamp ?? new Date().toISOString(),
          filename: this.extractFilename(p.path ?? ''),
          path: p.path ?? '',
          status: 'detected',
          message: `Detected ${p.status ?? 'new'} file`,
        });
        break;

      case 'watcher:file_ready':
        this.updateActivityStatus(state, p.path ?? '', 'detected', {
          message: 'File stable, ready for ingestion',
        });
        break;

      case 'watcher:ingest_started':
        this.updateActivityStatus(state, p.path ?? '', 'ingesting', {
          status: 'ingesting',
          message: 'Ingesting dataset...',
        });
        break;

      case 'watcher:ingest_succeeded':
        this.updateActivityStatus(state, p.path ?? '', 'success', {
          status: 'success',
          message: `Ingested ${p.rows ?? 0} rows, ${p.cols ?? 0} columns`,
          datasetId: p.datasetId ?? undefined,
          rows: p.rows ?? undefined,
          cols: p.cols ?? undefined,
        });
        this.actions.showToast(
          `Successfully ingested ${this.extractFilename(p.path ?? '')}`,
          'success'
        );
        break;

      case 'watcher:ingest_failed':
        this.updateActivityStatus(state, p.path ?? '', 'failed', {
          status: 'failed',
          message: p.message ?? 'Unknown error',
        });
        this.actions.showToast(`Failed to ingest ${this.extractFilename(p.path ?? '')}`, 'error');
        break;
    }

    this.actions.onStateChange();
  }

  private addActivity(state: AppState, activity: WatcherActivity): void {
    state.watcherActivities.unshift(activity);

    // Keep only last 50 activities
    if (state.watcherActivities.length > 50) {
      state.watcherActivities = state.watcherActivities.slice(0, 50);
    }
  }

  private updateActivityStatus(
    state: AppState,
    path: string,
    oldStatus: string,
    updates: Partial<WatcherActivity>
  ): void {
    const activity = state.watcherActivities.find(a => a.path === path && a.status === oldStatus);

    if (activity) {
      Object.assign(activity, updates);
    }
  }

  private extractFilename(path: string): string {
    return path.split(/[/\\]/).pop() ?? path;
  }
}
