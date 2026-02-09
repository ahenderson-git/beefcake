import { AnalysisResponse, CurrentDataset, DatasetVersion, TransformSpec } from '../../types';
import { escapeHtml, fmtBytes } from '../../utils';

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
  const nullColumns = (response.summary || []).filter(col => {
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
              ${(response.summary ?? [])
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
