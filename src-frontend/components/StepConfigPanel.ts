/**
 * Step Configuration Panel Component
 *
 * Provides dynamic forms for configuring pipeline step parameters.
 * Renders different form fields based on the step type.
 */

import { PipelineStep } from '../api-pipeline';

export interface StepConfigPanelState {
  step: PipelineStep | null;
  stepIndex: number | null;
  errors: Map<string, string>;
}

export class StepConfigPanel {
  private state: StepConfigPanelState;
  private container: HTMLElement;
  private onUpdate?: (step: PipelineStep) => void;

  constructor(container: HTMLElement) {
    this.container = container;
    this.state = {
      step: null,
      stepIndex: null,
      errors: new Map(),
    };
  }

  /**
   * Set the step to configure
   */
  setStep(step: PipelineStep | null, index: number | null): void {
    this.state.step = step;
    this.state.stepIndex = index;
    this.state.errors.clear();
    this.render();
    this.attachEventListeners();
  }

  /**
   * Set update callback
   */
  setOnUpdate(callback: (step: PipelineStep) => void): void {
    this.onUpdate = callback;
  }

  /**
   * Render the configuration panel
   */
  render(): void {
    if (!this.state.step) {
      this.container.innerHTML = `
                <div class="config-panel-empty">
                    <p>Select a step to configure its parameters</p>
                </div>
            `;
      return;
    }

    const stepObj = this.state.step as Record<string, unknown>;
    const stepType = stepObj.op as string;

    this.container.innerHTML = `
            <div class="config-panel">
                <div class="config-header">
                    <h3>Configure Step ${this.state.stepIndex !== null ? this.state.stepIndex + 1 : ''}</h3>
                    <span class="config-step-type">${this.getStepTypeName(stepType)}</span>
                </div>
                <div class="config-body">
                    ${this.renderConfigForm(stepType, stepObj)}
                </div>
                ${this.renderErrors()}
            </div>
        `;
  }

