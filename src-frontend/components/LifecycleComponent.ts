import * as api from '../api';
import * as renderers from '../renderers';
import { AppState, DiffSummary } from '../types';

import { Component, ComponentActions } from './Component';

export class LifecycleComponent extends Component {
  private publishModalVisible: boolean = false;

  constructor(containerId: string, actions: ComponentActions) {
    super(containerId, actions);
  }

  render(state: AppState): void {
    const container = this.getContainer();
    container.innerHTML = renderers.renderLifecycleView(state.currentDataset);
    this.bindEvents(state);

    // Render modal if visible
    if (this.publishModalVisible) {
      this.showPublishModal(state);
    }
  }

  override bindEvents(state: AppState): void {
    // Publish button
    document.getElementById('btn-publish-version')?.addEventListener('click', () => {
      if (!state.currentDataset) {
        this.actions.showToast('No dataset loaded', 'error');
        return;
      }
      this.publishModalVisible = true;
      this.showPublishModal(state);
    });

    // Version tree actions
    document.querySelectorAll('[data-action="set-active"]').forEach(btn => {
      btn.addEventListener('click', e => {
        void (async () => {
          const versionId = (e.currentTarget as HTMLElement).dataset.versionId!;
          if (!state.currentDataset) return;

          try {
            await api.setActiveVersion(state.currentDataset.id, versionId);
            state.currentDataset.activeVersionId = versionId;
            this.actions.showToast('Active version changed', 'success');
            this.actions.onStateChange();
          } catch (err) {
            this.actions.showToast(`Failed to set active version: ${String(err)}`, 'error');
          }
        })();
      });
    });

    document.querySelectorAll('[data-action="view-diff"]').forEach(btn => {
      btn.addEventListener('click', e => {
        void (async () => {
          const target = e.currentTarget as HTMLButtonElement;
          const versionId = target.dataset.versionId!;
          if (!state.currentDataset) return;

          try {
            const version = state.currentDataset.versions.find(v => v.id === versionId);
            if (!version?.parent_id) {
              this.actions.showToast(
                'Cannot compute diff - this is the first version with no parent',
                'info'
              );
              return;
            }

            // Show loading state
            target.disabled = true;
            target.textContent = 'Loading...';
            this.actions.showToast('Computing version diff...', 'info');

            const diff = await api.getVersionDiff(
              state.currentDataset.id,
              version.parent_id,
              versionId
            );

            // Reset button state
            target.disabled = false;
            target.textContent = 'View Diff';

            this.showDiffModal(diff, state);
          } catch (err) {
            // Reset button state on error
            target.disabled = false;
            target.textContent = 'View Diff';
            this.actions.showToast(`Failed to compute diff: ${String(err)}`, 'error');
          }
        })();
      });
    });

    // Stage click handlers (from lifecycle rail)
    document.querySelectorAll('.lifecycle-stage').forEach(stageEl => {
      stageEl.addEventListener('click', e => {
        void (async () => {
          const target = e.currentTarget as HTMLElement;
          const stage = target.dataset.stage!;

          if (target.classList.contains('stage-locked')) {
            this.actions.showToast(
              `Cannot switch to ${stage} stage yet - prerequisites not met`,
              'error'
            );
            return;
          }

          if (!state.currentDataset) return;

          const targetVersion = state.currentDataset.versions.find(v => v.stage === stage);
          if (targetVersion && targetVersion.id !== state.currentDataset.activeVersionId) {
            try {
              await api.setActiveVersion(state.currentDataset.id, targetVersion.id);
              state.currentDataset.activeVersionId = targetVersion.id;
              this.actions.showToast(`Switched to ${stage} version`, 'success');
              this.actions.onStateChange();
            } catch (err) {
              this.actions.showToast(`Failed to switch version: ${String(err)}`, 'error');
            }
          }
        })();
      });
    });
  }

  private showPublishModal(state: AppState): void {
    const modalContainer = document.getElementById('modal-container');
    if (!modalContainer) return;

    modalContainer.innerHTML = renderers.renderPublishModal();
    modalContainer.classList.add('active');
    this.bindPublishModalEvents(state);
  }

  private bindPublishModalEvents(state: AppState): void {
    document.getElementById('modal-close')?.addEventListener('click', () => {
      this.publishModalVisible = false;
      this.closeModal();
    });

    document.querySelector('.modal-overlay')?.addEventListener('click', e => {
      if (e.target === e.currentTarget) {
        this.publishModalVisible = false;
        this.closeModal();
      }
    });

    document.getElementById('btn-publish-view')?.addEventListener('click', () => {
      void this.handlePublish(state, 'view');
    });

    document.getElementById('btn-publish-snapshot')?.addEventListener('click', () => {
      void this.handlePublish(state, 'snapshot');
    });
  }

