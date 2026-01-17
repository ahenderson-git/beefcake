import * as api from '../api';
import * as renderers from '../renderers';
import { AppState, DbConnection } from '../types';

import { Component, ComponentActions } from './Component';

export class SettingsComponent extends Component {
  constructor(containerId: string, actions: ComponentActions) {
    super(containerId, actions);
  }

  render(state: AppState): void {
    if (!state.config) return;
    const container = this.getContainer();
    container.innerHTML = renderers.renderSettingsView(state.config, state.isAddingConnection);
    this.bindEvents(state);
  }

  override bindEvents(state: AppState): void {
    if (state.isAddingConnection) {
      document.getElementById('btn-save-new-conn')?.addEventListener('click', () => {
        void this.saveNewConnection(state);
      });

      document.getElementById('btn-cancel-new-conn')?.addEventListener('click', () => {
        state.isAddingConnection = false;
        this.actions.onStateChange();
      });

      document.getElementById('btn-test-new-conn')?.addEventListener('click', e => {
        void this.testNewConnection(e.currentTarget as HTMLButtonElement);
      });
    } else {
      document.getElementById('btn-add-conn')?.addEventListener('click', () => {
        state.isAddingConnection = true;
        this.actions.onStateChange();
      });
    }

    document.getElementById('select-import-id')?.addEventListener('change', e => {
      if (state.config) {
        state.config.active_import_id = (e.target as HTMLSelectElement).value || null;
        void api.saveAppConfig(state.config);
      }
    });

    document.getElementById('select-export-id')?.addEventListener('change', e => {
      if (state.config) {
        state.config.active_export_id = (e.target as HTMLSelectElement).value || null;
        void api.saveAppConfig(state.config);
      }
    });

    document.querySelectorAll('.btn-test-conn').forEach(btn => {
      btn.addEventListener('click', e => {
        const id = (e.currentTarget as HTMLElement).dataset.id!;
        void this.handleTestConnection(state, id, e.currentTarget as HTMLButtonElement);
      });
    });

    document.querySelectorAll('.btn-delete-conn').forEach(btn => {
      btn.addEventListener('click', e => {
        const id = (e.currentTarget as HTMLElement).dataset.id!;
        void this.deleteConnection(state, id);
      });
    });

    // Handle analysis sample size changes
    const sampleSizeInput = document.getElementById('analysis-sample-size') as HTMLInputElement;
    if (sampleSizeInput) {
      // Show initial warning
      this.updateSampleSizeWarning(parseInt(sampleSizeInput.value) || 10000);

      sampleSizeInput.addEventListener('input', e => {
        const value = parseInt((e.target as HTMLInputElement).value) || 10000;
        this.updateSampleSizeWarning(value);
      });

      sampleSizeInput.addEventListener('change', e => {
        if (state.config) {
          let value = parseInt((e.target as HTMLInputElement).value) || 10000;
          // Clamp value between min and max
          value = Math.max(1000, Math.min(500000, value));
          (e.target as HTMLInputElement).value = value.toString();

          state.config.analysis_sample_size = value;
          void api.saveAppConfig(state.config).then(() => {
            // Add impact summary to toast
            let impactSummary;
            if (value <= 25000) {
              impactSummary = ' (recommended)';
            } else if (value <= 50000) {
              impactSummary = ' (moderate performance impact)';
            } else if (value <= 100000) {
              impactSummary = ' (high performance impact)';
            } else {
              impactSummary = ' (very high performance impact)';
            }

            this.actions.showToast(
              `Sample size updated to ${value.toLocaleString()} rows${impactSummary}`,
              'success'
            );
          });
        }
      });
    }

    // Handle sampling strategy changes
    const samplingStrategySelect = document.getElementById(
      'sampling-strategy'
    ) as HTMLSelectElement;
    if (samplingStrategySelect) {
      // Show initial warning
      const initialStrategy = samplingStrategySelect.value || 'balanced';
      const initialSampleSize = parseInt(sampleSizeInput?.value || '10000');
      this.updateSamplingStrategyWarning(initialStrategy, initialSampleSize);

      samplingStrategySelect.addEventListener('change', e => {
        const strategy = (e.target as HTMLSelectElement).value;
        const sampleSize = parseInt(sampleSizeInput?.value || '10000');

        this.updateSamplingStrategyWarning(strategy, sampleSize);

        if (state.config) {
          state.config.sampling_strategy = strategy;
          void api.saveAppConfig(state.config).then(() => {
            // Add impact summary to toast
            let strategyLabel;
            let impactSummary;

            if (strategy === 'fast') {
              strategyLabel = 'Fast (sequential)';
              impactSummary = ' - fastest but may be biased';
            } else if (strategy === 'balanced') {
              strategyLabel = 'Balanced (stratified)';
              impactSummary = ' - recommended';
            } else {
              strategyLabel = 'Accurate (reservoir)';
              impactSummary = ' - most accurate';
            }

            this.actions.showToast(
              `Sampling strategy updated to ${strategyLabel}${impactSummary}`,
              'success'
            );
          });
        }
      });
    }
  }

