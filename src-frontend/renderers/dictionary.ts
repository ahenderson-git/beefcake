import { DataDictionary, SnapshotMetadata } from "../types";

/**
 * Render the Data Dictionary list view showing all snapshots.
 */
export function renderDictionaryList(snapshots: SnapshotMetadata[]): string {
  if (snapshots.length === 0) {
    return `
      <div class="empty-state">
        <h2>No Data Dictionary Snapshots</h2>
        <p>Data dictionary snapshots are created automatically when exporting datasets.</p>
        <p>Export a dataset to create your first snapshot.</p>
      </div>
    `;
  }

  const snapshotRows = snapshots.map(snapshot => {
    const date = new Date(snapshot.timestamp).toLocaleString();
    const completeness = snapshot.completeness_pct.toFixed(1);
    const completenessClass = snapshot.completeness_pct >= 70 ? 'high' : snapshot.completeness_pct >= 40 ? 'medium' : 'low';

    return `
      <tr class="snapshot-row" data-snapshot-id="${snapshot.snapshot_id}">
        <td><strong>${snapshot.dataset_name}</strong></td>
        <td>${date}</td>
        <td>${snapshot.row_count.toLocaleString()} × ${snapshot.column_count}</td>
        <td>
          <span class="completeness-badge completeness-${completenessClass}">
            ${completeness}%
          </span>
        </td>
        <td class="snapshot-actions">
          <button class="btn-view" data-snapshot-id="${snapshot.snapshot_id}" title="View/Edit">
            <i class="ph ph-eye"></i> View
          </button>
          <button class="btn-export-md" data-snapshot-id="${snapshot.snapshot_id}" title="Export Markdown">
            <i class="ph ph-file-text"></i> Export
          </button>
        </td>
      </tr>
    `;
  }).join('');

  return `
    <div class="dictionary-container">
      <div class="dictionary-header">
        <h1>Data Dictionary</h1>
        <div class="header-actions">
          <button id="btn-refresh-snapshots" class="btn-secondary">
            <i class="ph ph-arrow-clockwise"></i> Refresh
          </button>
        </div>
      </div>

      <table class="snapshots-table">
        <thead>
          <tr>
            <th>Dataset Name</th>
            <th>Created</th>
            <th>Dimensions</th>
            <th>Completeness</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          ${snapshotRows}
        </tbody>
      </table>
    </div>
  `;
}

/**
 * Calculate documentation completeness percentage.
 */
function calculateCompleteness(snapshot: DataDictionary): number {
  let totalFields = 0;
  let filledFields = 0;

  // Dataset level fields (6 fields in types.ts, tags is always array)
  const b = snapshot.dataset_metadata.business;
  const datasetFields: (keyof typeof b)[] = [
    'description', 'intended_use', 'owner_or_steward', 
    'refresh_expectation', 'sensitivity_classification', 'known_limitations'
  ];
  
  datasetFields.forEach(f => {
    totalFields++;
    if (b[f] && b[f] !== "") filledFields++;
  });

  // Column level fields (4 fields per column)
  snapshot.columns.forEach(col => {
    const cb = col.business;
    const colFields: (keyof typeof cb)[] = [
      'business_definition', 'business_rules', 'sensitivity_tag', 'notes'
    ];
    
    colFields.forEach(f => {
      totalFields++;
      if (cb[f] && cb[f] !== "") filledFields++;
    });
  });

  return totalFields > 0 ? (filledFields / totalFields) * 100 : 0;
}

/**
 * Render the Data Dictionary detail view for editing a snapshot.
 */
