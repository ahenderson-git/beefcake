import { AnalysisResponse, ColumnCleanConfig, ColumnSummary, LifecycleStage } from "../types";
import { escapeHtml, fmtBytes, fmtDuration } from "../utils";
import { CASE_OPTIONS, IMPUTE_OPTIONS, NORM_OPTIONS, renderSelect, ROUND_OPTIONS } from "./common";

export function renderAnalyserHeader(
  response: AnalysisResponse,
  currentStage: LifecycleStage | null = null,
  isReadOnly: boolean = false,
  useOriginalColumnNames: boolean = false,
  cleanAllActive: boolean = true
): string {
  const isSampled = response.total_row_count > response.row_count;
  const rowDisplay = isSampled
    ? `${response.total_row_count.toLocaleString()} rows <small>(Sampled ${response.row_count.toLocaleString()} for analysis)</small>`
    : `${response.row_count.toLocaleString()} rows`;

  return `
    <div id="lifecycle-rail-container"></div>
    ${isReadOnly ? `
      <div class="stage-banner stage-banner-readonly">
        <i class="ph ph-lock-key"></i>
        <div>
          <strong>Read-Only Analysis Mode</strong>
          <span>Review statistics and data quality. No modifications available in ${currentStage || 'current'} stage.</span>
        </div>
      </div>
    ` : ''}
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
        <button id="btn-open-file" class="btn-secondary btn-small">
          <i class="ph ph-file-plus"></i> Select File
        </button>
        <button id="btn-reanalyze" class="btn-secondary btn-small">Re-analyze</button>
        ${currentStage === 'Profiled' || currentStage === 'Raw' ? `
          <button id="btn-begin-cleaning" class="btn-primary btn-small">
            <i class="ph ph-broom"></i> Begin Cleaning
          </button>
        ` : currentStage === 'Cleaned' ? `
          <button id="btn-continue-advanced" class="btn-primary btn-small">
            <i class="ph ph-arrow-right"></i> Continue to Advanced
          </button>
        ` : `
          <button id="btn-export" class="btn-primary btn-small">
            <i class="ph ph-export"></i> Export / ETL
          </button>
        `}
      </div>
    </div>
    ${!isReadOnly ? `
      <div class="bulk-actions">
        <div class="bulk-group">
          <label><input type="checkbox" class="header-action" data-action="active-all" ${cleanAllActive ? 'checked' : ''}> Clean All</label>
        </div>
        <div class="bulk-group">
          <label><input type="checkbox" class="header-action" data-action="use-original-names" id="toggle-original-names" ${useOriginalColumnNames ? 'checked' : ''}> Original Names</label>
        </div>
        ${currentStage === 'Advanced' || currentStage === 'Validated' || currentStage === 'Published' ? `
          <div class="bulk-group">
            <label>Impute All:</label>
            ${renderSelect(IMPUTE_OPTIONS, 'None', 'header-action', { action: 'impute-all' }, 'Mixed')}
          </div>
        ` : ''}
        <div class="bulk-group">
          <label>Round All:</label>
          ${renderSelect(ROUND_OPTIONS, 'none', 'header-action', { action: 'round-all' }, 'Mixed')}
        </div>
        ${currentStage === 'Advanced' || currentStage === 'Validated' || currentStage === 'Published' ? `
          <div class="bulk-group">
            <label>Norm All:</label>
            ${renderSelect(NORM_OPTIONS, 'None', 'header-action', { action: 'norm-all' }, 'Mixed')}
          </div>
        ` : ''}
      </div>
    ` : ''}
  `;
}

interface DatasetStats {
  typeBreakdown: Record<string, number>;
  avgNullPct: number;
  avgCardinalityPct: number;
  highCardinalityCols: number;
  highQualityCols: number;
  needsAttentionCols: number;
}

function computeDatasetStats(response: AnalysisResponse): DatasetStats {
  const typeBreakdown: Record<string, number> = {};
  let totalNullPct = 0;
  let totalCardinalityPct = 0;
  let highCardinalityCols = 0;
  let highQualityCols = 0;
  let needsAttentionCols = 0;

  response.summary.forEach(col => {
    // Type breakdown
    typeBreakdown[col.kind] = (typeBreakdown[col.kind] || 0) + 1;

    // Null percentage
    const nullPct = (col.nulls / col.count) * 100;
    totalNullPct += nullPct;

    // Cardinality
    const uniqueCount = getUniqueCount(col);
    const cardinalityPct = (uniqueCount / col.count) * 100;
    totalCardinalityPct += cardinalityPct;

    if (cardinalityPct > 90) highCardinalityCols++;

    // Quality assessment
    if (nullPct < 5) {
      highQualityCols++;
    } else if (nullPct > 20) {
      needsAttentionCols++;
    }
  });

  return {
    typeBreakdown,
    avgNullPct: totalNullPct / response.summary.length,
    avgCardinalityPct: totalCardinalityPct / response.summary.length,
    highCardinalityCols,
    highQualityCols,
    needsAttentionCols
  };
}

