import { AnalysisResponse, ColumnCleanConfig, LifecycleStage } from '../types';

import { renderAnalyserHeader } from './analyser/header';
import { renderPublishedView } from './analyser/published';
import { renderAnalyserRow } from './analyser/row';
import { renderSchemaSidebar } from './analyser/sidebar';
import { renderDatasetOverview, renderEmptyAnalyser } from './analyser/summary';
import { renderValidatedSummary } from './analyser/validated';

export {
  renderAnalyserHeader,
  renderDatasetOverview,
  renderEmptyAnalyser,
  renderValidatedSummary,
  renderPublishedView,
  renderAnalyserRow,
  renderSchemaSidebar,
};

export function renderAnalyser(
  response: AnalysisResponse | null,
  expandedRows: Set<string>,
  configs: Record<string, ColumnCleanConfig>,
  currentStage: LifecycleStage | null = null,
  isReadOnly: boolean = false,
  selectedColumns: Set<string> = new Set(),
  _useOriginalColumnNames: boolean = false,
  advancedProcessingEnabled: boolean = false
): string {
  if (!response) {
    return renderEmptyAnalyser();
  }

  // If in Validated stage, show special summary
  if (currentStage === 'Validated') {
    return renderValidatedSummary(response, null); // dataset will be passed by component if available
  }

  // If in Published stage, show special view
  if (currentStage === 'Published') {
    return renderPublishedView(response, null); // dataset will be passed by component if available
  }

  // Find problematic columns
  const issues = (response.summary || []).filter(col => (col.nulls / col.count) * 100 > 10);

  return `
    <div class="analyser-container" data-testid="analyser-view">
      ${renderAnalyserHeader(
        response,
        currentStage,
        isReadOnly,
        _useOriginalColumnNames,
        true,
        advancedProcessingEnabled
      )}

      ${renderDatasetOverview(response)}

      ${
        issues.length > 0
          ? `
        <div class="analysis-alerts">
          <div class="alert-info">
            <i class="ph ph-warning-circle"></i>
            <span>Found ${issues.length} columns with potential quality issues (high null rates).</span>
          </div>
        </div>
      `
          : ''
      }

      <div class="analyser-layout">
        <div class="analyser-main">
          <div class="analyser-table">
            <div class="table-header">
              <div class="col-select">
                <input type="checkbox" id="select-all-columns" ${selectedColumns.size === (response.summary?.length ?? 0) ? 'checked' : ''} ${isReadOnly ? 'disabled' : ''}>
              </div>
              <div class="col-expander"></div>
              <div class="col-name">Column</div>
              <div class="col-type">Type</div>
              <div class="col-quality">Quality</div>
              <div class="col-stats-summary">Summary</div>
              <div class="col-actions"></div>
            </div>
            <div class="table-body">
              ${(response.summary ?? [])
                .map(col =>
                  renderAnalyserRow(
                    col,
                    expandedRows.has(col.name),
                    configs[col.name],
                    isReadOnly,
                    selectedColumns.has(col.name),
                    currentStage,
                    advancedProcessingEnabled
                  )
                )
                .join('')}
            </div>
          </div>
        </div>
        ${renderSchemaSidebar(response, configs)}
      </div>
    </div>
  `;
}
