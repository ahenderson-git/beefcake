import { Component, ComponentActions } from "./Component";
import { AppState, ExportSource } from "../types";
import * as renderers from "../renderers";
import * as api from "../api";
import * as monaco from 'monaco-editor';
import { ExportModal } from "./ExportModal";
import { AnsiUp } from 'ansi_up';

const DEFAULT_PYTHON_SCRIPT = `# Python script
import os
import polars as pl

# Note: Beefcake passes the currently analyzed dataset as BEEFCAKE_DATA_PATH.
# For large datasets, use "scan_*" (Lazy) instead of "read_*" to avoid OOM crashes.

data_path = os.environ.get("BEEFCAKE_DATA_PATH")
if data_path:
    print(f"Loading data from: {data_path}")
    # Handle both raw files (CSV/JSON) and prepared files (Parquet)
    if data_path.endswith(".parquet"):
        df = pl.scan_parquet(data_path)
    elif data_path.endswith(".json"):
        df = pl.read_json(data_path).lazy()
    else:
        df = pl.scan_csv(data_path, try_parse_dates=True)

    if df is not None:
        print("Lazy Dataset initialized successfully!")
        # Use .collect() only when you need the actual data (e.g. for preview or final export)
        # For 10M+ rows, ALWAYS use .limit() or .sample() before .collect() for previews.
        preview = df.head(10).collect()
        print(f"Schema: {df.schema}")
        print(preview)

        # 🎨 TIP: For fancy colored output, install 'rich':
        #   pip install rich
        # Then use:
        #   from rich.console import Console
        #   console = Console(force_terminal=True)
        #   console.print(df, style="bold cyan")
        #   console.print("[green]✓[/green] Success!")
else:
    print("No dataset loaded in Beefcake.")
    print("Tip: Load a file in the Analyser first.")`;

export class PythonComponent extends Component {
  private editor: monaco.editor.IStandaloneCodeEditor | null = null;
  private ansiConverter: AnsiUp;

  constructor(containerId: string, actions: ComponentActions) {
    super(containerId, actions);
    this.ansiConverter = new AnsiUp();
    // Configure ANSI converter for better output
    this.ansiConverter.use_classes = false; // Use inline styles for portability
  }

  render(state: AppState): void {
    const container = this.getContainer();
    container.innerHTML = renderers.renderPythonView(state);
    // Use setTimeout to ensure DOM is fully ready before initializing Monaco
    setTimeout(() => {
      this.initMonaco(state);
      this.bindEvents(state);
      this.bindSidebarEvents();
    }, 0);
  }

  private initMonaco(state: AppState) {
    const editorContainer = document.getElementById('py-editor');
    if (!editorContainer) {
      console.error('PythonComponent: py-editor container not found');
      return;
    }

    // Dispose old editor if it exists to prevent memory leaks
    if (this.editor) {
      try {
        this.editor.dispose();
      } catch (e) {
        console.warn('Failed to dispose Monaco editor:', e);
      }
      this.editor = null;
    }

    try {
      // Create new editor instance
      this.editor = monaco.editor.create(editorContainer, {
        value: state.pythonScript || DEFAULT_PYTHON_SCRIPT,
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

      console.log('Monaco editor created successfully');
    } catch (e) {
      console.error('Failed to create Monaco editor:', e);
    }
  }

  override bindEvents(state: AppState): void {
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
    document.getElementById('btn-copy-output-py')?.addEventListener('click', () => this.handleCopyOutput());
    document.getElementById('py-skip-cleaning')?.addEventListener('change', (e) => {
      state.pythonSkipCleaning = (e.target as HTMLInputElement).checked;
    });
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
      // Use cleaning configs only if skip cleaning is disabled
      const configs = state.pythonSkipCleaning ? undefined : state.cleaningConfigs;
      const result = await api.runPython(script, dataPath, configs);
      // Convert ANSI escape codes to HTML for colored output
      output.innerHTML = this.ansiConverter.ansi_to_html(result);
    } catch (err) {
      let errorMsg = String(err);
      if (errorMsg.includes("ModuleNotFoundError: No module named 'polars'")) {
        errorMsg += "\n\nTip: Click the 'Install Polars' button in the toolbar to install the required library.";
      }
      // Also convert errors to HTML (they might have ANSI codes too)
      output.innerHTML = this.ansiConverter.ansi_to_html(errorMsg);
    }
  }

  private async handleExport(state: AppState) {
    if (!this.editor) return;

    const source: ExportSource = {
      type: 'Python',
      content: this.editor.getValue(),
    };
    if (state.analysisResponse?.path) {
      source.path = state.analysisResponse.path;
    }

    const modal = new ExportModal('modal-container', this.actions, source);

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

  private async handleCopyOutput() {
    const output = document.getElementById('py-output');
    if (!output) return;

    try {
      // Get plain text content (strips HTML/ANSI formatting)
      const text = output.textContent || '';

      if (!text || text === 'Running script...') {
        this.actions.showToast('No output to copy', 'info');
        return;
      }

      await navigator.clipboard.writeText(text);
      this.actions.showToast('Output copied to clipboard', 'success');
    } catch (err) {
      this.actions.showToast(`Failed to copy: ${err}`, 'error');
    }
  }
}