function renderDatasetOverview(response: AnalysisResponse): string {
  const stats = computeDatasetStats(response);
  const totalCols = response.column_count;

  const typeOrder = ['Numeric', 'Text', 'Categorical', 'Temporal', 'Boolean', 'Nested'];
  const typeColors: Record<string, string> = {
    'Numeric': '#3b82f6',
    'Text': '#8b5cf6',
    'Categorical': '#ec4899',
    'Temporal': '#10b981',
    'Boolean': '#f59e0b',
    'Nested': '#6b7280'
  };

  const orderedTypes = typeOrder.filter(t => (stats.typeBreakdown[t] || 0) > 0);
  const otherTypes = Object.keys(stats.typeBreakdown).filter(t => !typeOrder.includes(t));
  const allTypes = [...orderedTypes, ...otherTypes];

  return `
    <div class="dataset-overview-card">
      <h4><i class="ph ph-chart-bar"></i> Dataset Overview</h4>
      <div class="type-breakdown">
        ${allTypes.map(type => {
          const count = stats.typeBreakdown[type] || 0;
          const pct = ((count / totalCols) * 100).toFixed(1);
          const color = typeColors[type] || '#6b7280';
          return `
            <div class="type-stat" style="border-left: 3px solid ${color}">
              <div class="type-count">${count}</div>
              <div class="type-label">${type}</div>
              <div class="type-pct">${pct}%</div>
            </div>
          `;
        }).join('')}
      </div>
      <div class="dataset-metrics">
        <div class="metric-item">
          <i class="ph ph-drop"></i>
          <span class="metric-label">Avg Nulls:</span>
          <span class="metric-value">${stats.avgNullPct.toFixed(1)}%</span>
        </div>
        <div class="metric-item">
          <i class="ph ph-fingerprint"></i>
          <span class="metric-label">Avg Cardinality:</span>
          <span class="metric-value">${stats.avgCardinalityPct.toFixed(1)}%</span>
        </div>
        <div class="metric-item">
          <i class="ph ph-flag"></i>
          <span class="metric-label">High-cardinality:</span>
          <span class="metric-value">${stats.highCardinalityCols} cols</span>
        </div>
      </div>
    </div>
  `;
}

export function renderAnalyser(
  response: AnalysisResponse,
  expandedRows: Set<string>,
  configs: Record<string, ColumnCleanConfig>,
  currentStage: LifecycleStage | null = null,
  isReadOnly: boolean = false,
  selectedColumns: Set<string> = new Set(),
  _useOriginalColumnNames: boolean = false
): string {
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
            : healthScore < 80
              ? '<p><i class="ph ph-info"></i> Data quality could be improved. Expand columns for details.</p>'
              : '<p><i class="ph ph-check-circle"></i> No critical health issues detected.</p>'}
        </div>
      </div>

      ${renderDatasetOverview(response)}

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
            ${!isReadOnly ? '<th>Cleaning Options</th>' : '<th>Actions</th>'}
          </tr>
        </thead>
        <tbody>
          ${response.summary.map(col => renderAnalyserRow(
            col,
            expandedRows.has(col.name),
            configs[col.name],
            isReadOnly,
            selectedColumns.size === 0 || selectedColumns.has(col.name),
            currentStage
          )).join('')}
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
    const firstEntry = entries.sort((a, b) => b[1] - a[1])[0];
    return firstEntry ? firstEntry[0] : 'N/A';
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

