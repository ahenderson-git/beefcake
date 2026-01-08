import { AnalysisResponse, AppConfig, ColumnSummary, View } from "./types";
import { escapeHtml, fmtBytes, fmtDuration } from "./utils";

export function renderDashboardView(state: any): string {
  return `
    <div class="dashboard">
      <div class="hero">
        <h1>beefcake <small>v0.1.0</small></h1>
        <p>Advanced Data Analysis & ETL Pipeline</p>
      </div>
      <div class="stats-grid">
        <div class="stat-card">
          <h3>Local Storage</h3>
          <div class="stat-value">Active</div>
          <p>~/.beefcake_config.json</p>
        </div>
        <div class="stat-card">
          <h3>Connections</h3>
          <div class="stat-value">${state.config?.connections.length || 0}</div>
          <p>Configured Endpoints</p>
        </div>
        <div class="stat-card">
          <h3>Last Analysis</h3>
          <div class="stat-value">${state.response ? escapeHtml(state.response.file_name) : 'None'}</div>
          <p>${state.response ? fmtBytes(state.response.file_size) : 'Ready for input'}</p>
        </div>
      </div>
      <div class="actions">
        <button id="btn-open" class="btn-primary">
          <i class="ph ph-cloud-arrow-up"></i> Load Dataset
        </button>
        <button id="btn-powershell" class="btn-secondary">
          <i class="ph ph-terminal"></i> PowerShell Console
        </button>
      </div>
    </div>
  `;
}

export function renderAnalyserHeader(response: AnalysisResponse, trimPct: number): string {
  return `
    <div class="analyser-header">
      <div class="file-info">
        <h2>${escapeHtml(response.file_name)}</h2>
        <div class="meta-tags">
          <span class="tag">${fmtBytes(response.file_size)}</span>
          <span class="tag">${response.row_count.toLocaleString()} rows</span>
          <span class="tag">${response.column_count} columns</span>
          <span class="tag">Took ${fmtDuration(response.analysis_duration)}</span>
        </div>
      </div>
      <div class="header-actions">
        <div class="trim-control">
          <label>Trim Mean %</label>
          <input type="range" id="trim-range" min="0" max="0.45" step="0.05" value="${trimPct}">
          <span>${Math.round(trimPct * 100)}%</span>
        </div>
        <button id="btn-push" class="btn-primary">
          <i class="ph ph-database"></i> Push to DB
        </button>
      </div>
    </div>
  `;
}

export function renderAnalyser(response: AnalysisResponse, expandedRows: Set<string>): string {
  return `
    <div class="analyser-view">
      <div id="analyser-header-container"></div>
      <div class="summary-grid">
        ${response.summary.map(col => renderSummaryCard(col, expandedRows.has(col.name))).join('')}
      </div>
    </div>
  `;
}

export function renderEmptyAnalyser(): string {
  return `
    <div class="empty-state">
      <i class="ph ph-magnifying-glass"></i>
      <h3>No Dataset Loaded</h3>
      <p>Please go to the Dashboard to load a file for analysis.</p>
    </div>
  `;
}

export function renderSummaryCard(col: ColumnSummary, isExpanded: boolean): string {
  const nullPct = ((col.nulls / col.count) * 100).toFixed(1);
  const healthClass = col.nulls > col.count * 0.1 ? 'warn' : 'ok';

  let statsHtml = '';
  if (col.stats.Numeric) {
    const s = col.stats.Numeric;
    statsHtml = `
      <div class="stats-mini">
        <span>Min: ${s.min?.toFixed(2) ?? '—'}</span>
        <span>Max: ${s.max?.toFixed(2) ?? '—'}</span>
        <span>Mean: ${s.mean?.toFixed(2) ?? '—'}</span>
      </div>
    `;
  } else if (col.stats.Categorical) {
    statsHtml = `<div class="stats-mini">Categorical (${Object.keys(col.stats.Categorical).length} unique)</div>`;
  } else if (col.stats.Temporal) {
    statsHtml = `<div class="stats-mini">Temporal Range: ${col.stats.Temporal.min} - ${col.stats.Temporal.max}</div>`;
  }

  return `
    <div class="summary-card ${isExpanded ? 'expanded' : ''}" data-col="${escapeHtml(col.name)}">
      <div class="card-header">
        <div class="col-name">
          <i class="ph ph-caret-${isExpanded ? 'down' : 'right'}"></i>
          <strong>${col.name}</strong>
          <span class="kind-tag">${col.kind}</span>
        </div>
        ${statsHtml}
        <div class="health-tag ${healthClass}">${nullPct}% nulls</div>
      </div>
      ${isExpanded ? renderExpandedDetails(col) : ''}
    </div>
  `;
}

