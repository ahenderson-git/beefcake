import { AnalysisResponse, AppConfig, ColumnCleanConfig, ColumnSummary, DbConnection, ExportSource } from "./types";
import { escapeHtml, fmtBytes, fmtDuration } from "./utils";

const IMPUTE_OPTIONS = [
  { value: 'None', label: 'None' },
  { value: 'Mean', label: 'Mean' },
  { value: 'Median', label: 'Median' },
  { value: 'Zero', label: 'Zero' },
  { value: 'Mode', label: 'Mode' },
];

const NORM_OPTIONS = [
  { value: 'None', label: 'None' },
  { value: 'ZScore', label: 'Z-Score' },
  { value: 'MinMax', label: 'Min-Max' },
];

const CASE_OPTIONS = [
  { value: 'None', label: 'None' },
  { value: 'Lowercase', label: 'Lower' },
  { value: 'Uppercase', label: 'Upper' },
  { value: 'TitleCase', label: 'Title' },
];

const ROUND_OPTIONS = [
  { value: 'none', label: 'None' },
  { value: '0', label: '0' },
  { value: '1', label: '1' },
  { value: '2', label: '2' },
  { value: '3', label: '3' },
  { value: '4', label: '4' },
];

function renderSelect(options: { value: string, label: string }[], selectedValue: string, className: string, dataAttrs: Record<string, string>, placeholder?: string, disabled?: boolean): string {
  const attrs = Object.entries(dataAttrs).map(([k, v]) => `data-${k}="${escapeHtml(v)}"`).join(' ');
  const placeholderHtml = placeholder ? `<option value="">${escapeHtml(placeholder)}</option>` : '';
  return `
    <select class="${className}" ${attrs} ${disabled ? 'disabled' : ''}>
      ${placeholderHtml}
      ${options.map(opt => `
        <option value="${escapeHtml(opt.value)}" ${opt.value === selectedValue ? 'selected' : ''}>${escapeHtml(opt.label)}</option>
      `).join('')}
    </select>
  `;
}

export function renderDashboardView(state: any): string {
  return `
    <div class="dashboard">
      <div class="hero">
        <h1>beefcake <small>v${state.version}</small></h1>
        <p>Developed by Anthony Henderson</p>
      </div>
      <div class="info-box">
        <div class="info-section">
          <h3>What is beefcake?</h3>
          <p>
            <strong>beefcake</strong> (v${state.version}) is a high-performance desktop application designed as an 
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
          <div class="stat-value">${state.analysisResponse ? escapeHtml(state.analysisResponse.file_name) : 'None'}</div>
          <p>${state.analysisResponse ? fmtBytes(state.analysisResponse.file_size) : 'Ready for input'}</p>
        </div>
      </div>
      <div class="actions">
        <button id="btn-open-file" class="btn-primary">
          <i class="ph ph-cloud-arrow-up"></i> Analyze New Dataset
        </button>
        <button id="btn-powershell" class="btn-secondary">
          <i class="ph ph-terminal"></i> PowerShell Console
        </button>
        <button id="btn-python" class="btn-secondary">
          <i class="ph ph-code"></i> Python IDE
        </button>
        <button id="btn-sql" class="btn-secondary">
          <i class="ph ph-database"></i> SQL IDE
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
          <span class="tag">${response.total_row_count.toLocaleString()} rows ${response.row_count < response.total_row_count ? `(sampled ${response.row_count.toLocaleString()})` : ''}</span>
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
        <button id="btn-open" class="btn-secondary">
          <i class="ph ph-cloud-arrow-up"></i> Load Dataset
        </button>
        <button id="btn-export-analyser" class="btn-primary">
          <i class="ph ph-export"></i> Export
        </button>
      </div>
    </div>
  `;
}