function renderEnhancedStats(col: ColumnSummary, nullPct: number, uniqueCount: number, uniquePct: number): string {
  // Base stats for all types
  let statsHTML = `
    <div class="stat-row"><span>Count</span> <span>${col.count.toLocaleString()}</span></div>
    <div class="stat-row"><span>Nulls</span> <span>${col.nulls.toLocaleString()} (${nullPct.toFixed(1)}%)</span></div>
    <div class="stat-row"><span>Unique</span> <span>${uniqueCount.toLocaleString()} (${uniquePct.toFixed(1)}%)</span></div>
  `;

  // Numeric-specific stats
  if (col.stats.Numeric) {
    const n = col.stats.Numeric;
    const iqr = (n.q3 && n.q1) ? (n.q3 - n.q1).toFixed(2) : 'N/A';
    statsHTML += `
      <div class="stat-section-header">Five-Number Summary</div>
      <div class="stat-row"><span>Min</span> <span>${n.min?.toFixed(2) || 'N/A'}</span></div>
      <div class="stat-row"><span>Q1 (25%)</span> <span>${n.q1?.toFixed(2) || 'N/A'}</span></div>
      <div class="stat-row"><span>Median (50%)</span> <span>${n.median?.toFixed(2) || 'N/A'}</span></div>
      <div class="stat-row"><span>Q3 (75%)</span> <span>${n.q3?.toFixed(2) || 'N/A'}</span></div>
      <div class="stat-row"><span>Max</span> <span>${n.max?.toFixed(2) || 'N/A'}</span></div>
      <div class="stat-row"><span>IQR</span> <span>${iqr}</span></div>

      <div class="stat-section-header">Distribution</div>
      <div class="stat-row"><span>Mean</span> <span>${n.mean?.toFixed(2) || 'N/A'}</span></div>
      <div class="stat-row"><span>Std Dev</span> <span>${n.std_dev?.toFixed(2) || 'N/A'}</span></div>
      <div class="stat-row"><span>Skewness</span> <span>${n.skew?.toFixed(3) || 'N/A'}</span></div>

      <div class="stat-section-header">Value Characteristics</div>
      <div class="stat-row"><span>Zeros</span> <span>${n.zero_count.toLocaleString()} (${((n.zero_count / col.count) * 100).toFixed(1)}%)</span></div>
      <div class="stat-row"><span>Negatives</span> <span>${n.negative_count.toLocaleString()} (${((n.negative_count / col.count) * 100).toFixed(1)}%)</span></div>
      <div class="stat-row"><span>Integer Type</span> <span>${n.is_integer ? '✓ Yes' : '✗ No'}</span></div>
      ${n.is_sorted || n.is_sorted_rev ? `
        <div class="stat-row">
          <span>Sorted</span>
          <span class="badge badge-sorted">${n.is_sorted ? '↑ Ascending' : '↓ Descending'}</span>
        </div>
      ` : ''}
    `;
  }

  // Text-specific stats
  if (col.stats.Text) {
    const t = col.stats.Text;
    statsHTML += `
      <div class="stat-section-header">Text Properties</div>
      <div class="stat-row"><span>Min length</span> <span>${t.min_length} chars</span></div>
      <div class="stat-row"><span>Max length</span> <span>${t.max_length} chars</span></div>
      <div class="stat-row"><span>Avg length</span> <span>${t.avg_length.toFixed(1)} chars</span></div>
      ${t.top_value ? `
        <div class="stat-row">
          <span>Top value</span>
          <span title="${escapeHtml(t.top_value[0])}">"${escapeHtml(t.top_value[0].substring(0, 20))}${t.top_value[0].length > 20 ? '...' : ''}"</span>
        </div>
        <div class="stat-row"><span>Top count</span> <span>${t.top_value[1].toLocaleString()}</span></div>
      ` : ''}
    `;
  }

  // Temporal-specific stats
  if (col.stats.Temporal) {
    const temp = col.stats.Temporal;
    statsHTML += `
      <div class="stat-section-header">Date Range</div>
      <div class="stat-row"><span>Earliest</span> <span>${temp.min || 'N/A'}</span></div>
      <div class="stat-row"><span>Latest</span> <span>${temp.max || 'N/A'}</span></div>
      ${temp.is_sorted || temp.is_sorted_rev ? `
        <div class="stat-row">
          <span>Sorted</span>
          <span class="badge badge-sorted">${temp.is_sorted ? '↑ Ascending' : '↓ Descending'}</span>
        </div>
      ` : ''}
    `;
  }

  // Categorical-specific stats
  if (col.stats.Categorical) {
    const catEntries = Object.entries(col.stats.Categorical)
      .sort((a, b) => b[1] - a[1])
      .slice(0, 3);
    if (catEntries.length > 0) {
      statsHTML += `
        <div class="stat-section-header">Top Values</div>
      `;
      catEntries.forEach(([val, count]) => {
        const pct = ((count / col.count) * 100).toFixed(1);
        statsHTML += `
          <div class="stat-row">
            <span title="${escapeHtml(val)}">${escapeHtml(val.substring(0, 15))}${val.length > 15 ? '...' : ''}</span>
            <span>${count.toLocaleString()} (${pct}%)</span>
          </div>
        `;
      });
    }
  }

  // Boolean-specific stats
  if (col.stats.Boolean) {
    const b = col.stats.Boolean;
    const truePct = ((b.true_count / col.count) * 100).toFixed(1);
    const falsePct = ((b.false_count / col.count) * 100).toFixed(1);
    statsHTML += `
      <div class="stat-section-header">Boolean Distribution</div>
      <div class="stat-row"><span>True</span> <span>${b.true_count.toLocaleString()} (${truePct}%)</span></div>
      <div class="stat-row"><span>False</span> <span>${b.false_count.toLocaleString()} (${falsePct}%)</span></div>
    `;
  }

  return statsHTML;
}

