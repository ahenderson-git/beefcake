import * as monaco from 'monaco-editor';
// @ts-ignore
import editorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker';
import Chart from 'chart.js/auto';

import {
  AnalysisResponse,
  AppConfig,
  ColumnCleanConfig,
  View
} from './types';

import * as api from './api';
import * as renderers from './renderers';

// @ts-ignore
self.MonacoEnvironment = {
  getWorker() {
    return new editorWorker();
  }
};

class BeefcakeApp {
  private editor: monaco.editor.IStandaloneCodeEditor | null = null;
  private outputEditor: monaco.editor.IStandaloneCodeEditor | null = null;
  private state: {
    currentView: View;
    response: AnalysisResponse | null;
    isLoading: boolean;
    expandedRows: Set<string>;
    trimPct: number;
    config: AppConfig | null;
    columnConfigs: Map<string, ColumnCleanConfig>;
    isAddingConnection: boolean;
  } = {
    currentView: 'Dashboard',
    response: null,
    isLoading: false,
    expandedRows: new Set(),
    trimPct: 0.1,
    config: null,
    columnConfigs: new Map(),
    isAddingConnection: false,
  };

  constructor() {
    this.init();
  }

  async init() {
    this.state.config = await api.loadAppConfig();
    this.renderInitialLayout();
    this.setupNavigation();
    this.render();
  }

  private renderInitialLayout() {
    const app = document.getElementById('app');
    if (app) {
      app.innerHTML = renderers.renderLayout();
    }
  }

  private setupNavigation() {
    document.querySelectorAll('.nav-item').forEach(item => {
      item.addEventListener('click', (e) => {
        const view = (e.currentTarget as HTMLElement).dataset.view as View;
        this.switchView(view);
      });
    });
  }

  private switchView(view: View) {
    this.state.currentView = view;
    this.state.isAddingConnection = false;
    document.querySelectorAll('.nav-item').forEach(item => {
      item.classList.toggle('active', (item as HTMLElement).dataset.view === view);
    });

    const title = document.getElementById('view-title');
    if (title) {
      if (view === 'CLI') title.innerText = 'CLI Help';
      else if (view === 'ActivityLog') title.innerText = 'Activity Log';
      else title.innerText = view;
    }

    this.render();
  }

  private render() {
    const main = document.getElementById('main-content');
    if (!main) return;

    if (this.state.isLoading) {
      main.innerHTML = '<div class="loading"><div class="spinner"></div>Analyzing dataset...</div>';
      return;
    }

    switch (this.state.currentView) {
      case 'Dashboard':
        main.innerHTML = renderers.renderDashboardView(this.state);
        this.bindDashboardEvents();
        break;
      case 'Analyser':
        if (this.state.response) {
          main.innerHTML = renderers.renderAnalyser(this.state.response, this.state.expandedRows, this.state.columnConfigs);
          this.renderAnalyserHeader();
          this.bindAnalyserEvents();
          this.initCharts();
        } else {
          main.innerHTML = renderers.renderEmptyAnalyser();
        }
        break;
      case 'PowerShell':
        main.innerHTML = renderers.renderPowerShellView(this.state.config?.powershell_font_size || 14);
        this.initMonaco();
        this.bindPowerShellEvents();
        break;
      case 'Settings':
        main.innerHTML = renderers.renderSettingsView(this.state.config, this.state.isAddingConnection);
        this.bindSettingsEvents();
        break;
      case 'CLI':
        main.innerHTML = renderers.renderCliHelpView();
        break;
      case 'ActivityLog':
        main.innerHTML = renderers.renderActivityLogView(this.state.config);
        this.bindActivityLogEvents();
        break;
    }
  }

  private bindActivityLogEvents() {
    document.getElementById('btn-clear-log')?.addEventListener('click', async () => {
      if (this.state.config) {
        this.state.config.audit_log = [];
        await api.saveAppConfig(this.state.config);
        this.render();
        this.showToast('Activity log cleared', 'success');
      }
    });
  }