function renderExpandedDetails(col: ColumnSummary): string {
  let detailsHtml = '';

  if (col.stats.Numeric) {
    const s = col.stats.Numeric;
    detailsHtml = `
      <div class="details-grid">
        <div class="detail-item">
          <label>Std Dev</label>
          <span>${s.std_dev?.toFixed(4) ?? '—'}</span>
        </div>
        <div class="detail-item">
          <label>Skew</label>
          <span>${s.skew?.toFixed(4) ?? '—'}</span>
        </div>
        <div class="detail-item">
          <label>P05 / P95</label>
          <span>${s.p05?.toFixed(2) ?? '—'} / ${s.p95?.toFixed(2) ?? '—'}</span>
        </div>
        <div class="detail-item">
          <label>Trimmed Mean</label>
          <span>${s.trimmed_mean?.toFixed(2) ?? '—'}</span>
        </div>
      </div>
    `;
  } else if (col.stats.Categorical) {
    const freq = col.stats.Categorical;
    const sorted = Object.entries(freq).sort((a: any, b: any) => b[1] - a[1]).slice(0, 5);
    detailsHtml = `
      <div class="freq-list">
        <label>Top Values</label>
        ${sorted.map(([val, count]: [any, any]) => `
          <div class="freq-item">
            <span class="f-val">${escapeHtml(String(val))}</span>
            <span class="f-count">${count}</span>
          </div>
        `).join('')}
      </div>
    `;
  }

  return `
    <div class="card-content">
      ${detailsHtml}
      <div class="advice-section">
        <div class="advice-block interpretation">
          <h4><i class="ph ph-info"></i> Interpretation</h4>
          <ul>${col.interpretation.map(i => `<li>${i}</li>`).join('')}</ul>
        </div>
        <div class="advice-block ml">
          <h4><i class="ph ph-flask"></i> ML Advice</h4>
          <ul>${col.ml_advice.map(i => `<li>${i}</li>`).join('')}</ul>
        </div>
      </div>
      ${col.samples && col.samples.length > 0 ? `
        <div class="samples-section">
          <label>Data Samples</label>
          <div class="samples-row">
            ${col.samples.map(s => `<span class="sample-tag">${escapeHtml(String(s))}</span>`).join('')}
          </div>
        </div>
      ` : ''}
    </div>
  `;
}

export function renderPowerShellView(): string {
  return `
    <div class="powershell-view">
      <div class="ps-header">
        <div class="ps-title">
          <i class="ph ph-terminal"></i>
          PowerShell Core
        </div>
        <div class="ps-actions">
          <button id="btn-load-ps" class="btn-secondary">
            <i class="ph ph-folder-open"></i> Load
          </button>
          <button id="btn-save-ps" class="btn-secondary">
            <i class="ph ph-floppy-disk"></i> Save
          </button>
          <button id="btn-run-ps" class="btn-primary">
            <i class="ph ph-play"></i> Run Script
          </button>
        </div>
      </div>
      <div class="ps-container">
        <div id="ps-editor" class="editor-frame"></div>
        <div id="ps-output" class="output-frame"></div>
      </div>
    </div>
  `;
}

export function renderCliHelpView(): string {
  return `
    <div class="cli-help-view">
      <h2>Beefcake CLI Reference</h2>
      <p>Beefcake can be used from the command line for automated pipelines.</p>
      
      <div class="cli-command">
        <h3><i class="ph ph-command"></i> import</h3>
        <p>Import a data file into a PostgreSQL database.</p>
        <pre>beefcake import --file ./data.csv --table my_table --db-url postgres://user:pass@host/db</pre>
        <ul>
          <li><code>--file, -f</code>: Path to CSV, Parquet, or JSON file</li>
          <li><code>--table, -t</code>: Target table name</li>
          <li><code>--schema</code>: Target schema (default: public)</li>
          <li><code>--clean</code>: Apply heuristic cleaning before import</li>
        </ul>
      </div>

      <div class="cli-command">
        <h3><i class="ph ph-command"></i> export</h3>
        <p>Convert data between formats (CSV, Parquet, JSON).</p>
        <pre>beefcake export --input ./raw.csv --output ./processed.parquet</pre>
        <ul>
          <li><code>--input, -i</code>: Path to input file</li>
          <li><code>--output, -o</code>: Path to output file</li>
          <li><code>--clean</code>: Apply heuristic cleaning</li>
        </ul>
      </div>

      <div class="cli-command">
        <h3><i class="ph ph-command"></i> clean</h3>
        <p>Clean a file and save the result.</p>
        <pre>beefcake clean --file ./dirty.csv --output ./clean.parquet</pre>
        <ul>
          <li><code>--file, -f</code>: Path to the input file</li>
          <li><code>--output, -o</code>: Path to the output file</li>
          <li><code>--config, -c</code>: Optional JSON cleaning configuration file</li>
        </ul>
      </div>

      <div class="cli-command">
        <h3><i class="ph ph-sparkle"></i> Cleaning Reference</h3>
        <p>When using the <code>--clean</code> flag or <code>clean</code> command, Beefcake performs these operations:</p>
        <div class="cleaning-grid">
          <div class="cleaning-item">
            <strong>Text Processing</strong>
            <p>Trims whitespace, removes special characters, standardizes NULL values, and handles case conversion.</p>
          </div>
          <div class="cleaning-item">
            <strong>Smart Casting</strong>
            <p>Automatically detects and converts data to Numeric, Boolean, or Temporal types with timezone support.</p>
          </div>
          <div class="cleaning-item">
            <strong>Imputation</strong>
            <p>Handles missing values by filling them with Mean, Median, Mode, Zero, or custom constants.</p>
          </div>
          <div class="cleaning-item">
            <strong>Refinement</strong>
            <p>Applies rounding, outlier clipping, and normalization (Min-Max or Z-Score).</p>
          </div>
          <div class="cleaning-item">
            <strong>Categorical</strong>
            <p>Groups rare values into "Other" and supports One-Hot Encoding for ML pipelines.</p>
          </div>
        </div>
      </div>
    </div>
  `;
}