export function renderDictionaryDetail(snapshot: DataDictionary): string {
  const date = new Date(snapshot.export_timestamp).toLocaleString();
  const completeness = calculateCompleteness(snapshot).toFixed(1);
  const tech = snapshot.dataset_metadata.technical;
  const business = snapshot.dataset_metadata.business;

  // Render dataset business metadata form
  const datasetForm = `
    <div class="metadata-section">
      <h3>Dataset Business Metadata</h3>
      <form id="dataset-business-form">
        <div class="form-group">
          <label for="description">Description</label>
          <textarea id="description" name="description" rows="3" placeholder="High-level description of dataset purpose and contents">${business.description || ''}</textarea>
        </div>

        <div class="form-group">
          <label for="intended_use">Intended Use</label>
          <textarea id="intended_use" name="intended_use" rows="2" placeholder="Intended use cases for this dataset">${business.intended_use || ''}</textarea>
        </div>

        <div class="form-row">
          <div class="form-group">
            <label for="owner_or_steward">Owner/Steward</label>
            <input type="text" id="owner_or_steward" name="owner_or_steward" placeholder="Person or team responsible" value="${business.owner_or_steward || ''}">
          </div>

          <div class="form-group">
            <label for="refresh_expectation">Refresh Expectation</label>
            <input type="text" id="refresh_expectation" name="refresh_expectation" placeholder="e.g., Daily, Weekly" value="${business.refresh_expectation || ''}">
          </div>
        </div>

        <div class="form-row">
          <div class="form-group">
            <label for="sensitivity_classification">Sensitivity</label>
            <select id="sensitivity_classification" name="sensitivity_classification">
              <option value="">Select...</option>
              <option value="Public" ${business.sensitivity_classification === 'Public' ? 'selected' : ''}>Public</option>
              <option value="Internal" ${business.sensitivity_classification === 'Internal' ? 'selected' : ''}>Internal</option>
              <option value="Confidential" ${business.sensitivity_classification === 'Confidential' ? 'selected' : ''}>Confidential</option>
              <option value="Restricted" ${business.sensitivity_classification === 'Restricted' ? 'selected' : ''}>Restricted</option>
            </select>
          </div>
        </div>

        <div class="form-group">
          <label for="known_limitations">Known Limitations</label>
          <textarea id="known_limitations" name="known_limitations" rows="3" placeholder="Known limitations, caveats, or warnings">${business.known_limitations || ''}</textarea>
        </div>
      </form>
    </div>
  `;

  // Render column metadata table
  const columnRows = snapshot.columns.map(col => {
    const hasWarnings = col.technical.warnings.length > 0;
    const warningClass = hasWarnings ? 'has-warning' : '';
    const warningsHtml = hasWarnings
      ? `<div class="warnings">${col.technical.warnings.map(w => `<span class="warning-badge">${w}</span>`).join('')}</div>`
      : '';

    return `
      <tr class="column-row ${warningClass}" data-column-name="${col.current_name}">
        <td>
          <strong>${col.current_name}</strong>
          ${col.original_name && col.original_name !== col.current_name ? `<br><small class="text-muted">Original: ${col.original_name}</small>` : ''}
          ${warningsHtml}
        </td>
        <td>
          <code>${col.technical.data_type}</code><br>
          <small>${col.technical.null_percentage.toFixed(1)}% null</small>
        </td>
        <td>
          <textarea class="column-definition" data-column="${col.current_name}" placeholder="Plain-English definition">${col.business.business_definition || ''}</textarea>
        </td>
        <td>
          <textarea class="column-rules" data-column="${col.current_name}" placeholder="Business rules or constraints">${col.business.business_rules || ''}</textarea>
        </td>
        <td>
          <select class="column-sensitivity" data-column="${col.current_name}">
            <option value="">None</option>
            <option value="PII" ${col.business.sensitivity_tag === 'PII' ? 'selected' : ''}>PII</option>
            <option value="Financial" ${col.business.sensitivity_tag === 'Financial' ? 'selected' : ''}>Financial</option>
            <option value="Public" ${col.business.sensitivity_tag === 'Public' ? 'selected' : ''}>Public</option>
          </select>
        </td>
        <td>
          <textarea class="column-notes" data-column="${col.current_name}" placeholder="Additional notes">${col.business.notes || ''}</textarea>
        </td>
      </tr>
    `;
  }).join('');

  return `
    <div class="dictionary-detail-container">
      <div class="detail-header">
        <button id="btn-back-to-list" class="btn-secondary">
          <i class="ph ph-arrow-left"></i> Back to List
        </button>
        <div class="detail-title">
          <h1>${snapshot.dataset_name}</h1>
          <p class="text-muted">Snapshot ID: ${snapshot.snapshot_id}</p>
        </div>
        <div class="detail-actions">
          <button id="btn-save-metadata" class="btn-primary">
            <i class="ph ph-floppy-disk"></i> Save Changes
          </button>
        </div>
      </div>

      <div class="metadata-overview">
        <div class="overview-card">
          <label>Created</label>
          <div>${date}</div>
        </div>
        <div class="overview-card">
          <label>Dimensions</label>
          <div>${tech.row_count.toLocaleString()} × ${tech.column_count}</div>
        </div>
        <div class="overview-card">
          <label>Format</label>
          <div>${tech.export_format}</div>
        </div>
        <div class="overview-card">
          <label>Quality Score</label>
          <div>${tech.quality_summary.overall_score.toFixed(1)}%</div>
        </div>
        <div class="overview-card">
          <label>Documentation</label>
          <div>${completeness}%</div>
        </div>
      </div>

      ${datasetForm}

      <div class="metadata-section">
        <h3>Column Metadata</h3>
        <div class="columns-table-wrapper">
          <table class="columns-metadata-table">
            <thead>
              <tr>
                <th>Column Name</th>
                <th>Technical</th>
                <th>Business Definition</th>
                <th>Business Rules</th>
                <th>Sensitivity</th>
                <th>Notes</th>
              </tr>
            </thead>
            <tbody>
              ${columnRows}
            </tbody>
          </table>
        </div>
      </div>

      <div class="detail-footer">
        <button id="btn-save-metadata-bottom" class="btn-primary">
          <i class="ph ph-floppy-disk"></i> Save Changes
        </button>
        <button id="btn-export-markdown" class="btn-secondary">
          <i class="ph ph-file-text"></i> Export Markdown
        </button>
      </div>
    </div>
  `;
}
