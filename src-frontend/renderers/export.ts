import { DbConnection, ExportSource } from "../types";
import { escapeHtml } from "../utils";

export function renderExportModal(
  source: ExportSource,
  connections: DbConnection[],
  activeExportId: string | null | undefined,
  destType: 'File' | 'Database',
  isLoading: boolean,
  isAborting: boolean
): string {
  return `
    <div class="modal-overlay">
      <div class="modal export-modal">
        <div class="modal-header">
          <h3><i class="ph ph-export"></i> Export Data</h3>
          <button class="btn-close-modal"><i class="ph ph-x"></i></button>
        </div>
        <div class="modal-body">
          <div class="export-step">
            <label>1. Select Destination Type</label>
            <div class="dest-toggle">
              <button class="toggle-btn ${destType === 'File' ? 'active' : ''}" data-dest="File">
                <i class="ph ph-file-arrow-down"></i> Local File
              </button>
              <button class="toggle-btn ${destType === 'Database' ? 'active' : ''}" data-dest="Database">
                <i class="ph ph-database"></i> Database
              </button>
            </div>
          </div>

          ${destType === 'File' ? `
            <div class="export-step">
              <label>2. Choose Format & Location</label>
              <div class="input-with-button">
                <input type="text" id="export-file-path" placeholder="C:\\path\\to\\export.parquet" readonly>
                <button id="btn-browse-export" class="btn-secondary btn-small">Browse</button>
              </div>
              <p class="help-text">Recommended formats: .parquet (high performance), .csv, .json</p>
            </div>
          ` : `
            <div class="export-step">
              <label>2. Select Connection</label>
              <select id="export-connection-id">
                <option value="">-- Choose Connection --</option>
                ${connections.map(conn => `
                  <option value="${conn.id}" ${conn.id === activeExportId ? 'selected' : ''}>${escapeHtml(conn.name)} (${escapeHtml(conn.settings.database)}.${escapeHtml(conn.settings.table)})</option>
                `).join('')}
              </select>
            </div>
          `}

          <div class="export-summary">
            <h4>Export Summary</h4>
            <div class="summary-grid">
              <div class="summary-item"><span>Source</span> <span>${source.type}</span></div>
              <div class="summary-item"><span>Processing</span> <span>Optimized Streaming</span></div>
            </div>
          </div>
        </div>
        <div class="modal-footer">
          <button class="btn-secondary btn-close-modal">Cancel</button>
          ${isLoading 
            ? `<button class="btn-primary" disabled><div class="spinner-small"></div> Processing...</button>
               ${isAborting ? '<span>Aborting...</span>' : '<button id="btn-abort-export" class="btn-danger btn-small">Abort</button>'}`
            : `<button id="btn-start-export" class="btn-primary" ${destType === 'Database' && !activeExportId ? 'disabled' : ''}>
                <i class="ph ph-rocket-launch"></i> Start Export
              </button>`
          }
        </div>
      </div>
    </div>
  `;
}
