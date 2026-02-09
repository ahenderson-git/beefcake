import { ColumnCleanConfig, ColumnSummary, LifecycleStage } from '../../types';
import { escapeHtml } from '../../utils';
import { CASE_OPTIONS, getImputeOptionsForColumn, NORM_OPTIONS, renderSelect } from '../common';

export function getUniqueCount(col: ColumnSummary): number {
  // Extract distinct count based on column stats type
  if (col.stats.Numeric?.distinct_count !== undefined) {
    return col.stats.Numeric.distinct_count;
  }
  if (col.stats.Text?.distinct !== undefined) {
    return col.stats.Text.distinct;
  }
  if (col.stats.Temporal?.distinct_count !== undefined) {
    return col.stats.Temporal.distinct_count;
  }
  if (col.stats.Categorical) {
    return Object.keys(col.stats.Categorical).length;
  }
  if (col.stats.Boolean) {
    // Boolean columns have at most 2 distinct values
    const { true_count, false_count } = col.stats.Boolean;
    return (true_count > 0 ? 1 : 0) + (false_count > 0 ? 1 : 0);
  }
  // Final fallback
  return 0;
}

export function getMeanOrMode(col: ColumnSummary): string | number {
  if (
    col.kind === 'Numeric' &&
    col.stats.Numeric?.mean !== undefined &&
    col.stats.Numeric.mean !== null
  ) {
    return col.stats.Numeric.mean.toFixed(2);
  }
  if (col.kind === 'Text' && col.stats.Text?.top_value) {
    return col.stats.Text.top_value[0];
  }
  if (col.kind === 'Categorical' && col.stats.Categorical) {
    // Categorical is a HashMap - convert to sorted entries
    const entries = Object.entries(col.stats.Categorical).sort((a, b) => b[1] - a[1]);
    return entries.length > 0 && entries[0] ? entries[0][0] : '-';
  }
  return '-';
}

export function getMedian(col: ColumnSummary): string | number {
  if (
    col.kind === 'Numeric' &&
    col.stats.Numeric?.median !== undefined &&
    col.stats.Numeric.median !== null
  ) {
    return col.stats.Numeric.median.toFixed(2);
  }
  return '-';
}

export function getStdDev(col: ColumnSummary): string | number {
  if (
    col.kind === 'Numeric' &&
    col.stats.Numeric?.std_dev !== undefined &&
    col.stats.Numeric.std_dev !== null
  ) {
    return col.stats.Numeric.std_dev.toFixed(2);
  }
  return '-';
}

export function getMinMax(col: ColumnSummary): string {
  const numericStats = col.kind === 'Numeric' ? col.stats.Numeric : undefined;
  if (numericStats && numericStats.min !== null && numericStats.max !== null) {
    return `${numericStats.min.toFixed(1)} / ${numericStats.max.toFixed(1)}`;
  }
  return '-';
}

export function renderEnhancedStats(
  col: ColumnSummary,
  nullPct: number,
  uniqueCount: number,
  uniquePct: number
): string {
  const isNumeric = col.kind === 'Numeric';
  const isText = col.kind === 'Text' || col.kind === 'Categorical';

  return `
    <div class="enhanced-stats-grid">
      <div class="stat-group">
        <h5>Quality & Cardinality</h5>
        <div class="stat-row">
          <span>Nulls:</span>
          <span class="stat-value ${nullPct > 5 ? 'warn' : ''}">${col.nulls.toLocaleString()} (${nullPct.toFixed(1)}%)</span>
        </div>
        <div class="stat-row">
          <span>Unique:</span>
          <span class="stat-value">${uniqueCount.toLocaleString()} (${uniquePct.toFixed(1)}%)</span>
        </div>
      </div>

      <div class="stat-group">
        <h5>Distribution</h5>
        <div class="stat-row">
          <span>Mean/Mode:</span>
          <span class="stat-value mono">${escapeHtml(getMeanOrMode(col).toString())}</span>
        </div>
        ${
          isNumeric
            ? `
          <div class="stat-row">
            <span>Median:</span>
            <span class="stat-value mono">${getMedian(col)}</span>
          </div>
          <div class="stat-row">
            <span>Std Dev:</span>
            <span class="stat-value mono">${getStdDev(col)}</span>
          </div>
        `
            : ''
        }
      </div>

      ${
        isNumeric
          ? `
        <div class="stat-group">
          <h5>Range</h5>
          <div class="stat-row">
            <span>Min / Max:</span>
            <span class="stat-value mono">${getMinMax(col)}</span>
          </div>
        </div>
      `
          : ''
      }

      ${
        isText
          ? `
        <div class="stat-group">
          <h5>Top Values</h5>
          <div class="top-values-list">
            ${(col.stats.Text?.top_value
              ? [col.stats.Text.top_value]
              : col.stats.Categorical
                ? Object.entries(col.stats.Categorical)
                    .sort((a, b) => b[1] - a[1])
                    .slice(0, 3)
                : []
            )
              .map(
                ([val, count]: [string | number | boolean, number]) => `
              <div class="top-value-item">
                <span class="val mono">${escapeHtml(val.toString())}</span>
                <span class="count">${count.toLocaleString()}</span>
              </div>
            `
              )
              .join('')}
          </div>
        </div>
      `
          : ''
      }
    </div>
  `;
}

