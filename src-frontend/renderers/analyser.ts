import { AnalysisResponse, ColumnCleanConfig, ColumnSummary } from "../types";
import { escapeHtml, fmtBytes, fmtDuration } from "../utils";
import { CASE_OPTIONS, IMPUTE_OPTIONS, NORM_OPTIONS, renderSelect, ROUND_OPTIONS } from "./common";

export function renderAnalyserHeader(response: AnalysisResponse, trimPct: number): string {
  const isSampled = response.total_row_count > response.row_count;
  const rowDisplay = isSampled 
    ? `${response.total_row_count.toLocaleString()} rows <small>(Sampled ${response.row_count.toLocaleString()} for analysis)</small>`
    : `${response.row_count.toLocaleString()} rows`;

  return `
    <div class="analyser-header">
      <div class="header-main">
        <h2>${escapeHtml(response.file_name)} <small>(${fmtBytes(response.file_size)})</small></h2>
        <div class="meta-info">
          <span><i class="ph ph-rows"></i> ${rowDisplay}</span>
          <span><i class="ph ph-columns"></i> ${response.column_count} columns</span>
          <span><i class="ph ph-timer"></i> Analyzed in ${fmtDuration(response.analysis_duration)}</span>
        </div>
      </div>
      <div class="header-actions">
        <div class="trim-control">
          <label title="Lower/Upper percentage to ignore for outlier detection">Outlier Trim: <span id="trim-value">${Math.round(trimPct * 100)}%</span></label>
          <input type="range" id="trim-range" min="0" max="0.25" step="0.01" value="${trimPct}">
        </div>
        <button id="btn-open-file" class="btn-secondary btn-small">
          <i class="ph ph-file-plus"></i> Select File
        </button>
        <button id="btn-reanalyze" class="btn-secondary btn-small">Re-analyze</button>
        <button id="btn-export" class="btn-primary btn-small">
          <i class="ph ph-export"></i> Export / ETL
        </button>
      </div>
    </div>
    <div class="bulk-actions">
      <div class="bulk-group">
        <label><input type="checkbox" class="header-action" data-action="active-all" checked> All Active</label>
      </div>
      <div class="bulk-group">
        <label>Impute All:</label>
        ${renderSelect(IMPUTE_OPTIONS, 'None', 'header-action', { action: 'impute-all' }, 'Mixed')}
      </div>
      <div class="bulk-group">
        <label>Round All:</label>
        ${renderSelect(ROUND_OPTIONS, 'none', 'header-action', { action: 'round-all' }, 'Mixed')}
      </div>
      <div class="bulk-group">
        <label>Norm All:</label>
        ${renderSelect(NORM_OPTIONS, 'None', 'header-action', { action: 'norm-all' }, 'Mixed')}
      </div>
    </div>
  `;
}

export function renderAnalyser(response: AnalysisResponse, expandedRows: Set<string>, configs: Record<string, ColumnCleanConfig>): string {
  const healthScore = Math.round(response.health.score * 100);
  const healthClass = healthScore > 80 ? 'health-good' : healthScore > 50 ? 'health-warn' : 'health-poor';
  
  return `
    <div class="analyser-container">
      <div id="analyser-header-container"></div>
      
      <div class="health-banner ${healthClass}">
        <div class="health-score">
          <span class="score-label">Health Score</span>
          <span class="score-value">${healthScore}%</span>
        </div>
        <div class="health-issues">
          ${response.health.risks.length > 0 
            ? `<ul>${response.health.risks.map(issue => `<li><i class="ph ph-warning"></i> ${escapeHtml(issue)}</li>`).join('')}</ul>`
            : '<p><i class="ph ph-check-circle"></i> No critical health issues detected.</p>'}
        </div>
      </div>

      <table class="analyser-table">
        <thead>
          <tr>
            <th style="width: 30px;"></th>
            <th>Column</th>
            <th>Type</th>
            <th>Quality</th>
            <th>Mean / Mode</th>
            <th>Min</th>
            <th>Max</th>
            <th>Cleaning Options</th>
          </tr>
        </thead>
        <tbody>
          ${response.summary.map(col => renderAnalyserRow(col, expandedRows.has(col.name), configs[col.name])).join('')}
        </tbody>
      </table>
    </div>
  `;
}

export function renderEmptyAnalyser(): string {
  return `
    <div class="empty-state">
      <i class="ph ph-file-search"></i>
      <h3>No data analyzed yet</h3>
      <p>Select a file to begin advanced profiling and cleaning.</p>
      <button id="btn-open-file" class="btn-primary">
        <i class="ph ph-cloud-arrow-up"></i> Select File
      </button>
    </div>
  `;
}

function getUniqueCount(col: ColumnSummary): number {
  if (col.stats.Numeric) return col.stats.Numeric.distinct_count;
  if (col.stats.Temporal) return col.stats.Temporal.distinct_count;
  if (col.stats.Text) return col.stats.Text.distinct;
  if (col.stats.Categorical) return Object.keys(col.stats.Categorical).length;
  if (col.stats.Boolean) {
    let count = 0;
    if (col.stats.Boolean.true_count > 0) count++;
    if (col.stats.Boolean.false_count > 0) count++;
    return count;
  }
  return 0;
}

