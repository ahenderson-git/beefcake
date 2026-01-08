import { AnalysisResponse, AppConfig, ColumnCleanConfig, ColumnSummary } from "./types";
import { escapeHtml, fmtBytes, fmtDuration } from "./utils";

export function renderDashboardView(state: any): string {
  return `
    <div class="dashboard">
      <div class="hero">
        <h1>beefcake <small>v0.1.0</small></h1>
        <p>Advanced Data Analysis & ETL Pipeline</p>
      </div>
      <div class="info-box">
        <div class="info-section">
          <h3>What is beefcake?</h3>
          <p>
            <strong>beefcake</strong> (v0.1.0) is a high-performance desktop application designed as an 
            <strong>Advanced Data Analysis and ETL (Extract, Transform, Load) Pipeline</strong>. 
            Built with <strong>Tauri</strong>, it leverages the speed of <strong>Rust</strong> and <strong>Polars</strong> 
            to provide a robust environment for inspecting, cleaning, and moving data from local files into production-ready databases.
          </p>
        </div>
        
        <div class="info-grid">
          <div class="info-item">
            <strong><i class="ph ph-stethoscope"></i> Data Profiling</strong>
            <span>Automatic health scores, risk identification, and detailed column statistics.</span>
          </div>
          <div class="info-item">
            <strong><i class="ph ph-magic-wand"></i> Smart Cleaning</strong>
            <span>Interactive tools for normalization, imputation, case conversion, and encoding.</span>
          </div>
          <div class="info-item">
            <strong><i class="ph ph-database"></i> Seamless ETL</strong>
            <span>Push cleaned data directly to PostgreSQL with high-speed COPY commands.</span>
          </div>
          <div class="info-item">
            <strong><i class="ph ph-brain"></i> ML Insights</strong>
            <span>Train predictive models (Regression, Trees) directly on your analyzed datasets.</span>
          </div>
        </div>
        
        <div class="info-footer">
          <p><i class="ph ph-terminal-window"></i> <strong>Technical Foundation:</strong> High-performance processing powered by Rust & Polars DataFrames.</p>
        </div>
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
  const healthScore = Math.round(response.health.score * 100);
  const healthClass = healthScore > 80 ? 'ok' : (healthScore > 50 ? 'warn' : 'error');

  return `
    <div class="analyser-header">
      <div class="file-info">
        <div class="title-row">
          <h2>${escapeHtml(response.file_name)}</h2>
          <div class="health-score-badge ${healthClass}" title="${response.health.risks.join('\n')}">
            <i class="ph ph-heartbeat"></i>
            Health: ${healthScore}%
          </div>
        </div>
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

export function renderAnalyser(response: AnalysisResponse, expandedRows: Set<string>, configs: Map<string, ColumnCleanConfig>): string {
  const allActive = Array.from(configs.values()).every(c => c.active);

  return `
    <div class="analyser-view">
      <div id="analyser-header-container"></div>
      <div class="table-container">
        <table class="analyser-table">
          <thead>
            <tr>
              <th class="col-active">
                <input type="checkbox" class="header-action" data-action="active-all" title="Toggle all active" ${allActive ? 'checked' : ''}>
              </th>
              <th class="col-name">Column</th>
              <th class="col-stats">Stats</th>
              <th class="col-impute">
                Imputation
                <select class="header-action" data-action="impute-all">
                  <option value="">Set all...</option>
                  <option value="None">None</option>
                  <option value="Mean">Mean</option>
                  <option value="Median">Median</option>
                  <option value="Zero">Zero</option>
                  <option value="Mode">Mode</option>
                </select>
              </th>
              <th class="col-round">
                Rounding
                <select class="header-action" data-action="round-all">
                  <option value="">Set all...</option>
                  <option value="none">None</option>
                  <option value="0">0</option>
                  <option value="1">1</option>
                  <option value="2">2</option>
                  <option value="3">3</option>
                  <option value="4">4</option>
                </select>
              </th>
              <th class="col-norm">
                Normalization
                <select class="header-action" data-action="norm-all">
                  <option value="">Set all...</option>
                  <option value="None">None</option>
                  <option value="ZScore">Z-Score</option>
                  <option value="MinMax">Min-Max</option>
                </select>
              </th>
              <th class="col-case">
                Case
                <select class="header-action" data-action="case-all">
                  <option value="">Set all...</option>
                  <option value="None">None</option>
                  <option value="Lowercase">Lower</option>
                  <option value="Uppercase">Upper</option>
                  <option value="TitleCase">Title</option>
                </select>
              </th>
              <th class="col-onehot">
                One-Hot
                <input type="checkbox" class="header-action" data-action="onehot-all" title="Toggle all">
              </th>
            </tr>
          </thead>
          <tbody>
            ${response.summary.map(col => renderAnalyserRow(col, expandedRows.has(col.name), configs.get(col.name))).join('')}
          </tbody>
        </table>
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

export function renderAnalyserRow(col: ColumnSummary, isExpanded: boolean, config?: ColumnCleanConfig): string {
  const nullPct = ((col.nulls / col.count) * 100).toFixed(1);
  const healthClass = col.nulls > col.count * 0.1 ? 'warn' : 'ok';

  let statsHtml = '';
  if (col.stats.Numeric) {
    const s = col.stats.Numeric;
    statsHtml = `Min: ${s.min?.toFixed(2) ?? '—'} Max: ${s.max?.toFixed(2) ?? '—'} Mean: ${s.mean?.toFixed(2) ?? '—'}`;
  } else if (col.stats.Categorical) {
    statsHtml = `Categorical (${Object.keys(col.stats.Categorical).length} unique)`;
  } else if (col.stats.Temporal) {
    statsHtml = `Range: ${col.stats.Temporal.min} - ${col.stats.Temporal.max}`;
  } else if (col.stats.Boolean) {
    const s = col.stats.Boolean;
    statsHtml = `True: ${s.true_count} False: ${s.false_count}`;
  } else if (col.stats.Text) {
    const s = col.stats.Text;
    statsHtml = `Avg Len: ${s.avg_length.toFixed(1)}`;
  }

  const c = config || {} as ColumnCleanConfig;
  const isActive = c.active;

  return `
    <tr class="analyser-row ${isExpanded ? 'expanded' : ''} ${isActive ? '' : 'inactive'}" data-col="${escapeHtml(col.name)}">
      <td class="col-active">
        <input type="checkbox" class="row-action" data-col="${escapeHtml(col.name)}" data-prop="active" ${isActive ? 'checked' : ''}>
      </td>
      <td class="col-name">
        <div class="name-wrapper">
          <i class="ph ph-caret-${isExpanded ? 'down' : 'right'} expand-toggle"></i>
          <strong>${col.name}</strong>
          <span class="kind-tag">${col.kind}</span>
        </div>
      </td>
      <td class="col-stats">
        <div class="stats-mini">${statsHtml}</div>
        <div class="health-tag ${healthClass}">${nullPct}% nulls</div>
      </td>
      <td class="col-impute">
        <select class="row-action" data-col="${escapeHtml(col.name)}" data-prop="impute_mode" ${isActive ? '' : 'disabled'}>
          <option value="None" ${c.impute_mode === 'None' ? 'selected' : ''}>None</option>
          <option value="Mean" ${c.impute_mode === 'Mean' ? 'selected' : ''}>Mean</option>
          <option value="Median" ${c.impute_mode === 'Median' ? 'selected' : ''}>Median</option>
          <option value="Zero" ${c.impute_mode === 'Zero' ? 'selected' : ''}>Zero</option>
          <option value="Mode" ${c.impute_mode === 'Mode' ? 'selected' : ''}>Mode</option>
        </select>
      </td>
      <td class="col-round">
        <select class="row-action" data-col="${escapeHtml(col.name)}" data-prop="rounding" ${isActive ? '' : 'disabled'}>
          <option value="none" ${c.rounding === null ? 'selected' : ''}>None</option>
          <option value="0" ${c.rounding === 0 ? 'selected' : ''}>0</option>
          <option value="1" ${c.rounding === 1 ? 'selected' : ''}>1</option>
          <option value="2" ${c.rounding === 2 ? 'selected' : ''}>2</option>
          <option value="3" ${c.rounding === 3 ? 'selected' : ''}>3</option>
          <option value="4" ${c.rounding === 4 ? 'selected' : ''}>4</option>
        </select>
      </td>
      <td class="col-norm">
        <select class="row-action" data-col="${escapeHtml(col.name)}" data-prop="normalization" ${isActive ? '' : 'disabled'}>
          <option value="None" ${c.normalization === 'None' ? 'selected' : ''}>None</option>
          <option value="ZScore" ${c.normalization === 'ZScore' ? 'selected' : ''}>Z-Score</option>
          <option value="MinMax" ${c.normalization === 'MinMax' ? 'selected' : ''}>Min-Max</option>
        </select>
      </td>
      <td class="col-case">
        <select class="row-action" data-col="${escapeHtml(col.name)}" data-prop="text_case" ${isActive ? '' : 'disabled'}>
          <option value="None" ${c.text_case === 'None' ? 'selected' : ''}>None</option>
          <option value="Lowercase" ${c.text_case === 'Lowercase' ? 'selected' : ''}>Lower</option>
          <option value="Uppercase" ${c.text_case === 'Uppercase' ? 'selected' : ''}>Upper</option>
          <option value="TitleCase" ${c.text_case === 'TitleCase' ? 'selected' : ''}>Title</option>
        </select>
      </td>
      <td class="col-onehot">
        <input type="checkbox" class="row-action" data-col="${escapeHtml(col.name)}" data-prop="one_hot_encode" ${c.one_hot_encode ? 'checked' : ''} ${isActive ? '' : 'disabled'}>
      </td>
    </tr>
    ${isExpanded ? `
      <tr class="details-row ${isActive ? '' : 'inactive'}">
        <td colspan="8">
          ${renderExpandedDetails(col)}
        </td>
      </tr>
    ` : ''}
  `;
}

function renderExpandedDetails(col: ColumnSummary): string {
  let detailsHtml = '';

  if (col.stats.Numeric) {
    const s = col.stats.Numeric;
    detailsHtml = `
      <div class="details-grid-wrapper">
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
        ${s.histogram ? `
          <div class="histogram-container">
            <canvas id="chart-${col.name}"></canvas>
          </div>
        ` : ''}
      </div>
    `;
  } else if (col.stats.Temporal) {
    const s = col.stats.Temporal;
    detailsHtml = `
      <div class="details-grid-wrapper">
        <div class="details-grid">
          <div class="detail-item">
            <label>Min</label>
            <span>${s.min ?? '—'}</span>
          </div>
          <div class="detail-item">
            <label>Max</label>
            <span>${s.max ?? '—'}</span>
          </div>
          <div class="detail-item">
            <label>Distinct</label>
            <span>${s.distinct_count}</span>
          </div>
          <div class="detail-item">
            <label>Sorted</label>
            <span>${s.is_sorted ? 'Yes' : (s.is_sorted_rev ? 'Reverse' : 'No')}</span>
          </div>
        </div>
        ${s.histogram ? `
          <div class="histogram-container">
            <canvas id="chart-${col.name}"></canvas>
          </div>
        ` : ''}
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

export function renderPowerShellView(fontSize: number): string {
  return `
    <div class="powershell-view">
      <div class="ps-header">
        <div class="ps-title">
          <i class="ph ph-terminal"></i>
          PowerShell Core
        </div>
        <div class="ps-actions">
          <div class="ps-font-controls">
            <button id="btn-dec-font" class="btn-icon" title="Decrease Font Size">
              <i class="ph ph-minus"></i>
            </button>
            <span id="ps-font-size-label">${fontSize}</span>
            <button id="btn-inc-font" class="btn-icon" title="Increase Font Size">
              <i class="ph ph-plus"></i>
            </button>
          </div>
          <div class="ps-divider"></div>
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
            <p>Trims whitespace, removes special characters (including non-ASCII), standardizes NULL values, and extracts numeric values from strings.</p>
          </div>
          <div class="cleaning-item">
            <strong>Smart Casting</strong>
            <p>Automatically detects and converts data to Numeric, Boolean, or Temporal (Date/Time) types with custom format and UTC support.</p>
          </div>
        </div>
        <p style="margin-top: var(--spacing-medium); font-size: 0.85rem; color: #888; font-style: italic;">
          Note: Advanced features like case conversion, imputation, rounding, normalization, and One-Hot encoding are available via the GUI only.
        </p>
      </div>
    </div>
  `;
}

export function renderSettingsView(config: AppConfig | null, isAddingConnection: boolean = false): string {
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
                <button class="btn-icon btn-test-conn" data-id="${escapeHtml(conn.id)}" title="Test Connection">
                  <i class="ph ph-plugs-connected"></i>
                </button>
                <button class="btn-icon btn-delete-conn" data-id="${escapeHtml(conn.id)}" title="Delete Connection">
                  <i class="ph ph-trash"></i>
                </button>
              </div>
            </div>
          `).join('')}
          
          ${isAddingConnection ? `
            <div class="add-conn-form">
              <h4>Add New Connection</h4>
              <div class="settings-grid">
                <label>Connection Name</label>
                <input type="text" id="new-conn-name" class="settings-input" placeholder="e.g. Local Postgres" />
                
                <label>Host</label>
                <input type="text" id="new-conn-host" class="settings-input" value="localhost" />
                
                <label>Port</label>
                <input type="text" id="new-conn-port" class="settings-input" value="5432" />
                
                <label>User</label>
                <input type="text" id="new-conn-user" class="settings-input" value="postgres" />
                
                <label>Password</label>
                <input type="password" id="new-conn-pass" class="settings-input" />
                
                <label>Database</label>
                <input type="text" id="new-conn-db" class="settings-input" />
              </div>
              <div class="form-actions">
                <button id="btn-save-new-conn" class="primary">Save Connection</button>
                <button id="btn-test-new-conn">
                  <i class="ph ph-plugs-connected"></i> Test Connection
                </button>
                <button id="btn-cancel-new-conn">Cancel</button>
              </div>
            </div>
          ` : `
            <button id="btn-add-conn" class="btn-secondary">
              <i class="ph ph-plus"></i> Add New Connection
            </button>
          `}
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

export function renderActivityLogView(config: AppConfig | null): string {
  if (!config) return '<div class="loading">Loading activity log...</div>';

  return `
    <div class="activity-log-view">
      <div class="view-header">
        <h2>Activity Log</h2>
        <button id="btn-clear-log" class="btn-secondary btn-small">
          <i class="ph ph-trash"></i> Clear Log
        </button>
      </div>
      
      <div class="log-container">
        ${config.audit_log.length === 0 ? `
          <div class="empty-state">
            <i class="ph ph-clock-counter-clockwise"></i>
            <p>No activity recorded yet.</p>
          </div>
        ` : `
          <div class="log-list">
            ${config.audit_log.slice().reverse().map(entry => {
              const date = new Date(entry.timestamp);
              const timeStr = date.toLocaleTimeString();
              const dateStr = date.toLocaleDateString();
              return `
                <div class="log-entry">
                  <div class="log-time" title="${dateStr} ${timeStr}">
                    ${timeStr}
                  </div>
                  <div class="log-action">${escapeHtml(entry.action)}</div>
                  <div class="log-details">${escapeHtml(entry.details)}</div>
                </div>
              `;
            }).join('')}
          </div>
        `}
      </div>
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
          <button class="nav-item" data-view="ActivityLog">
            <i class="ph ph-clock-counter-clockwise"></i> Activity Log
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
