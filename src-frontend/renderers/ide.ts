import { AppState, ColumnSummary, DatasetVersion, ColumnInfo } from '../types';
import { escapeHtml, fmtBytes, getStageIcon, getStageOrder } from '../utils';

/**
 * Renders the stage selector dropdown for IDE toolbars.
 *
 * Shows a traditional "Skip Cleaning" checkbox when no dataset versions exist (backward compatibility).
 * Shows a dropdown with all available versions when they exist, sorted by stage order.
 *
 * @param state - The application state
 * @param ideType - Either 'python' or 'sql' to generate unique IDs
 * @returns HTML string for the stage selector UI
 */
function renderStageSelector(state: AppState, ideType: 'python' | 'sql'): string {
  if (!state.currentDataset?.versions.length) {
    return `
      <label class="toggle-label" title="When enabled, uses original file. When disabled, applies Analyser cleaning configs." data-testid="${ideType}-skip-cleaning-label">
        <input type="checkbox" id="${ideType}-skip-cleaning" ${ideType === 'python' ? (state.pythonSkipCleaning ? 'checked' : '') : state.sqlSkipCleaning ? 'checked' : ''} data-testid="${ideType}-skip-cleaning-checkbox" />
        <span>Skip Cleaning</span>
      </label>
    `;
  }

  const selectedVersionId = state.selectedVersionId ?? state.currentDataset.activeVersionId;
  // Sort versions by stage order (Raw → Profiled → Cleaned → Advanced → Validated → Published)
  const sortedVersions = [...state.currentDataset.versions].sort(
    (a, b) => getStageOrder(a.stage) - getStageOrder(b.stage)
  );

  return `
    <div class="stage-selector" title="Select which data version/stage to use for execution" data-testid="${ideType}-stage-selector">
      <label for="${ideType}-stage-select" class="stage-label">Data Stage:</label>
      <select id="${ideType}-stage-select" class="stage-select" data-testid="${ideType}-stage-select">
        ${sortedVersions
          .map((v: DatasetVersion) => {
            const icon = getStageIcon(v.stage);
            const rowInfo = v.metadata.row_count
              ? ` (${v.metadata.row_count.toLocaleString()} rows`
              : '';
            const sizeInfo = v.metadata.file_size_bytes
              ? `, ${fmtBytes(v.metadata.file_size_bytes)}`
              : '';
            const closeParen = rowInfo ? ')' : '';
            const description = v.metadata.description
              ? ` - ${escapeHtml(v.metadata.description)}`
              : '';

            return `
            <option value="${escapeHtml(v.id)}" ${v.id === selectedVersionId ? 'selected' : ''} data-testid="${ideType}-stage-option-${v.stage}">
              ${icon} ${escapeHtml(v.stage)}${rowInfo}${sizeInfo}${closeParen}${description}
            </option>
          `;
          })
          .join('')}
      </select>
    </div>
  `;
}

export function renderPowerShellView(fontSize: number): string {
  return `
    <div class="ide-view" data-testid="powershell-ide-view">
      <div class="ide-toolbar" data-testid="powershell-ide-toolbar">
        <div class="ide-title"><i class="ph ph-terminal"></i> PowerShell Console</div>
        <div class="ide-actions">
          <button id="btn-run-ps" class="btn-primary btn-small" data-testid="powershell-ide-run-button"><i class="ph ph-play"></i> Run Selection</button>
          <button id="btn-clear-ps" class="btn-secondary btn-small" data-testid="powershell-ide-clear-button">Clear</button>
          <button id="btn-load-ps" class="btn-secondary btn-small" title="Load Script" data-testid="powershell-ide-load-button"><i class="ph ph-folder-open"></i></button>
          <button id="btn-save-ps" class="btn-secondary btn-small" title="Save Script" data-testid="powershell-ide-save-button"><i class="ph ph-floppy-disk"></i></button>
          <div class="font-controls">
            <button id="btn-dec-font" class="btn-secondary btn-small" title="Decrease Font" data-testid="powershell-ide-dec-font-button"><i class="ph ph-minus"></i></button>
            <span id="ps-font-size-label" data-testid="powershell-ide-font-size">${fontSize}</span>
            <button id="btn-inc-font" class="btn-secondary btn-small" title="Increase Font" data-testid="powershell-ide-inc-font-button"><i class="ph ph-plus"></i></button>
          </div>
        </div>
      </div>
      <div id="ps-editor" class="ide-editor" style="height: 400px;" data-testid="powershell-ide-editor"></div>
      <div class="ide-output" data-testid="powershell-ide-output-panel">
        <div class="output-header">Output</div>
        <pre id="ps-output" class="output-content" data-testid="powershell-ide-output"></pre>
      </div>
    </div>
  `;
}

export function renderPythonView(state: AppState): string {
  const fontSize = state.config?.python_font_size ?? 14;
  const showRefactorBtn =
    state.currentDataset && state.previousVersionId && state.selectedVersionId;

  return `
    <div class="ide-layout" data-testid="python-ide-view">
      <div class="ide-main">
        <div class="ide-toolbar" data-testid="python-ide-toolbar">
          <div class="ide-title">
            <i class="ph ph-code"></i>
            <span>Python IDE</span>
            <small>(Polars v${state.polarsVersion ?? 'unknown'})</small>
          </div>
          <div class="ide-actions">
            <!-- Run Actions Group -->
            <div class="btn-group btn-group-primary">
              <button id="btn-run-py" class="btn-primary btn-small" title="Execute Python script" data-testid="python-ide-run-button">
                <i class="ph ph-play"></i>
                <span>Run Script</span>
              </button>
              <button id="btn-clear-py" class="btn-secondary btn-small" title="Clear output" data-testid="python-ide-clear-button">
                <i class="ph ph-eraser"></i>
              </button>
            </div>

            <div class="action-divider"></div>

            <!-- File Actions Group -->
            <div class="btn-group">
              <button id="btn-load-py" class="btn-secondary btn-small" title="Load script from file" data-testid="python-ide-load-button">
                <i class="ph ph-folder-open"></i>
              </button>
              <button id="btn-save-py" class="btn-secondary btn-small" title="Save script to file" data-testid="python-ide-save-button">
                <i class="ph ph-floppy-disk"></i>
              </button>
              <button id="btn-export-py" class="btn-secondary btn-small" title="Export result to file" data-testid="python-ide-export-button">
                <i class="ph ph-export"></i>
              </button>
            </div>

            <div class="action-divider"></div>

            <!-- Tools Group -->
            <div class="btn-group">
              <button id="btn-install-polars" class="btn-secondary btn-small" title="Install Polars library via pip" data-testid="python-ide-install-button">
                <i class="ph ph-package"></i>
              </button>
              ${
                showRefactorBtn
                  ? `
              <button id="btn-refactor-py" class="btn-secondary btn-small" title="Update column names in script to match selected stage" data-testid="python-ide-refactor-button">
                <i class="ph ph-arrows-counter-clockwise"></i>
              </button>`
                  : ''
              }
            </div>

            <div class="action-divider"></div>

            <!-- View Controls Group -->
            <div class="btn-group">
              ${renderStageSelector(state, 'python')}
              <div class="font-controls">
                <button id="btn-dec-font-py" class="btn-secondary btn-small" title="Decrease font size" data-testid="python-ide-dec-font-button">
                  <i class="ph ph-minus"></i>
                </button>
                <span id="py-font-size-label" class="font-size-display" data-testid="python-ide-font-size">${fontSize}px</span>
                <button id="btn-inc-font-py" class="btn-secondary btn-small" title="Increase font size" data-testid="python-ide-inc-font-button">
                  <i class="ph ph-plus"></i>
                </button>
              </div>
            </div>
          </div>
        </div>
        <div id="py-editor" class="ide-editor" style="height: 450px;" data-testid="python-ide-editor"></div>
        <div class="ide-output" data-testid="python-ide-output-panel">
          <div class="output-header">
            <div class="output-header-left">
              <i class="ph ph-terminal-window"></i>
              <span>Console Output</span>
            </div>
            <div class="output-header-right">
              <button id="btn-copy-output-py" class="btn-ghost btn-xs" title="Copy output to clipboard" data-testid="python-ide-copy-button">
                <i class="ph ph-copy"></i> Copy
              </button>
            </div>
          </div>
          <pre id="py-output" class="output-content" data-testid="python-ide-output"></pre>
          <div class="output-status-bar" id="py-status-bar">
            <div class="status-item">
              <i class="ph ph-circle"></i>
              <span id="py-status-text">Ready</span>
            </div>
            <div class="status-item" id="py-exec-time" style="display: none;">
              <i class="ph ph-clock"></i>
              <span id="py-exec-time-text">0s</span>
            </div>
          </div>
        </div>
      </div>
      ${renderColumnSidebar(state)}
    </div>
  `;
}

export function renderSQLView(state: AppState): string {
  const fontSize = state.config?.sql_font_size ?? 14;
  const showRefactorBtn =
    state.currentDataset && state.previousVersionId && state.selectedVersionId;

  return `
    <div class="ide-layout" data-testid="sql-ide-view">
      <div class="ide-main">
        <div class="ide-toolbar" data-testid="sql-ide-toolbar">
          <div class="ide-title">
            <i class="ph ph-database"></i>
            <span>SQL Lab</span>
            <small>(Polars v${state.polarsVersion ?? 'unknown'})</small>
          </div>
          <div class="ide-actions">
            <!-- Run Actions Group -->
            <div class="btn-group btn-group-primary">
              <button id="btn-run-sql" class="btn-primary btn-small" title="Execute SQL query" data-testid="sql-ide-run-button">
                <i class="ph ph-play"></i>
                <span>Execute Query</span>
              </button>
              <button id="btn-clear-sql" class="btn-secondary btn-small" title="Clear output" data-testid="sql-ide-clear-button">
                <i class="ph ph-eraser"></i>
              </button>
            </div>

            <div class="action-divider"></div>

            <!-- File Actions Group -->
            <div class="btn-group">
              <button id="btn-load-sql" class="btn-secondary btn-small" title="Load query from file" data-testid="sql-ide-load-button">
                <i class="ph ph-folder-open"></i>
              </button>
              <button id="btn-save-sql" class="btn-secondary btn-small" title="Save query to file" data-testid="sql-ide-save-button">
                <i class="ph ph-floppy-disk"></i>
              </button>
              <button id="btn-export-sql" class="btn-secondary btn-small" title="Export result to file" data-testid="sql-ide-export-button">
                <i class="ph ph-export"></i>
              </button>
            </div>

            <div class="action-divider"></div>

            <!-- Tools Group -->
            <div class="btn-group">
              <button id="btn-install-polars" class="btn-secondary btn-small" title="Install Polars library via pip" data-testid="sql-ide-install-button">
                <i class="ph ph-package"></i>
              </button>
              <button id="btn-sql-docs" class="btn-secondary btn-small" title="Open Polars SQL documentation" data-testid="sql-ide-docs-button">
                <i class="ph ph-question"></i>
              </button>
              ${
                showRefactorBtn
                  ? `
              <button id="btn-refactor-sql" class="btn-secondary btn-small" title="Update column names in query to match selected stage" data-testid="sql-ide-refactor-button">
                <i class="ph ph-arrows-counter-clockwise"></i>
              </button>`
                  : ''
              }
            </div>

            <div class="action-divider"></div>

            <!-- View Controls Group -->
            <div class="btn-group">
              ${renderStageSelector(state, 'sql')}
              <div class="font-controls">
                <button id="btn-dec-font-sql" class="btn-secondary btn-small" title="Decrease font size" data-testid="sql-ide-dec-font-button">
                  <i class="ph ph-minus"></i>
                </button>
                <span id="sql-font-size-label" class="font-size-display" data-testid="sql-ide-font-size">${fontSize}px</span>
                <button id="btn-inc-font-sql" class="btn-secondary btn-small" title="Increase font size" data-testid="sql-ide-inc-font-button">
                  <i class="ph ph-plus"></i>
                </button>
              </div>
            </div>
          </div>
        </div>
        <div id="sql-editor" class="ide-editor" style="height: 450px;" data-testid="sql-ide-editor"></div>
        <div class="ide-output" data-testid="sql-ide-output-panel">
          <div class="output-header">
            <div class="output-header-left">
              <i class="ph ph-table"></i>
              <span>Result Set</span>
            </div>
            <div class="output-header-right">
              <button id="btn-copy-output-sql" class="btn-ghost btn-xs" title="Copy output to clipboard" data-testid="sql-ide-copy-button">
                <i class="ph ph-copy"></i> Copy
              </button>
            </div>
          </div>
          <pre id="sql-output" class="output-content" data-testid="sql-ide-output"></pre>
          <div class="output-status-bar" id="sql-status-bar">
            <div class="status-item">
              <i class="ph ph-circle"></i>
              <span id="sql-status-text">Ready</span>
            </div>
            <div class="status-item" id="sql-exec-time" style="display: none;">
              <i class="ph ph-clock"></i>
              <span id="sql-exec-time-text">0s</span>
            </div>
            <div class="status-item" id="sql-row-count" style="display: none;">
              <i class="ph ph-rows"></i>
              <span id="sql-row-count-text">0 rows</span>
            </div>
          </div>
        </div>
      </div>
      ${renderColumnSidebar(state)}
    </div>
  `;
}

export function renderColumnSidebar(state: AppState): string {
  // If we have stage-specific columns, use those; otherwise fall back to analysisResponse
  const columns = state.currentIdeColumns;

  if (!columns && !state.analysisResponse) {
    return `
      <div class="ide-sidebar" id="ide-sidebar">
        <div class="ide-sidebar-collapsed-tab" id="ide-collapsed-tab">
          <span>Columns</span>
        </div>
        <div class="ide-sidebar-content">
          <div class="sidebar-header" id="ide-sidebar-header">
            <div class="sidebar-title">
              <i class="ph ph-table"></i>
              <span>Columns</span>
            </div>
            <div class="sidebar-header-controls">
              <button class="sidebar-collapse-btn" id="ide-collapse-btn" title="Collapse sidebar">
                <i class="ph ph-caret-right"></i>
              </button>
            </div>
          </div>
          <div class="sidebar-info">No dataset loaded.</div>
        </div>
      </div>
    `;
  }

  // Use currentIdeColumns if available, otherwise use analysisResponse.summary
  const columnsToRender = columns
    ? columns.map((col: ColumnInfo) => ({
        name: col.name,
        dtype: col.dtype,
      }))
    : state.analysisResponse!.summary.map((col: ColumnSummary) => ({
        name: col.name,
        dtype: col.kind,
      }));

  return `
    <div class="ide-sidebar" id="ide-sidebar" data-testid="ide-column-sidebar">
      <div class="ide-sidebar-collapsed-tab" id="ide-collapsed-tab">
        <span>Columns</span>
      </div>
      <div class="ide-sidebar-content">
        <div class="sidebar-header" id="ide-sidebar-header">
          <div class="sidebar-title">
            <i class="ph ph-table"></i>
            <span>Columns</span>
          </div>
          <div class="sidebar-header-controls">
            <button class="sidebar-collapse-btn" id="ide-collapse-btn" title="Collapse sidebar">
              <i class="ph ph-caret-right"></i>
            </button>
          </div>
        </div>
        <div class="sidebar-list">
          ${columnsToRender
            .map(
              col => `
            <div class="sidebar-item" data-testid="ide-column-item-${escapeHtml(col.name)}">
              <div class="col-info">
                <span class="col-name" title="${escapeHtml(col.name)}" data-testid="ide-column-name-${escapeHtml(col.name)}">${escapeHtml(col.name)}</span>
                <span class="col-dtype" title="${escapeHtml(col.dtype)}">${escapeHtml(col.dtype)}</span>
              </div>
              <button class="btn-insert-col btn-ghost btn-xs" data-col="${escapeHtml(col.name)}" title="Insert column name into editor" data-testid="ide-insert-column-${escapeHtml(col.name)}">
                <i class="ph ph-plus-circle"></i>
              </button>
            </div>
          `
            )
            .join('')}
        </div>
      </div>
    </div>
  `;
}