function getMeanOrMode(col: ColumnSummary): string {
  if (col.stats.Numeric) return col.stats.Numeric.mean?.toFixed(2) || 'N/A';
  if (col.stats.Text) return col.stats.Text.top_value ? col.stats.Text.top_value[0] : 'N/A';
  if (col.stats.Categorical) {
    const entries = Object.entries(col.stats.Categorical);
    if (entries.length === 0) return 'N/A';
    return entries.sort((a, b) => b[1] - a[1])[0][0];
  }
  if (col.stats.Boolean) return col.stats.Boolean.true_count >= col.stats.Boolean.false_count ? 'True' : 'False';
  return 'N/A';
}

function getMinMax(col: ColumnSummary): [string, string] {
  if (col.stats.Numeric) return [col.stats.Numeric.min?.toString() || 'N/A', col.stats.Numeric.max?.toString() || 'N/A'];
  if (col.stats.Temporal) return [col.stats.Temporal.min || 'N/A', col.stats.Temporal.max || 'N/A'];
  if (col.stats.Text) return [col.stats.Text.min_length + ' chars', col.stats.Text.max_length + ' chars'];
  return ['N/A', 'N/A'];
}

export function renderAnalyserRow(col: ColumnSummary, isExpanded: boolean, config?: ColumnCleanConfig): string {
  const nullPct = (col.nulls / col.count) * 100;
  const uniqueCount = getUniqueCount(col);
  const uniquePct = (uniqueCount / col.count) * 100;
  
  const qualityClass = nullPct > 20 ? 'quality-poor' : nullPct > 5 ? 'quality-warn' : 'quality-good';
  const typeIcon = col.kind === 'Numeric' ? 'ph-hash' : col.kind === 'Text' ? 'ph-text-t' : col.kind === 'Temporal' ? 'ph-calendar' : 'ph-check-square';

  const [min, max] = getMinMax(col);
  const meanOrMode = getMeanOrMode(col);

  return `
    <tr class="analyser-row ${isExpanded ? 'expanded' : ''}" data-col="${escapeHtml(col.name)}">
      <td><i class="ph ${isExpanded ? 'ph-caret-down' : 'ph-caret-right'} expand-toggle"></i></td>
      <td>
        <div class="col-name-box">
          <span class="col-name">${escapeHtml(col.name)}</span>
        </div>
      </td>
      <td><span class="col-type"><i class="ph ${typeIcon}"></i> ${col.kind}</span></td>
      <td>
        <div class="quality-bar-container" title="${col.nulls} nulls (${nullPct.toFixed(1)}%)">
          <div class="quality-bar ${qualityClass}" style="width: ${Math.max(5, 100 - nullPct)}%"></div>
          <span class="quality-text">${(100 - nullPct).toFixed(0)}%</span>
        </div>
      </td>
      <td class="mono">${escapeHtml(meanOrMode)}</td>
      <td class="mono">${escapeHtml(min)}</td>
      <td class="mono">${escapeHtml(max)}</td>
      <td class="cleaning-cell">
        <div class="cleaning-summary">
          ${config?.active ? '<span class="active-dot" title="Cleaning Active"></span>' : ''}
          ${config?.impute_mode !== 'None' ? `<span class="clean-tag">Impute: ${config?.impute_mode}</span>` : ''}
          ${config?.normalization !== 'None' ? `<span class="clean-tag">Norm: ${config?.normalization}</span>` : ''}
        </div>
      </td>
    </tr>
    ${isExpanded ? `
      <tr class="details-row">
        <td colspan="8">
          <div class="details-expanded">
            <div class="details-grid">
              <div class="details-stats">
                <h4>Statistics</h4>
                <div class="stats-list">
                  <div class="stat-row"><span>Nulls</span> <span>${col.nulls.toLocaleString()} (${nullPct.toFixed(1)}%)</span></div>
                  <div class="stat-row"><span>Unique</span> <span>${uniqueCount.toLocaleString()} (${uniquePct.toFixed(1)}%)</span></div>
                  ${col.stats.Numeric ? `
                    <div class="stat-row"><span>Std Dev</span> <span>${col.stats.Numeric.std_dev?.toFixed(2) || 'N/A'}</span></div>
                    <div class="stat-row"><span>Skew</span> <span>${col.stats.Numeric.skew?.toFixed(2) || 'N/A'}</span></div>
                  ` : ''}
                </div>
              </div>
              
              <div class="details-cleaning">
                <h4>Cleaning Pipeline</h4>
                <div class="cleaning-controls">
                  <div class="control-group">
                    <label><input type="checkbox" class="row-action" data-prop="active" ${config?.active ? 'checked' : ''} data-col="${escapeHtml(col.name)}"> Enable Cleaning</label>
                  </div>
                  
                  <div class="control-grid">
                    <div class="control-item">
                      <label>Handle Nulls (Impute)</label>
                      ${renderSelect(IMPUTE_OPTIONS, config?.impute_mode || 'None', 'row-action', { col: col.name, prop: 'impute_mode' })}
                    </div>
                    <div class="control-item">
                      <label>Normalization</label>
                      ${renderSelect(NORM_OPTIONS, config?.normalization || 'None', 'row-action', { col: col.name, prop: 'normalization' })}
                    </div>
                    ${col.kind === 'Text' ? `
                      <div class="control-item">
                        <label>Text Case</label>
                        ${renderSelect(CASE_OPTIONS, config?.text_case || 'None', 'row-action', { col: col.name, prop: 'text_case' })}
                      </div>
                    ` : ''}
                    ${col.kind === 'Numeric' ? `
                      <div class="control-item">
                        <label>Rounding</label>
                        ${renderSelect(ROUND_OPTIONS, config?.rounding?.toString() || 'none', 'row-action', { col: col.name, prop: 'rounding' })}
                      </div>
                    ` : ''}
                  </div>

                  <div class="control-advanced">
                    <label title="Automatic outlier handling and normalization"><input type="checkbox" class="row-action" data-prop="ml_preprocessing" ${config?.ml_preprocessing ? 'checked' : ''} data-col="${escapeHtml(col.name)}"> ML Preprocessing</label>
                    <label title="Clip values to 3x std dev"><input type="checkbox" class="row-action" data-prop="clip_outliers" ${config?.clip_outliers ? 'checked' : ''} data-col="${escapeHtml(col.name)}"> Clip Outliers</label>
                  </div>
                </div>
              </div>

              ${renderDistribution(col)}
              ${renderInsights(col)}
            </div>
          </div>
        </td>
      </tr>
    ` : ''}
  `;
}

