import * as monaco from 'monaco-editor';

import * as api from '../api';
import * as renderers from '../renderers';
import { AppState } from '../types';

import { Component, ComponentActions } from './Component';
import { ExportModal } from './ExportModal';

export class SQLComponent extends Component {
  private editor: monaco.editor.IStandaloneCodeEditor | null = null;

  constructor(containerId: string, actions: ComponentActions) {
    super(containerId, actions);
  }

  render(state: AppState): void {
    const container = this.getContainer();
    container.innerHTML = renderers.renderSQLView(state);
    this.initMonaco(state);
    this.bindEvents(state);
    this.bindSidebarEvents();
  }

  private initMonaco(state: AppState): void {
    const editorContainer = document.getElementById('sql-editor');
    if (editorContainer) {
      // noinspection SqlNoDataSourceInspection
      const defaultValue =
        '-- SQL Query\n' +
        '-- The loaded dataset is automatically registered as the table "data"\n' +
        'SELECT * FROM data LIMIT 10;';

      this.editor = monaco.editor.create(editorContainer, {
        value: state.sqlScript ?? defaultValue,
        language: 'sql',
        theme: 'vs-dark',
        automaticLayout: true,
        fontSize: state.config?.sql_font_size ?? 14,
        fontFamily: "'Fira Code', monospace",
        fontLigatures: true,
        minimap: { enabled: false },
      });

      this.editor.onDidChangeModelContent(() => {
        state.sqlScript = this.editor?.getValue() ?? null;
      });
    }
  }

  override bindEvents(state: AppState): void {
    document.getElementById('btn-run-sql')?.addEventListener('click', () => {
      void this.runSql(state);
    });
    document.getElementById('btn-clear-sql')?.addEventListener('click', () => {
      const output = document.getElementById('sql-output');
      if (output) output.textContent = '';
    });
    document.getElementById('btn-export-sql')?.addEventListener('click', () => {
      void this.handleExport(state);
    });
    document.getElementById('btn-inc-font-sql')?.addEventListener('click', () => {
      void this.updateFontSize(state, 1);
    });
    document.getElementById('btn-dec-font-sql')?.addEventListener('click', () => {
      void this.updateFontSize(state, -1);
    });
    document.getElementById('btn-load-sql')?.addEventListener('click', () => {
      void this.handleLoadScript();
    });
    document.getElementById('btn-save-sql')?.addEventListener('click', () => {
      void this.handleSaveScript();
    });
    document.getElementById('btn-install-polars')?.addEventListener('click', () => {
      void this.handleInstallPolars();
    });
    document.getElementById('btn-sql-docs')?.addEventListener('click', () => {
      window.open('https://docs.pola.rs/user-guide/sql/intro/', '_blank');
    });
    document.getElementById('btn-copy-output-sql')?.addEventListener('click', () => {
      this.handleCopyOutput();
    });
    document.getElementById('sql-skip-cleaning')?.addEventListener('change', e => {
      state.sqlSkipCleaning = (e.target as HTMLInputElement).checked;
      this.actions.onStateChange();
    });
  }

  private bindSidebarEvents(): void {
    document.querySelectorAll('.btn-insert-col').forEach(btn => {
      btn.addEventListener('click', e => {
        const colName = (e.currentTarget as HTMLElement).dataset.col;
        if (colName) {
          this.insertColumnName(colName);
        }
      });
    });
  }

  private insertColumnName(colName: string): void {
    if (!this.editor) return;

    const selection = this.editor.getSelection();
    if (!selection) return;

    const range = new monaco.Range(
      selection.startLineNumber,
      selection.startColumn,
      selection.endLineNumber,
      selection.endColumn
    );

    // In SQL, double quotes are used for identifiers if they have spaces
    const text = colName.includes(' ') ? `"${colName}"` : colName;

    this.editor.executeEdits('insert-column', [
      {
        range: range,
        text: text,
        forceMoveMarkers: true,
      },
    ]);

    this.editor.focus();
  }

