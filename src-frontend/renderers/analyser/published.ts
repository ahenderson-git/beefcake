import { AnalysisResponse, CurrentDataset, DatasetVersion } from '../../types';
import { escapeHtml, fmtBytes } from '../../utils';

export function renderPublishedView(
  response: AnalysisResponse,
  dataset: CurrentDataset | null
): string {
  // Get the Published version details
  const publishedVersion = dataset?.versions?.find((v: DatasetVersion) => v.stage === 'Published');
  const publishMode: 'view' | 'snapshot' = 'snapshot'; // Default to snapshot mode for display
  const publishDate = publishedVersion
    ? new Date(publishedVersion.created_at).toLocaleString()
    : 'Unknown';

  // Calculate quality metrics
  const nullColumns = (response.summary ?? []).filter(col => {
    const nullPct = (col.nulls / col.count) * 100;
    return nullPct > 0;
  });
  const avgNullPct =
    nullColumns.length > 0
      ? nullColumns.reduce((sum, col) => sum + (col.nulls / col.count) * 100, 0) /
        nullColumns.length
      : 0;

  return `
    <div class="published-view">
      <div class="published-banner">
        <i class="ph ph-rocket-launch"></i>
        <div>
          <h3>🎉 Dataset Published to Production</h3>
          <p>This dataset is now immutable and ready for production use. Published as <strong>${publishMode}</strong> on ${publishDate}.</p>
        </div>
      </div>

      <div class="metrics-dashboard">
        <div class="metric-card">
          <h4><i class="ph ph-shield-check"></i> Production Metrics</h4>
          <div class="metric-stats">
            <div class="metric-row">
              <span>Health Score:</span>
              <span class="metric-value">${Math.round(response.health.score * 100)}%</span>
            </div>
            <div class="metric-row">
              <span>Rows:</span>
              <span class="metric-value">${response.total_row_count.toLocaleString()}</span>
            </div>
            <div class="metric-row">
              <span>Columns:</span>
              <span class="metric-value">${response.column_count}</span>
            </div>
            <div class="metric-row">
              <span>Avg Nulls:</span>
              <span class="metric-value">${avgNullPct.toFixed(1)}%</span>
            </div>
          </div>
        </div>

        <div class="metric-card">
          <h4><i class="ph ph-lock-key"></i> Version Info</h4>
          <div class="metric-stats">
            <div class="metric-row">
              <span>Mode:</span>
              <span class="metric-value">Snapshot (Materialized)</span>
            </div>
            <div class="metric-row">
              <span>Published:</span>
              <span class="metric-value">${publishDate}</span>
            </div>
            <div class="metric-row">
              <span>Size:</span>
              <span class="metric-value">${fmtBytes(response.file_size)}</span>
            </div>
            <div class="metric-row">
              <span>Status:</span>
              <span class="metric-value">🔒 Immutable</span>
            </div>
          </div>
        </div>
      </div>

      <div class="published-reusability">
        <h4><i class="ph ph-code"></i> Reusability & Automation</h4>
        <p class="section-description">Generate scripts to reproduce this pipeline or export the configuration.</p>

        <div class="command-tabs">
          <button class="command-tab active" data-tab="powershell" id="tab-powershell">
            <i class="ph ph-terminal"></i> PowerShell
          </button>
          <button class="command-tab" data-tab="python" id="tab-python">
            <i class="ph ph-code"></i> Python
          </button>
          <button class="command-tab" data-tab="json" id="tab-json">
            <i class="ph ph-brackets-curly"></i> JSON Config
          </button>
        </div>

        <div class="tab-content" id="published-tab-content">
          <div class="code-snippet-container">
            <pre class="mono"><code id="published-code-snippet"># Loading Published Dataset in PowerShell
$DatasetId = "${dataset?.id}"
$VersionId = "${publishedVersion?.id}"
$DataPath = "${publishedVersion?.data_location?.ParquetFile?.replace(/\\/g, '\\\\') ?? publishedVersion?.data_location?.OriginalFile?.replace(/\\/g, '\\\\') ?? ''}"

# Load using Beefcake CLI
beefcake pipeline execute --input $DataPath --spec ./pipeline.json</code></pre>
            <button class="btn-copy" title="Copy to clipboard">
              <i class="ph ph-copy"></i>
            </button>
          </div>
        </div>
      </div>

      <div class="final-dataset-preview">
        <h4><i class="ph ph-table"></i> Production Snapshot Overview</h4>
        <p class="preview-description">Read-only preview of the published production data.</p>
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

      <div class="published-actions">
        <button id="btn-back-to-validated" class="btn-secondary">
          <i class="ph ph-arrow-left"></i> Back to Validation
        </button>
        <button id="btn-view-audit-trail" class="btn-ghost">
          <i class="ph ph-fingerprint"></i> View Audit Trail
        </button>
      </div>
    </div>
  `;
}
