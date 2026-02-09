import { invoke } from '@tauri-apps/api/core';

import * as api from '../api';
import * as renderers from '../renderers';
import { AppState, DbConnection, StandardPaths } from '../types';

import { Component, ComponentActions } from './Component';

export class SettingsComponent extends Component {
  private standardPaths: StandardPaths | null = null;
  private trustedPaths: string[] | null = null;
  private isLoadingPaths = false;

  constructor(containerId: string, actions: ComponentActions) {
    super(containerId, actions);
  }

  render(state: AppState): void {
    if (!state.config) return;
    const container = this.getContainer();
    container.innerHTML = renderers.renderSettingsView(
      state.config,
      state.isAddingConnection,
      this.standardPaths,
      this.trustedPaths
    );
    this.bindEvents(state);
    // Check and display API key status on render
    void this.updateAPIKeyStatus();
    void this.loadPaths();
    void this.loadLogPath();
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
        state.config.settings.active_import_id = (e.target as HTMLSelectElement).value || null;
        void api.saveAppConfig(state.config);
      }
    });

    document.getElementById('select-export-id')?.addEventListener('change', e => {
      if (state.config) {
        state.config.settings.active_export_id = (e.target as HTMLSelectElement).value || null;
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

          state.config.settings.analysis_sample_size = value;
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
          state.config.settings.sampling_strategy = strategy;
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

    // Handle AI settings
    this.bindAISettings(state);

    // Folder quick actions
    document.querySelectorAll<HTMLButtonElement>('.folder-btn').forEach(btn => {
      btn.addEventListener('click', () => {
        const path = btn.dataset.path;
        if (path) {
          void api.openPath(path);
        }
      });
    });

    document.querySelectorAll<HTMLButtonElement>('.btn-copy-path').forEach(btn => {
      btn.addEventListener('click', () => {
        const path = btn.dataset.copy;
        if (path) {
          const label = (btn.title || btn.getAttribute('aria-label')) ?? 'Path';
          void this.copyPath(path, label);
        }
      });
    });

    document.getElementById('btn-add-trusted-path')?.addEventListener('click', () => {
      void this.handleAddTrustedPath();
    });

    document.getElementById('btn-show-onboarding')?.addEventListener('click', () => {
      this.actions.showFirstRunWizard?.();
    });

    // Log file access buttons
    document.getElementById('btn-open-log-dir')?.addEventListener('click', () => {
      void this.openLogDirectory();
    });

    document.getElementById('btn-open-main-log')?.addEventListener('click', () => {
      void this.openLogFile('main');
    });

    document.getElementById('btn-open-error-log')?.addEventListener('click', () => {
      void this.openLogFile('error');
    });

    document.querySelectorAll<HTMLButtonElement>('.btn-open-trusted-path').forEach(btn => {
      btn.addEventListener('click', () => {
        const path = btn.dataset.path;
        if (path) {
          void api.openPath(path);
        }
      });
    });

    document.querySelectorAll<HTMLButtonElement>('.btn-remove-trusted-path').forEach(btn => {
      btn.addEventListener('click', () => {
        const path = btn.dataset.path;
        if (path) {
          void this.handleRemoveTrustedPath(path);
        }
      });
    });
  }

  private async loadPaths(): Promise<void> {
    if (this.isLoadingPaths || (this.standardPaths && this.trustedPaths)) {
      return;
    }
    this.isLoadingPaths = true;
    try {
      const [paths, trusted] = await Promise.all([api.getStandardPaths(), api.listTrustedPaths()]);
      this.standardPaths = paths;
      this.trustedPaths = trusted;
      this.actions.onStateChange();
    } catch (error) {
      console.error('Failed to load folders:', error);
    } finally {
      this.isLoadingPaths = false;
    }
  }

  private async handleAddTrustedPath(): Promise<void> {
    try {
      const selected = await api.openFolderDialog();
      if (!selected) return;
      this.trustedPaths = await api.addTrustedPath(selected);
      this.actions.showToast('Trusted folder added', 'success');
      this.actions.onStateChange();
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      this.actions.showToast(`Failed to add trusted folder: ${message}`, 'error');
    }
  }

  private async handleRemoveTrustedPath(path: string): Promise<void> {
    try {
      this.trustedPaths = await api.removeTrustedPath(path);
      this.actions.showToast('Trusted folder removed', 'success');
      this.actions.onStateChange();
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      this.actions.showToast(`Failed to remove trusted folder: ${message}`, 'error');
    }
  }

  private async copyPath(path: string, label?: string): Promise<void> {
    try {
      await navigator.clipboard.writeText(path);
      const msg = label ? `${label} copied` : 'Path copied to clipboard';
      this.actions.showToast(msg, 'success');
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      this.actions.showToast(`Failed to copy path: ${message}`, 'error');
    }
  }

  private bindAISettings(state: AppState): void {
    const aiEnabled = document.getElementById('ai-enabled') as HTMLInputElement;
    const aiApiKey = document.getElementById('ai-api-key') as HTMLInputElement;
    const btnToggleApiKey = document.getElementById('btn-toggle-api-key');
    const btnSaveApiKey = document.getElementById('btn-save-api-key');
    const btnTestAI = document.getElementById('btn-test-ai');
    const btnDeleteApiKey = document.getElementById('btn-delete-api-key');
    const aiModel = document.getElementById('ai-model') as HTMLSelectElement;
    const aiTemperature = document.getElementById('ai-temperature') as HTMLInputElement;
    const aiMaxTokens = document.getElementById('ai-max-tokens') as HTMLInputElement;

    // Toggle AI enabled
    aiEnabled?.addEventListener('change', () => {
      void (async (): Promise<void> => {
        if (state.config) {
          state.config.settings.ai_config = state.config.settings.ai_config ?? {
            enabled: false,
            model: 'gpt-3.5-turbo',
            temperature: 0.7,
            max_tokens: 1000,
          };
          state.config.settings.ai_config.enabled = aiEnabled.checked;
          await api.saveAppConfig(state.config);
          this.actions.showToast(
            `AI Assistant ${aiEnabled.checked ? 'enabled' : 'disabled'}`,
            'success'
          );
        }
      })();
    });

    // Toggle API key visibility
    btnToggleApiKey?.addEventListener('click', () => {
      if (aiApiKey) {
        const isPassword = aiApiKey.type === 'password';
        aiApiKey.type = isPassword ? 'text' : 'password';
        const icon = btnToggleApiKey.querySelector('i');
        if (icon) {
          icon.className = isPassword ? 'ph ph-eye-slash' : 'ph ph-eye';
        }
      }
    });

    // Save API key
    btnSaveApiKey?.addEventListener('click', () => {
      void (async (): Promise<void> => {
        const key = aiApiKey?.value.trim();
        if (!key) {
          this.showAIStatus('Please enter an API key', 'error');
          return;
        }

        try {
          await invoke<void>('ai_set_api_key', { apiKey: key });
          if (aiApiKey) aiApiKey.value = '';

          // Verify it was saved by checking keyring
          const hasKey = await invoke<boolean>('ai_has_api_key');
          if (hasKey) {
            this.showAIStatus('✓ API key saved securely', 'success');
            await this.updateAPIKeyStatus();
          } else {
            this.showAIStatus('Warning: API key may not have been saved correctly', 'error');
          }
        } catch (error: unknown) {
          const message = error instanceof Error ? error.message : String(error);
          this.showAIStatus(`Failed to save API key: ${message}`, 'error');
        }
      })();
    });

    // Test AI connection
    btnTestAI?.addEventListener('click', () => {
      void (async (): Promise<void> => {
        this.showAIStatus('Testing connection...', 'info');
        try {
          await invoke<void>('ai_test_connection');
          this.showAIStatus('✓ Connection successful!', 'success');
        } catch (error: unknown) {
          const message = error instanceof Error ? error.message : String(error);
          this.showAIStatus(`Connection failed: ${message}`, 'error');
        }
      })();
    });

    // Delete API key
    btnDeleteApiKey?.addEventListener('click', () => {
      void (async (): Promise<void> => {
        if (!confirm('Are you sure you want to delete your API key?')) {
          return;
        }

        try {
          await invoke<void>('ai_delete_api_key');
          this.showAIStatus('API key deleted', 'success');
          if (aiApiKey) aiApiKey.value = '';
          await this.updateAPIKeyStatus();
        } catch (error: unknown) {
          const message = error instanceof Error ? error.message : String(error);
          this.showAIStatus(`Failed to delete API key: ${message}`, 'error');
        }
      })();
    });

    // Update AI config on change
    const updateAIConfig = async (): Promise<void> => {
      if (state.config) {
        state.config.settings.ai_config = {
          enabled: aiEnabled?.checked ?? false,
          model: aiModel?.value ?? 'gpt-3.5-turbo',
          temperature: parseFloat(aiTemperature?.value ?? '0.7'),
          max_tokens: parseInt(aiMaxTokens?.value ?? '1000'),
        };

        try {
          await invoke<void>('ai_update_config', { aiConfig: state.config.settings.ai_config });
          await api.saveAppConfig(state.config);
          this.actions.showToast('AI configuration updated', 'success');
        } catch (error: unknown) {
          const message = error instanceof Error ? error.message : String(error);
          this.actions.showToast(`Failed to update AI config: ${message}`, 'error');
        }
      }
    };

    aiModel?.addEventListener('change', () => void updateAIConfig());
    aiTemperature?.addEventListener('change', () => void updateAIConfig());
    aiMaxTokens?.addEventListener('change', () => void updateAIConfig());
  }

  private async updateAPIKeyStatus(): Promise<void> {
    try {
      const hasKey = await invoke<boolean>('ai_has_api_key');
      const apiKeyLabel = document.querySelector('label[for="ai-api-key"]');

      if (apiKeyLabel) {
        // Remove any existing status indicator
        const existingStatus = apiKeyLabel.querySelector('.api-key-status');
        if (existingStatus) {
          existingStatus.remove();
        }

        // Add status indicator
        if (hasKey) {
          const statusSpan = document.createElement('span');
          statusSpan.className = 'api-key-status configured';
          statusSpan.textContent = ' ✓ Configured';
          statusSpan.style.color = '#22c55e';
          statusSpan.style.fontWeight = 'normal';
          statusSpan.style.marginLeft = '8px';
          apiKeyLabel.appendChild(statusSpan);
        }
      }
    } catch (error) {
      console.error('Failed to check API key status:', error);
    }
  }

  private showAIStatus(message: string, type: 'success' | 'error' | 'info'): void {
    const statusDiv = document.getElementById('ai-status');
    if (statusDiv) {
      statusDiv.textContent = message;
      statusDiv.className = `ai-status-message ${type}`;
      // Clear after 5 seconds
      setTimeout(() => {
        statusDiv.textContent = '';
        statusDiv.className = 'ai-status-message';
      }, 5000);
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
      state.config.settings.connections.push(newConn);
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
    const conn = state.config?.settings.connections.find(c => c.id === id);
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
      state.config.settings.connections = state.config.settings.connections.filter(
        c => c.id !== id
      );
      if (state.config.settings.active_import_id === id)
        state.config.settings.active_import_id = null;
      if (state.config.settings.active_export_id === id)
        state.config.settings.active_export_id = null;

      await api.deleteConnection(id);
      await api.saveAppConfig(state.config);
      this.actions.onStateChange();
      this.actions.showToast('Connection deleted', 'success');
    }
  }

  private async openLogDirectory(): Promise<void> {
    try {
      const logDir = await api.getLogDirectory();
      const pathDisplay = document.getElementById('log-directory-path');
      if (pathDisplay) {
        pathDisplay.textContent = logDir;
      }
      await api.openPath(logDir);
      this.actions.showToast('Log directory opened', 'success');
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      this.actions.showToast(`Failed to open log directory: ${message}`, 'error');
    }
  }

  private async openLogFile(type: 'main' | 'error'): Promise<void> {
    try {
      const logPath =
        type === 'main' ? await api.getCurrentLogFile() : await api.getCurrentErrorLogFile();
      await api.openPath(logPath);
      this.actions.showToast(`${type === 'main' ? 'Main' : 'Error'} log file opened`, 'success');
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      this.actions.showToast(`Failed to open log file: ${message}`, 'error');
    }
  }

  private async loadLogPath(): Promise<void> {
    try {
      const logDir = await api.getLogDirectory();
      const pathDisplay = document.getElementById('log-directory-path');
      if (pathDisplay) {
        pathDisplay.textContent = logDir;
      }
    } catch (error) {
      console.error('Failed to load log directory path:', error);
    }
  }
}
