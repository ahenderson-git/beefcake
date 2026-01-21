import {
  AnalysisResponse,
  ColumnCleanConfig,
  ColumnSummary,
  CurrentDataset,
  DatasetVersion,
  LifecycleStage,
  TransformSpec,
} from '../types';
import { escapeHtml, fmtBytes, fmtDuration } from '../utils';

import {
  CASE_OPTIONS,
  getImputeOptionsForColumn,
  IMPUTE_OPTIONS,
  NORM_OPTIONS,
  renderSelect,
  ROUND_OPTIONS,
} from './common';

function renderCleaningInfoBox(): string {
  return `
    <div class="cleaning-info-box">
      <div class="cleaning-info-header">
        <i class="ph ph-info"></i>
        <h4>What does cleaning include?</h4>
        <button class="cleaning-info-toggle" aria-label="Toggle info">
          <i class="ph ph-caret-up"></i>
        </button>
      </div>
      <div class="cleaning-info-content">
        <p class="cleaning-info-intro">The Cleaning stage applies <strong>reversible text and type transformations</strong> to prepare your data:</p>
        <div class="cleaning-info-grid">
          <div class="cleaning-info-section">
            <strong><i class="ph ph-text-t"></i> Text Cleaning:</strong>
            <ul>
              <li>Trim whitespace</li>
              <li>Convert case (lower/upper/title)</li>
              <li>Remove special characters</li>
              <li>Standardize null representations</li>
            </ul>
          </div>
          <div class="cleaning-info-section">
            <strong><i class="ph ph-swap"></i> Type Casting:</strong>
            <ul>
              <li>Convert to Numeric, Text, Boolean</li>
              <li>Parse Temporal (dates/times)</li>
              <li>Detect Categorical patterns</li>
            </ul>
          </div>
          <div class="cleaning-info-section">
            <strong><i class="ph ph-tag"></i> Column Renaming:</strong>
            <ul>
              <li>Standardize column names</li>
              <li>Apply custom naming conventions</li>
            </ul>
          </div>
        </div>
        <p class="cleaning-info-note">
          <i class="ph ph-arrow-counter-clockwise"></i>
          <strong>Note:</strong> All cleaning operations in this stage are reversible.
          Advanced operations (imputation, normalisation, encoding) are available in the <strong>Advanced</strong> stage.
          <a href="#" class="cleaning-info-link" data-view="reference">View full documentation →</a>
        </p>
      </div>
    </div>
  `;
}

