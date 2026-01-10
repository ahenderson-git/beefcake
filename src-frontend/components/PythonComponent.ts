import { Component, ComponentActions } from "./Component";
import { AppState } from "../types";
import * as renderers from "../renderers";
import * as api from "../api";
import * as monaco from 'monaco-editor';
import { ExportModal } from "./ExportModal";

export class PythonComponent extends Component {
  private editor: monaco.editor.IStandaloneCodeEditor | null = null;

  constructor(containerId: string, actions: ComponentActions) {
    super(containerId, actions);
  }

  render(state: AppState): void {
    this.container.innerHTML = renderers.renderPythonView(state);
    this.initMonaco(state);
    this.bindEvents(state);
    this.bindSidebarEvents();
  }

  private initMonaco(state: AppState) {
    const editorContainer = document.getElementById('py-editor');
    if (editorContainer) {
      const defaultValue = '# Python script\n' +
        'import os\n' +
        'import polars as pl\n\n' +
        '# Note: Beefcake passes the currently analyzed dataset as BEEFCAKE_DATA_PATH.\n' +
        '# For large datasets, use "scan_*" (Lazy) instead of "read_*" to avoid OOM crashes.\n\n' +
        'data_path = os.environ.get("BEEFCAKE_DATA_PATH")\n' +
        'if data_path:\n' +
        '    print(f"Loading data from: {data_path}")\n' +
        '    # Handle both raw files (CSV/JSON) and prepared files (Parquet)\n' +
        '    if data_path.endswith(".parquet"):\n' +
        '        df = pl.scan_parquet(data_path)\n' +
        '    elif data_path.endswith(".json"):\n' +
        '        df = pl.read_json(data_path).lazy()\n' +
        '    else:\n' +
        '        df = pl.scan_csv(data_path, try_parse_dates=True)\n\n' +
        '    if df is not None:\n' +
        '        print("Lazy Dataset initialized successfully!")\n' +
        '        # Use .collect() only when you need the actual data (e.g. for preview or final export)\n' +
        '        # For 10M+ rows, ALWAYS use .limit() or .sample() before .collect() for previews.\n' +
        '        preview = df.head(10).collect()\n' +
        '        print(f"Schema: {df.schema}")\n' +
        '        print(preview)\n' +
        'else:\n' +
        '    print("No dataset loaded in Beefcake.")\n' +
        '    print("Tip: Load a file in the Analyser first.")';

      this.editor = monaco.editor.create(editorContainer, {
        value: state.pythonScript || defaultValue,
        language: 'python',
        theme: 'vs-dark',
        automaticLayout: true,
        fontSize: state.config?.python_font_size || 14,
        fontFamily: "'Fira Code', monospace",
        fontLigatures: true,
        minimap: { enabled: false }
      });

      this.editor.onDidChangeModelContent(() => {
        state.pythonScript = this.editor?.getValue() || null;
      });
    }
  }

  bindEvents(state: AppState): void {
    document.getElementById('btn-run-py')?.addEventListener('click', () => this.runPython(state));
    document.getElementById('btn-clear-py')?.addEventListener('click', () => {
      const output = document.getElementById('py-output');
      if (output) output.textContent = '';
    });
    document.getElementById('btn-export-py')?.addEventListener('click', () => this.handleExport(state));
    document.getElementById('btn-inc-font-py')?.addEventListener('click', () => this.updateFontSize(state, 1));
    document.getElementById('btn-dec-font-py')?.addEventListener('click', () => this.updateFontSize(state, -1));
    document.getElementById('btn-load-py')?.addEventListener('click', () => this.handleLoadScript());
    document.getElementById('btn-save-py')?.addEventListener('click', () => this.handleSaveScript());
    document.getElementById('btn-install-polars')?.addEventListener('click', () => this.handleInstallPolars());
  }

  private bindSidebarEvents() {
    document.querySelectorAll('.btn-insert-col').forEach(btn => {
      btn.addEventListener('click', (e) => {
        const colName = (e.currentTarget as HTMLElement).dataset.col;
        if (colName) {
          this.insertColumnName(colName);
        }
      });
    });
  }