  private renderAnalyserHeader() {
    const container = document.getElementById('analyser-header-container');
    if (container && this.state.response) {
      container.innerHTML = renderers.renderAnalyserHeader(this.state.response, this.state.trimPct);
      this.bindAnalyserHeaderEvents();
    }
  }

  private bindDashboardEvents() {
    document.getElementById('btn-open')?.addEventListener('click', () => this.handleOpenFile());
    document.getElementById('btn-powershell')?.addEventListener('click', () => this.switchView('PowerShell'));
  }

  private bindAnalyserEvents() {
    // Expand/Collapse
    document.querySelectorAll('.name-wrapper').forEach(wrapper => {
      wrapper.addEventListener('click', (e) => {
        const row = (e.currentTarget as HTMLElement).closest('.analyser-row') as HTMLElement;
        const colName = row.dataset.col;
        if (colName) {
          if (this.state.expandedRows.has(colName)) {
            this.state.expandedRows.delete(colName);
          } else {
            this.state.expandedRows.add(colName);
          }
          this.render();
        }
      });
    });

    // Row Actions (Individual Settings)
    document.querySelectorAll('.row-action').forEach(action => {
      action.addEventListener('change', (e) => {
        const el = e.target as HTMLSelectElement | HTMLInputElement;
        const colName = el.dataset.col;
        const prop = el.dataset.prop;
        if (colName && prop) {
          const config = this.state.columnConfigs.get(colName);
          if (config) {
            if (el.type === 'checkbox') {
              (config as any)[prop] = (el as HTMLInputElement).checked;
            } else {
              let value: any = el.value;
              if (prop === 'rounding') {
                value = value === 'none' ? null : parseInt(value);
              }
              (config as any)[prop] = value;
            }
            if (prop === 'active') {
              this.render();
            }
          }
        }
      });
    });

    // Header Actions (Set All)
    document.querySelectorAll('.header-action').forEach(action => {
      action.addEventListener('change', (e) => {
        const el = e.target as HTMLSelectElement | HTMLInputElement;
        const actionType = el.dataset.action;
        let value: any;
        let prop: string;

        if (el.type === 'checkbox') {
          value = (el as HTMLInputElement).checked;
        } else {
          value = el.value;
          if (!value) return; // "Set all..." placeholder
        }

        switch (actionType) {
          case 'active-all':
            prop = 'active';
            break;
          case 'impute-all':
            prop = 'impute_mode';
            break;
          case 'round-all':
            prop = 'rounding';
            value = value === 'none' ? null : parseInt(value);
            break;
          case 'norm-all':
            prop = 'normalization';
            break;
          case 'case-all':
            prop = 'text_case';
            break;
          case 'onehot-all':
            prop = 'one_hot_encode';
            break;
          default:
            return;
        }

        this.state.columnConfigs.forEach(config => {
          (config as any)[prop] = value;
        });
        this.render();
      });
    });
  }
  
  private initCharts() {
    if (!this.state.response) return;

    this.state.expandedRows.forEach(colName => {
      const col = this.state.response?.summary.find(s => s.name === colName);
      if (!col) return;

      const canvas = document.getElementById(`chart-${col.name}`) as HTMLCanvasElement;
      if (!canvas) return;

      let histData: [number, number][] | null = null;
      let labelPrefix = '';

      if (col.stats.Numeric?.histogram) {
        histData = col.stats.Numeric.histogram;
      } else if (col.stats.Temporal?.histogram) {
        histData = col.stats.Temporal.histogram;
        labelPrefix = 'TS: ';
      }

      if (!histData || histData.length === 0) return;

      new Chart(canvas, {
        type: 'bar',
        data: {
          labels: histData.map(d => {
            if (col.stats.Temporal) {
              return new Date(d[0]).toLocaleDateString();
            }
            return d[0].toFixed(2);
          }),
          datasets: [{
            label: 'Frequency',
            data: histData.map(d => d[1]),
            backgroundColor: '#4a90e2',
            borderWidth: 0,
            barPercentage: 1.0,
            categoryPercentage: 1.0
          }]
        },
        options: {
          responsive: true,
          maintainAspectRatio: false,
          plugins: {
            legend: { display: false },
            tooltip: {
              callbacks: {
                label: (context) => `Count: ${context.parsed.y}`
              }
            }
          },
          scales: {
            x: {
              ticks: {
                maxRotation: 0,
                autoSkip: true,
                maxTicksLimit: 10
              },
              grid: { display: false }
            },
            y: {
              beginAtZero: true,
              ticks: { precision: 0 }
            }
          }
        }
      });
    });
  }

