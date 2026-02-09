import { AnalysisResponse, ColumnCleanConfig, LifecycleStage } from '../../types';
import { escapeHtml } from '../../utils';

export function renderSchemaSidebar(
  response: AnalysisResponse,
  configs: Record<string, ColumnCleanConfig>,
  currentStage: LifecycleStage | null = null,
  _isReadOnly: boolean = false,
  selectedColumns: Set<string> = new Set(),
  hasDataset: boolean = false
): string {
  const activeCount = Object.values(configs || {}).filter(c => c.active).length;

  // Determine which action button to show based on stage
  let actionButton = '';

  // Show button if we have analysis data and are in an appropriate stage
  // This works for both: dataset-based analysis AND ad-hoc file analysis
  if ((currentStage === 'Profiled' || currentStage === 'Raw') && response) {
    const selectedCount = selectedColumns.size;
    const isDisabled = selectedCount === 0;
    actionButton = `
        <div class="sidebar-footer">
          <button id="btn-begin-cleaning" class="btn-primary btn-full-width" ${isDisabled ? 'disabled' : ''}>
            <i class="ph ph-broom"></i> Begin Cleaning
          </button>
          ${selectedCount > 0 ? `<p class="sidebar-hint">${selectedCount} column${selectedCount !== 1 ? 's' : ''} selected</p>` : '<p class="sidebar-hint warn">Select at least one column to continue</p>'}
        </div>
      `;
  } else if (currentStage === 'Cleaned' && hasDataset) {
    actionButton = `
        <div class="sidebar-footer">
          <button id="btn-continue-advanced" class="btn-primary btn-full-width">
            <i class="ph ph-arrow-right"></i> Continue to Advanced
          </button>
          <p class="sidebar-hint">Enable ML preprocessing features</p>
        </div>
      `;
  } else if (currentStage === 'Advanced' && hasDataset) {
    actionButton = `
        <div class="sidebar-footer">
          <button id="btn-move-to-validated" class="btn-primary btn-full-width">
            <i class="ph ph-check-circle"></i> Move to Validated
          </button>
          <p class="sidebar-hint">Finalize and prepare for publishing</p>
        </div>
      `;
  }

  return `
    <div class="schema-sidebar">
      <div class="sidebar-header">
        <h4><i class="ph ph-table"></i> Schema</h4>
        <span class="column-count">${activeCount} / ${response.column_count} active</span>
      </div>
      <div class="column-list-compact">
        ${(response.summary || [])
          .map(col => {
            const config = configs[col.name];
            const isActive = config?.active ?? true;
            return `
            <div class="column-item-compact ${isActive ? 'active' : 'inactive'}" data-col="${escapeHtml(col.name)}">
              <i class="ph ${isActive ? 'ph-check-circle' : 'ph-circle'}"></i>
              <span class="col-name">${escapeHtml(config?.new_name || col.name)}</span>
            </div>
          `;
          })
          .join('')}
      </div>
      ${actionButton}
    </div>
  `;
}
