import { AnalysisResponse, LifecycleStage } from '../../types';
import { escapeHtml, fmtBytes, fmtDuration } from '../../utils';

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
          <span>
            <strong>Note:</strong> All cleaning operations in this stage are reversible.
            Advanced operations (imputation, normalisation, encoding) are available in the <strong>Advanced</strong> stage.
            <a href="#" class="cleaning-info-link" data-view="reference">View full documentation →</a>
          </span>
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
  const rowCount = response.row_count;
  const totalRowCount = response.total_row_count;
  const isSampled = totalRowCount > rowCount;
  const rowDisplay = isSampled
    ? `${totalRowCount.toLocaleString()} rows <small>(Analysed ${rowCount.toLocaleString()} rows)</small>`
    : `${rowCount.toLocaleString()} rows`;

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
    <div class="analyser-header" data-testid="analyser-header">
      <div class="header-main">
        <h2 data-testid="analyser-file-name">${escapeHtml(response.file_name)} <small data-testid="analyser-file-size">(${fmtBytes(response.file_size)})</small></h2>
        <div class="meta-info" data-testid="analyser-metadata">
          <span data-testid="analyser-row-count"><i class="ph ph-rows"></i> ${rowDisplay}</span>
          <span data-testid="analyser-column-count"><i class="ph ph-columns"></i> ${response.column_count} columns</span>
          <span data-testid="analyser-analysis-duration"><i class="ph ph-timer"></i> Analysed in ${fmtDuration(response.analysis_duration)}</span>
        </div>
      </div>
      <div class="header-actions">
        <button id="btn-open-file" class="btn-secondary btn-small" data-testid="analyser-open-file-button">
          <i class="ph ph-file-plus"></i> Select File
        </button>
        ${
          currentStage === 'Advanced'
            ? `
          <div class="action-divider"></div>
          <label class="toggle-control" title="Enable Advanced stage for ML preprocessing features (imputation, normalisation, encoding)">
            <input type="checkbox" id="toggle-advanced-mode" ${advancedProcessingEnabled ? 'checked' : ''}>
            <span class="toggle-slider"></span>
            <span class="toggle-label">Advanced Stage</span>
          </label>
          <div class="action-divider"></div>`
            : ''
        }
        ${
          !isReadOnly
            ? `
          <div class="action-divider"></div>
          <button id="btn-toggle-names" class="btn-ghost btn-small ${useOriginalColumnNames ? 'active' : ''}">
            <i class="ph ${useOriginalColumnNames ? 'ph-tag-simple' : 'ph-tag'}"></i>
            ${useOriginalColumnNames ? 'Using Original Names' : 'Using Standardised Names'}
          </button>
          <button id="btn-clean-all" class="btn-ghost btn-small ${cleanAllActive ? 'active' : ''}">
            <i class="ph ${cleanAllActive ? 'ph-check-square' : 'ph-square'}"></i>
            Clean All
          </button>
        `
            : ''
        }
        <div class="action-divider"></div>
        <button id="btn-export" class="btn-primary btn-small" data-testid="analyser-export-button">
          <i class="ph ph-export"></i> Export
        </button>
      </div>
    </div>
  `;
}