  private bindAnalyserHeaderEvents() {
    const range = document.getElementById('trim-range') as HTMLInputElement;
    range?.addEventListener('input', (e) => {
      this.state.trimPct = parseFloat((e.target as HTMLInputElement).value);
      this.renderAnalyserHeader();
    });
    range?.addEventListener('change', () => {
      if (this.state.response) {
        this.handleAnalysis(this.state.response.file_path);
      }
    });

    document.getElementById('btn-push')?.addEventListener('click', () => this.handlePushToDb());
  }

  private bindPowerShellEvents() {
    document.getElementById('btn-run-ps')?.addEventListener('click', () => this.runPowerShell());
    document.getElementById('btn-load-ps')?.addEventListener('click', () => this.handleLoadScript());
    document.getElementById('btn-save-ps')?.addEventListener('click', () => this.handleSaveScript());
    document.getElementById('btn-inc-font')?.addEventListener('click', () => this.updateFontSize(1));
    document.getElementById('btn-dec-font')?.addEventListener('click', () => this.updateFontSize(-1));
  }

  private async updateFontSize(delta: number) {
    if (this.state.config) {
      this.state.config.powershell_font_size = Math.max(8, Math.min(72, this.state.config.powershell_font_size + delta));
      
      this.editor?.updateOptions({ fontSize: this.state.config.powershell_font_size });
      this.outputEditor?.updateOptions({ fontSize: this.state.config.powershell_font_size });
      
      const label = document.getElementById('ps-font-size-label');
      if (label) label.innerText = String(this.state.config.powershell_font_size);
      
      await api.saveAppConfig(this.state.config);
    }
  }

  private bindSettingsEvents() {
    const settingsView = document.querySelector('.settings-view');
    if (!settingsView) return;

    settingsView.addEventListener('click', (e) => {
      const target = e.target as HTMLElement;
      const btn = target.closest('button');
      if (!btn) return;

      e.preventDefault();

      if (btn.id === 'btn-add-conn') {
        this.addConnection();
      } else if (btn.id === 'btn-save-new-conn') {
        this.saveNewConnection();
      } else if (btn.id === 'btn-test-new-conn') {
        this.testNewConnection(btn);
      } else if (btn.id === 'btn-cancel-new-conn') {
        this.cancelAddConnection();
      } else if (btn.classList.contains('btn-delete-conn')) {
        const id = btn.dataset.id;
        if (id) this.deleteConnection(id);
      } else if (btn.classList.contains('btn-test-conn')) {
        const id = btn.dataset.id;
        if (id) this.handleTestConnection(id, btn);
      }
    });

    document.getElementById('select-import-id')?.addEventListener('change', async (e) => {
      if (this.state.config) {
        this.state.config.active_import_id = (e.target as HTMLSelectElement).value || null;
        await api.saveAppConfig(this.state.config);
        this.showToast('Default import connection updated', 'success');
      }
    });

    document.getElementById('select-export-id')?.addEventListener('change', async (e) => {
      if (this.state.config) {
        this.state.config.active_export_id = (e.target as HTMLSelectElement).value || null;
        await api.saveAppConfig(this.state.config);
        this.showToast('Default export connection updated', 'success');
      }
    });
  }

  private async handleOpenFile() {
    const path = await api.openFileDialog();
    if (path) {
      this.handleAnalysis(path);
    }
  }

