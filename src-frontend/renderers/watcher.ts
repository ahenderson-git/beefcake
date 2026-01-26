import type { WatcherState, WatcherActivity } from '../types';
import { escapeHtml } from '../utils';

export function renderWatcherPanel(
  watcherState: WatcherState | null,
  activities: WatcherActivity[]
): string {
  const acts = activities || [];
  return `
    <div class="watcher-container">
      ${renderWatcherControls(watcherState)}
      ${renderWatcherStatus(watcherState)}
      ${renderActivityFeed(acts)}
    </div>
  `;
}

function renderWatcherControls(state: WatcherState | null): string {
  const enabled = state?.enabled ?? false;
  const folder = state?.folder ?? '';
  const hasFolder = folder.length > 0;

  return `
    <div class="watcher-controls">
      <h3><i class="ph ph-eye"></i> Watch Folder</h3>
      <p class="watcher-help-text">
        Automatically ingest CSV and JSON files dropped into a watched folder.
      </p>

      <div class="control-group">
        <label>Watch Folder:</label>
        <div class="folder-input-group">
          <input
            type="text"
            id="txt-watch-folder"
            class="folder-input"
            value="${escapeHtml(folder)}"
            readonly
            placeholder="No folder selected"
          />
          <button id="btn-select-folder" class="btn-secondary btn-small">
            <i class="ph ph-folder-open"></i> Browse...
          </button>
        </div>
      </div>

      <div class="watcher-control-row">
        <label class="control-label">
          <span>Watcher Status</span>
          <button id="btn-toggle-watcher" class="btn-toggle ${enabled ? 'active' : ''}" ${!hasFolder ? 'disabled' : ''}>
            <i class="ph ph-${enabled ? 'pause' : 'play'}"></i>
            ${enabled ? 'Stop Watching' : 'Start Watching'}
          </button>
        </label>
      </div>

      <div class="watcher-setting">
        <label>
          <input type="checkbox" id="chk-auto-ingest" ${enabled ? 'checked' : ''}>
          <span>Auto-ingest new files</span>
        </label>
      </div>
    </div>
  `;
}

function renderWatcherStatus(state: WatcherState | null): string {
  if (!state) {
    return `
      <div class="watcher-status-section">
        <h3>Status</h3>
        <div class="watcher-status">
          <div class="status-row">
            <label>Status:</label>
            <span class="status-badge status-idle">
              <i class="ph ph-circle"></i>
              Not Initialized
            </span>
          </div>
        </div>
      </div>
    `;
  }

  const stateClass = `status-${state.state}`;
  const stateIcon =
    {
      idle: 'ph-circle',
      watching: 'ph-eye',
      ingesting: 'ph-arrow-circle-down',
      error: 'ph-warning-circle',
    }[state.state] || 'ph-circle';

  const stateLabel =
    {
      idle: 'Idle',
      watching: 'Watching',
      ingesting: 'Ingesting',
      error: 'Error',
    }[state.state] || 'Unknown';

  return `
    <div class="watcher-status-section">
      <h3>Status</h3>
      <div class="watcher-status">
        <div class="status-row">
          <label>Status:</label>
          <span class="status-badge ${stateClass}">
            <i class="ph ${stateIcon}"></i>
            ${stateLabel}
          </span>
        </div>

        ${
          state.folder
            ? `
          <div class="watcher-folder">
            <label>Watching:</label>
            <div class="folder-path">
              <i class="ph ph-folder-open"></i>
              <span>${escapeHtml(state.folder)}</span>
            </div>
          </div>
        `
            : ''
        }

        ${
          state.message
            ? `
          <div class="watcher-message ${state.state === 'error' ? 'error' : 'info'}">
            <i class="ph ph-info"></i>
            <span>${escapeHtml(state.message)}</span>
          </div>
        `
            : ''
        }
      </div>
    </div>
  `;
}

function renderActivityFeed(activities: WatcherActivity[]): string {
  const acts = activities || [];
  return `
    <div class="watcher-activity-section">
      <div class="activity-header">
        <h3>Recent Activity</h3>
        ${
          acts.length > 0
            ? `
          <button id="btn-clear-activities" class="btn-text btn-small">
            <i class="ph ph-trash"></i> Clear
          </button>
        `
            : ''
        }
      </div>
      ${
        acts.length === 0
          ? `
        <div class="activity-empty">
          <i class="ph ph-tray"></i>
          <p>No recent activity</p>
        </div>
      `
          : `
        <div class="activity-list">
          ${acts.map(activity => renderActivityItem(activity)).join('')}
        </div>
      `
      }
    </div>
  `;
}

function renderActivityItem(activity: WatcherActivity): string {
  const statusClass = `activity-${activity.status}`;
  const statusIcon =
    {
      detected: 'ph-file-magnifying-glass',
      ingesting: 'ph-spinner',
      success: 'ph-check-circle',
      failed: 'ph-x-circle',
    }[activity.status] || 'ph-circle';

  const statusLabel =
    {
      detected: 'Detected',
      ingesting: 'Ingesting',
      success: 'Success',
      failed: 'Failed',
    }[activity.status] || 'Unknown';

  return `
    <div class="activity-item ${statusClass}">
      <div class="activity-icon">
        <i class="ph ${statusIcon}"></i>
      </div>
      <div class="activity-details">
        <div class="activity-filename">${escapeHtml(activity.filename)}</div>
        <div class="activity-meta">
          <span class="activity-status">${statusLabel}</span>
          <span class="activity-time">${formatTimestamp(activity.timestamp)}</span>
        </div>
        ${
          activity.message
            ? `
          <div class="activity-message">${escapeHtml(activity.message)}</div>
        `
            : ''
        }
        ${
          activity.rows && activity.cols
            ? `
          <div class="activity-stats">
            <span>${activity.rows.toLocaleString()} rows</span>
            <span>${activity.cols} columns</span>
          </div>
        `
            : ''
        }
      </div>
      <div class="activity-actions">
        ${
          activity.status === 'failed'
            ? `
          <button class="btn-retry-ingest btn-text btn-small" data-activity-id="${activity.id}">
            <i class="ph ph-arrow-clockwise"></i> Retry
          </button>
        `
            : ''
        }
        ${
          activity.status === 'success' && activity.datasetId
            ? `
          <button class="btn-view-dataset btn-primary btn-small" data-dataset-id="${activity.datasetId}">
            <i class="ph ph-arrow-right"></i> View
          </button>
        `
            : ''
        }
      </div>
    </div>
  `;
}

function formatTimestamp(timestamp: string): string {
  try {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);

    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins}m ago`;

    const diffHours = Math.floor(diffMins / 60);
    if (diffHours < 24) return `${diffHours}h ago`;

    const diffDays = Math.floor(diffHours / 24);
    return `${diffDays}d ago`;
  } catch {
    return timestamp;
  }
}