export function renderDistribution(col: ColumnSummary): string {
  if (col.stats.Numeric && col.stats.Numeric.histogram) {
    const hist = col.stats.Numeric.histogram;
    const maxCount = Math.max(...hist.map(h => h[1]));
    return `
      <div class="details-distribution">
        <h4>Distribution</h4>
        <div class="histogram">
          <div class="hist-bars">
            ${hist.map(([val, count]) => `
              <div class="hist-bar" style="height: ${(count / maxCount) * 100}%" title="${val.toFixed(2)}: ${count}"></div>
            `).join('')}
          </div>
        </div>
      </div>
    `;
  } else if (col.stats.Categorical) {
    const top = Object.entries(col.stats.Categorical)
      .sort((a, b) => b[1] - a[1])
      .slice(0, 10);
    const maxCount = Math.max(...top.map(t => t[1]));
    return `
      <div class="details-distribution">
        <h4>Top Categories</h4>
        <div class="top-values">
          ${top.map(([val, count]) => `
            <div class="top-val-row">
              <span class="top-val-label" title="${escapeHtml(val)}">${escapeHtml(val || '(empty)')}</span>
              <div class="top-val-bar-container">
                <div class="top-val-bar" style="width: ${(count / maxCount) * 100}%"></div>
              </div>
              <span class="top-val-count">${count}</span>
            </div>
          `).join('')}
        </div>
      </div>
    `;
  }
  return '';
}

export function renderInsights(col: ColumnSummary): string {
  if (col.business_summary && col.business_summary.length > 0) {
    return `
      <div class="details-insights">
        <h4>Insights</h4>
        <div class="business-summary">
          <ul>
            ${col.business_summary.map(s => `<li>${escapeHtml(s)}</li>`).join('')}
          </ul>
        </div>
      </div>
    `;
  }
  return '';
}

export function renderSchemaSidebar(response: AnalysisResponse, configs: Record<string, ColumnCleanConfig>): string {
  return `
    <div class="schema-sidebar">
      <h3>Cleaning Schema</h3>
      <div class="schema-stats">
        <span>${Object.values(configs).filter(c => c.active).length} active transforms</span>
      </div>
      <div class="schema-list">
        ${response.summary.map(col => {
          const config = configs[col.name];
          if (!config || !config.active) return '';
          return `
            <div class="schema-item">
              <span class="schema-col-name">${escapeHtml(col.name)}</span>
              <div class="schema-badges">
                ${config.impute_mode !== 'None' ? `<span class="badge">Impute</span>` : ''}
                ${config.normalization !== 'None' ? `<span class="badge">Norm</span>` : ''}
                ${config.text_case !== 'None' ? `<span class="badge">Case</span>` : ''}
                ${config.ml_preprocessing ? `<span class="badge-ml">ML</span>` : ''}
              </div>
            </div>
          `;
        }).join('')}
      </div>
      <div class="schema-actions">
        <button id="btn-clear-schema" class="btn-secondary btn-block">Clear All</button>
      </div>
    </div>
  `;
}