  private async handleAnalysis(path: string) {
    this.state.isLoading = true;
    this.render();
    try {
      const response = await api.analyseFile(path, this.state.trimPct);
      this.state.response = response;
      this.state.columnConfigs.clear();
      response.summary.forEach(col => {
        this.state.columnConfigs.set(col.name, {
          new_name: col.name,
          target_dtype: null,
          active: true,
          advanced_cleaning: true,
          ml_preprocessing: false,
          trim_whitespace: false,
          remove_special_chars: false,
          text_case: "None",
          standardize_nulls: false,
          remove_non_ascii: false,
          regex_find: "",
          regex_replace: "",
          rounding: null,
          extract_numbers: false,
          clip_outliers: false,
          temporal_format: "",
          timezone_utc: false,
          freq_threshold: null,
          normalization: "None",
          one_hot_encode: false,
          impute_mode: "None",
        });
      });
      this.state.currentView = 'Analyser';
      this.showToast(`Analysis complete: ${this.state.response.file_name}`, 'success');
    } catch (err) {
      this.showToast(String(err), 'error');
    } finally {
      this.state.isLoading = false;
      this.render();
    }
  }

  private async handlePushToDb() {
    if (!this.state.response || !this.state.config) return;

    if (this.state.config.connections.length === 0) {
      this.showToast('No database connections configured. Please go to Settings.', 'error');
      return;
    }

    const connId = this.state.config.active_import_id || this.state.config.connections[0].id;

    try {
      this.showToast('Pushing data to PostgreSQL...', 'info');
      const configsObj = Object.fromEntries(this.state.columnConfigs);
      await api.pushToDb(this.state.response.file_path, connId, configsObj);
      this.showToast('Data successfully pushed to database', 'success');
    } catch (err) {
      this.showToast(String(err), 'error');
    }
  }

  private initMonaco() {
    const editorEl = document.getElementById('ps-editor');
    const outputEl = document.getElementById('ps-output');
    const fontSize = this.state.config?.powershell_font_size || 14;

    if (editorEl) {
      const value = this.editor ? this.editor.getValue() : '# PowerShell Script\nGet-Process | Select-Object -First 10';
      if (this.editor) this.editor.dispose();
      
      this.editor = monaco.editor.create(editorEl, {
        value,
        language: 'powershell',
        theme: 'vs-dark',
        automaticLayout: true,
        minimap: { enabled: false },
        fontSize: fontSize
      });
    }

    if (outputEl) {
      const value = this.outputEditor ? this.outputEditor.getValue() : '';
      if (this.outputEditor) this.outputEditor.dispose();

      this.outputEditor = monaco.editor.create(outputEl, {
        value,
        language: 'text',
        theme: 'vs-dark',
        readOnly: true,
        automaticLayout: true,
        minimap: { enabled: false },
        fontSize: fontSize
      });
    }
  }

  private async runPowerShell() {
    const script = this.editor?.getValue();
    if (!script) return;

    try {
      this.outputEditor?.setValue('Running...');
      const result = await api.runPowerShell(script);
      this.outputEditor?.setValue(result);
    } catch (err) {
      this.outputEditor?.setValue(`Error: ${err}`);
    }
  }

  private async handleLoadScript() {
    try {
      const path = await api.openFileDialog([{ name: 'PowerShell Scripts', extensions: ['ps1'] }]);
      if (path) {
        const content = await api.readTextFile(path);
        this.editor?.setValue(content);
        this.showToast('Script loaded successfully', 'success');
      }
    } catch (err) {
      this.showToast(`Failed to load script: ${err}`, 'error');
    }
  }

  private async handleSaveScript() {
    const script = this.editor?.getValue();
    if (!script) return;

    try {
      const path = await api.saveFileDialog([{ name: 'PowerShell Scripts', extensions: ['ps1'] }]);
      if (path) {
        await api.writeTextFile(path, script);
        this.showToast('Script saved successfully', 'success');
      }
    } catch (err) {
      this.showToast(`Failed to save script: ${err}`, 'error');
    }
  }

  private async addConnection() {
    this.state.isAddingConnection = true;
    this.render();
  }

  private cancelAddConnection() {
    this.state.isAddingConnection = false;
    this.render();
  }

