import { DbConnection, ExportSource } from '../types';
import { escapeHtml } from '../utils';

export function renderExportConfig(
  destType: 'File' | 'Database',
  connections: DbConnection[],
  activeExportId: string | null | undefined
): string {
  if (destType === 'File') {
    return `
      <div class="export-step">
        <label>2. Choose Format & Location</label>
        <div class="input-with-button">
          <input type="text" id="export-file-path" placeholder="C:\\path\\to\\export.parquet" readonly>
          <button type="button" id="btn-browse-export" class="btn-secondary btn-small">Browse</button>
        </div>
        <p class="help-text">Recommended formats: .parquet (high performance), .csv, .json</p>
      </div>
      <div class="export-step">
        <label class="checkbox-label">
          <input type="checkbox" id="export-create-dictionary" checked>
          <span>Create data dictionary snapshot</span>
        </label>
        <p class="help-text">Automatically generates metadata documentation (JSON + Markdown)</p>
      </div>
    `;
  } else {
    return `
      <div class="export-step">
        <label>2. Select Connection</label>
        <select id="export-connection-id">
          <option value="">-- Choose Connection --</option>
          ${connections
            .map(
              conn => `
            <option value="${conn.id}" ${conn.id === activeExportId ? 'selected' : ''}>${escapeHtml(conn.name)} (${escapeHtml(conn.settings.database)}.${escapeHtml(conn.settings.table)})</option>
          `
            )
            .join('')}
        </select>
      </div>
    `;
  }
}

export function renderExportModal(
  source: ExportSource,
  connections: DbConnection[],
  activeExportId: string | null | undefined,
  destType: 'File' | 'Database',
  isLoading: boolean,
  isAborting: boolean
): string {
  return `
    <div class="modal-overlay" id="export-modal" data-testid="export-modal-overlay">
      <div class="modal export-modal" data-testid="export-modal">
        <div class="modal-header">
          <h3><i class="ph ph-export"></i> Export Data</h3>
          <button type="button" class="btn-close-modal" data-testid="export-modal-close"><i class="ph ph-x"></i></button>
        </div>
        <div class="modal-body" data-testid="export-modal-body">
          <div class="export-step">
            <label>1. Select Destination Type</label>
            <div class="dest-toggle">
              <button type="button" class="toggle-btn ${destType === 'File' ? 'active' : ''}" data-dest="File" data-testid="export-dest-file">
                <i class="ph ph-file-arrow-down"></i> Local File
              </button>
              <button type="button" class="toggle-btn ${destType === 'Database' ? 'active' : ''}" data-dest="Database" data-testid="export-dest-database">
                <i class="ph ph-database"></i> Database
              </button>
            </div>
          </div>

          <div id="export-config-container">
            ${renderExportConfig(destType, connections, activeExportId)}
          </div>

          <div class="export-summary">
            <h4>Export Summary</h4>
            <div class="summary-grid">
              <div class="summary-item"><span>Source</span> <span>${source.type}</span></div>
              <div class="summary-item"><span>Processing</span> <span>Optimised Streaming</span></div>
            </div>
          </div>
        </div>
        <div class="modal-footer ${isLoading ? 'modal-footer-loading' : ''}">
          ${!isLoading ? '<button type="button" class="btn-secondary btn-close-modal" data-testid="export-cancel-button">Cancel</button>' : ''}
          ${
            isLoading
              ? `<div class="loading-button-group">
                <button type="button" class="btn-primary" disabled><div class="spinner-small"></div> Processing...</button>
                <button type="button" id="btn-abort-export" class="btn-danger btn-small" data-testid="export-abort-button">${isAborting ? 'Aborting...' : 'Abort'}</button>
              </div>`
              : `<button type="button" id="btn-start-export" class="btn-primary" data-testid="export-confirm-button" ${destType === 'Database' && !activeExportId ? 'disabled' : ''}>
                <i class="ph ph-rocket-launch"></i> Start Export
              </button>`
          }
        </div>
      </div>
    </div>
  `;
}