export function renderAnalyser(response: AnalysisResponse, expandedRows: Set<string>, configs: Record<string, ColumnCleanConfig>): string {
  const allActive = Object.values(configs).every(c => c.active);

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
              <th class="col-name">
                Column
                <button class="header-action-icon" data-action="standardize-all" title="Standardize all column names">
                  <i class="ph ph-magic-wand"></i>
                </button>
              </th>
              <th class="col-stats">Stats</th>
              <th class="col-impute">
                Imputation
                ${renderSelect(IMPUTE_OPTIONS, '', 'header-action', { action: 'impute-all' }, 'Set all...')}
              </th>
              <th class="col-round">
                Rounding
                ${renderSelect(ROUND_OPTIONS, '', 'header-action', { action: 'round-all' }, 'Set all...')}
              </th>
              <th class="col-norm">
                Normalization
                ${renderSelect(NORM_OPTIONS, '', 'header-action', { action: 'norm-all' }, 'Set all...')}
              </th>
              <th class="col-case">
                Case
                ${renderSelect(CASE_OPTIONS, '', 'header-action', { action: 'case-all' }, 'Set all...')}
              </th>
              <th class="col-onehot">
                One-Hot
                <input type="checkbox" class="header-action" data-action="onehot-all" title="Toggle all">
              </th>
            </tr>
          </thead>
          <tbody>
            ${response.summary.map(col => renderAnalyserRow(col, expandedRows.has(col.name), configs[col.name])).join('')}
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
      <p>Select a file to begin analysis and cleaning.</p>
      <button id="btn-open" class="btn-primary">
          <i class="ph ph-cloud-arrow-up"></i> Load Dataset
      </button>
    </div>
  `;
}

export function renderLoading(message: string = "Analyzing dataset..."): string {
  return `
    <div class="loading">
      <div class="spinner"></div>
      <p>${escapeHtml(message)}</p>
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
          <div class="name-edit-group">
            <input type="text" class="row-action new-name-input" 
                   data-col="${escapeHtml(col.name)}" data-prop="new_name" 
                   value="${escapeHtml(c.new_name || col.name)}" 
                   ${isActive ? '' : 'disabled'}>
            <span class="original-name-label" title="Original: ${escapeHtml(col.name)}">${escapeHtml(col.name)}</span>
          </div>
          <span class="kind-tag">${col.kind}</span>
        </div>
      </td>
      <td class="col-stats">
        <div class="stats-mini">${statsHtml}</div>
        <div class="health-tag ${healthClass}">${nullPct}% nulls</div>
      </td>
      <td class="col-impute">
        ${renderSelect(IMPUTE_OPTIONS, c.impute_mode, 'row-action', { col: col.name, prop: 'impute_mode' }, undefined, !isActive)}
      </td>
      <td class="col-round">
        ${renderSelect(ROUND_OPTIONS, (c.rounding ?? 'none').toString(), 'row-action', { col: col.name, prop: 'rounding' }, undefined, !isActive)}
      </td>
      <td class="col-norm">
        ${renderSelect(NORM_OPTIONS, c.normalization, 'row-action', { col: col.name, prop: 'normalization' }, undefined, !isActive)}
      </td>
      <td class="col-case">
        ${renderSelect(CASE_OPTIONS, c.text_case, 'row-action', { col: col.name, prop: 'text_case' }, undefined, !isActive)}
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
          <button id="btn-clear-ps" class="btn-secondary">
            <i class="ph ph-trash"></i> Clear
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

export function renderPythonView(state: any): string {
  const fontSize = state.config?.python_font_size || 14;
  return `
    <div class="python-view">
      <div class="py-header">
        <div class="py-title">
          <i class="ph ph-code"></i>
          Python IDE
        </div>
        <div class="py-actions">
          <div class="py-font-controls">
            <button id="btn-dec-font-py" class="btn-icon" title="Decrease Font Size">
              <i class="ph ph-minus"></i>
            </button>
            <span id="py-font-size-label">${fontSize}</span>
            <button id="btn-inc-font-py" class="btn-icon" title="Increase Font Size">
              <i class="ph ph-plus"></i>
            </button>
          </div>
          <div class="py-divider"></div>
          <button id="btn-load-py" class="btn-secondary">
            <i class="ph ph-folder-open"></i> Load
          </button>
          <button id="btn-save-py" class="btn-secondary">
            <i class="ph ph-floppy-disk"></i> Save
          </button>
          <button id="btn-install-polars" class="btn-secondary" title="Install Polars Library">
            <i class="ph ph-package"></i> Install Polars
          </button>
          <button id="btn-clear-py" class="btn-secondary">
            <i class="ph ph-trash"></i> Clear
          </button>
          <button id="btn-export-py" class="btn-secondary" title="Export 'df' variable">
            <i class="ph ph-export"></i> Export
          </button>
          <button id="btn-run-py" class="btn-primary">
            <i class="ph ph-play"></i> Run Script
          </button>
        </div>
      </div>
      <div class="py-main-layout">
        <div class="py-container">
          <div id="py-editor" class="editor-frame"></div>
          <div id="py-output" class="output-frame"></div>
        </div>
        ${renderSchemaSidebar(state.analysisResponse, state.cleaningConfigs)}
      </div>
    </div>
  `;
}

export function renderSQLView(state: any): string {
  const fontSize = state.config?.sql_font_size || 14;
  return `
    <div class="sql-view">
      <div class="sql-header">
        <div class="sql-title">
          <i class="ph ph-database"></i>
          SQL IDE
        </div>
        <div class="sql-actions">
          <div class="sql-font-controls">
            <button id="btn-dec-font-sql" class="btn-icon" title="Decrease Font Size">
              <i class="ph ph-minus"></i>
            </button>
            <span id="sql-font-size-label">${fontSize}</span>
            <button id="btn-inc-font-sql" class="btn-icon" title="Increase Font Size">
              <i class="ph ph-plus"></i>
            </button>
          </div>
          <div class="sql-divider"></div>
          <button id="btn-load-sql" class="btn-secondary">
            <i class="ph ph-folder-open"></i> Load
          </button>
          <button id="btn-save-sql" class="btn-secondary">
            <i class="ph ph-floppy-disk"></i> Save
          </button>
          <button id="btn-sql-docs" class="btn-secondary" title="View SQL Documentation">
            <i class="ph ph-book-open"></i> Docs
          </button>
          <button id="btn-clear-sql" class="btn-secondary">
            <i class="ph ph-trash"></i> Clear
          </button>
          <button id="btn-export-sql" class="btn-secondary" title="Export current results">
            <i class="ph ph-export"></i> Export
          </button>
          <button id="btn-run-sql" class="btn-primary">
            <i class="ph ph-play"></i> Run Query
          </button>
        </div>
      </div>
      <div class="sql-main-layout">
        <div class="sql-container">
          <div id="sql-editor" class="editor-frame"></div>
          <div id="sql-output" class="output-frame"></div>
        </div>
        ${renderSchemaSidebar(state.analysisResponse, state.cleaningConfigs)}
      </div>
    </div>
  `;
}

export function renderSchemaSidebar(response: AnalysisResponse | null, configs: Record<string, ColumnCleanConfig> = {}): string {
  if (!response) {
    return `
      <div class="schema-sidebar empty">
        <div class="sidebar-header">Dataset Schema</div>
        <div class="empty-msg">
          <i class="ph ph-file-search"></i>
          <p>No dataset loaded. Load a file in the Analyser first.</p>
        </div>
      </div>
    `;
  }

  return `
    <div class="schema-sidebar">
      <div class="sidebar-header">Dataset Schema</div>
      <div class="column-list">
        ${response.summary.map(col => {
          const config = configs[col.name];
          const displayName = config?.new_name || col.name;
          const isActive = config ? config.active : true;
          
          if (!isActive) {
            return `
              <div class="column-item inactive" title="${escapeHtml(col.name)} (Will be dropped)">
                <div class="col-info">
                  <span class="col-name">${escapeHtml(displayName)}</span>
                  <span class="col-type">Dropped</span>
                </div>
              </div>
            `;
          }

          return `
            <div class="column-item" title="${escapeHtml(col.name)} (${col.kind})">
              <div class="col-info">
                <span class="col-name">${escapeHtml(displayName)}</span>
                <span class="col-type">${escapeHtml(col.kind)}</span>
              </div>
              <button class="btn-insert-col" data-col="${escapeHtml(displayName)}" title="Insert column name">
                <i class="ph ph-arrow-square-in"></i>
              </button>
            </div>
          `;
        }).join('')}
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

export function renderExportModal(
  source: ExportSource,
  connections: DbConnection[],
  activeExportId: string | null | undefined,
  destType: 'File' | 'Database',
  isLoading: boolean = false
): string {
  const isFile = destType === 'File';
  const isDb = destType === 'Database';

  return `
    <div id="export-modal" class="modal-overlay">
      <div class="modal-content export-modal">
        <div class="modal-header">
          <h3><i class="ph ph-export"></i> Export Data</h3>
          <button id="btn-close-export" class="btn-icon" ${isLoading ? 'disabled' : ''}><i class="ph ph-x"></i></button>
        </div>
        <div class="modal-body">
          ${isLoading ? `
            <div class="export-loading">
              <div class="spinner"></div>
              <p>Processing and exporting data...</p>
              <p class="help-text">This may take a moment for large datasets.</p>
            </div>
          ` : `
            <div class="export-source-info">
              <label>Source:</label>
              <span>${source.type} ${source.type === 'Analyser' ? `(${escapeHtml(source.path?.split(/[\\/]/).pop() || '')})` : ''}</span>
            </div>

            <div class="export-dest-toggle">
              <label class="toggle-option">
                <input type="radio" name="dest-type" value="File" ${isFile ? 'checked' : ''}>
                <div class="toggle-card">
                  <i class="ph ph-file-arrow-down"></i>
                  <span>To File</span>
                </div>
              </label>
              <label class="toggle-option">
                <input type="radio" name="dest-type" value="Database" ${isDb ? 'checked' : ''}>
                <div class="toggle-card">
                  <i class="ph ph-database"></i>
                  <span>To Database</span>
                </div>
              </label>
            </div>

            <div class="export-config">
              ${isFile ? `
                <div class="config-group">
                  <label for="export-format">File Format</label>
                  <select id="export-format" class="form-control">
                    <option value="csv">CSV (Comma Separated Values)</option>
                    <option value="parquet">Parquet (Apache Parquet)</option>
                    <option value="json">JSON (JavaScript Object Notation)</option>
                  </select>
                </div>
              ` : `
                <div class="config-group">
                  <label for="export-connection">Database Connection</label>
                  <select id="export-connection" class="form-control">
                    <option value="">-- Select Connection --</option>
                    ${connections.map(conn => `
                      <option value="${conn.id}" ${conn.id === activeExportId ? 'selected' : ''}>
                        ${escapeHtml(conn.name)} (${escapeHtml(conn.settings.database)}.${escapeHtml(conn.settings.table)})
                      </option>
                    `).join('')}
                  </select>
                  <p class="help-text">Select a connection configured in Settings.</p>
                </div>
              `}
            </div>
          `}
        </div>
        <div class="modal-footer">
          <button id="btn-confirm-export" class="btn-primary" ${isLoading ? 'disabled' : ''}>
            <i class="ph ph-check"></i> ${isLoading ? 'Exporting...' : 'Export Now'}
          </button>
        </div>
      </div>
    </div>
  `;
}

export function renderReferenceView(): string {
  return `
    <div class="reference-view">
      <div class="reference-header">
        <h2>Reference Material</h2>
        <p>A collection of useful resources for data engineering and Rust development.</p>
      </div>

      <div class="reference-grid">
        <!-- Rust Section -->
        <section class="ref-section">
          <h3><i class="ph ph-read-cv-logo"></i> Rust Programming</h3>
          <div class="ref-links">
            <a href="https://doc.rust-lang.org/book/" target="_blank" class="ref-link">
              <div class="link-icon"><i class="ph ph-book"></i></div>
              <div class="link-content">
                <strong>The Rust Book</strong>
                <span>The definitive guide to Rust.</span>
              </div>
            </a>
            <a href="https://doc.rust-lang.org/rust-by-example/" target="_blank" class="ref-link">
              <div class="link-icon"><i class="ph ph-code"></i></div>
              <div class="link-content">
                <strong>Rust by Example</strong>
                <span>Learn Rust through annotated examples.</span>
              </div>
            </a>
          </div>
        </section>

        <!-- Polars Section -->
        <section class="ref-section">
          <h3><i class="ph ph-lightning"></i> Data Analysis (Polars)</h3>
          <div class="ref-links">
            <a href="https://docs.pola.rs/user-guide/" target="_blank" class="ref-link">
              <div class="link-icon"><i class="ph ph-map-trifold"></i></div>
              <div class="link-content">
                <strong>Polars User Guide</strong>
                <span>Concepts and usage patterns.</span>
              </div>
            </a>
            <a href="https://docs.rs/polars/latest/polars/" target="_blank" class="ref-link">
              <div class="link-icon"><i class="ph ph-file-text"></i></div>
              <div class="link-content">
                <strong>Polars API (Rust)</strong>
                <span>Detailed Rust documentation.</span>
              </div>
            </a>
          </div>
        </section>

        <!-- Tauri & Frontend -->
        <section class="ref-section">
          <h3><i class="ph ph-desktop"></i> App Development</h3>
          <div class="ref-links">
            <a href="https://tauri.app/" target="_blank" class="ref-link">
              <div class="link-icon"><i class="ph ph-browser"></i></div>
              <div class="link-content">
                <strong>Tauri Docs</strong>
                <span>Framework for tiny, fast binaries.</span>
              </div>
            </a>
            <a href="https://www.typescriptlang.org/docs/" target="_blank" class="ref-link">
              <div class="link-icon"><i class="ph ph-scroll"></i></div>
              <div class="link-content">
                <strong>TypeScript Handbook</strong>
                <span>The language for this frontend.</span>
              </div>
            </a>
          </div>
        </section>

        <!-- Training -->
        <section class="ref-section">
          <h3><i class="ph ph-graduation-cap"></i> Training & Courses</h3>
          <div class="ref-links">
            <a href="https://www.udemy.com/course/learn-to-code-with-rust/" target="_blank" class="ref-link">
              <div class="link-icon"><i class="ph ph-video"></i></div>
              <div class="link-content">
                <strong>Udemy: Rust Course</strong>
                <span>Learn to Code with Rust.</span>
              </div>
            </a>
          </div>
        </section>

        <!-- Tools -->
        <section class="ref-section">
          <h3><i class="ph ph-wrench"></i> Inspiration & Tools</h3>
          <div class="ref-links">
            <a href="https://phosphoricons.com/" target="_blank" class="ref-link">
              <div class="link-icon"><i class="ph ph-paint-brush"></i></div>
              <div class="link-content">
                <strong>Phosphor Icons</strong>
                <span>The icon library used in beefcake.</span>
              </div>
            </a>
            <a href="https://github.com/rust-unofficial/awesome-rust" target="_blank" class="ref-link">
              <div class="link-icon"><i class="ph ph-star"></i></div>
              <div class="link-content">
                <strong>Awesome Rust</strong>
                <span>A curated list of Rust resources.</span>
              </div>
            </a>
          </div>
        </section>
      </div>

      <div class="reference-content">
        <div class="content-card">
          <h3><i class="ph ph-chart-line"></i> Understanding Data Skewness</h3>
          <div class="skew-grid">
            <div class="skew-item">
              <strong>Right Skew (Positive)</strong>
              <p>The mean is greater than the median. High-value outliers pull the average up.</p>
            </div>
            <div class="skew-item">
              <strong>Left Skew (Negative)</strong>
              <p>The mean is less than the median. Low-value outliers pull the average down.</p>
            </div>
          </div>
        </div>

        <div class="content-card">
          <h3><i class="ph ph-magic-wand"></i> Preprocessing for Machine Learning</h3>
          <div class="ml-grid">
            <div class="ml-item">
              <h4>Normalization (Scaling)</h4>
              <ul>
                <li><strong>Min-Max:</strong> Rescales to [0, 1]. Best when bounds are known.</li>
                <li><strong>Z-Score:</strong> Mean=0, StdDev=1. Robust to outliers.</li>
              </ul>
            </div>
            <div class="ml-item">
              <h4>Categorical Encoding</h4>
              <ul>
                <li><strong>One-Hot:</strong> Binary columns for each category. Best for non-ordered data.</li>
                <li><strong>Label:</strong> Assigns integers. Better for ordered data (Ordinal).</li>
              </ul>
            </div>
            <div class="ml-item">
              <h4>Imputation (Handling Nulls)</h4>
              <ul>
                <li><strong>Mean/Median:</strong> Good for numeric fields.</li>
                <li><strong>Mode:</strong> Best for categorical fields.</li>
                <li><strong>Constant:</strong> When 'missing' has a business meaning.</li>
              </ul>
            </div>
          </div>
        </div>

        <div class="content-card">
          <h3><i class="ph ph-database"></i> PostgreSQL Export Guide</h3>
          <p>Beefcake handles metadata (file size, health score) and summaries automatically. When exporting to PostgreSQL:</p>
          <ul>
            <li>Column types are inferred and mapped to Postgres types.</li>
            <li>High-speed COPY commands are used for efficiency.</li>
            <li>Metadata and statistics are saved to the target database.</li>
          </ul>
        </div>
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
          <button class="nav-item" data-view="Python">
            <i class="ph ph-code"></i> Python IDE
          </button>
          <button class="nav-item" data-view="SQL">
            <i class="ph ph-database"></i> SQL IDE
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
          <button class="nav-item" data-view="Reference">
            <i class="ph ph-book"></i> Reference
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
        <main id="view-container"></main>
      </div>
    </div>
    <div id="toast-container"></div>
  `;
}