export function renderAnalyserRow(
  col: ColumnSummary,
  isExpanded: boolean,
  config: ColumnCleanConfig | undefined,
  isReadOnly: boolean,
  isSelected: boolean,
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

  const isAdvancedStage = currentStage === 'Advanced' || advancedProcessingEnabled;

  return `
    <div class="analyser-row ${isExpanded ? 'expanded' : ''} ${!config?.active ? 'inactive' : ''} ${isSelected ? 'selected' : ''}" data-col="${escapeHtml(col.name)}" data-testid="analyser-column-row">
      <div class="row-main">
        <div class="col-select">
          <input type="checkbox" class="col-select-checkbox" data-col="${escapeHtml(col.name)}" data-testid="analyser-column-checkbox" ${isSelected ? 'checked' : ''} ${isReadOnly && currentStage !== 'Profiled' ? 'disabled' : ''}>
        </div>
        <div class="col-expander" data-testid="analyser-column-expander">
          <i class="ph ph-caret-right"></i>
        </div>
        <div class="col-name" title="${escapeHtml(col.name)}" data-testid="analyser-column-name">
          <i class="ph ${typeIcon} type-icon"></i>
          <span class="name-text">${escapeHtml(config?.new_name || col.name)}</span>
          ${config?.new_name && config.new_name !== col.name ? `<span class="original-name">(${escapeHtml(col.name)})</span>` : ''}
        </div>
        <div class="col-type" data-testid="analyser-column-type">
          <span class="type-badge">${col.kind}</span>
        </div>
        <div class="col-quality" data-testid="analyser-column-quality">
          <div class="quality-bar-container">
            <div class="quality-bar ${qualityClass}" style="width: ${Math.max(5, 100 - nullPct)}%"></div>
            <span class="quality-text">${(100 - nullPct).toFixed(0)}% quality</span>
          </div>
        </div>
        <div class="col-stats-summary" data-testid="analyser-column-stats">
          <span class="stat-pill" data-testid="analyser-column-null-pct">Nulls: ${nullPct.toFixed(1)}%</span>
          <span class="stat-pill" data-testid="analyser-column-unique-pct">Unique: ${uniquePct.toFixed(1)}%</span>
        </div>
        <div class="col-actions">
           ${
             !isReadOnly
               ? `
            <button class="btn-icon btn-active-toggle ${config?.active ? 'active' : ''}" title="${config?.active ? 'Disable column' : 'Enable column'}">
              <i class="ph ${config?.active ? 'ph-eye' : 'ph-eye-slash'}"></i>
            </button>
          `
               : ''
           }
        </div>
      </div>

      <div class="row-details">
        <div class="details-grid">
          <div class="details-stats">
            ${renderEnhancedStats(col, nullPct, uniqueCount, uniquePct)}
            ${renderDistribution(col)}
            ${renderInsights(col)}
          </div>

          ${
            !isReadOnly && config
              ? `
          <div class="details-config">
            <h5><i class="ph ph-gear"></i> Transformations</h5>

            <div class="config-section">
              <label>Target Name</label>
              <input type="text" class="config-input config-name" value="${escapeHtml(config.new_name)}" placeholder="Enter new name...">
            </div>

            <div class="config-grid">
              <div class="config-section">
                <label>Text Cleaning</label>
                <div class="checkbox-group">
                  <label class="checkbox-control">
                    <input type="checkbox" class="config-trim" ${config.trim_whitespace ? 'checked' : ''}>
                    <span>Trim Whitespace</span>
                  </label>
                  <label class="checkbox-control">
                    <input type="checkbox" class="config-nulls" ${config.standardise_nulls ? 'checked' : ''}>
                    <span>Standardise Nulls</span>
                  </label>
                  <label class="checkbox-control">
                    <input type="checkbox" class="config-special" ${config.remove_special_chars ? 'checked' : ''}>
                    <span>Remove Special Chars</span>
                  </label>
                </div>
              </div>

              <div class="config-section">
                <label>Text Case</label>
                ${renderSelect(CASE_OPTIONS, config.text_case, 'config-case', {})}
              </div>
            </div>

            ${
              isAdvancedStage
                ? `
              <div class="advanced-config-divider">
                <span>Advanced transformations</span>
              </div>

              <div class="config-grid">
                <div class="config-section">
                  <label>Imputation (Handle Nulls)</label>
                  ${renderSelect(getImputeOptionsForColumn(col.kind), config.impute_mode, 'config-impute', {})}
                </div>
                <div class="config-section">
                  <label>Normalisation</label>
                  ${renderSelect(NORM_OPTIONS, config.normalisation, 'config-norm', {})}
                </div>
              </div>

              <div class="config-section">
                <div class="checkbox-group inline">
                  <label class="checkbox-control" title="Handle outliers by clipping to 1st/99th percentiles">
                    <input type="checkbox" class="config-outliers" ${config.clip_outliers ? 'checked' : ''}>
                    <span>Clip Outliers</span>
                  </label>
                  <label class="checkbox-control" title="Convert categorical values to numeric columns">
                    <input type="checkbox" class="config-onehot" ${config.one_hot_encode ? 'checked' : ''}>
                    <span>One-Hot Encode</span>
                  </label>
                </div>
              </div>
            `
                : ''
            }
          </div>
          `
              : ''
          }
        </div>
      </div>
    </div>
  `;
}