export function renderSettingsView(config: AppConfig | null): string {
  if (!config) return '<div class="loading">Loading configuration...</div>';

  return `
    <div class="settings-view">
      <h2>System Settings</h2>
      
      <div class="settings-section">
        <h3>CLI Defaults</h3>
        <p class="section-desc">Assign default database connections for Beefcake CLI operations.</p>
        <div class="settings-grid">
          <label>Default Import</label>
          <select id="select-import-id" class="settings-select">
            <option value="">None (Specify via --db-url)</option>
            ${config.connections.map(c => `
              <option value="${escapeHtml(c.id)}" ${config.active_import_id === c.id ? 'selected' : ''}>${escapeHtml(c.name)}</option>
            `).join('')}
          </select>

          <label>Default Export</label>
          <select id="select-export-id" class="settings-select">
            <option value="">None (Specify via --db-url)</option>
            ${config.connections.map(c => `
              <option value="${escapeHtml(c.id)}" ${config.active_export_id === c.id ? 'selected' : ''}>${escapeHtml(c.name)}</option>
            `).join('')}
          </select>
        </div>
      </div>

      <div class="settings-section">
        <h3>Database Connections</h3>
        <div class="connection-list">
          ${config.connections.map(conn => `
            <div class="conn-item">
              <div class="conn-info">
                <strong>${escapeHtml(conn.name)}</strong>
                <span>${escapeHtml(conn.settings.user)}@${escapeHtml(conn.settings.host)}:${escapeHtml(conn.settings.port)}/${escapeHtml(conn.settings.database)}</span>
              </div>
              <div class="conn-actions">
                <button class="btn-icon btn-delete-conn" data-id="${escapeHtml(conn.id)}" title="Delete Connection">
                  <i class="ph ph-trash"></i>
                </button>
              </div>
            </div>
          `).join('')}
          <button id="btn-add-conn" class="btn-secondary">
            <i class="ph ph-plus"></i> Add New Connection
          </button>
        </div>
      </div>
    </div>
  `;
}

export function renderToast(message: string, type: 'info' | 'error' | 'success' = 'info'): string {
  return `
    <div class="toast ${type}">
      <i class="ph ph-${type === 'error' ? 'warning-circle' : (type === 'success' ? 'check-circle' : 'info')}"></i>
      <span>${escapeHtml(message)}</span>
    </div>
  `;
}

export function renderLayout(): string {
  return `
    <div class="layout">
      <aside class="sidebar">
        <div class="sidebar-logo">
          <i class="ph ph-cake"></i>
          beefcake
        </div>
        <nav>
          <button class="nav-item active" data-view="Dashboard">
            <i class="ph ph-layout"></i> Dashboard
          </button>
          <button class="nav-item" data-view="Analyser">
            <i class="ph ph-chart-bar"></i> Analyser
          </button>
          <button class="nav-item" data-view="PowerShell">
            <i class="ph ph-terminal"></i> PowerShell
          </button>
          <button class="nav-item" data-view="Settings">
            <i class="ph ph-gear"></i> Settings
          </button>
          <button class="nav-item" data-view="CLI">
            <i class="ph ph-command"></i> CLI Help
          </button>
        </nav>
        <div class="sidebar-footer">
          <div class="status-indicator">
            <div class="status-dot"></div>
            <span>System Ready</span>
          </div>
        </div>
      </aside>
      <div class="main-content">
        <header>
          <h2 id="view-title">Dashboard</h2>
        </header>
        <main id="main-content"></main>
      </div>
    </div>
    <div id="toast-container"></div>
  `;
}
