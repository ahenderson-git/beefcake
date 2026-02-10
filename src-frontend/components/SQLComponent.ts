import * as monaco from 'monaco-editor';

import * as api from '../api';
import * as renderers from '../renderers';
import { renderColumnSidebar } from '../renderers/ide';
import { AppState } from '../types';
import { getDataPathForExecution } from '../utils';
import { setupIDESidebarToggle } from '../utils/ide-sidebar';

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
    void this.updateColumnSchema(state);
  }

  private async updateColumnSchema(state: AppState): Promise<void> {
    // If we have a dataset with versions, fetch the schema for the selected version
    if (state.currentDataset && (state.currentDataset.versions?.length ?? 0) > 0) {
      const selectedVersionId = state.selectedVersionId ?? state.currentDataset.activeVersionId;
      try {
        state.currentIdeColumns = await api.getVersionSchema(
          state.currentDataset.id,
          selectedVersionId
        );
        // Update only the sidebar, not the entire component
        this.updateSidebarDisplay(state);
      } catch (err) {
        console.error('Failed to fetch version schema:', err);
        // Fall back to analysisResponse columns if API fails
        state.currentIdeColumns = null;
        this.updateSidebarDisplay(state);
      }
    } else {
      // No dataset versions, use analysisResponse
      state.currentIdeColumns = null;
      this.updateSidebarDisplay(state);
    }
  }

  private updateSidebarDisplay(state: AppState): void {
    // Find the sidebar container in the current layout
    const sidebarContainer = document.querySelector('.ide-layout .ide-sidebar');
    if (!sidebarContainer) return;

    // Get the current collapsed state before re-rendering
    const wasCollapsed = sidebarContainer.classList.contains('collapsed');

    // Re-render just the sidebar HTML
    const newSidebarHTML = renderColumnSidebar(state);
    const tempDiv = document.createElement('div');
    tempDiv.innerHTML = newSidebarHTML;
    const newSidebar = tempDiv.firstElementChild;

    if (newSidebar) {
      // Preserve collapsed state
      if (wasCollapsed) {
        newSidebar.classList.add('collapsed');
      }

      // Replace the old sidebar with the new one
      sidebarContainer.replaceWith(newSidebar);

      // Re-bind sidebar events (insert column buttons)
      this.bindSidebarEvents();

      // Re-bind collapse/expand functionality after DOM replacement
      // This ensures the collapse button works after sidebar updates
      setupIDESidebarToggle();
    }
  }

  private initMonaco(state: AppState): void {
    const editorContainer = document.getElementById('sql-editor');
    if (editorContainer) {
      //noinspection SqlNoDataSourceInspection,SqlDialectInspection
      const defaultValue =
        '-- SQL Query\n' +
        '-- The loaded dataset is automatically registered as the table "data"\n' +
        'SELECT * FROM data LIMIT 10;';

      this.editor = monaco.editor.create(editorContainer, {
        value: state.sqlScript ?? defaultValue,
        language: 'sql',
        theme: 'vs-dark',
        automaticLayout: true,
        fontSize: state.config?.settings.sql_font_size ?? 14,
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
      void this.handleInstallPolars(state);
    });
    document.getElementById('btn-sql-docs')?.addEventListener('click', () => {
      window.open('https://docs.pola.rs/user-guide/sql/intro/', '_blank');
    });
    document.getElementById('btn-copy-output-sql')?.addEventListener('click', () => {
      this.handleCopyOutput();
    });
    document.getElementById('btn-refactor-sql')?.addEventListener('click', () => {
      void this.handleRefactorColumnNames(state);
    });
    document.getElementById('sql-skip-cleaning')?.addEventListener('change', e => {
      state.sqlSkipCleaning = (e.target as HTMLInputElement).checked;
      this.actions.onStateChange();
    });
    document.getElementById('sql-stage-select')?.addEventListener('change', e => {
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

  private updateStatus(
    status: 'ready' | 'running' | 'success' | 'error',
    message: string,
    execTime?: number,
    rowCount?: number
  ): void {
    const statusItem = document.querySelector('#sql-status-text')?.parentElement;
    const statusText = document.getElementById('sql-status-text');
    const execTimeItem = document.getElementById('sql-exec-time');
    const execTimeText = document.getElementById('sql-exec-time-text');
    const rowCountItem = document.getElementById('sql-row-count');
    const rowCountText = document.getElementById('sql-row-count-text');

    if (statusItem && statusText) {
      statusItem.className = `status-item status-${status}`;
      statusText.textContent = message;
    }

    if (execTimeItem && execTimeText) {
      if (execTime !== undefined) {
        execTimeItem.style.display = 'flex';
        execTimeText.textContent = `${execTime.toFixed(2)}s`;
      } else {
        execTimeItem.style.display = 'none';
      }
    }

    if (rowCountItem && rowCountText) {
      if (rowCount !== undefined) {
        rowCountItem.style.display = 'flex';
        rowCountText.textContent = `${rowCount.toLocaleString()} rows`;
      } else {
        rowCountItem.style.display = 'none';
      }
    }
  }

  private async runSql(state: AppState): Promise<void> {
    if (!this.editor) return;
    if (!(await this.ensureSecurityAcknowledged(state))) return;
    const query = this.editor.getValue();
    const output = document.getElementById('sql-output');
    if (!output) return;

    if (!state.analysisResponse) {
      output.textContent =
        'Error: No dataset loaded in Beefcake Analyser.\nPlease go to Dashboard or Analyser to load a file first.';
      this.actions.showToast('No dataset loaded', 'error');
      this.updateStatus('error', 'No dataset loaded');
      return;
    }

    const dataPath = getDataPathForExecution(state);
    if (!dataPath) {
      output.textContent =
        'Error: Dataset path is missing from analysis response.\nTry re-loading the file in the Analyser.';
      this.actions.showToast('Dataset path missing', 'error');
      this.updateStatus('error', 'Missing dataset path');
      return;
    }

    output.textContent = 'Executing query...';
    this.updateStatus('running', 'Executing...');
    const startTime = performance.now();

    try {
      // Cleaning config behavior:
      // - When dataset lifecycle is used (currentDataset exists): cleaning configs are NEVER applied
      //   because the versioned data is already cleaned and stored in Parquet format
      // - When working with raw files (no lifecycle): cleaning configs are applied unless skipCleaning is enabled
      // This ensures backward compatibility while supporting the new lifecycle workflow
      const useCleaningConfigs = !state.currentDataset && !state.sqlSkipCleaning;
      const configs = useCleaningConfigs ? state.cleaningConfigs : undefined;
      const result = await api.runSql(query, dataPath, configs);
      const execTime = (performance.now() - startTime) / 1000;

      output.textContent = result;

      // Try to extract row count from result
      const rowMatch = result.match(/(\d+)\s+rows?\s+selected/i);
      const rowCount = rowMatch?.[1] ? parseInt(rowMatch[1]) : undefined;

      this.updateStatus('success', 'Success', execTime, rowCount);
    } catch (err) {
      const execTime = (performance.now() - startTime) / 1000;
      let errorMsg = String(err);
      if (errorMsg.includes("ModuleNotFoundError: No module named 'polars'")) {
        errorMsg +=
          "\n\nTip: Click the 'Install Polars' button in the SQL IDE toolbar to install the required library.";
      }
      output.textContent = errorMsg;
      this.updateStatus('error', 'Query failed', execTime);
    }
  }

  private async handleInstallPolars(state: AppState): Promise<void> {
    const output = document.getElementById('sql-output');
    if (!output) return;
    if (!(await this.ensureSecurityAcknowledged(state))) return;

    output.textContent = 'Installing polars package...\n';
    try {
      output.textContent = await api.installPythonPackage('polars');
      this.actions.showToast('Polars installed successfully', 'success');
      // Refresh Polars version in UI
      await this.refreshPolarsVersion(state);
    } catch (err) {
      output.textContent = `Installation failed:\n${String(err)}`;
      this.actions.showToast('Installation failed', 'error');
    }
  }

  private async refreshPolarsVersion(state: AppState): Promise<void> {
    try {
      const result = await api.checkPythonEnvironment();
      const match = result.match(/Polars\s+(\d+\.\d+\.\d+)\s+installed/);
      if (match?.[1]) {
        state.polarsVersion = match[1];
        this.render(state); // Re-render to show updated version
      }
    } catch (err) {
      /* eslint-disable-next-line no-console */
      console.debug('Failed to refresh Polars version:', err);
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
    if (!this.editor) return;

    // Use the same data path as execution for consistency
    const dataPath = getDataPathForExecution(state);

    const modal = new ExportModal('modal-container', this.actions, {
      type: 'SQL',
      content: this.editor.getValue(),
      ...(dataPath ? { path: dataPath } : {}),
    });

    document.getElementById('modal-container')?.classList.add('active');
    await modal.show(state);
    document.getElementById('modal-container')?.classList.remove('active');
  }

  private async updateFontSize(state: AppState, delta: number): Promise<void> {
    if (state.config) {
      state.config.settings.sql_font_size = Math.max(
        8,
        Math.min(32, (state.config.settings.sql_font_size ?? 14) + delta)
      );
      this.editor?.updateOptions({ fontSize: state.config.settings.sql_font_size });

      const label = document.getElementById('sql-font-size-label');
      if (label) label.textContent = state.config.settings.sql_font_size.toString();

      await api.saveAppConfig(state.config);
      this.actions.onStateChange();
    }
  }

  private async ensureSecurityAcknowledged(state: AppState): Promise<boolean> {
    if (!state.config) {
      this.actions.showToast('App configuration missing', 'error');
      return false;
    }

    if (state.config.settings.security_warning_acknowledged) {
      return true;
    }

    const confirmed = confirm(
      'Running scripts can execute arbitrary code on your machine. Do you want to continue?'
    );
    if (!confirmed) {
      return false;
    }

    state.config.settings.security_warning_acknowledged = true;
    await api.saveAppConfig(state.config);
    this.actions.showToast('Security warning acknowledged', 'info');
    return true;
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

  /**
   * Handles intelligent refactoring of column names in SQL queries when switching between
   * data lifecycle stages (e.g., Raw → Cleaned).
   *
   * **What it does:**
   * - Compares the previous and current data stage versions to find renamed columns
   * - Shows a confirmation dialog listing all column name changes
   * - Uses Monaco Editor's find/replace to update column references
   * - Handles multiple SQL identifier styles: unquoted, double-quoted, backticks, brackets
   * - Preserves the original quoting style used in the query
   * - Shows toast notifications for success/failure/info messages
   *
   * **When it's called:**
   * - User clicks the "Refactor" button after switching data stages
   * - Button only appears when `previousVersionId` and `selectedVersionId` are both set
   *
   * **SQL identifier support:**
   * - Unquoted: `SELECT customer_name FROM data`
   * - Double quotes: `SELECT "customer name" FROM data`
   * - Backticks: `SELECT \`customer name\` FROM data` (MySQL)
   * - Brackets: `SELECT [customer name] FROM data` (SQL Server)
   *
   * **Example workflow:**
   * ```sql
   * -- Before (Raw stage):
   * SELECT "Customer Name", "Order Date" FROM data
   *
   * -- After switching to Cleaned stage and clicking Refactor:
   * SELECT "customer_name", "order_date" FROM data
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
      this.actions.showToast('Query is empty - nothing to refactor', 'info');
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
        `\n\nUpdate all references in your SQL query?\n\n` +
        `💡 Tip: You can undo changes with Ctrl+Z`;

      if (!confirm(message)) {
        return;
      }

      const model = this.editor.getModel();
      if (!model) return;

      let totalReplacements = 0;

      // Common SQL keywords to avoid replacing (lowercase for comparison)
      const sqlKeywords = new Set([
        'select',
        'from',
        'where',
        'join',
        'left',
        'right',
        'inner',
        'outer',
        'on',
        'group',
        'by',
        'order',
        'having',
        'limit',
        'offset',
        'union',
        'all',
        'distinct',
        'as',
        'and',
        'or',
        'not',
        'in',
        'exists',
        'between',
        'like',
        'is',
        'null',
        'case',
        'when',
        'then',
        'else',
        'end',
        'cast',
        'count',
        'sum',
        'avg',
        'min',
        'max',
        'date',
        'time',
        'timestamp',
        'true',
        'false',
      ]);

      // Perform replacements for each renamed column
      for (const [oldName, newName] of renamedColumns) {
        // Build patterns to try (skip unquoted if it's a SQL keyword or has spaces)
        const isKeyword = sqlKeywords.has(oldName.toLowerCase());
        const hasSpaces = oldName.includes(' ');

        const patterns = [];

        // Only try unquoted if it's not a keyword and has no spaces
        if (!isKeyword && !hasSpaces) {
          patterns.push(oldName);
        }

        // Always try quoted forms
        patterns.push(
          `"${oldName}"`, // Double quoted
          `\`${oldName}\``, // Backtick quoted (MySQL)
          `[${oldName}]` // Bracket quoted (SQL Server)
        );

        for (const searchPattern of patterns) {
          const matches = model.findMatches(
            searchPattern,
            true, // Search full model
            false, // Not regex
            true, // Match case
            null, // Word separators
            false // Capture matches
          );

          if (matches.length > 0) {
            // Determine the quote style to use for replacement (preserve original style)
            let replacement = newName;
            if (searchPattern.startsWith('"')) {
              replacement = `"${newName}"`;
            } else if (searchPattern.startsWith('`')) {
              replacement = `\`${newName}\``;
            } else if (searchPattern.startsWith('[')) {
              replacement = `[${newName}]`;
            }

            this.editor.executeEdits(
              'refactor-columns',
              matches.map(match => ({
                range: match.range,
                text: replacement,
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
        this.actions.showToast('No column references found in query', 'info');
      }
    } catch (err) {
      this.actions.showToast(`Refactor failed: ${String(err)}`, 'error');
      console.error('Column refactoring error:', err);
    }
  }
}