export function renderDistribution(col: ColumnSummary): string {
  if (col.kind === 'Numeric' && col.stats.Numeric?.histogram) {
    const hist = col.stats.Numeric.histogram;
    const maxCount = Math.max(...hist.map(h => h[1])); // h[1] is count in 2-tuple

    return `
      <div class="distribution-chart">
        <h5>Distribution</h5>
        <div class="histogram">
          ${hist
            .map(
              ([binCentre, count]) => `
            <div class="hist-bar" style="height: ${maxCount > 0 ? (count / maxCount) * 100 : 0}%" title="${binCentre.toFixed(2)}: ${count.toLocaleString()}"></div>
          `
            )
            .join('')}
        </div>
      </div>
    `;
  }

  if (col.kind === 'Categorical' && col.stats.Categorical) {
    // Convert HashMap to entries array
    const topValues = Object.entries(col.stats.Categorical);
    if (topValues.length === 0) return '';
    const maxCount = Math.max(...topValues.map(([_, count]) => count));

    return `
      <div class="distribution-chart">
        <h5>Top Categories</h5>
        <div class="cat-chart">
          ${topValues
            .sort((a, b) => b[1] - a[1])
            .slice(0, 5)
            .map(
              ([val, count]: [string | number | boolean, number]) => `
            <div class="cat-row">
              <span class="cat-label mono">${escapeHtml(val.toString())}</span>
              <div class="cat-bar-container">
                <div class="cat-bar" style="width: ${maxCount > 0 ? (count / maxCount) * 100 : 0}%"></div>
              </div>
              <span class="cat-count">${count.toLocaleString()}</span>
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
  const insights: string[] = [];

  if (col.nulls / col.count > 0.5) insights.push('Very high null rate (>50%)');
  if (getUniqueCount(col) === col.count) insights.push('Possible primary key (100% unique)');

  if (insights.length === 0) return '';

  return `
    <div class="column-insights">
      <h5><i class="ph ph-lightbulb"></i> Insights</h5>
      <ul>
        ${insights.map(s => `<li>${s}</li>`).join('')}
      </ul>
    </div>
  `;
}
