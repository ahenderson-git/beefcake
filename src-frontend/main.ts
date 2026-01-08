import * as monaco from 'monaco-editor';
// @ts-ignore
import editorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker';

import {
  AnalysisResponse,
  AppConfig,
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
  } = {
    currentView: 'Dashboard',
    response: null,
    isLoading: false,
    expandedRows: new Set(),
    trimPct: 0.1,
    config: null,
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
    document.querySelectorAll('.nav-item').forEach(item => {
      item.classList.toggle('active', (item as HTMLElement).dataset.view === view);
    });

    const title = document.getElementById('view-title');
    if (title) {
      if (view === 'CLI') title.innerText = 'CLI Help';
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
          main.innerHTML = renderers.renderAnalyser(this.state.response, this.state.expandedRows);
          this.renderAnalyserHeader();
          this.bindAnalyserEvents();
        } else {
          main.innerHTML = renderers.renderEmptyAnalyser();
        }
        break;
      case 'PowerShell':
        main.innerHTML = renderers.renderPowerShellView();
        this.initMonaco();
        this.bindPowerShellEvents();
        break;
      case 'Settings':
        main.innerHTML = renderers.renderSettingsView(this.state.config);
        this.bindSettingsEvents();
        break;
      case 'CLI':
        main.innerHTML = renderers.renderCliHelpView();
        break;
    }
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
    document.querySelectorAll('.summary-card').forEach(card => {
      card.addEventListener('click', (e) => {
        const colName = (e.currentTarget as HTMLElement).dataset.col;
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
  }

  private bindSettingsEvents() {
    document.getElementById('btn-add-conn')?.addEventListener('click', () => this.addConnection());
    document.querySelectorAll('.btn-delete-conn').forEach(btn => {
      btn.addEventListener('click', (e) => {
        const id = (e.currentTarget as HTMLElement).dataset.id;
        if (id) this.deleteConnection(id);
      });
    });

    document.getElementById('select-import-id')?.addEventListener('change', (e) => {
      if (this.state.config) {
        this.state.config.active_import_id = (e.target as HTMLSelectElement).value || null;
        api.saveAppConfig(this.state.config);
        this.showToast('Default import connection updated', 'success');
      }
    });

    document.getElementById('select-export-id')?.addEventListener('change', (e) => {
      if (this.state.config) {
        this.state.config.active_export_id = (e.target as HTMLSelectElement).value || null;
        api.saveAppConfig(this.state.config);
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
      this.state.response = await api.analyseFile(path, this.state.trimPct);
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
      await api.pushToDb(this.state.response.file_path, connId);
      this.showToast('Data successfully pushed to database', 'success');
    } catch (err) {
      this.showToast(String(err), 'error');
    }
  }

  private initMonaco() {
    const editorEl = document.getElementById('ps-editor');
    const outputEl = document.getElementById('ps-output');

    if (editorEl) {
      const value = this.editor ? this.editor.getValue() : '# PowerShell Script\nGet-Process | Select-Object -First 10';
      if (this.editor) this.editor.dispose();
      
      this.editor = monaco.editor.create(editorEl, {
        value,
        language: 'powershell',
        theme: 'vs-dark',
        automaticLayout: true,
        minimap: { enabled: false }
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
        minimap: { enabled: false }
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
    const name = prompt('Connection Name:');
    if (!name) return;

    const host = prompt('Host:', 'localhost') || 'localhost';
    const port = prompt('Port:', '5432') || '5432';
    const user = prompt('User:', 'postgres') || 'postgres';
    const password = prompt('Password:') || '';
    const database = prompt('Database:') || '';

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
      this.render();
      this.showToast('Connection added', 'success');
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