  /**
   * Get human-readable step type name
   */
  private getStepTypeName(stepType: string): string {
    return stepType.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase());
  }

  /**
   * Render configuration form based on step type
   */
  private renderConfigForm(stepType: string, stepObj: Record<string, unknown>): string {
    switch (stepType) {
      case 'drop_columns':
        return this.renderDropColumnsForm(stepObj);
      case 'rename_columns':
        return this.renderRenameColumnsForm(stepObj);
      case 'trim_whitespace':
        return this.renderTrimWhitespaceForm(stepObj);
      case 'cast_types':
        return this.renderCastTypesForm(stepObj);
      case 'parse_dates':
        return this.renderParseDatesForm(stepObj);
      case 'impute':
        return this.renderImputeForm(stepObj);
      case 'normalise_columns':
        return this.renderNormaliseForm(stepObj);
      case 'clip_outliers':
        return this.renderClipOutliersForm(stepObj);
      case 'one_hot_encode':
        return this.renderOneHotEncodeForm(stepObj);
      case 'extract_numbers':
        return this.renderExtractNumbersForm(stepObj);
      case 'regex_replace':
        return this.renderRegexReplaceForm(stepObj);
      default:
        return '<p>Configuration for this step type is not yet implemented.</p>';
    }
  }

  /**
   * Render form for drop_columns step
   */
  private renderDropColumnsForm(stepObj: Record<string, unknown>): string {
    const columns = (stepObj.columns as string[]) || [];
    const columnsText = columns.join(', ');

    return `
            <div class="form-group">
                <label for="drop-columns-input">Columns to Drop</label>
                <textarea
                    id="drop-columns-input"
                    class="form-control"
                    rows="3"
                    placeholder="Enter column names separated by commas (e.g., col1, col2, col3)"
                >${this.escapeHtml(columnsText)}</textarea>
                <small class="form-hint">Comma-separated list of column names to remove</small>
            </div>
        `;
  }

  /**
   * Render form for rename_columns step
   */
  private renderRenameColumnsForm(stepObj: Record<string, unknown>): string {
    const mapping = (stepObj.mapping as Record<string, string>) || {};
    const mappingText = Object.entries(mapping)
      .map(([old, new_]) => `${old} -> ${new_}`)
      .join('\n');

    return `
            <div class="form-group">
                <label for="rename-mapping-input">Column Renaming</label>
                <textarea
                    id="rename-mapping-input"
                    class="form-control"
                    rows="5"
                    placeholder="Enter one mapping per line: old_name -> new_name"
                >${this.escapeHtml(mappingText)}</textarea>
                <small class="form-hint">One mapping per line: old_name -> new_name</small>
            </div>
        `;
  }

  /**
   * Render form for trim_whitespace step
   */
  private renderTrimWhitespaceForm(stepObj: Record<string, unknown>): string {
    const columns = (stepObj.columns as string[]) || [];
    const columnsText = columns.join(', ');

    return `
            <div class="form-group">
                <label for="trim-columns-input">Columns to Trim</label>
                <textarea
                    id="trim-columns-input"
                    class="form-control"
                    rows="3"
                    placeholder="Enter column names (comma-separated) or leave empty for all text columns"
                >${this.escapeHtml(columnsText)}</textarea>
                <small class="form-hint">Leave empty to trim all text columns</small>
            </div>
        `;
  }

  /**
   * Render form for cast_types step
   */
  private renderCastTypesForm(stepObj: Record<string, unknown>): string {
    const columns = (stepObj.columns as Record<string, string>) || {};
    const mappingText = Object.entries(columns)
      .map(([col, type]) => `${col} -> ${type}`)
      .join('\n');

    return `
            <div class="form-group">
                <label for="cast-mapping-input">Type Casting</label>
                <textarea
                    id="cast-mapping-input"
                    class="form-control"
                    rows="5"
                    placeholder="Enter one mapping per line: column_name -> type"
                >${this.escapeHtml(mappingText)}</textarea>
                <small class="form-hint">Types: i64, f64, String, bool</small>
            </div>
        `;
  }

  /**
   * Render form for parse_dates step
   */
  private renderParseDatesForm(stepObj: Record<string, unknown>): string {
    const columns = (stepObj.columns as Record<string, string>) || {};
    const mappingText = Object.entries(columns)
      .map(([col, format]) => `${col} -> ${format}`)
      .join('\n');

    return `
            <div class="form-group">
                <label for="parse-dates-input">Date Parsing</label>
                <textarea
                    id="parse-dates-input"
                    class="form-control"
                    rows="5"
                    placeholder="Enter one mapping per line: column_name -> format"
                >${this.escapeHtml(mappingText)}</textarea>
                <small class="form-hint">Format: %Y-%m-%d, %Y/%m/%d %H:%M:%S, etc.</small>
            </div>
        `;
  }

  /**
   * Render form for impute step
   */
  private renderImputeForm(stepObj: Record<string, unknown>): string {
    const strategy = (stepObj.strategy as string) || 'mean';
    const columns = (stepObj.columns as string[]) || [];
    const columnsText = columns.join(', ');

    return `
            <div class="form-group">
                <label for="impute-strategy-select">Imputation Strategy</label>
                <select id="impute-strategy-select" class="form-control">
                    <option value="mean" ${strategy === 'mean' ? 'selected' : ''}>Mean</option>
                    <option value="median" ${strategy === 'median' ? 'selected' : ''}>Median</option>
                    <option value="mode" ${strategy === 'mode' ? 'selected' : ''}>Mode</option>
                    <option value="zero" ${strategy === 'zero' ? 'selected' : ''}>Zero</option>
                </select>
            </div>
            <div class="form-group">
                <label for="impute-columns-input">Columns</label>
                <textarea
                    id="impute-columns-input"
                    class="form-control"
                    rows="3"
                    placeholder="Column names (comma-separated) or leave empty for all numeric columns"
                >${this.escapeHtml(columnsText)}</textarea>
                <small class="form-hint">Leave empty to impute all numeric columns</small>
            </div>
        `;
  }

  /**
   * Render form for normalize_columns step
   */
  private renderNormaliseForm(stepObj: Record<string, unknown>): string {
    const method = (stepObj.method as string) || 'z_score';
    const columns = (stepObj.columns as string[]) || [];
    const columnsText = columns.join(', ');

    return `
            <div class="form-group">
                <label for="normalize-method-select">Normalization Method</label>
                <select id="normalize-method-select" class="form-control">
                    <option value="z_score" ${method === 'z_score' ? 'selected' : ''}>Z-Score (standardize)</option>
                    <option value="min_max" ${method === 'min_max' ? 'selected' : ''}>Min-Max (0-1 range)</option>
                </select>
            </div>
            <div class="form-group">
                <label for="normalize-columns-input">Columns</label>
                <textarea
                    id="normalize-columns-input"
                    class="form-control"
                    rows="3"
                    placeholder="Column names (comma-separated)"
                >${this.escapeHtml(columnsText)}</textarea>
            </div>
        `;
  }

  /**
   * Render form for clip_outliers step
   */
  private renderClipOutliersForm(stepObj: Record<string, unknown>): string {
    const columns = (stepObj.columns as string[]) || [];
    const lowerQuantile = (stepObj.lower_quantile as number) || 0.05;
    const upperQuantile = (stepObj.upper_quantile as number) || 0.95;
    const columnsText = columns.join(', ');

    return `
            <div class="form-group">
                <label for="clip-columns-input">Columns</label>
                <textarea
                    id="clip-columns-input"
                    class="form-control"
                    rows="3"
                    placeholder="Column names (comma-separated)"
                >${this.escapeHtml(columnsText)}</textarea>
            </div>
            <div class="form-row">
                <div class="form-group">
                    <label for="clip-lower-input">Lower Quantile</label>
                    <input
                        type="number"
                        id="clip-lower-input"
                        class="form-control"
                        min="0"
                        max="1"
                        step="0.01"
                        value="${lowerQuantile}"
                    />
                    <small class="form-hint">e.g., 0.05 = 5th percentile</small>
                </div>
                <div class="form-group">
                    <label for="clip-upper-input">Upper Quantile</label>
                    <input
                        type="number"
                        id="clip-upper-input"
                        class="form-control"
                        min="0"
                        max="1"
                        step="0.01"
                        value="${upperQuantile}"
                    />
                    <small class="form-hint">e.g., 0.95 = 95th percentile</small>
                </div>
            </div>
        `;
  }

  /**
   * Render form for one_hot_encode step
   */
  private renderOneHotEncodeForm(stepObj: Record<string, unknown>): string {
    const columns = (stepObj.columns as string[]) || [];
    const dropOriginal = (stepObj.drop_original as boolean) ?? true;
    const columnsText = columns.join(', ');

    return `
            <div class="form-group">
                <label for="onehot-columns-input">Columns to Encode</label>
                <textarea
                    id="onehot-columns-input"
                    class="form-control"
                    rows="3"
                    placeholder="Column names (comma-separated)"
                >${this.escapeHtml(columnsText)}</textarea>
            </div>
            <div class="form-group">
                <label class="form-checkbox">
                    <input
                        type="checkbox"
                        id="onehot-drop-original"
                        ${dropOriginal ? 'checked' : ''}
                    />
                    Drop original columns after encoding
                </label>
            </div>
        `;
  }

  /**
   * Render form for extract_numbers step
   */
  private renderExtractNumbersForm(stepObj: Record<string, unknown>): string {
    const columns = (stepObj.columns as string[]) || [];
    const columnsText = columns.join(', ');

    return `
            <div class="form-group">
                <label for="extract-columns-input">Columns</label>
                <textarea
                    id="extract-columns-input"
                    class="form-control"
                    rows="3"
                    placeholder="Column names (comma-separated)"
                >${this.escapeHtml(columnsText)}</textarea>
                <small class="form-hint">Extracts first numeric value found in text</small>
            </div>
        `;
  }

  /**
   * Render form for regex_replace step
   */
  private renderRegexReplaceForm(stepObj: Record<string, unknown>): string {
    const columns = (stepObj.columns as string[]) || [];
    const pattern = (stepObj.pattern as string) || '';
    const replacement = (stepObj.replacement as string) || '';
    const columnsText = columns.join(', ');

    return `
            <div class="form-group">
                <label for="regex-columns-input">Columns</label>
                <textarea
                    id="regex-columns-input"
                    class="form-control"
                    rows="2"
                    placeholder="Column names (comma-separated)"
                >${this.escapeHtml(columnsText)}</textarea>
            </div>
            <div class="form-group">
                <label for="regex-pattern-input">Pattern (Regex)</label>
                <input
                    type="text"
                    id="regex-pattern-input"
                    class="form-control"
                    placeholder="e.g., [0-9]+"
                    value="${this.escapeHtml(pattern)}"
                />
            </div>
            <div class="form-group">
                <label for="regex-replacement-input">Replacement</label>
                <input
                    type="text"
                    id="regex-replacement-input"
                    class="form-control"
                    placeholder="e.g., X"
                    value="${this.escapeHtml(replacement)}"
                />
            </div>
        `;
  }

  /**
   * Render validation errors
   */
  private renderErrors(): string {
    if (this.state.errors.size === 0) {
      return '';
    }

    const errors = Array.from(this.state.errors.values());
    return `
            <div class="config-errors">
                ${errors.map(err => `<div class="error-message">${this.escapeHtml(err)}</div>`).join('')}
            </div>
        `;
  }

  /**
   * Escape HTML
   */
  private escapeHtml(text: string): string {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  }

  /**
   * Attach event listeners
   */
  private attachEventListeners(): void {
    if (!this.state.step) return;

    const stepObj = this.state.step as Record<string, unknown>;
    const stepType = stepObj.op as string;

    // Add change listeners based on step type
    switch (stepType) {
      case 'drop_columns':
        this.attachDropColumnsListeners();
        break;
      case 'rename_columns':
        this.attachRenameColumnsListeners();
        break;
      case 'trim_whitespace':
        this.attachTrimWhitespaceListeners();
        break;
      case 'cast_types':
        this.attachCastTypesListeners();
        break;
      case 'parse_dates':
        this.attachParseDatesListeners();
        break;
      case 'impute':
        this.attachImputeListeners();
        break;
      case 'normalise_columns':
        this.attachNormaliseListeners();
        break;
      case 'clip_outliers':
        this.attachClipOutliersListeners();
        break;
      case 'one_hot_encode':
        this.attachOneHotEncodeListeners();
        break;
      case 'extract_numbers':
        this.attachExtractNumbersListeners();
        break;
      case 'regex_replace':
        this.attachRegexReplaceListeners();
        break;
    }
  }

  // Individual attachment methods for each step type
  private attachDropColumnsListeners(): void {
    const input = this.container.querySelector<HTMLTextAreaElement>('#drop-columns-input');
    input?.addEventListener('blur', () => {
      const columnsText = input.value.trim();
      const columns = columnsText
        ? columnsText
            .split(',')
            .map(c => c.trim())
            .filter(c => c)
        : [];
      this.updateStep({ columns });
    });
  }

  private attachRenameColumnsListeners(): void {
    const input = this.container.querySelector<HTMLTextAreaElement>('#rename-mapping-input');
    input?.addEventListener('blur', () => {
      const mappingText = input.value.trim();
      const mapping: Record<string, string> = {};
      mappingText.split('\n').forEach(line => {
        const parts = line.split('->').map(p => p.trim());
        if (parts.length === 2 && parts[0] && parts[1]) {
          mapping[parts[0]] = parts[1];
        }
      });
      this.updateStep({ mapping });
    });
  }

  private attachTrimWhitespaceListeners(): void {
    const input = this.container.querySelector<HTMLTextAreaElement>('#trim-columns-input');
    input?.addEventListener('blur', () => {
      const columnsText = input.value.trim();
      const columns = columnsText
        ? columnsText
            .split(',')
            .map(c => c.trim())
            .filter(c => c)
        : [];
      this.updateStep({ columns });
    });
  }

  private attachCastTypesListeners(): void {
    const input = this.container.querySelector<HTMLTextAreaElement>('#cast-mapping-input');
    input?.addEventListener('blur', () => {
      const mappingText = input.value.trim();
      const columns: Record<string, string> = {};
      mappingText.split('\n').forEach(line => {
        const parts = line.split('->').map(p => p.trim());
        if (parts.length === 2 && parts[0] && parts[1]) {
          columns[parts[0]] = parts[1];
        }
      });
      this.updateStep({ columns });
    });
  }

  private attachParseDatesListeners(): void {
    const input = this.container.querySelector<HTMLTextAreaElement>('#parse-dates-input');
    input?.addEventListener('blur', () => {
      const mappingText = input.value.trim();
      const columns: Record<string, string> = {};
      mappingText.split('\n').forEach(line => {
        const parts = line.split('->').map(p => p.trim());
        if (parts.length === 2 && parts[0] && parts[1]) {
          columns[parts[0]] = parts[1];
        }
      });
      this.updateStep({ columns });
    });
  }

  private attachImputeListeners(): void {
    const strategySelect =
      this.container.querySelector<HTMLSelectElement>('#impute-strategy-select');
    const columnsInput = this.container.querySelector<HTMLTextAreaElement>('#impute-columns-input');

    strategySelect?.addEventListener('change', () => {
      this.updateStep({ strategy: strategySelect.value });
    });

    columnsInput?.addEventListener('blur', () => {
      const columnsText = columnsInput.value.trim();
      const columns = columnsText
        ? columnsText
            .split(',')
            .map(c => c.trim())
            .filter(c => c)
        : [];
      this.updateStep({ columns });
    });
  }

  private attachNormaliseListeners(): void {
    const methodSelect = this.container.querySelector<HTMLSelectElement>(
      '#normalize-method-select'
    );
    const columnsInput = this.container.querySelector<HTMLTextAreaElement>(
      '#normalize-columns-input'
    );

    methodSelect?.addEventListener('change', () => {
      this.updateStep({ method: methodSelect.value });
    });

    columnsInput?.addEventListener('blur', () => {
      const columnsText = columnsInput.value.trim();
      const columns = columnsText
        ? columnsText
            .split(',')
            .map(c => c.trim())
            .filter(c => c)
        : [];
      this.updateStep({ columns });
    });
  }

  private attachClipOutliersListeners(): void {
    const columnsInput = this.container.querySelector<HTMLTextAreaElement>('#clip-columns-input');
    const lowerInput = this.container.querySelector<HTMLInputElement>('#clip-lower-input');
    const upperInput = this.container.querySelector<HTMLInputElement>('#clip-upper-input');

    columnsInput?.addEventListener('blur', () => {
      const columnsText = columnsInput.value.trim();
      const columns = columnsText
        ? columnsText
            .split(',')
            .map(c => c.trim())
            .filter(c => c)
        : [];
      this.updateStep({ columns });
    });

    lowerInput?.addEventListener('change', () => {
      this.updateStep({ lower_quantile: parseFloat(lowerInput.value) });
    });

    upperInput?.addEventListener('change', () => {
      this.updateStep({ upper_quantile: parseFloat(upperInput.value) });
    });
  }

  private attachOneHotEncodeListeners(): void {
    const columnsInput = this.container.querySelector<HTMLTextAreaElement>('#onehot-columns-input');
    const dropCheckbox = this.container.querySelector<HTMLInputElement>('#onehot-drop-original');

    columnsInput?.addEventListener('blur', () => {
      const columnsText = columnsInput.value.trim();
      const columns = columnsText
        ? columnsText
            .split(',')
            .map(c => c.trim())
            .filter(c => c)
        : [];
      this.updateStep({ columns });
    });

    dropCheckbox?.addEventListener('change', () => {
      this.updateStep({ drop_original: dropCheckbox.checked });
    });
  }

  private attachExtractNumbersListeners(): void {
    const input = this.container.querySelector<HTMLTextAreaElement>('#extract-columns-input');
    input?.addEventListener('blur', () => {
      const columnsText = input.value.trim();
      const columns = columnsText
        ? columnsText
            .split(',')
            .map(c => c.trim())
            .filter(c => c)
        : [];
      this.updateStep({ columns });
    });
  }

  private attachRegexReplaceListeners(): void {
    const columnsInput = this.container.querySelector<HTMLTextAreaElement>('#regex-columns-input');
    const patternInput = this.container.querySelector<HTMLInputElement>('#regex-pattern-input');
    const replacementInput = this.container.querySelector<HTMLInputElement>(
      '#regex-replacement-input'
    );

    columnsInput?.addEventListener('blur', () => {
      const columnsText = columnsInput.value.trim();
      const columns = columnsText
        ? columnsText
            .split(',')
            .map(c => c.trim())
            .filter(c => c)
        : [];
      this.updateStep({ columns });
    });

    patternInput?.addEventListener('blur', () => {
      this.updateStep({ pattern: patternInput.value });
    });

    replacementInput?.addEventListener('blur', () => {
      this.updateStep({ replacement: replacementInput.value });
    });
  }

  /**
   * Update step with new values
   */
  private updateStep(updates: Record<string, unknown>): void {
    if (!this.state.step) return;

    // Merge updates into current step
    const updatedStep = { ...this.state.step, ...updates } as PipelineStep;
    this.state.step = updatedStep;

    // Validate
    this.validate();

    // Notify parent
    if (this.onUpdate && this.state.errors.size === 0) {
      this.onUpdate(updatedStep);
    }
  }

  /**
   * Validate current step configuration
   */
  private validate(): void {
    this.state.errors.clear();

    if (!this.state.step) return;

    const stepObj = this.state.step as Record<string, unknown>;
    const stepType = stepObj.op as string;

    // Basic validation based on step type
    switch (stepType) {
      case 'drop_columns':
      case 'trim_whitespace':
      case 'extract_numbers':
        if (!stepObj.columns || (stepObj.columns as unknown[]).length === 0) {
          this.state.errors.set('columns', 'At least one column is required');
        }
        break;
      case 'rename_columns':
      case 'cast_types':
      case 'parse_dates':
        if (!stepObj.columns && !stepObj.mapping) {
          this.state.errors.set('mapping', 'At least one mapping is required');
        }
        break;
      case 'regex_replace':
        if (!stepObj.pattern || (stepObj.pattern as string).trim() === '') {
          this.state.errors.set('pattern', 'Pattern is required');
        }
        break;
    }

    // Re-render if there are errors
    if (this.state.errors.size > 0) {
      this.render();
      this.attachEventListeners();
    }
  }
}