  private async handlePublish(state: AppState, mode: 'view' | 'snapshot'): Promise<void> {
    if (!state.currentDataset) return;

    try {
      this.actions.showToast(`Publishing as ${mode}...`, 'info');
      await api.publishVersion(state.currentDataset.id, state.currentDataset.activeVersionId, mode);

      // Reload versions to include the new published version
      const versionsJson = await api.listVersions(state.currentDataset.id);
      state.currentDataset.versions = JSON.parse(
        versionsJson
      ) as import('../types').DatasetVersion[];

      this.publishModalVisible = false;
      this.closeModal();
      this.actions.showToast(`Successfully published as ${mode}`, 'success');
      this.actions.onStateChange();
    } catch (err) {
      this.actions.showToast(`Failed to publish: ${String(err)}`, 'error');
    }
  }

  private closeModal(): void {
    const modalContainer = document.getElementById('modal-container');
    if (modalContainer) {
      modalContainer.classList.remove('active');
      modalContainer.innerHTML = '';
    }
  }

  private showDiffModal(diff: DiffSummary, state: AppState): void {
    const modalContainer = document.getElementById('modal-container');
    if (!modalContainer) return;

    modalContainer.innerHTML = this.renderDiffModal(diff, state);
    modalContainer.classList.add('active');

    // Close modal handlers
    const closeHandlers = () => {
      this.closeModal();
    };

    document.getElementById('modal-close')?.addEventListener('click', closeHandlers);
    document.getElementById('modal-close-footer')?.addEventListener('click', closeHandlers);

    document.querySelector('.modal-overlay')?.addEventListener('click', e => {
      if (e.target === e.currentTarget) {
        this.closeModal();
      }
    });

    // Keyboard shortcut - ESC to close
    const escHandler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        this.closeModal();
        document.removeEventListener('keydown', escHandler);
      }
    };
    document.addEventListener('keydown', escHandler);
  }

  private renderDiffModal(diff: DiffSummary, state: AppState): string {
    const hasSchemaChanges =
      diff.schema_changes.columns_added.length > 0 ||
      diff.schema_changes.columns_removed.length > 0 ||
      diff.schema_changes.columns_renamed.length > 0;
    const hasRowChanges = diff.row_changes.rows_v1 !== diff.row_changes.rows_v2;
    const rowChange = diff.row_changes.rows_v2 - diff.row_changes.rows_v1;

    // Get version labels
    const getVersionLabel = (versionId: string): string => {
      if (!state.currentDataset) return versionId;
      const version = state.currentDataset.versions.find(v => v.id === versionId);
      if (!version) return versionId;
      const index = state.currentDataset.versions.indexOf(version);
      return `v${index} (${version.stage})`;
    };

    const v1Label = getVersionLabel(diff.version1_id);
    const v2Label = getVersionLabel(diff.version2_id);

    return `
      <div class="modal-overlay" data-testid="diff-modal-overlay">
        <div class="modal-content modal-diff" data-testid="diff-modal">
          <div class="modal-header">
            <div>
              <h3>Version Diff: ${v1Label} → ${v2Label}</h3>
              <p class="diff-subtitle">Comparing changes between versions</p>
            </div>
            <button class="modal-close" id="modal-close" data-testid="diff-modal-close">
              <i class="ph ph-x"></i>
            </button>
          </div>

          <div class="modal-body">
            <div class="diff-summary-badges">
              ${diff.schema_changes.columns_added.length > 0 ? `<span class="badge badge-success">+${diff.schema_changes.columns_added.length} columns</span>` : ''}
              ${diff.schema_changes.columns_removed.length > 0 ? `<span class="badge badge-danger">-${diff.schema_changes.columns_removed.length} columns</span>` : ''}
              ${hasRowChanges ? `<span class="badge ${rowChange > 0 ? 'badge-success' : 'badge-danger'}">${rowChange > 0 ? '+' : ''}${rowChange.toLocaleString()} rows</span>` : ''}
              ${diff.statistical_changes.length > 0 ? `<span class="badge badge-info">${diff.statistical_changes.length} statistical changes</span>` : ''}
            </div>

          <div class="modal-body">
            <div class="diff-summary-badges">
              ${diff.schema_changes.columns_added.length > 0 ? `<span class="badge badge-success">+${diff.schema_changes.columns_added.length} columns</span>` : ''}
              ${diff.schema_changes.columns_removed.length > 0 ? `<span class="badge badge-danger">-${diff.schema_changes.columns_removed.length} columns</span>` : ''}
              ${hasRowChanges ? `<span class="badge ${rowChange > 0 ? 'badge-success' : 'badge-danger'}">${rowChange > 0 ? '+' : ''}${rowChange.toLocaleString()} rows</span>` : ''}
              ${diff.statistical_changes.length > 0 ? `<span class="badge badge-info">${diff.statistical_changes.length} statistical changes</span>` : ''}
            </div>

            <div class="diff-section">
              <h4>Row Changes</h4>
              <div class="diff-stats">
                <div class="diff-stat">
                  <span class="diff-stat-label">Version 1</span>
                  <span class="diff-stat-value">${diff.row_changes.rows_v1.toLocaleString()} rows</span>
                </div>
                <div class="diff-stat">
                  <span class="diff-stat-label">Version 2</span>
                  <span class="diff-stat-value">${diff.row_changes.rows_v2.toLocaleString()} rows</span>
                </div>
                ${
                  hasRowChanges
                    ? `
                  <div class="diff-stat">
                    <span class="diff-stat-label">Change</span>
                    <span class="diff-stat-value ${diff.row_changes.rows_v2 > diff.row_changes.rows_v1 ? 'diff-positive' : 'diff-negative'}">
                      ${diff.row_changes.rows_v2 > diff.row_changes.rows_v1 ? '+' : ''}${(diff.row_changes.rows_v2 - diff.row_changes.rows_v1).toLocaleString()} rows
                    </span>
                  </div>
                `
                    : ''
                }
              </div>
            </div>

            ${
              hasSchemaChanges
                ? `
              <div class="diff-section">
                <h4>Schema Changes</h4>
                ${
                  diff.schema_changes.columns_added.length > 0
                    ? `
                  <div class="diff-change">
                    <strong>Columns Added:</strong>
                    <ul>
                      ${diff.schema_changes.columns_added.map(col => `<li class="diff-added">${col}</li>`).join('')}
                    </ul>
                  </div>
                `
                    : ''
                }
                ${
                  diff.schema_changes.columns_removed.length > 0
                    ? `
                  <div class="diff-change">
                    <strong>Columns Removed:</strong>
                    <ul>
                      ${diff.schema_changes.columns_removed.map(col => `<li class="diff-removed">${col}</li>`).join('')}
                    </ul>
                  </div>
                `
                    : ''
                }
                ${
                  diff.schema_changes.columns_renamed.length > 0
                    ? `
                  <div class="diff-change">
                    <strong>Columns Renamed:</strong>
                    <ul>
                      ${diff.schema_changes.columns_renamed.map(([old, newName]) => `<li>${old} → ${newName}</li>`).join('')}
                    </ul>
                  </div>
                `
                    : ''
                }
              </div>
            `
                : ''
            }

            ${
              diff.statistical_changes.length > 0
                ? `
              <div class="diff-section">
                <h4>Statistical Changes</h4>
                <table class="diff-table">
                  <thead>
                    <tr>
                      <th>Column</th>
                      <th>Metric</th>
                      <th>Before</th>
                      <th>After</th>
                      <th>Change</th>
                    </tr>
                  </thead>
                  <tbody>
                    ${diff.statistical_changes
                      .slice(0, 10)
                      .map(
                        change => `
                      <tr>
                        <td><code>${change.column}</code></td>
                        <td>${change.metric}</td>
                        <td>${change.value_v1?.toFixed(2) ?? 'N/A'}</td>
                        <td>${change.value_v2?.toFixed(2) ?? 'N/A'}</td>
                        <td class="${(change.change_percent ?? 0) > 0 ? 'diff-positive' : 'diff-negative'}">
                          ${change.change_percent ? `${change.change_percent > 0 ? '+' : ''}${change.change_percent.toFixed(1)}%` : 'N/A'}
                        </td>
                      </tr>
                    `
                      )
                      .join('')}
                  </tbody>
                </table>
                ${diff.statistical_changes.length > 10 ? `<p class="diff-note">Showing 10 of ${diff.statistical_changes.length} changes</p>` : ''}
              </div>
            `
                : ''
            }
          </div>

          <div class="modal-footer">
            <button class="btn btn-secondary" id="modal-close-footer" data-testid="modal-close-footer">
              <i class="ph ph-x"></i> Close
            </button>
          </div>
        </div>
      </div>
    `;
  }
}