  private async saveNewConnection() {
    const name = (document.getElementById('new-conn-name') as HTMLInputElement)?.value;
    if (!name) {
      this.showToast('Connection name is required', 'error');
      return;
    }

    const host = (document.getElementById('new-conn-host') as HTMLInputElement)?.value || 'localhost';
    const port = (document.getElementById('new-conn-port') as HTMLInputElement)?.value || '5432';
    const user = (document.getElementById('new-conn-user') as HTMLInputElement)?.value || 'postgres';
    const password = (document.getElementById('new-conn-pass') as HTMLInputElement)?.value || '';
    const database = (document.getElementById('new-conn-db') as HTMLInputElement)?.value || '';

    const newConn = {
      id: crypto.randomUUID(),
      name,
      settings: {
        db_type: 'postgres',
        host,
        port,
        user,
        password,
        database,
        schema: 'public',
        table: ''
      }
    };

    if (this.state.config) {
      this.state.config.connections.push(newConn);
      await api.saveAppConfig(this.state.config);
      this.state.isAddingConnection = false;
      this.render();
      this.showToast('Connection added', 'success');
    }
  }

  private async testNewConnection(btnElement?: HTMLButtonElement) {
    const host = (document.getElementById('new-conn-host') as HTMLInputElement)?.value || 'localhost';
    const port = (document.getElementById('new-conn-port') as HTMLInputElement)?.value || '5432';
    const user = (document.getElementById('new-conn-user') as HTMLInputElement)?.value || 'postgres';
    const password = (document.getElementById('new-conn-pass') as HTMLInputElement)?.value || '';
    const database = (document.getElementById('new-conn-db') as HTMLInputElement)?.value || '';

    const settings = {
      db_type: 'postgres',
      host,
      port,
      user,
      password,
      database,
      schema: 'public',
      table: ''
    };

    const btn = btnElement || document.getElementById('btn-test-new-conn') as HTMLButtonElement;
    const icon = btn?.querySelector('i');
    
    if (btn && icon) {
      btn.disabled = true;
      icon.className = 'ph ph-circle-notch loading-spin';
    }

    try {
      this.showToast('Testing connection...', 'info');
      const result = await api.testConnection(settings);
      this.showToast(result, 'success');
    } catch (err) {
      this.showToast(String(err), 'error');
    } finally {
      if (btn && icon) {
        btn.disabled = false;
        icon.className = 'ph ph-plugs-connected';
      }
    }
  }

  private async handleTestConnection(id: string, btnElement?: HTMLButtonElement) {
    const conn = this.state.config?.connections.find(c => c.id === id);
    if (!conn) {
      this.showToast('Connection not found in configuration', 'error');
      return;
    }

    const btn = btnElement || document.querySelector(`.btn-test-conn[data-id="${id}"]`) as HTMLButtonElement;
    const icon = btn?.querySelector('i');

    if (btn && icon) {
      btn.disabled = true;
      icon.className = 'ph ph-circle-notch loading-spin';
    }

    try {
      this.showToast(`Testing connection "${conn.name}"...`, 'info');
      const result = await api.testConnection(conn.settings);
      this.showToast(result, 'success');
    } catch (err) {
      this.showToast(String(err), 'error');
    } finally {
      if (btn && icon) {
        btn.disabled = false;
        icon.className = 'ph ph-plugs-connected';
      }
    }
  }

  private async deleteConnection(id: string) {
    if (this.state.config) {
      if (!confirm('Are you sure you want to delete this connection?')) return;
      
      this.state.config.connections = this.state.config.connections.filter(c => c.id !== id);
      
      // Clear active IDs if they were deleted
      if (this.state.config.active_import_id === id) this.state.config.active_import_id = null;
      if (this.state.config.active_export_id === id) this.state.config.active_export_id = null;
      
      await api.saveAppConfig(this.state.config);
      this.render();
      this.showToast('Connection deleted', 'success');
    }
  }

  private showToast(message: string, type: 'info' | 'error' | 'success' = 'info') {
    const container = document.getElementById('toast-container');
    if (!container) return;

    const toast = document.createElement('div');
    toast.innerHTML = renderers.renderToast(message, type);
    const toastEl = toast.firstElementChild!;
    container.appendChild(toastEl);

    setTimeout(() => {
      toastEl.classList.add('fade-out');
      setTimeout(() => toastEl.remove(), 500);
    }, 3000);
  }
}

window.addEventListener('DOMContentLoaded', () => {
  new BeefcakeApp();
});