export function renderAnalyserRow(col: ColumnSummary, isExpanded: boolean, config?: ColumnCleanConfig, isReadOnly: boolean = false, isSelected: boolean = true, currentStage: LifecycleStage | null = null): string {
  const nullPct = (col.nulls / col.count) * 100;
  const uniqueCount = getUniqueCount(col);
  const uniquePct = (uniqueCount / col.count) * 100;

  const qualityClass = nullPct > 20 ? 'quality-poor' : nullPct > 5 ? 'quality-warn' : 'quality-good';
  const typeIcon = col.kind === 'Numeric' ? 'ph-hash' : col.kind === 'Text' ? 'ph-text-t' : col.kind === 'Temporal' ? 'ph-calendar' : 'ph-check-square';

  const [min, max] = getMinMax(col);
  const meanOrMode = getMeanOrMode(col);

  // Determine which name to show as primary (proposed) vs secondary (original)
  const proposedName = config?.new_name || col.name;
  const hasNameChange = proposedName !== col.name;

  return `
    <tr class="analyser-row ${isExpanded ? 'expanded' : ''}" data-col="${escapeHtml(col.name)}">
      <td><i class="ph ${isExpanded ? 'ph-caret-down' : 'ph-caret-right'} expand-toggle"></i></td>
      <td>
        <div class="col-name-box">
          ${isReadOnly ? `
            <input type="checkbox" class="col-select-checkbox" data-col="${escapeHtml(col.name)}" ${isSelected ? 'checked' : ''} title="Include in cleaning">
          ` : ''}
          <div class="col-name-display">
            <span class="col-name">${escapeHtml(proposedName)}</span>
            ${hasNameChange ? `
              <span class="col-name-original">Originally: "${escapeHtml(col.name)}"</span>
            ` : ''}
          </div>
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
        ${isReadOnly ? `
          <div class="readonly-actions">
            ${nullPct > 50 ? '<span class="recommendation-badge badge-warn" title="High null percentage">High Nulls</span>' : ''}
            ${uniquePct > 95 ? '<span class="recommendation-badge badge-info" title="Highly unique - consider dropping">Unique</span>' : ''}
          </div>
        ` : `
          <div class="cleaning-summary">
            ${config?.active ? '<span class="active-dot" title="Cleaning Active"></span>' : ''}
            ${config?.impute_mode !== 'None' ? `<span class="clean-tag">Impute: ${config?.impute_mode}</span>` : ''}
            ${config?.normalization !== 'None' ? `<span class="clean-tag">Norm: ${config?.normalization}</span>` : ''}
          </div>
        `}
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
                  ${renderEnhancedStats(col, nullPct, uniqueCount, uniquePct)}
                </div>
              </div>

              ${!isReadOnly ? `
                <div class="details-cleaning">
                  <h4>Cleaning Pipeline</h4>
                  <div class="cleaning-controls">
                    <div class="control-group">
                      <label><input type="checkbox" class="row-action" data-prop="active" ${config?.active ? 'checked' : ''} data-col="${escapeHtml(col.name)}"> Enable Cleaning</label>
                    </div>

                    <div class="control-grid">
                      ${currentStage === 'Advanced' || currentStage === 'Validated' || currentStage === 'Published' ? `
                        <div class="control-item">
                          <label>Handle Nulls (Impute)</label>
                          ${renderSelect(IMPUTE_OPTIONS, config?.impute_mode || 'None', 'row-action', { col: col.name, prop: 'impute_mode' })}
                        </div>
                        <div class="control-item">
                          <label>Normalization</label>
                          ${renderSelect(NORM_OPTIONS, config?.normalization || 'None', 'row-action', { col: col.name, prop: 'normalization' })}
                        </div>
                      ` : ''}
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

                    ${currentStage === 'Advanced' || currentStage === 'Validated' || currentStage === 'Published' ? `
                      <div class="control-advanced">
                        <label title="Automatic outlier handling and normalization"><input type="checkbox" class="row-action" data-prop="ml_preprocessing" ${config?.ml_preprocessing ? 'checked' : ''} data-col="${escapeHtml(col.name)}"> ML Preprocessing</label>
                        <label title="Clip values to 3x std dev"><input type="checkbox" class="row-action" data-prop="clip_outliers" ${config?.clip_outliers ? 'checked' : ''} data-col="${escapeHtml(col.name)}"> Clip Outliers</label>
                      </div>
                    ` : ''}
                  </div>
                </div>
              ` : ''}

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
