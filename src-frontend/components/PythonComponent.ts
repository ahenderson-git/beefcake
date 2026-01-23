import { AnsiUp } from 'ansi_up';
import DOMPurify from 'dompurify';
import * as monaco from 'monaco-editor';

import * as api from '../api';
import * as renderers from '../renderers';
import { AppState, ExportSource } from '../types';
import { getDataPathForExecution } from '../utils';

import { Component, ComponentActions } from './Component';
import { ExportModal } from './ExportModal';

const DEFAULT_PYTHON_SCRIPT = `# Python script
import os
import polars as pl

# Note: Beefcake passes the currently analysed dataset as BEEFCAKE_DATA_PATH.
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
      void this.updateColumnSchema(state);
    }, 0);
  }

  private async updateColumnSchema(state: AppState): Promise<void> {
    // If we have a dataset with versions, fetch the schema for the selected version
    if (state.currentDataset && state.currentDataset.versions.length > 0) {
      const selectedVersionId = state.selectedVersionId ?? state.currentDataset.activeVersionId;
      try {
        state.currentIdeColumns = await api.getVersionSchema(
          state.currentDataset.id,
          selectedVersionId
        );
      } catch (err) {
        console.error('Failed to fetch version schema:', err);
        // Fall back to analysisResponse columns if API fails
        state.currentIdeColumns = null;
      }
    } else {
      // No dataset versions, use analysisResponse
      state.currentIdeColumns = null;
    }
  }

  private initMonaco(state: AppState): void {
    const editorContainer = document.getElementById('py-editor');
    if (!editorContainer) {
      return;
    }

    // Dispose old editor if it exists to prevent memory leaks
    if (this.editor) {
      try {
        this.editor.dispose();
      } catch (e) {
        // Ignore dispose errors
      }
      this.editor = null;
    }

    try {
      // Create new editor instance
      this.editor = monaco.editor.create(editorContainer, {
        value: state.pythonScript ?? DEFAULT_PYTHON_SCRIPT,
        language: 'python',
        theme: 'vs-dark',
        automaticLayout: true,
        fontSize: state.config?.python_font_size ?? 14,
        fontFamily: "'Fira Code', monospace",
        fontLigatures: true,
        minimap: { enabled: false },
      });

      this.editor.onDidChangeModelContent(() => {
        state.pythonScript = this.editor?.getValue() ?? null;
      });
    } catch (e) {
      // Failed to create editor
    }
  }

  override bindEvents(state: AppState): void {
    document.getElementById('btn-run-py')?.addEventListener('click', () => {
      void this.runPython(state);
    });
    document.getElementById('btn-clear-py')?.addEventListener('click', () => {
      const output = document.getElementById('py-output');
      if (output) output.textContent = '';
    });
    document.getElementById('btn-export-py')?.addEventListener('click', () => {
      void this.handleExport(state);
    });
    document.getElementById('btn-inc-font-py')?.addEventListener('click', () => {
      void this.updateFontSize(state, 1);
    });
    document.getElementById('btn-dec-font-py')?.addEventListener('click', () => {
      void this.updateFontSize(state, -1);
    });
    document.getElementById('btn-load-py')?.addEventListener('click', () => {
      void this.handleLoadScript();
    });
    document.getElementById('btn-save-py')?.addEventListener('click', () => {
      void this.handleSaveScript();
    });
    document.getElementById('btn-install-polars')?.addEventListener('click', () => {
      void this.handleInstallPolars(state);
    });
    document.getElementById('btn-copy-output-py')?.addEventListener('click', () => {
      void this.handleCopyOutput();
    });
    document.getElementById('btn-refactor-py')?.addEventListener('click', () => {
      void this.handleRefactorColumnNames(state);
    });
    document.getElementById('py-skip-cleaning')?.addEventListener('change', e => {
      state.pythonSkipCleaning = (e.target as HTMLInputElement).checked;
    });
    document.getElementById('python-stage-select')?.addEventListener('change', e => {
      void (async () => {
        // Track previous version for diffing
        state.previousVersionId = state.selectedVersionId;
        state.selectedVersionId = (e.target as HTMLSelectElement).value;
        await this.updateColumnSchema(state);
        this.actions.onStateChange();
      })();
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

    // In Python, we might want it quoted if it contains spaces or just as a string
    // But usually in code we just want the name. Let's provide it as a string literal.
    const text = `"${colName}"`;

    this.editor.executeEdits('insert-column', [
      {
        range: range,
        text: text,
        forceMoveMarkers: true,
      },
    ]);

    this.editor.focus();
  }

  private async handleInstallPolars(state: AppState): Promise<void> {
    const output = document.getElementById('py-output');
    if (!output) return;
    if (!(await this.ensureSecurityAcknowledged(state))) return;

    output.textContent = 'Installing polars... (this may take a minute)';
    try {
      output.textContent = await api.installPythonPackage('polars');
      this.actions.showToast('Polars installed successfully', 'success');
    } catch (err) {
      output.textContent = `Error installing polars: ${String(err)}`;
      this.actions.showToast('Failed to install polars', 'error');
    }
  }

  private async runPython(state: AppState): Promise<void> {
    if (!this.editor) return;
    if (!(await this.ensureSecurityAcknowledged(state))) return;
    const script = this.editor.getValue();
    const output = document.getElementById('py-output');
    if (!output) return;

    if (!state.analysisResponse) {
      output.textContent =
        'Error: No dataset loaded in Beefcake Analyser.\nPlease go to Dashboard or Analyser to load a file first.';
      this.actions.showToast('No dataset loaded', 'error');
      return;
    }

    const dataPath = getDataPathForExecution(state);
    if (!dataPath) {
      output.textContent =
        'Error: Dataset path is missing from analysis response.\nThis might be a bug. Try re-loading the file in the Analyser.';
      this.actions.showToast('Dataset path missing', 'error');
      return;
    }

    output.textContent = 'Running script...';
    try {
      // Cleaning config behavior:
      // - When dataset lifecycle is used (currentDataset exists): cleaning configs are NEVER applied
      //   because the versioned data is already cleaned and stored in Parquet format
      // - When working with raw files (no lifecycle): cleaning configs are applied unless skipCleaning is enabled
      // This ensures backward compatibility while supporting the new lifecycle workflow
      const useCleaningConfigs = !state.currentDataset && !state.pythonSkipCleaning;
      const configs = useCleaningConfigs ? state.cleaningConfigs : undefined;
      const result = await api.runPython(script, dataPath, configs);
      // Convert ANSI escape codes to HTML for colored output
      const html = this.ansiConverter.ansi_to_html(result);
      output.innerHTML = DOMPurify.sanitize(html);
    } catch (err) {
      let errorMsg = String(err);
      if (errorMsg.includes("ModuleNotFoundError: No module named 'polars'")) {
        errorMsg +=
          "\n\nTip: Click the 'Install Polars' button in the toolbar to install the required library.";
      }
      // Also convert errors to HTML (they might have ANSI codes too)
      const html = this.ansiConverter.ansi_to_html(errorMsg);
      output.innerHTML = DOMPurify.sanitize(html);
    }
  }

  private async handleExport(state: AppState): Promise<void> {
    if (!this.editor) return;

    const source: ExportSource = {
      type: 'Python',
      content: this.editor.getValue(),
    };
    // Use the same data path as execution for consistency
    const dataPath = getDataPathForExecution(state);
    if (dataPath) {
      source.path = dataPath;
    }

    const modal = new ExportModal('modal-container', this.actions, source);

    document.getElementById('modal-container')?.classList.add('active');
    await modal.show(state);
    document.getElementById('modal-container')?.classList.remove('active');
  }

  private async updateFontSize(state: AppState, delta: number): Promise<void> {
    if (state.config) {
      state.config.python_font_size = Math.max(
        8,
        Math.min(32, state.config.python_font_size + delta)
      );
      this.editor?.updateOptions({ fontSize: state.config.python_font_size });

      const label = document.getElementById('py-font-size-label');
      if (label) label.textContent = state.config.python_font_size.toString();

      await api.saveAppConfig(state.config);
      this.actions.onStateChange();
    }
  }

  private async ensureSecurityAcknowledged(state: AppState): Promise<boolean> {
    const config = state.config;
    if (!config) {
      this.actions.showToast('App configuration missing', 'error');
      return false;
    }

    if (config.security_warning_acknowledged) {
      return true;
    }

    const confirmed = confirm(
      'Running scripts can execute arbitrary code on your machine. Do you want to continue?'
    );
    if (!confirmed) {
      return false;
    }

    config.security_warning_acknowledged = true;
    await api.saveAppConfig(config);
    this.actions.showToast('Security warning acknowledged', 'info');
    return true;
  }

  private async handleLoadScript(): Promise<void> {
    try {
      const path = await api.openFileDialog([{ name: 'Python', extensions: ['py'] }]);
      if (path) {
        const content = await api.readTextFile(path);
        this.editor?.setValue(content);
        this.actions.showToast('Script loaded', 'success');
      }
    } catch (err) {
      this.actions.showToast(`Error loading script: ${String(err)}`, 'error');
    }
  }

  private async handleSaveScript(): Promise<void> {
    try {
      const path = await api.saveFileDialog([{ name: 'Python', extensions: ['py'] }]);
      if (path) {
        const content = this.editor?.getValue() ?? '';
        await api.writeTextFile(path, content);
        this.actions.showToast('Script saved', 'success');
      }
    } catch (err) {
      this.actions.showToast(`Error saving script: ${String(err)}`, 'error');
    }
  }

  private async handleCopyOutput(): Promise<void> {
    const output = document.getElementById('py-output');
    if (!output) return;

    try {
      // Get plain text content (strips HTML/ANSI formatting)
      const text = output.textContent ?? '';

      if (!text || text === 'Running script...') {
        this.actions.showToast('No output to copy', 'info');
        return;
      }

      await navigator.clipboard.writeText(text);
      this.actions.showToast('Output copied to clipboard', 'success');
    } catch (err) {
      this.actions.showToast(`Failed to copy: ${String(err)}`, 'error');
    }
  }

  /**
   * Handles intelligent refactoring of column names in the Python script when switching between
   * data lifecycle stages (e.g., Raw → Cleaned).
   *
   * **What it does:**
   * - Compares the previous and current data stage versions to find renamed columns
   * - Shows a confirmation dialog listing all column name changes
   * - Uses Monaco Editor's find/replace to update all quoted column references
   * - Preserves the original quote style (single or double quotes)
   * - Shows toast notifications for success/failure/info messages
   *
   * **When it's called:**
   * - User clicks the "Refactor" button after switching data stages
   * - Button only appears when `previousVersionId` and `selectedVersionId` are both set
   *
   * **Example workflow:**
   * ```python
   * # Before (Raw stage):
   * df.select("Customer Name", "Order Date")
   *
   * # After switching to Cleaned stage and clicking Refactor:
   * df.select("customer_name", "order_date")
   * ```
   *
   * @param state - Application state containing dataset and version information
   */
  private async handleRefactorColumnNames(state: AppState): Promise<void> {
    if (
      !this.editor ||
      !state.currentDataset ||
      !state.previousVersionId ||
      !state.selectedVersionId
    ) {
      this.actions.showToast('Cannot refactor: missing version information', 'error');
      return;
    }

    // Check if editor has any content
    const content = this.editor.getValue().trim();
    if (!content) {
      this.actions.showToast('Script is empty - nothing to refactor', 'info');
      return;
    }

    try {
      // Get the diff between previous and current version
      const diff = await api.getVersionDiff(
        state.currentDataset.id,
        state.previousVersionId,
        state.selectedVersionId
      );

      const renamedColumns = diff.schema_changes.columns_renamed;
      if (renamedColumns.length === 0) {
        this.actions.showToast('No column renames detected between stages', 'info');
        return;
      }

      // Build confirmation message with clear formatting
      const renameList = renamedColumns
        .map(([oldName, newName]) => `  • "${oldName}" → "${newName}"`)
        .join('\n');

      const message =
        `🔄 Column Refactoring\n\n` +
        `Found ${renamedColumns.length} renamed column${renamedColumns.length === 1 ? '' : 's'}:\n\n` +
        renameList +
        `\n\nUpdate all references in your Python script?\n\n` +
        `💡 Tip: You can undo changes with Ctrl+Z`;

      if (!confirm(message)) {
        return;
      }

      const model = this.editor.getModel();
      if (!model) return;

      let totalReplacements = 0;

      // Perform replacements for each renamed column
      for (const [oldName, newName] of renamedColumns) {
        // Try both single and double quotes to preserve user's style
        for (const quote of ['"', "'"]) {
          const searchPattern = `${quote}${oldName}${quote}`;
          const matches = model.findMatches(
            searchPattern,
            true, // Search full model
            false, // Not regex
            true, // Match case
            null, // Word separators
            false // Capture matches
          );

          if (matches.length > 0) {
            this.editor.executeEdits(
              'refactor-columns',
              matches.map(match => ({
                range: match.range,
                text: `${quote}${newName}${quote}`,
              }))
            );
            totalReplacements += matches.length;
          }
        }
      }

      if (totalReplacements > 0) {
        const refWord = totalReplacements === 1 ? 'reference' : 'references';
        this.actions.showToast(`✓ Updated ${totalReplacements} column ${refWord}`, 'success');
        // Clear previous version ID so button doesn't show again until next stage change
        state.previousVersionId = null;
        this.actions.onStateChange();
      } else {
        this.actions.showToast('No column references found in script', 'info');
      }
    } catch (err) {
      this.actions.showToast(`Refactor failed: ${String(err)}`, 'error');
      console.error('Column refactoring error:', err);
    }
  }
}
