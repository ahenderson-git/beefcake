import { AnalysisResponse } from '../../types';

export class DatasetStats {
  totalCells: number = 0;
  totalNulls: number = 0;
  overallQuality: number = 0;
  numericCols: number = 0;
  textCols: number = 0;
  temporalCols: number = 0;
  categoricalCols: number = 0;
  highCardinalityCols: number = 0;
}

export function computeDatasetStats(response: AnalysisResponse): DatasetStats {
  const stats = new DatasetStats();
  stats.totalCells = response.total_row_count * response.column_count;

  response.summary?.forEach(col => {
    stats.totalNulls += col.nulls;
    if (col.kind === 'Numeric') stats.numericCols++;
    if (col.kind === 'Text') stats.textCols++;
    if (col.kind === 'Temporal') stats.temporalCols++;
    if (col.kind === 'Categorical') stats.categoricalCols++;

    // High cardinality heuristic (>50% unique and not a tiny dataset)
    if (col.count > 10) {
      let uniqueCount = 0;
      if (col.stats.Numeric?.distinct_count !== undefined) {
        uniqueCount = col.stats.Numeric.distinct_count;
      } else if (col.stats.Text?.distinct !== undefined) {
        uniqueCount = col.stats.Text.distinct;
      } else if (col.stats.Temporal?.distinct_count !== undefined) {
        uniqueCount = col.stats.Temporal.distinct_count;
      } else if (col.stats.Categorical) {
        uniqueCount = Object.keys(col.stats.Categorical).length;
      } else if (col.stats.Boolean) {
        const { true_count, false_count } = col.stats.Boolean;
        uniqueCount = (true_count > 0 ? 1 : 0) + (false_count > 0 ? 1 : 0);
      }
      if (uniqueCount / col.count > 0.5) {
        stats.highCardinalityCols++;
      }
    }
  });

  stats.overallQuality =
    stats.totalCells > 0 ? ((stats.totalCells - stats.totalNulls) / stats.totalCells) * 100 : 0;

  return stats;
}

export function renderDatasetOverview(response: AnalysisResponse): string {
  const stats = computeDatasetStats(response);

  return `
    <div class="metrics-dashboard" data-testid="analyser-metrics-dashboard">
      <div class="metric-card" data-testid="analyser-quality-score-card">
        <h4><i class="ph ph-shield-check"></i> Quality Score</h4>
        <div class="quality-gauge">
          <div class="gauge-value" data-testid="analyser-quality-score">${stats.overallQuality.toFixed(1)}%</div>
          <div class="gauge-bar">
            <div class="gauge-fill" style="width: ${stats.overallQuality}%"></div>
          </div>
        </div>
        <p class="metric-hint">${stats.totalNulls.toLocaleString()} missing values across all columns</p>
      </div>

      <div class="metric-card" data-testid="analyser-type-distribution-card">
        <h4><i class="ph ph-chart-pie"></i> Type Distribution</h4>
        <div class="type-distribution">
          ${[
            { label: 'Num', count: stats.numericCols, icon: 'ph-hash' },
            { label: 'Txt', count: stats.textCols, icon: 'ph-text-t' },
            { label: 'Time', count: stats.temporalCols, icon: 'ph-calendar' },
            { label: 'Cat', count: stats.categoricalCols, icon: 'ph-check-square' },
          ]
            .filter(t => t.count > 0)
            .map(
              t => `
            <div class="type-pill">
              <i class="ph ${t.icon}"></i>
              <span>${t.label}: ${t.count}</span>
            </div>
          `
            )
            .join('')}
        </div>
      </div>

      <div class="metric-card">
        <h4><i class="ph ph-database"></i> Dataset Size</h4>
        <div class="metric-stats">
          <div class="metric-row">
            <span class="metric-label"><i class="ph ph-rows"></i> Rows:</span>
            <span class="metric-value">${response.total_row_count.toLocaleString()}</span>
          </div>
          <div class="metric-row">
            <span class="metric-label"><i class="ph ph-columns"></i> Columns:</span>
            <span class="metric-value">${response.column_count.toLocaleString()}</span>
          </div>
          <div class="metric-row">
            <span class="metric-label"><i class="ph ph-grid-four"></i> Total Cells:</span>
            <span class="metric-value">${stats.totalCells.toLocaleString()}</span>
          </div>
        </div>
      </div>

      <div class="metric-card">
        <h4><i class="ph ph-lightning"></i> Complexity</h4>
        <div class="metric-stats">
          <div class="metric-row">
            <span class="metric-label"><i class="ph ph-flag"></i> High-cardinality:</span>
            <span class="metric-value">${stats.highCardinalityCols} cols</span>
          </div>
          <div class="metric-row">
            <span class="metric-label"><i class="ph ph-warning-circle"></i> Has Nulls:</span>
            <span class="metric-value">${(response.summary || []).filter(c => c.nulls > 0).length} cols</span>
          </div>
          ${
            stats.temporalCols > 0
              ? `<div class="metric-row">
            <span class="metric-label"><i class="ph ph-calendar"></i> Time-series:</span>
            <span class="metric-value">${stats.temporalCols} cols</span>
          </div>`
              : ''
          }
        </div>
      </div>
    </div>
  `;
}

export function renderEmptyAnalyser(): string {
  return `
    <div class="empty-analyser-container" data-testid="analyser-view">
      <div class="empty-analyser-card">
        <i class="ph ph-file-search empty-analyser-icon"></i>
        <h2>No Data Loaded</h2>
        <p class="empty-analyser-description">Get started by selecting a file to analyze and profile</p>
        <button id="btn-open-file-empty" class="btn-primary btn-select-file">
          <i class="ph ph-file-plus"></i> Select File
        </button>
        <div class="supported-formats">
          <span class="format-badge">CSV</span>
          <span class="format-badge">Parquet</span>
          <span class="format-badge">JSON</span>
        </div>
      </div>
    </div>
  `;
}
