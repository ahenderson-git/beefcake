import { AppState, WatcherActivity } from "../types";
import * as api from "../api";
import { renderWatcherPanel } from "../renderers/watcher";
import { Component, ComponentActions } from "./Component";

export class WatcherComponent extends Component {
  constructor(containerId: string, actions: ComponentActions) {
    super(containerId, actions);
  }

  public override render(state: AppState): void {
    const container = this.getContainer();

    container.innerHTML = renderWatcherPanel(
      state.watcherState,
      state.watcherActivities
    );

    this.bindEvents(state);
  }

  public override bindEvents(state: AppState): void {
    // Toggle watcher on/off
    document.getElementById('btn-toggle-watcher')?.addEventListener('click', async () => {
      await this.handleToggleWatcher(state);
    });

    // Open folder picker dialog
    document.getElementById('btn-select-folder')?.addEventListener('click', async () => {
      await this.handleSelectFolder(state);
    });

    // Toggle auto-ingest setting
    document.getElementById('chk-auto-ingest')?.addEventListener('change', async (e) => {
      const target = e.target as HTMLInputElement;
      await this.handleToggleAutoIngest(state, target.checked);
    });

    // Retry failed ingestion
    document.querySelectorAll('.btn-retry-ingest').forEach(btn => {
      btn.addEventListener('click', async (e) => {
        const target = e.target as HTMLElement;
        const activityId = target.dataset.activityId;
        if (activityId) {
          await this.handleRetryIngest(state, activityId);
        }
      });
    });

    // View dataset from successful ingestion
    document.querySelectorAll('.btn-view-dataset').forEach(btn => {
      btn.addEventListener('click', (e) => {
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
      const currentlyEnabled = state.watcherState?.enabled || false;

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
      this.actions.showToast(`Failed to toggle watcher: ${err}`, 'error');
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

      state.watcherState = await api.watcherSetFolder(selected);

      this.actions.showToast(`Watch folder set: ${selected}`, 'success');
      this.actions.onStateChange();
    } catch (err) {
      this.actions.showToast(`Failed to select folder: ${err}`, 'error');
    }
  }

  private async handleToggleAutoIngest(_state: AppState, enabled: boolean): Promise<void> {
    try {
      // This will be implemented when we add auto-ingest config to backend
      // For now, just show a message
      this.actions.showToast(
        `Auto-ingest ${enabled ? 'enabled' : 'disabled'}`,
        'info'
      );
    } catch (err) {
      this.actions.showToast(`Failed to update auto-ingest: ${err}`, 'error');
    }
  }

  private async handleRetryIngest(state: AppState, activityId: string): Promise<void> {
    const activity = state.watcherActivities.find(a => a.id === activityId);
    if (!activity) return;

    try {
      await api.watcherIngestNow(activity.path);
      this.actions.showToast(`Retrying ingestion: ${activity.filename}`, 'info');
    } catch (err) {
      this.actions.showToast(`Failed to retry: ${err}`, 'error');
    }
  }

  private handleViewDataset(datasetId: string): void {
    if (this.actions.navigateTo) {
      this.actions.navigateTo('analyser', datasetId);
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
  public handleWatcherEvent(state: AppState, eventType: string, payload: any): void {
    switch (eventType) {
      case 'watcher:status':
        state.watcherState = payload;
        break;

      case 'watcher:file_detected':
        this.addActivity(state, {
          id: crypto.randomUUID(),
          timestamp: payload.detected_at,
          filename: this.extractFilename(payload.path),
          path: payload.path,
          status: 'detected',
          message: `Detected ${payload.file_type} file`,
        });
        break;

      case 'watcher:file_ready':
        this.updateActivityStatus(state, payload.path, 'detected', {
          message: 'File stable, ready for ingestion',
        });
        break;

      case 'watcher:ingest_started':
        this.updateActivityStatus(state, payload.path, 'ingesting', {
          status: 'ingesting',
          message: 'Ingesting dataset...',
        });
        break;

      case 'watcher:ingest_succeeded':
        this.updateActivityStatus(state, payload.path, 'success', {
          status: 'success',
          message: `Ingested ${payload.rows} rows, ${payload.cols} columns`,
          datasetId: payload.dataset_id,
          rows: payload.rows,
          cols: payload.cols,
        });
        this.actions.showToast(
          `Successfully ingested ${this.extractFilename(payload.path)}`,
          'success'
        );
        break;

      case 'watcher:ingest_failed':
        this.updateActivityStatus(state, payload.path, 'failed', {
          status: 'failed',
          message: payload.error,
        });
        this.actions.showToast(
          `Failed to ingest ${this.extractFilename(payload.path)}`,
          'error'
        );
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
    const activity = state.watcherActivities.find(
      a => a.path === path && a.status === oldStatus
    );

    if (activity) {
      Object.assign(activity, updates);
    }
  }

  private extractFilename(path: string): string {
    return path.split(/[/\\]/).pop() || path;
  }
}