export function renderAnalyserHeader(
  response: AnalysisResponse,
  currentStage: LifecycleStage | null = null,
  isReadOnly: boolean = false,
  useOriginalColumnNames: boolean = false,
  cleanAllActive: boolean = true,
  advancedProcessingEnabled: boolean = false
): string {
  const isSampled = response.total_row_count > response.row_count;
  const rowDisplay = isSampled
    ? `${response.total_row_count.toLocaleString()} rows <small>(Analysed ${response.row_count.toLocaleString()} rows)</small>`
    : `${response.row_count.toLocaleString()} rows`;

  return `
    ${
      isReadOnly
        ? `
      <div class="stage-banner stage-banner-readonly">
        <i class="ph ph-lock-key"></i>
        <div>
          <strong>Read-Only Analysis Mode</strong>
          <span>Review statistics and data quality – remove unnecessary columns. No modifications available in ${currentStage ?? 'current'} stage.</span>
        </div>
      </div>
    `
        : ''
    }
    ${currentStage === 'Cleaned' && !isReadOnly ? renderCleaningInfoBox() : ''}
    <div class="analyser-header">
      <div class="header-main">
        <h2>${escapeHtml(response.file_name)} <small>(${fmtBytes(response.file_size)})</small></h2>
        <div class="meta-info">
          <span data-testid="analyser-row-count"><i class="ph ph-rows"></i> ${rowDisplay}</span>
          <span data-testid="analyser-column-count"><i class="ph ph-columns"></i> ${response.column_count} columns</span>
          <span><i class="ph ph-timer"></i> Analysed in ${fmtDuration(response.analysis_duration)}</span>
        </div>
      </div>
      <div class="header-actions">
        <button id="btn-open-file" class="btn-secondary btn-small" data-testid="analyser-open-file-button">
          <i class="ph ph-file-plus"></i> Select File
        </button>
        <button id="btn-reanalyze" class="btn-secondary btn-small" data-testid="analyser-reanalyze-button">Re-analyse</button>
        ${
          currentStage === 'Profiled' || currentStage === 'Raw'
            ? `
          <button id="btn-begin-cleaning" class="btn-primary btn-small" data-testid="btn-begin-cleaning">
            <i class="ph ph-broom"></i> Begin Cleaning
          </button>
        `
            : currentStage === 'Cleaned'
              ? `
          <button id="btn-continue-advanced" class="btn-primary btn-small" data-testid="btn-continue-advanced">
            <i class="ph ph-arrow-right"></i> Continue to Advanced
          </button>
        `
              : currentStage === 'Advanced'
                ? `
          <button id="btn-move-to-validated" class="btn-primary btn-small" data-testid="btn-move-to-validated">
            <i class="ph ph-check-circle"></i> Move to Validated
          </button>
        `
                : `
          <button id="btn-export" class="btn-primary btn-small" data-testid="btn-export-analyser">
            <i class="ph ph-export"></i> Export / ETL
          </button>
        `
        }
      </div>
    </div>
    ${
      !isReadOnly
        ? `
      <div class="bulk-actions">
        ${
          currentStage === 'Cleaned' || currentStage === 'Profiled' || currentStage === 'Raw'
            ? `
          <div class="bulk-group">
            <label><input type="checkbox" name="clean-all" class="header-action" data-action="active-all" data-testid="header-active-all" ${cleanAllActive ? 'checked' : ''}> Clean All</label>
          </div>
          <div class="bulk-group">
            <label><input type="checkbox" name="use-original-names" class="header-action" data-action="use-original-names" id="toggle-original-names" data-testid="header-use-original-names" ${useOriginalColumnNames ? 'checked' : ''}> Original Names</label>
          </div>
        `
            : ''
        }
        ${
          currentStage === 'Advanced' ||
          currentStage === 'Validated' ||
          currentStage === 'Published'
            ? `
          <div class="bulk-group">
            <label title="Enable machine learning preprocessing features: imputation, normalisation, one-hot encoding, and outlier clipping">
              <input type="checkbox" name="activate-advanced" class="header-action" data-action="activate-advanced" data-testid="header-activate-advanced" ${advancedProcessingEnabled ? 'checked' : ''}>
              <i class="ph ph-brain"></i> Advanced Processing
              ${advancedProcessingEnabled ? '<span class="badge-ml" style="margin-left: 8px;">ACTIVE</span>' : ''}
            </label>
          </div>
          <div class="bulk-group">
            <label>Impute All:</label>
            ${renderSelect(IMPUTE_OPTIONS, 'None', 'header-action', { action: 'impute-all' }, 'Mixed', !advancedProcessingEnabled)}
          </div>
          <div class="bulk-group">
            <label>Norm All:</label>
            ${renderSelect(NORM_OPTIONS, 'None', 'header-action', { action: 'norm-all' }, 'Mixed', !advancedProcessingEnabled)}
          </div>
        `
            : ''
        }
        ${
          currentStage === 'Cleaned' || currentStage === 'Profiled' || currentStage === 'Raw'
            ? `
          <div class="bulk-group">
            <label>Round All:</label>
            ${renderSelect(ROUND_OPTIONS, 'none', 'header-action', { action: 'round-all' }, 'Mixed')}
          </div>
        `
            : ''
        }
      </div>
    `
        : ''
    }
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
    typeBreakdown[col.kind] = (typeBreakdown[col.kind] ?? 0) + 1;

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
    needsAttentionCols,
  };
}

function renderDatasetOverview(response: AnalysisResponse): string {
  const stats = computeDatasetStats(response);
  const totalCols = response.column_count;

  const typeOrder = ['Numeric', 'Text', 'Categorical', 'Temporal', 'Boolean', 'Nested'];
  const typeColors: Record<string, string> = {
    Numeric: '#3b82f6',
    Text: '#8b5cf6',
    Categorical: '#ec4899',
    Temporal: '#10b981',
    Boolean: '#f59e0b',
    Nested: '#6b7280',
  };

  const orderedTypes = typeOrder.filter(t => (stats.typeBreakdown[t] ?? 0) > 0);
  const otherTypes = Object.keys(stats.typeBreakdown).filter(t => !typeOrder.includes(t));
  const allTypes = [...orderedTypes, ...otherTypes];

  return `
    <div class="dataset-overview-card">
      <h4><i class="ph ph-chart-bar"></i> Dataset Overview</h4>
      <div class="type-breakdown">
        ${allTypes
          .map(type => {
            const count = stats.typeBreakdown[type] ?? 0;
            const pct = ((count / totalCols) * 100).toFixed(1);
            const color = typeColors[type] ?? '#6b7280';
            return `
            <div class="type-stat" style="border-left: 3px solid ${color}">
              <div class="type-count">${count}</div>
              <div class="type-label">${type}</div>
              <div class="type-pct">${pct}%</div>
            </div>
          `;
          })
          .join('')}
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

export function renderValidatedSummary(
  response: AnalysisResponse,
  dataset: CurrentDataset | null
): string {
  // Calculate transformation summary
  const versions = dataset?.versions ?? [];
  const rawVersion = versions.find((v: DatasetVersion) => v.stage === 'Raw');

  const initialColumns = rawVersion?.metadata?.column_count ?? response.column_count;
  const currentColumns = response.column_count;
  const columnDelta = currentColumns - initialColumns;
  const columnDeltaSign = columnDelta >= 0 ? '+' : '';

  const initialRows = rawVersion?.metadata?.row_count ?? response.total_row_count;
  const currentRows = response.total_row_count;
  const rowDelta = currentRows - initialRows;
  const rowDeltaSign = rowDelta >= 0 ? '+' : '';

  // Calculate quality metrics
  const nullColumns = response.summary.filter(col => {
    const nullPct = (col.nulls / col.count) * 100;
    return nullPct > 0;
  });
  const avgNullPct =
    nullColumns.length > 0
      ? nullColumns.reduce((sum, col) => sum + (col.nulls / col.count) * 100, 0) /
        nullColumns.length
      : 0;

  // Build transformation timeline
  const sortedVersions = [...versions].sort(
    (a: DatasetVersion, b: DatasetVersion) =>
      new Date(a.created_at).getTime() - new Date(b.created_at).getTime()
  );

  const timelineHTML = sortedVersions
    .map((v: DatasetVersion) => {
      const isActive = v.id === dataset?.activeVersionId;
      const stageIcon =
        v.stage === 'Raw'
          ? 'ph-file'
          : v.stage === 'Profiled'
            ? 'ph-chart-line'
            : v.stage === 'Cleaned'
              ? 'ph-broom'
              : v.stage === 'Advanced'
                ? 'ph-gear-six'
                : v.stage === 'Validated'
                  ? 'ph-check-circle'
                  : 'ph-rocket-launch';

      const transformSummary =
        (v.pipeline?.transforms?.length ?? 0) > 0
          ? (v.pipeline?.transforms ?? [])
              .map((t: TransformSpec) => {
                if (t.transform_type === 'select_columns') {
                  const columns = t.parameters.columns as string[] | undefined;
                  return `Selected ${columns?.length ?? 0} columns`;
                }
                if (t.transform_type === 'clean') {
                  return 'Applied cleaning transformations';
                }
                return t.transform_type;
              })
              .join(', ')
          : 'No transformations';

      return `
      <div class="timeline-item ${isActive ? 'timeline-item-active' : ''}">
        <div class="timeline-marker">
          <i class="ph ${stageIcon}"></i>
        </div>
        <div class="timeline-content">
          <strong>${v.stage}</strong>
          ${isActive ? '<span class="timeline-badge">Current</span>' : ''}
          <div class="timeline-details">${transformSummary}</div>
        </div>
      </div>
    `;
    })
    .join('');

  return `
    <div class="validated-summary">
      <div class="validation-status-banner validation-passed">
        <i class="ph ph-check-circle"></i>
        <div>
          <h3>Dataset Ready for Publication</h3>
          <p>Review the transformation summary below and publish when ready.</p>
        </div>
      </div>

      <div class="metrics-dashboard">
        <div class="metric-card">
          <h4><i class="ph ph-git-branch"></i> Transformation Journey</h4>
          <div class="metric-stats">
            <div class="metric-row">
              <span>Versions:</span>
              <span class="metric-value">${versions.length}</span>
            </div>
            <div class="metric-row">
              <span>Columns:</span>
              <span class="metric-value">${initialColumns} → ${currentColumns} <small>(${columnDeltaSign}${columnDelta})</small></span>
            </div>
            <div class="metric-row">
              <span>Rows:</span>
              <span class="metric-value">${currentRows.toLocaleString()} <small>(${rowDeltaSign}${rowDelta})</small></span>
            </div>
            <div class="metric-row">
              <span>Size:</span>
              <span class="metric-value">${fmtBytes(response.file_size)}</span>
            </div>
          </div>
        </div>

        <div class="metric-card">
          <h4><i class="ph ph-shield-check"></i> Quality Metrics</h4>
          <div class="metric-stats">
            <div class="metric-row metric-check">
              <i class="ph ph-check-circle"></i>
              <span>Schema validated</span>
            </div>
            <div class="metric-row metric-check">
              <i class="ph ph-check-circle"></i>
              <span>Avg nulls: ${avgNullPct.toFixed(1)}%</span>
            </div>
            <div class="metric-row metric-check">
              <i class="ph ph-check-circle"></i>
              <span>Row count: ${currentRows.toLocaleString()}</span>
            </div>
            <div class="metric-row metric-check">
              <i class="ph ph-check-circle"></i>
              <span>${response.column_count} columns</span>
            </div>
          </div>
        </div>
      </div>

      <div class="transformation-timeline">
        <h4><i class="ph ph-list-checks"></i> Applied Transformations</h4>
        <div class="timeline">
          ${timelineHTML}
        </div>
      </div>

      <div class="final-dataset-preview">
        <h4><i class="ph ph-table"></i> Final Dataset Overview</h4>
        <p class="preview-description">Read-only preview of your dataset. All transformations have been applied.</p>
        <div class="dataset-overview-compact">
          <table class="preview-table">
            <thead>
              <tr>
                <th>Column</th>
                <th>Type</th>
                <th>Quality</th>
                <th>Mean/Mode</th>
                <th>Median</th>
              </tr>
            </thead>
            <tbody>
              ${response.summary
                .map(col => {
                  const nullPct = (col.nulls / col.count) * 100;
                  const qualityClass =
                    nullPct > 20 ? 'quality-poor' : nullPct > 5 ? 'quality-warn' : 'quality-good';
                  const typeIcon =
                    col.kind === 'Numeric'
                      ? 'ph-hash'
                      : col.kind === 'Text'
                        ? 'ph-text-t'
                        : col.kind === 'Temporal'
                          ? 'ph-calendar'
                          : 'ph-check-square';
                  const meanOrMode =
                    (col.stats.Numeric?.mean !== null && col.stats.Numeric?.mean !== undefined
                      ? col.stats.Numeric.mean.toFixed(2)
                      : null) ??
                    (col.stats.Text?.top_value ? col.stats.Text.top_value[0] : null) ??
                    (col.stats.Boolean?.true_count !== null &&
                    col.stats.Boolean?.true_count !== undefined
                      ? col.stats.Boolean.true_count.toString()
                      : null) ??
                    '-';
                  const median =
                    col.stats.Numeric?.median !== null && col.stats.Numeric?.median !== undefined
                      ? col.stats.Numeric.median.toFixed(2)
                      : '-';

                  return `
                  <tr>
                    <td><i class="ph ${typeIcon}"></i> ${escapeHtml(col.name)}</td>
                    <td>${col.kind}</td>
                    <td>
                      <div class="quality-bar-container">
                        <div class="quality-bar ${qualityClass}" style="width: ${Math.max(5, 100 - nullPct)}%"></div>
                        <span class="quality-text">${(100 - nullPct).toFixed(0)}%</span>
                      </div>
                    </td>
                    <td class="mono">${escapeHtml(meanOrMode)}</td>
                    <td class="mono">${escapeHtml(median)}</td>
                  </tr>
                `;
                })
                .join('')}
            </tbody>
          </table>
        </div>
      </div>

      <div class="validated-actions">
        <button id="btn-back-to-advanced" class="btn-secondary">
          <i class="ph ph-arrow-left"></i> Back to Advanced
        </button>
        <button id="btn-publish-dataset" class="btn-primary">
          <i class="ph ph-rocket-launch"></i> Publish Dataset
        </button>
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
  _useOriginalColumnNames: boolean = false,
  advancedProcessingEnabled: boolean = false
): string {
  const healthScore = Math.round(response.health.score * 100);
  const healthClass =
    healthScore > 80 ? 'health-good' : healthScore > 50 ? 'health-warn' : 'health-poor';

  return `
    <div class="analyser-wrapper" data-testid="analyser-view">
      <div id="lifecycle-rail-container"></div>
      <div id="analyser-content-container" class="analyser-container">
        <div id="analyser-header-container"></div>

        <div class="health-banner ${healthClass}" data-testid="health-score-banner">
        <div class="health-score" data-testid="health-score-badge">
          <span class="score-label">Health Score</span>
          <span class="score-value" data-testid="health-score-value">${healthScore}%</span>
        </div>
        <div class="health-issues">
          ${
            response.health.risks.length > 0
              ? `<ul>${response.health.risks.map(issue => `<li><i class="ph ph-warning"></i> ${escapeHtml(issue)}</li>`).join('')}</ul>`
              : healthScore < 80
                ? '<p><i class="ph ph-info"></i> Data quality could be improved. Expand columns for details.</p>'
                : '<p><i class="ph ph-check-circle"></i> No critical health issues detected.</p>'
          }
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
            <th>Median</th>
            <th>Min</th>
            <th>Max</th>
            <th>Std Dev</th>
            ${!isReadOnly ? '<th>Cleaning Options</th>' : '<th>Actions</th>'}
          </tr>
        </thead>
        <tbody>
          ${response.summary
            .map(col =>
              renderAnalyserRow(
                col,
                expandedRows.has(col.name),
                configs[col.name],
                isReadOnly,
                selectedColumns.size === 0 || selectedColumns.has(col.name),
                currentStage,
                advancedProcessingEnabled
              )
            )
            .join('')}
        </tbody>
      </table>
      </div>
    </div>
  `;
}

export function renderEmptyAnalyser(): string {
  return `
    <div class="empty-state" data-testid="analyser-empty-state">
      <i class="ph ph-file-search"></i>
      <h3>No data analysed yet</h3>
      <p>Select a file to begin advanced profiling and cleaning.</p>
      <button id="btn-open-file" class="btn-primary" data-testid="empty-open-file-button">
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
  if (col.stats.Numeric)
    return col.stats.Numeric.mean !== null && col.stats.Numeric.mean !== undefined
      ? col.stats.Numeric.mean.toFixed(2)
      : 'N/A';
  if (col.stats.Text) return col.stats.Text.top_value ? col.stats.Text.top_value[0] : 'N/A';
  if (col.stats.Categorical) {
    const entries = Object.entries(col.stats.Categorical);
    if (entries.length === 0) return 'N/A';
    const firstEntry = entries.sort((a, b) => b[1] - a[1])[0];
    return firstEntry ? firstEntry[0] : 'N/A';
  }
  if (col.stats.Boolean)
    return col.stats.Boolean.true_count >= col.stats.Boolean.false_count ? 'True' : 'False';
  return 'N/A';
}

function getMedian(col: ColumnSummary): string {
  if (col.stats.Numeric)
    return col.stats.Numeric.median !== null && col.stats.Numeric.median !== undefined
      ? col.stats.Numeric.median.toFixed(2)
      : 'N/A';
  return 'N/A';
}

function getStdDev(col: ColumnSummary): string {
  if (col.stats.Numeric)
    return col.stats.Numeric.std_dev !== null && col.stats.Numeric.std_dev !== undefined
      ? col.stats.Numeric.std_dev.toFixed(2)
      : 'N/A';
  return 'N/A';
}

function getMinMax(col: ColumnSummary): [string, string] {
  if (col.stats.Numeric)
    return [col.stats.Numeric.min?.toString() ?? 'N/A', col.stats.Numeric.max?.toString() ?? 'N/A'];
  if (col.stats.Temporal) return [col.stats.Temporal.min ?? 'N/A', col.stats.Temporal.max ?? 'N/A'];
  if (col.stats.Text)
    return [col.stats.Text.min_length + ' chars', col.stats.Text.max_length + ' chars'];
  return ['N/A', 'N/A'];
}

function renderEnhancedStats(
  col: ColumnSummary,
  nullPct: number,
  uniqueCount: number,
  uniquePct: number
): string {
  // Base stats for all types
  let statsHTML = `
    <div class="stat-row"><span>Count</span> <span>${col.count.toLocaleString()}</span></div>
    <div class="stat-row"><span>Nulls</span> <span>${col.nulls.toLocaleString()} (${nullPct.toFixed(1)}%)</span></div>
    <div class="stat-row"><span>Unique</span> <span>${uniqueCount.toLocaleString()} (${uniquePct.toFixed(1)}%)</span></div>
  `;

  // Numeric-specific stats
  if (col.stats.Numeric) {
    const n = col.stats.Numeric;
    const iqr = n.q3?.toFixed(2) && n.q1?.toFixed(2) ? (n.q3 - n.q1).toFixed(2) : 'N/A';
    statsHTML += `
      <div class="stat-section-header">Five-Number Summary</div>
      <div class="stat-row"><span>Min</span> <span>${n.min?.toFixed(2) ?? 'N/A'}</span></div>
      <div class="stat-row"><span>Q1 (25%)</span> <span>${n.q1?.toFixed(2) ?? 'N/A'}</span></div>
      <div class="stat-row"><span>Median (50%)</span> <span>${n.median?.toFixed(2) ?? 'N/A'}</span></div>
      <div class="stat-row"><span>Q3 (75%)</span> <span>${n.q3?.toFixed(2) ?? 'N/A'}</span></div>
      <div class="stat-row"><span>Max</span> <span>${n.max?.toFixed(2) ?? 'N/A'}</span></div>
      <div class="stat-row"><span>IQR</span> <span>${iqr}</span></div>

      <div class="stat-section-header">Distribution</div>
      <div class="stat-row"><span>Mean</span> <span>${n.mean?.toFixed(2) ?? 'N/A'}</span></div>
      <div class="stat-row"><span>Std Dev</span> <span>${n.std_dev?.toFixed(2) ?? 'N/A'}</span></div>
      <div class="stat-row"><span>Skewness</span> <span>${n.skew?.toFixed(3) ?? 'N/A'}</span></div>

      <div class="stat-section-header">Value Characteristics</div>
      <div class="stat-row"><span>Zeros</span> <span>${n.zero_count.toLocaleString()} (${((n.zero_count / col.count) * 100).toFixed(1)}%)</span></div>
      <div class="stat-row"><span>Negatives</span> <span>${n.negative_count.toLocaleString()} (${((n.negative_count / col.count) * 100).toFixed(1)}%)</span></div>
      <div class="stat-row"><span>Integer Type</span> <span>${n.is_integer ? '✓ Yes' : '✗ No'}</span></div>
      ${
        n.is_sorted || n.is_sorted_rev
          ? `
        <div class="stat-row">
          <span>Sorted</span>
          <span class="badge badge-sorted">${n.is_sorted ? '↑ Ascending' : '↓ Descending'}</span>
        </div>
      `
          : ''
      }
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
      ${
        t.top_value
          ? `
        <div class="stat-row">
          <span>Top value</span>
          <span title="${escapeHtml(t.top_value[0])}">"${escapeHtml(t.top_value[0].substring(0, 20))}${t.top_value[0].length > 20 ? '...' : ''}"</span>
        </div>
        <div class="stat-row"><span>Top count</span> <span>${t.top_value[1].toLocaleString()}</span></div>
      `
          : ''
      }
    `;
  }

  // Temporal-specific stats
  if (col.stats.Temporal) {
    const temp = col.stats.Temporal;
    statsHTML += `
      <div class="stat-section-header">Date Range</div>
      <div class="stat-row"><span>Earliest</span> <span>${temp.min ?? 'N/A'}</span></div>
      <div class="stat-row"><span>Latest</span> <span>${temp.max ?? 'N/A'}</span></div>
      ${
        temp.is_sorted || temp.is_sorted_rev
          ? `
        <div class="stat-row">
          <span>Sorted</span>
          <span class="badge badge-sorted">${temp.is_sorted ? '↑ Ascending' : '↓ Descending'}</span>
        </div>
      `
          : ''
      }
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

export function renderAnalyserRow(
  col: ColumnSummary,
  isExpanded: boolean,
  config?: ColumnCleanConfig,
  isReadOnly: boolean = false,
  isSelected: boolean = true,
  currentStage: LifecycleStage | null = null,
  advancedProcessingEnabled: boolean = false
): string {
  const nullPct = (col.nulls / col.count) * 100;
  const uniqueCount = getUniqueCount(col);
  const uniquePct = (uniqueCount / col.count) * 100;

  const qualityClass =
    nullPct > 20 ? 'quality-poor' : nullPct > 5 ? 'quality-warn' : 'quality-good';
  const typeIcon =
    col.kind === 'Numeric'
      ? 'ph-hash'
      : col.kind === 'Text'
        ? 'ph-text-t'
        : col.kind === 'Temporal'
          ? 'ph-calendar'
          : 'ph-check-square';

  const [min, max] = getMinMax(col);
  const meanOrMode = getMeanOrMode(col);
  const median = getMedian(col);
  const stdDev = getStdDev(col);

  // Calculate skewness for badge display
  const skew = col.stats.Numeric?.skew;
  const isHighlySkewed = (skew ?? null) !== null && Math.abs(skew!) > 1;
  const stdDevValue = col.stats.Numeric?.std_dev;
  const meanValue = col.stats.Numeric?.mean;
  const isHighVariance =
    (stdDevValue ?? null) !== null &&
    (meanValue ?? null) !== null &&
    meanValue !== 0 &&
    stdDevValue! / Math.abs(meanValue!) > 0.5;

  // Determine which name to show as primary (proposed) vs secondary (original)
  const proposedName = config?.new_name ?? col.name;
  const hasNameChange = proposedName !== col.name;

  return `
    <tr class="analyser-row ${isExpanded ? 'expanded' : ''}" data-col="${escapeHtml(col.name)}" data-testid="analyser-row-${escapeHtml(col.name)}">
      <td><i class="ph ${isExpanded ? 'ph-caret-down' : 'ph-caret-right'} expand-toggle"></i></td>
      <td>
        <div class="col-name-box">
          ${
            isReadOnly
              ? `
            <input type="checkbox" name="select-col-${escapeHtml(col.name)}" class="col-select-checkbox" data-col="${escapeHtml(col.name)}" data-testid="col-select-checkbox-${escapeHtml(col.name)}" ${isSelected ? 'checked' : ''} title="Include in cleaning">
          `
              : ''
          }
          <div class="col-name-display">
            <span class="col-name">${escapeHtml(proposedName)}</span>
            ${
              hasNameChange
                ? `
              <span class="col-name-original">Originally: "${escapeHtml(col.name)}"</span>
            `
                : ''
            }
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
      <td class="mono">${escapeHtml(median)}</td>
      <td class="mono">${escapeHtml(min)}</td>
      <td class="mono">${escapeHtml(max)}</td>
      <td class="mono">${escapeHtml(stdDev)}</td>
      <td class="cleaning-cell">
        ${
          isReadOnly
            ? `
          <div class="readonly-actions">
            ${nullPct > 50 ? '<span class="recommendation-badge badge-warn" title="High null percentage">High Nulls</span>' : ''}
            ${uniquePct > 95 ? '<span class="recommendation-badge badge-info" title="Highly unique - consider dropping">Unique</span>' : ''}
            ${isHighlySkewed ? '<span class="recommendation-badge badge-warn" title="Skewed distribution">Skewed</span>' : ''}
            ${isHighVariance ? '<span class="recommendation-badge badge-info" title="High coefficient of variation">High Variance</span>' : ''}
            ${uniqueCount === col.count - col.nulls ? '<span class="recommendation-badge badge-info" title="All values are unique">All Unique</span>' : ''}
          </div>
        `
            : `
          <div class="cleaning-summary">
            ${config?.active ? '<span class="active-dot" title="Cleaning Active"></span>' : ''}
            ${config?.impute_mode !== 'None' ? `<span class="clean-tag">Impute: ${config?.impute_mode}</span>` : ''}
            ${config?.normalisation !== 'None' ? `<span class="clean-tag">Norm: ${config?.normalisation}</span>` : ''}
          </div>
        `
        }
      </td>
    </tr>
    ${
      isExpanded
        ? `
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

              ${
                !isReadOnly
                  ? `
                <div class="details-cleaning">
                  <h4>Cleaning Pipeline</h4>
                  <div class="cleaning-controls">
                    <div class="control-group">
                      ${
                        currentStage === 'Cleaned' ||
                        currentStage === 'Profiled' ||
                        currentStage === 'Raw'
                          ? `
                        <label><input type="checkbox" name="enable-cleaning-${escapeHtml(col.name)}" class="row-action" data-prop="active" ${config?.active ? 'checked' : ''} data-col="${escapeHtml(col.name)}" data-testid="clean-active-${escapeHtml(col.name)}"> Enable Cleaning</label>
                      `
                          : `
                        <div class="cleaning-status-indicator">
                          <i class="ph ${config?.active ? 'ph-check-circle' : 'ph-x-circle'}"></i>
                          <span>Cleaning is ${config?.active ? 'enabled' : 'disabled'} (change in Cleaning stage)</span>
                        </div>
                      `
                      }
                    </div>

                    <div class="control-grid">
                      ${
                        currentStage === 'Advanced' ||
                        currentStage === 'Validated' ||
                        currentStage === 'Published'
                          ? `
                        <div class="control-item">
                          <label title="Fill missing values - options based on column type">Handle Nulls (Impute)</label>
                          ${renderSelect(getImputeOptionsForColumn(col.kind), config?.impute_mode ?? 'None', 'row-action', { col: col.name, prop: 'impute_mode' }, undefined, !advancedProcessingEnabled)}
                        </div>
                        ${
                          col.kind === 'Numeric'
                            ? `
                        <div class="control-item">
                          <label title="Scale numeric values to standard range">Normalisation</label>
                          ${renderSelect(NORM_OPTIONS, config?.normalisation ?? 'None', 'row-action', { col: col.name, prop: 'normalisation' }, undefined, !advancedProcessingEnabled)}
                        </div>
                        `
                            : ''
                        }
                        ${
                          col.kind === 'Categorical'
                            ? `
                        <div class="control-item">
                          <label title="Convert categorical column into binary columns (one column per category)">
                            <input type="checkbox" name="one-hot-encode-${escapeHtml(col.name)}" class="row-action" data-prop="one_hot_encode" ${config?.one_hot_encode ? 'checked' : ''} data-col="${escapeHtml(col.name)}" data-testid="one-hot-encode-${escapeHtml(col.name)}" ${!advancedProcessingEnabled ? 'disabled' : ''}>
                            One-Hot Encode
                          </label>
                        </div>
                        `
                            : ''
                        }
                      `
                          : ''
                      }
                      ${
                        currentStage !== 'Advanced' &&
                        currentStage !== 'Validated' &&
                        currentStage !== 'Published'
                          ? `
                        ${
                          col.kind === 'Text'
                            ? `
                          <div class="control-item">
                            <label>Text Case</label>
                            ${renderSelect(CASE_OPTIONS, config?.text_case ?? 'None', 'row-action', { col: col.name, prop: 'text_case' })}
                          </div>
                        `
                            : ''
                        }
                        ${
                          col.kind === 'Numeric'
                            ? `
                          <div class="control-item">
                            <label>Rounding</label>
                            ${renderSelect(ROUND_OPTIONS, config?.rounding?.toString() ?? 'none', 'row-action', { col: col.name, prop: 'rounding' })}
                          </div>
                        `
                            : ''
                        }
                      `
                          : ''
                      }
                    </div>

                    ${
                      (currentStage === 'Advanced' ||
                        currentStage === 'Validated' ||
                        currentStage === 'Published') &&
                      col.kind === 'Numeric'
                        ? `
                      <div class="control-advanced">
                        <label title="Clip extreme values to 5th and 95th percentiles to reduce outlier impact">
                          <input type="checkbox" name="clip-outliers-${escapeHtml(col.name)}" class="row-action" data-prop="clip_outliers" ${config?.clip_outliers ? 'checked' : ''} data-col="${escapeHtml(col.name)}" data-testid="clip-outliers-${escapeHtml(col.name)}" ${!advancedProcessingEnabled ? 'disabled' : ''}>
                          Clip Outliers
                        </label>
                      </div>
                    `
                        : ''
                    }
                  </div>
                </div>
              `
                  : ''
              }

              ${renderDistribution(col)}
              ${renderInsights(col)}
            </div>
          </div>
        </td>
      </tr>
    `
        : ''
    }
  `;
}

export function renderDistribution(col: ColumnSummary): string {
  if (col.stats.Numeric?.histogram) {
    const hist = col.stats.Numeric.histogram;
    const maxCount = Math.max(...hist.map(h => h[1]));
    return `
      <div class="details-distribution">
        <h4>Distribution</h4>
        <div class="histogram">
          <div class="hist-bars">
            ${hist
              .map(
                ([val, count]) => `
              <div class="hist-bar" style="height: ${(count / maxCount) * 100}%" title="${val.toFixed(2)}: ${count}"></div>
            `
              )
              .join('')}
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
          ${top
            .map(
              ([val, count]) => `
            <div class="top-val-row">
              <span class="top-val-label" title="${escapeHtml(val)}">${escapeHtml(val || '(empty)')}</span>
              <div class="top-val-bar-container">
                <div class="top-val-bar" style="width: ${(count / maxCount) * 100}%"></div>
              </div>
              <span class="top-val-count">${count}</span>
            </div>
          `
            )
            .join('')}
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

export function renderSchemaSidebar(
  response: AnalysisResponse,
  configs: Record<string, ColumnCleanConfig>
): string {
  return `
    <div class="schema-sidebar">
      <h3>Cleaning Schema</h3>
      <div class="schema-stats">
        <span>${Object.values(configs).filter(c => c.active).length} active transforms</span>
      </div>
      <div class="schema-list">
        ${response.summary
          .map(col => {
            const config = configs[col.name];
            if (!config || !config.active) return '';
            return `
            <div class="schema-item">
              <span class="schema-col-name">${escapeHtml(col.name)}</span>
              <div class="schema-badges">
                ${config.impute_mode !== 'None' ? `<span class="badge">Impute</span>` : ''}
                ${config.normalisation !== 'None' ? `<span class="badge">Norm</span>` : ''}
                ${config.text_case !== 'None' ? `<span class="badge">Case</span>` : ''}
                ${config.ml_preprocessing ? `<span class="badge-ml">ML</span>` : ''}
              </div>
            </div>
          `;
          })
          .join('')}
      </div>
      <div class="schema-actions">
        <button id="btn-clear-schema" class="btn-secondary btn-block">Clear All</button>
      </div>
    </div>
  `;
}