  private async runSql(state: AppState): Promise<void> {
    if (!this.editor) return;
    const query = this.editor.getValue();
    const output = document.getElementById('sql-output');
    if (!output) return;

    if (!state.analysisResponse) {
      output.textContent =
        'Error: No dataset loaded in Beefcake Analyser.\nPlease go to Dashboard or Analyser to load a file first.';
      this.actions.showToast('No dataset loaded', 'error');
      return;
    }

    const dataPath = state.analysisResponse.path;
    if (!dataPath) {
      output.textContent =
        'Error: Dataset path is missing from analysis response.\nTry re-loading the file in the Analyser.';
      this.actions.showToast('Dataset path missing', 'error');
      return;
    }

    output.textContent = 'Executing query...';
    try {
      const configs = state.sqlSkipCleaning ? undefined : state.cleaningConfigs;
      output.textContent = await api.runSql(query, dataPath, configs);
    } catch (err) {
      let errorMsg = String(err);
      if (errorMsg.includes("ModuleNotFoundError: No module named 'polars'")) {
        errorMsg +=
          "\n\nTip: Click the 'Install Polars' button in the SQL IDE toolbar to install the required library.";
      }
      output.textContent = errorMsg;
    }
  }

  private async handleInstallPolars(): Promise<void> {
    const output = document.getElementById('sql-output');
    if (!output) return;

    output.textContent = 'Installing polars package...\n';
    try {
      output.textContent = await api.installPythonPackage('polars');
      this.actions.showToast('Polars installed successfully', 'success');
    } catch (err) {
      output.textContent = `Installation failed:\n${String(err)}`;
      this.actions.showToast('Installation failed', 'error');
    }
  }

  private handleCopyOutput(): void {
    const output = document.getElementById('sql-output');
    if (!output) return;

    const text = output.textContent ?? '';
    void navigator.clipboard.writeText(text).then(
      () => {
        this.actions.showToast('Output copied to clipboard', 'success');
      },
      () => {
        this.actions.showToast('Failed to copy output', 'error');
      }
    );
  }

  private async handleExport(state: AppState): Promise<void> {
    if (!this.editor || !state.analysisResponse) return;

    const modal = new ExportModal('modal-container', this.actions, {
      type: 'SQL',
      content: this.editor.getValue(),
      path: state.analysisResponse.path,
    });

    document.getElementById('modal-container')?.classList.add('active');
    await modal.show(state);
    document.getElementById('modal-container')?.classList.remove('active');
  }

  private async updateFontSize(state: AppState, delta: number): Promise<void> {
    if (state.config) {
      state.config.sql_font_size = Math.max(
        8,
        Math.min(32, (state.config.sql_font_size ?? 14) + delta)
      );
      this.editor?.updateOptions({ fontSize: state.config.sql_font_size });

      const label = document.getElementById('sql-font-size-label');
      if (label) label.textContent = state.config.sql_font_size.toString();

      await api.saveAppConfig(state.config);
      this.actions.onStateChange();
    }
  }

  private async handleLoadScript(): Promise<void> {
    try {
      const path = await api.openFileDialog([{ name: 'SQL', extensions: ['sql'] }]);
      if (path) {
        const content = await api.readTextFile(path);
        this.editor?.setValue(content);
        this.actions.showToast('Query loaded', 'success');
      }
    } catch (err) {
      this.actions.showToast(`Error loading query: ${String(err)}`, 'error');
    }
  }

  private async handleSaveScript(): Promise<void> {
    try {
      const path = await api.saveFileDialog([{ name: 'SQL', extensions: ['sql'] }]);
      if (path) {
        const content = this.editor?.getValue() ?? '';
        await api.writeTextFile(path, content);
        this.actions.showToast('Query saved', 'success');
      }
    } catch (err) {
      this.actions.showToast(`Error saving query: ${String(err)}`, 'error');
    }
  }
}
