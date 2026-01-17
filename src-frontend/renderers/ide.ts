import { AppState, ColumnSummary } from '../types';
import { escapeHtml } from '../utils';

export function renderPowerShellView(fontSize: number): string {
  return `
    <div class="ide-view">
      <div class="ide-toolbar">
        <div class="ide-title"><i class="ph ph-terminal"></i> PowerShell Console</div>
        <div class="ide-actions">
          <button id="btn-run-ps" class="btn-primary btn-small"><i class="ph ph-play"></i> Run Selection</button>
          <button id="btn-clear-ps" class="btn-secondary btn-small">Clear</button>
          <button id="btn-load-ps" class="btn-secondary btn-small" title="Load Script"><i class="ph ph-folder-open"></i></button>
          <button id="btn-save-ps" class="btn-secondary btn-small" title="Save Script"><i class="ph ph-floppy-disk"></i></button>
          <div class="font-controls">
            <button id="btn-dec-font" class="btn-secondary btn-small" title="Decrease Font"><i class="ph ph-minus"></i></button>
            <span id="ps-font-size-label">${fontSize}</span>
            <button id="btn-inc-font" class="btn-secondary btn-small" title="Increase Font"><i class="ph ph-plus"></i></button>
          </div>
        </div>
      </div>
      <div id="ps-editor" class="ide-editor" style="height: 400px;"></div>
      <div class="ide-output">
        <div class="output-header">Output</div>
        <pre id="ps-output" class="output-content"></pre>
      </div>
    </div>
  `;
}

export function renderPythonView(state: AppState): string {
  const fontSize = state.config?.python_font_size ?? 14;
  return `
    <div class="ide-layout">
      <div class="ide-main">
        <div class="ide-toolbar">
          <div class="ide-title"><i class="ph ph-code"></i> Python IDE <small>(Polars v${state.polarsVersion ?? 'unknown'})</small></div>
          <div class="ide-actions">
            <button id="btn-run-py" class="btn-primary btn-small"><i class="ph ph-play"></i> Run Script</button>
            <button id="btn-clear-py" class="btn-secondary btn-small">Clear</button>
            <button id="btn-export-py" class="btn-secondary btn-small" title="Export Result"><i class="ph ph-export"></i></button>
            <button id="btn-load-py" class="btn-secondary btn-small" title="Load Script"><i class="ph ph-folder-open"></i></button>
            <button id="btn-save-py" class="btn-secondary btn-small" title="Save Script"><i class="ph ph-floppy-disk"></i></button>
            <button id="btn-install-polars" class="btn-secondary btn-small"><i class="ph ph-package"></i> Install Polars</button>
            <label class="toggle-label" title="When enabled, uses original file. When disabled, applies Analyser cleaning configs.">
              <input type="checkbox" id="py-skip-cleaning" ${state.pythonSkipCleaning ? 'checked' : ''} />
              <span>Skip Cleaning</span>
            </label>
            <div class="font-controls">
              <button id="btn-dec-font-py" class="btn-secondary btn-small"><i class="ph ph-minus"></i></button>
              <span id="py-font-size-label">${fontSize}</span>
              <button id="btn-inc-font-py" class="btn-secondary btn-small"><i class="ph ph-plus"></i></button>
            </div>
          </div>
        </div>
        <div id="py-editor" class="ide-editor" style="height: 450px;"></div>
        <div class="ide-output">
          <div class="output-header">
            Console Output
            <button id="btn-copy-output-py" class="btn-ghost btn-xs" title="Copy output to clipboard" style="float: right;">
              <i class="ph ph-copy"></i>
            </button>
          </div>
          <pre id="py-output" class="output-content"></pre>
        </div>
      </div>
      ${renderColumnSidebar(state)}
    </div>
  `;
}

export function renderSQLView(state: AppState): string {
  const fontSize = state.config?.sql_font_size ?? 14;
  return `
    <div class="ide-layout">
      <div class="ide-main">
        <div class="ide-toolbar">
          <div class="ide-title"><i class="ph ph-database"></i> SQL Lab <small>(Polars v${state.polarsVersion ?? 'unknown'})</small></div>
          <div class="ide-actions">
            <button id="btn-run-sql" class="btn-primary btn-small"><i class="ph ph-play"></i> Execute Query</button>
            <button id="btn-clear-sql" class="btn-secondary btn-small">Clear</button>
            <button id="btn-export-sql" class="btn-secondary btn-small" title="Export Result"><i class="ph ph-export"></i></button>
            <button id="btn-load-sql" class="btn-secondary btn-small" title="Load Query"><i class="ph ph-folder-open"></i></button>
            <button id="btn-save-sql" class="btn-secondary btn-small" title="Save Query"><i class="ph ph-floppy-disk"></i></button>
            <button id="btn-install-polars" class="btn-secondary btn-small"><i class="ph ph-package"></i> Install Polars</button>
            <button id="btn-sql-docs" class="btn-secondary btn-small" title="Polars SQL Docs"><i class="ph ph-question"></i></button>
            <label class="toggle-label" title="When enabled, uses original file. When disabled, applies Analyser cleaning configs.">
              <input type="checkbox" id="sql-skip-cleaning" ${state.sqlSkipCleaning ? 'checked' : ''} />
              <span>Skip Cleaning</span>
            </label>
            <div class="font-controls">
              <button id="btn-dec-font-sql" class="btn-secondary btn-small"><i class="ph ph-minus"></i></button>
              <span id="sql-font-size-label">${fontSize}</span>
              <button id="btn-inc-font-sql" class="btn-secondary btn-small"><i class="ph ph-plus"></i></button>
            </div>
          </div>
        </div>
        <div id="sql-editor" class="ide-editor" style="height: 450px;"></div>
        <div class="ide-output">
          <div class="output-header">
            Result Set
            <button id="btn-copy-output-sql" class="btn-ghost btn-xs" title="Copy output to clipboard" style="float: right;">
              <i class="ph ph-copy"></i>
            </button>
          </div>
          <pre id="sql-output" class="output-content"></pre>
        </div>
      </div>
      ${renderColumnSidebar(state)}
    </div>
  `;
}

function renderColumnSidebar(state: AppState): string {
  if (!state.analysisResponse) {
    return `
      <div class="ide-sidebar">
        <div class="sidebar-info">No dataset loaded.</div>
      </div>
    `;
  }

  return `
    <div class="ide-sidebar">
      <div class="sidebar-header">Columns</div>
      <div class="sidebar-list">
        ${state.analysisResponse.summary
          .map(
            (col: ColumnSummary) => `
          <div class="sidebar-item">
            <span class="col-name" title="${escapeHtml(col.name)}">${escapeHtml(col.name)}</span>
            <button class="btn-insert-col btn-ghost btn-xs" data-col="${escapeHtml(col.name)}" title="Insert into editor">
              <i class="ph ph-plus-circle"></i>
            </button>
          </div>
        `
          )
          .join('')}
      </div>
    </div>
  `;
}
