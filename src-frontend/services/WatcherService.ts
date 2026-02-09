import { listen, UnlistenFn } from '@tauri-apps/api/event';

import * as api from '../api';
import { WatcherComponent } from '../components/WatcherComponent';
import { WatcherState, WatcherActivity, AppState } from '../types';

export class WatcherService {
  private unlisteners: UnlistenFn[] = [];

  constructor(
    private state: AppState,
    private components: Record<string, unknown>,
    private onStateUpdate: () => void,
    private showToast: (message: string, type?: 'success' | 'error' | 'info') => void
  ) {}

  async init(): Promise<void> {
    try {
      // Listen for watcher status updates
      this.unlisteners.push(
        await listen<WatcherState>('watcher:status', event => {
          this.state.watcherState = event.payload;
          this.onStateUpdate();
        })
      );

      // Helper to dispatch to WatcherComponent
      const dispatchToWatcher = (eventName: string, payload: WatcherActivity): void => {
        const watcherComp = this.components['Watcher'] as WatcherComponent;
        if (watcherComp) {
          watcherComp.handleWatcherEvent(this.state, eventName, payload);
        }
      };

      // Listen for various watcher activities
      const events: string[] = [
        'watcher:file_detected',
        'watcher:file_ready',
        'watcher:ingest_started',
        'watcher:ingest_succeeded',
        'watcher:ingest_failed',
      ];

      for (const eventName of events) {
        this.unlisteners.push(
          await listen<WatcherActivity>(eventName, event => {
            dispatchToWatcher(eventName, event.payload);
          })
        );
      }

      // Load initial watcher state
      try {
        this.state.watcherState = await api.watcherGetState();
      } catch (err) {
        this.showToast(`Failed to load watcher state: ${String(err)}`, 'error');
      }
    } catch (err) {
      this.showToast(`Failed to setup watcher events: ${String(err)}`, 'error');
    }
  }

  destroy(): void {
    this.unlisteners.forEach(unlisten => unlisten());
    this.unlisteners = [];
  }
}