  private insertColumnName(colName: string) {
    if (!this.editor) return;

    const selection = this.editor.getSelection();
    if (!selection) return;

    const range = new monaco.Range(
      selection.startLineNumber,
      selection.startColumn,
      selection.endLineNumber,
      selection.endColumn
    );

    // In Python, we might want it quoted if it contains spaces or just as a string
    // But usually in code we just want the name. Let's provide it as a string literal.
    const text = `"${colName}"`;
    
    this.editor.executeEdits('insert-column', [
      {
        range: range,
        text: text,
        forceMoveMarkers: true
      }
    ]);
    
    this.editor.focus();
  }

  private async handleInstallPolars() {
    const output = document.getElementById('py-output');
    if (!output) return;

    output.textContent = 'Installing polars... (this may take a minute)';
    try {
      output.textContent = await api.installPythonPackage('polars');
      this.actions.showToast('Polars installed successfully', 'success');
    } catch (err) {
      output.textContent = `Error installing polars: ${err}`;
      this.actions.showToast('Failed to install polars', 'error');
    }
  }

  private async runPython(state: AppState) {
    if (!this.editor) return;
    const script = this.editor.getValue();
    const output = document.getElementById('py-output');
    if (!output) return;

    if (!state.analysisResponse) {
      output.textContent = 'Error: No dataset loaded in Beefcake Analyser.\nPlease go to Dashboard or Analyser to load a file first.';
      this.actions.showToast('No dataset loaded', 'error');
      return;
    }

    const dataPath = state.analysisResponse.path;
    if (!dataPath) {
      output.textContent = 'Error: Dataset path is missing from analysis response.\nThis might be a bug. Try re-loading the file in the Analyser.';
      this.actions.showToast('Dataset path missing', 'error');
      return;
    }

    output.textContent = 'Running script...';
    try {
      output.innerHTML = await api.runPython(script, dataPath, state.cleaningConfigs);
    } catch (err) {
      let errorMsg = String(err);
      if (errorMsg.includes("ModuleNotFoundError: No module named 'polars'")) {
        errorMsg += "\n\nTip: Click the 'Install Polars' button in the toolbar to install the required library.";
      }
      output.textContent = errorMsg;
    }
  }

  private async handleExport(state: AppState) {
    if (!this.editor) return;

    const modal = new ExportModal('modal-container', this.actions, {
      type: 'Python',
      content: this.editor.getValue(),
      path: state.analysisResponse?.path
    });

    document.getElementById('modal-container')?.classList.add('active');
    await modal.show(state);
    document.getElementById('modal-container')?.classList.remove('active');
  }

  private async updateFontSize(state: AppState, delta: number) {
    if (state.config) {
      state.config.python_font_size = Math.max(8, Math.min(32, state.config.python_font_size + delta));
      this.editor?.updateOptions({ fontSize: state.config.python_font_size });
      
      const label = document.getElementById('py-font-size-label');
      if (label) label.textContent = state.config.python_font_size.toString();

      await api.saveAppConfig(state.config);
      this.actions.onStateChange();
    }
  }

  private async handleLoadScript() {
    try {
      const path = await api.openFileDialog([{ name: 'Python', extensions: ['py'] }]);
      if (path) {
        const content = await api.readTextFile(path);
        this.editor?.setValue(content);
        this.actions.showToast('Script loaded', 'success');
      }
    } catch (err) {
      this.actions.showToast(`Error loading script: ${err}`, 'error');
    }
  }

  private async handleSaveScript() {
    try {
      const path = await api.saveFileDialog([{ name: 'Python', extensions: ['py'] }]);
      if (path) {
        const content = this.editor?.getValue() || '';
        await api.writeTextFile(path, content);
        this.actions.showToast('Script saved', 'success');
      }
    } catch (err) {
      this.actions.showToast(`Error saving script: ${err}`, 'error');
    }
  }
}