  private updateSampleSizeWarning(value: number): void {
    const warningDiv = document.getElementById('sample-size-warning') as HTMLElement;
    if (!warningDiv) return;

    let message;
    let className;

    if (value < 10000) {
      message = '⚠️ May reduce accuracy for distribution analysis';
      className = 'warning-low';
    } else if (value <= 25000) {
      message = '✓ Recommended • Good balance of speed and accuracy';
      className = 'warning-good';
    } else if (value <= 50000) {
      message = '⚠️ Moderate impact • ~5-10x slower for large files';
      className = 'warning-moderate';
    } else if (value <= 100000) {
      message = '⚠️ High impact • ~20-50x slower, use with caution';
      className = 'warning-high';
    } else {
      message = '🔴 Very high impact • May take minutes for billion-row files';
      className = 'warning-critical';
    }

    warningDiv.textContent = message;
    warningDiv.className = `sample-size-warning ${className}`;
  }

  private updateSamplingStrategyWarning(strategy: string, sampleSize: number): void {
    const warningDiv = document.getElementById('sampling-strategy-info') as HTMLElement;
    if (!warningDiv) return;

    let message;
    let className;

    switch (strategy) {
      case 'fast':
        message =
          '⚡ Fastest • Samples from first ~1M rows • May miss patterns in sorted data • Best for <10M rows';
        className = 'warning-moderate';
        break;

      case 'balanced':
        if (sampleSize > 100000) {
          message = `✓ Recommended • Even sampling across entire file • ~5-10s for billion rows (${sampleSize.toLocaleString()} samples)`;
        } else {
          message =
            '✓ Recommended • Even sampling across entire file • Good speed-accuracy balance';
        }
        className = 'warning-good';
        break;

      case 'accurate':
        message = `⏱️ Reservoir sampling (Phase 2) • Perfectly unbiased • ~30-60s for billion rows`;
        className = 'warning-info';
        break;

      default:
        message = '';
        className = '';
    }

    warningDiv.textContent = message;
    warningDiv.className = `sampling-strategy-info ${className}`;
  }

  private getNewConnectionSettings(): DbConnection['settings'] {
    const host = (document.getElementById('new-conn-host') as HTMLInputElement)?.value;
    const port = (document.getElementById('new-conn-port') as HTMLInputElement)?.value;
    const user = (document.getElementById('new-conn-user') as HTMLInputElement)?.value;
    const password = (document.getElementById('new-conn-pass') as HTMLInputElement)?.value;
    const database = (document.getElementById('new-conn-db') as HTMLInputElement)?.value;
    const schema =
      (document.getElementById('new-conn-schema') as HTMLInputElement)?.value || 'public';
    const table = (document.getElementById('new-conn-table') as HTMLInputElement)?.value || '';

    return {
      db_type: 'postgres',
      host: host ?? '',
      port: port ?? '',
      user: user ?? '',
      password: password ?? '',
      database: database ?? '',
      schema,
      table,
    };
  }

  private async saveNewConnection(state: AppState): Promise<void> {
    const name = (document.getElementById('new-conn-name') as HTMLInputElement)?.value;
    if (!name) {
      this.actions.showToast('Connection name is required', 'error');
      return;
    }

    const newConn: DbConnection = {
      id: crypto.randomUUID(),
      name,
      settings: this.getNewConnectionSettings(),
    };

    if (state.config) {
      state.config.connections.push(newConn);
      await api.saveAppConfig(state.config);
      state.isAddingConnection = false;
      this.actions.onStateChange();
      this.actions.showToast('Connection added', 'success');
    }
  }

  private async testNewConnection(btn: HTMLButtonElement): Promise<void> {
    const settings = this.getNewConnectionSettings();
    const icon = btn.querySelector('i');

    if (icon) {
      btn.disabled = true;
      icon.className = 'ph ph-circle-notch loading-spin';
    }

    try {
      this.actions.showToast('Testing connection...', 'info');
      const result = await api.testConnection(settings);
      this.actions.showToast(result, 'success');
    } catch (err) {
      this.actions.showToast(String(err), 'error');
    } finally {
      if (icon) {
        btn.disabled = false;
        icon.className = 'ph ph-plugs-connected';
      }
    }
  }

  private async handleTestConnection(
    state: AppState,
    id: string,
    btn: HTMLButtonElement
  ): Promise<void> {
    const conn = state.config?.connections.find(c => c.id === id);
    if (!conn) return;

    const icon = btn.querySelector('i');
    if (icon) {
      btn.disabled = true;
      icon.className = 'ph ph-circle-notch loading-spin';
    }

    try {
      this.actions.showToast(`Testing connection "${conn.name}"...`, 'info');
      const result = await api.testConnection(conn.settings, id);
      this.actions.showToast(result, 'success');
    } catch (err) {
      this.actions.showToast(String(err), 'error');
    } finally {
      if (icon) {
        btn.disabled = false;
        icon.className = 'ph ph-plugs-connected';
      }
    }
  }

  private async deleteConnection(state: AppState, id: string): Promise<void> {
    if (state.config) {
      if (!confirm('Are you sure you want to delete this connection?')) return;
      state.config.connections = state.config.connections.filter(c => c.id !== id);
      if (state.config.active_import_id === id) state.config.active_import_id = null;
      if (state.config.active_export_id === id) state.config.active_export_id = null;

      await api.deleteConnection(id);
      await api.saveAppConfig(state.config);
      this.actions.onStateChange();
      this.actions.showToast('Connection deleted', 'success');
    }
  }
}
