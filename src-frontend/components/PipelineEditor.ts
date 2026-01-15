/**
 * Pipeline Editor Component
 *
 * Visual editor for creating and modifying data transformation pipelines.
 * Features a two-panel layout with step palette on left and pipeline canvas on right.
 */

import { PipelineSpec, PipelineStep } from '../api-pipeline';
import { StepConfigPanel } from './StepConfigPanel';
import { StepPalette } from './StepPalette';

export interface PipelineEditorState {
    /** Current pipeline being edited */
    spec: PipelineSpec;

    /** Index of currently selected step for configuration */
    selectedStepIndex: number | null;

    /** Whether pipeline has unsaved changes */
    isDirty: boolean;

    /** Validation errors */
    errors: string[];
}

export class PipelineEditor {
    private state: PipelineEditorState;
    private container: HTMLElement;
    private stepPalette: StepPalette | null = null;
    private configPanel: StepConfigPanel | null = null;
    private onSave?: (spec: PipelineSpec) => void;
    private onBack?: () => void;
    private onExecute?: (spec: PipelineSpec) => void;
    private draggedStepIndex: number | null = null;

    constructor(container: HTMLElement, spec?: PipelineSpec) {
        this.container = container;
        this.state = {
            spec: spec || this.createEmptyPipeline(),
            selectedStepIndex: null,
            isDirty: false,
            errors: [],
        };
    }

    /**
     * Create an empty pipeline spec
     */
    private createEmptyPipeline(): PipelineSpec {
        return {
            name: 'New Pipeline',
            version: '0.1',
            steps: [],
            input: {
                format: 'csv'
            },
            output: {
                format: 'parquet',
                path: ''
            }
        };
    }

    /**
     * Initialize the editor
     */
    init(): void {
        this.render();
        this.attachEventListeners();
        this.initializePalette();
        this.initializeConfigPanel();
    }

    /**
     * Initialize step palette
     */
    private initializePalette(): void {
        const paletteContainer = this.container.querySelector('#step-palette-container') as HTMLElement;
        if (paletteContainer) {
            this.stepPalette = new StepPalette(paletteContainer);
            this.stepPalette.setOnStepAdd((step) => {
                this.addStep(step);
            });
            this.stepPalette.init();
        } else {
            console.error('Step palette container not found');
        }
    }

    /**
     * Initialize config panel
     */
    private initializeConfigPanel(): void {
        const configContainer = this.container.querySelector('#step-config-container') as HTMLElement;
        if (configContainer) {
            this.configPanel = new StepConfigPanel(configContainer);
            this.configPanel.setOnUpdate((updatedStep: PipelineStep) => {
                this.handleStepUpdate(updatedStep);
            });
            // Update with current selection
            const selectedStep = this.state.selectedStepIndex !== null
                ? this.state.spec.steps[this.state.selectedStepIndex] || null
                : null;
            this.configPanel.setStep(selectedStep, this.state.selectedStepIndex);
        } else {
            console.error('Config panel container not found');
        }
    }

    /**
     * Set save callback
     */
    setOnSave(callback: (spec: PipelineSpec) => void): void {
        this.onSave = callback;
    }

    /**
     * Set back callback
     */
    setOnBack(callback: () => void): void {
        this.onBack = callback;
    }

    /**
     * Set execute callback
     */
    setOnExecute(callback: (spec: PipelineSpec) => void): void {
        this.onExecute = callback;
    }

    /**
     * Render the editor UI
     */
    render(): void {
        this.container.innerHTML = `
            <div class="pipeline-editor">
                <div class="editor-header">
                    <button id="editor-back-btn" class="btn-secondary">← Back to Library</button>
                    <input
                        type="text"
                        id="pipeline-name-input"
                        class="pipeline-name-input"
                        value="${this.escapeHtml(this.state.spec.name)}"
                        placeholder="Pipeline Name"
                    />
                    <div class="editor-actions">
                        ${this.state.isDirty ? '<span class="unsaved-indicator">●</span>' : ''}
                        <button id="editor-execute-btn" class="btn-secondary" ${this.state.spec.steps.length === 0 ? 'disabled' : ''}>▶ Execute</button>
                        <button id="editor-save-btn" class="btn-primary">Save</button>
                    </div>
                </div>

                <div class="editor-body">
                    <div id="step-palette-container" class="editor-sidebar">
                        <!-- Step Palette will be rendered here -->
                    </div>

                    <div class="editor-main">
                        <div class="pipeline-canvas">
                            <h3>Pipeline Steps</h3>
                            ${this.renderPipelineSteps()}
                        </div>
                    </div>

                    <div id="step-config-container" class="editor-config-panel">
                        <!-- Step Configuration Panel will be rendered here -->
                    </div>
                </div>

                ${this.renderErrors()}
            </div>
        `;
    }

    /**
     * Render pipeline steps
     */
    private renderPipelineSteps(): string {
        if (this.state.spec.steps.length === 0) {
            return `
                <div class="empty-pipeline">
                    <p>No steps yet</p>
                    <p class="hint">Add steps from the palette to build your pipeline</p>
                </div>
            `;
        }

        return `
            <div class="steps-list">
                ${this.state.spec.steps.map((step, index) => this.renderStepCard(step, index)).join('')}
            </div>
        `;
    }

    /**
     * Render a single step card
     */
    private renderStepCard(step: PipelineStep, index: number): string {
        const isSelected = this.state.selectedStepIndex === index;
        const stepType = this.getStepType(step);
        const stepSummary = this.getStepSummary(step);

        return `
            <div class="step-card ${isSelected ? 'selected' : ''}"
                 data-index="${index}"
                 draggable="true">
                <div class="step-card-header">
                    <span class="step-drag-handle" title="Drag to reorder">⋮⋮</span>
                    <span class="step-number">${index + 1}</span>
                    <span class="step-type">${this.escapeHtml(stepType)}</span>
                    <div class="step-card-actions">
                        ${index > 0 ? `<button class="btn-icon step-move-up" data-index="${index}" title="Move Up">↑</button>` : ''}
                        ${index < this.state.spec.steps.length - 1 ? `<button class="btn-icon step-move-down" data-index="${index}" title="Move Down">↓</button>` : ''}
                        <button class="btn-icon step-delete" data-index="${index}" title="Delete">🗑️</button>
                    </div>
                </div>
                <div class="step-card-body">
                    <p class="step-summary">${this.escapeHtml(stepSummary)}</p>
                </div>
            </div>
        `;
    }

    /**
     * Get step type from step object
     */
    private getStepType(step: PipelineStep): string {
        // Step is a tagged enum with "op" field
        const stepObj = step as Record<string, unknown>;
        if (stepObj.op) {
            return String(stepObj.op).replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase());
        }
        // Fallback: get first key
        const keys = Object.keys(step);
        return keys[0] || 'Unknown';
    }

    /**
     * Get human-readable summary of step
     */
    private getStepSummary(step: PipelineStep): string {
        const stepObj = step as Record<string, unknown>;
        const op = stepObj.op as string;

        switch (op) {
            case 'drop_columns':
                const dropCols = (stepObj.columns as string[]) || [];
                return `Drop ${dropCols.length} column(s)`;
            case 'rename_columns':
                const mapping = (stepObj.mapping as Record<string, string>) || {};
                return `Rename ${Object.keys(mapping).length} column(s)`;
            case 'trim_whitespace':
                const trimCols = (stepObj.columns as string[]) || [];
                return `Trim whitespace in ${trimCols.length} column(s)`;
            case 'cast_types':
                const castCols = (stepObj.columns as Record<string, string>) || {};
                return `Cast ${Object.keys(castCols).length} column(s)`;
            case 'impute':
                const strategy = stepObj.strategy || 'unknown';
                return `Impute using ${strategy}`;
            case 'normalize_columns':
                const method = stepObj.method || 'unknown';
                return `Normalize using ${method}`;
            default:
                return 'Transform data';
        }
    }

    /**
     * Render validation errors
     */
    private renderErrors(): string {
        if (this.state.errors.length === 0) {
            return '';
        }

        return `
            <div class="editor-errors">
                <h4>Validation Errors:</h4>
                <ul>
                    ${this.state.errors.map(err => `<li>${this.escapeHtml(err)}</li>`).join('')}
                </ul>
            </div>
        `;
    }

    /**
     * Escape HTML to prevent XSS
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
        // Back button
        const backBtn = this.container.querySelector('#editor-back-btn');
        backBtn?.addEventListener('click', () => {
            if (this.state.isDirty) {
                const confirm = window.confirm('You have unsaved changes. Are you sure you want to go back?');
                if (!confirm) return;
            }
            if (this.onBack) this.onBack();
        });

        // Save button
        const saveBtn = this.container.querySelector('#editor-save-btn');
        saveBtn?.addEventListener('click', () => this.handleSave());

        // Execute button
        const executeBtn = this.container.querySelector('#editor-execute-btn');
        executeBtn?.addEventListener('click', () => {
            if (this.onExecute) {
                this.onExecute(this.state.spec);
            }
        });

        // Pipeline name input
        const nameInput = this.container.querySelector<HTMLInputElement>('#pipeline-name-input');
        nameInput?.addEventListener('input', (e) => {
            this.state.spec.name = (e.target as HTMLInputElement).value;
            this.state.isDirty = true;
            this.render();
            this.attachEventListeners();
            this.initializePalette();
            this.initializeConfigPanel();
        });

        // Step card actions
        this.container.querySelectorAll('.step-move-up').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const index = parseInt((e.currentTarget as HTMLElement).getAttribute('data-index') || '0');
                this.moveStepUp(index);
            });
        });

        this.container.querySelectorAll('.step-move-down').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const index = parseInt((e.currentTarget as HTMLElement).getAttribute('data-index') || '0');
                this.moveStepDown(index);
            });
        });

        this.container.querySelectorAll('.step-delete').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const index = parseInt((e.currentTarget as HTMLElement).getAttribute('data-index') || '0');
                this.deleteStep(index);
            });
        });

        // Step card selection
        this.container.querySelectorAll('.step-card').forEach(card => {
            card.addEventListener('click', (e) => {
                // Don't select if clicking action buttons
                if ((e.target as HTMLElement).closest('.step-card-actions')) return;

                const index = parseInt((card as HTMLElement).getAttribute('data-index') || '0');
                this.selectStep(index);
            });
        });

        // Drag-and-drop event listeners
        this.container.querySelectorAll('.step-card').forEach(card => {
            const cardElement = card as HTMLElement;

            cardElement.addEventListener('dragstart', (e: DragEvent) => {
                this.draggedStepIndex = parseInt(cardElement.getAttribute('data-index') || '0');
                cardElement.classList.add('dragging');
                if (e.dataTransfer) {
                    e.dataTransfer.effectAllowed = 'move';
                    e.dataTransfer.setData('text/html', cardElement.innerHTML);
                }
            });

            cardElement.addEventListener('dragend', () => {
                cardElement.classList.remove('dragging');
                this.draggedStepIndex = null;
                // Remove all drag-over classes
                this.container.querySelectorAll('.step-card').forEach(c => {
                    c.classList.remove('drag-over');
                });
            });

            cardElement.addEventListener('dragover', (e: DragEvent) => {
                e.preventDefault();
                if (e.dataTransfer) {
                    e.dataTransfer.dropEffect = 'move';
                }
                cardElement.classList.add('drag-over');
            });

            cardElement.addEventListener('dragleave', () => {
                cardElement.classList.remove('drag-over');
            });

            cardElement.addEventListener('drop', (e: DragEvent) => {
                e.preventDefault();
                cardElement.classList.remove('drag-over');

                const dropIndex = parseInt(cardElement.getAttribute('data-index') || '0');
                if (this.draggedStepIndex !== null && this.draggedStepIndex !== dropIndex) {
                    this.moveStepToIndex(this.draggedStepIndex, dropIndex);
                }
            });
        });
    }

    /**
     * Add a step to the pipeline
     */
    addStep(step: PipelineStep): void {
        this.state.spec.steps.push(step);
        this.state.isDirty = true;
        this.render();
        this.attachEventListeners();
        // Reinitialize palette and config panel after render
        this.initializePalette();
        this.initializeConfigPanel();
    }

    /**
     * Move step up in pipeline
     */
    private moveStepUp(index: number): void {
        if (index > 0 && this.state.spec.steps[index] && this.state.spec.steps[index - 1]) {
            const steps = this.state.spec.steps;
            const temp = steps[index - 1]!;
            steps[index - 1] = steps[index]!;
            steps[index] = temp;
            this.state.isDirty = true;
            this.render();
            this.attachEventListeners();
            this.initializePalette();
            this.initializeConfigPanel();
        }
    }

    /**
     * Move step down in pipeline
     */
    private moveStepDown(index: number): void {
        if (index < this.state.spec.steps.length - 1 && this.state.spec.steps[index] && this.state.spec.steps[index + 1]) {
            const steps = this.state.spec.steps;
            const temp = steps[index]!;
            steps[index] = steps[index + 1]!;
            steps[index + 1] = temp;
            this.state.isDirty = true;
            this.render();
            this.attachEventListeners();
            this.initializePalette();
            this.initializeConfigPanel();
        }
    }

    /**
     * Delete a step
     */
    private deleteStep(index: number): void {
        const confirm = window.confirm('Delete this step?');
        if (confirm) {
            this.state.spec.steps.splice(index, 1);
            this.state.isDirty = true;
            if (this.state.selectedStepIndex === index) {
                this.state.selectedStepIndex = null;
            }
            this.render();
            this.attachEventListeners();
            this.initializePalette();
            this.initializeConfigPanel();
        }
    }

    /**
     * Move step from one index to another (used for drag-and-drop)
     */
    private moveStepToIndex(fromIndex: number, toIndex: number): void {
        if (fromIndex === toIndex) return;

        const steps = this.state.spec.steps;
        const movedStep = steps[fromIndex];
        if (!movedStep) return; // Safety check
        steps.splice(fromIndex, 1);
        steps.splice(toIndex, 0, movedStep);

        // Update selected index if necessary
        if (this.state.selectedStepIndex === fromIndex) {
            this.state.selectedStepIndex = toIndex;
        } else if (fromIndex < this.state.selectedStepIndex! && toIndex >= this.state.selectedStepIndex!) {
            this.state.selectedStepIndex!--;
        } else if (fromIndex > this.state.selectedStepIndex! && toIndex <= this.state.selectedStepIndex!) {
            this.state.selectedStepIndex!++;
        }

        this.state.isDirty = true;
        this.render();
        this.attachEventListeners();
        this.initializePalette();
        this.initializeConfigPanel();
    }

    /**
     * Handle step update from config panel
     */
    private handleStepUpdate(updatedStep: PipelineStep): void {
        if (this.state.selectedStepIndex === null) return;

        this.state.spec.steps[this.state.selectedStepIndex] = updatedStep;
        this.state.isDirty = true;

        // Re-render to update step card summary
        this.render();
        this.attachEventListeners();
        this.initializePalette();
        this.initializeConfigPanel();
    }

    /**
     * Select a step for configuration
     */
    private selectStep(index: number): void {
        this.state.selectedStepIndex = index;

        // Update config panel
        if (this.configPanel) {
            const selectedStep = this.state.spec.steps[index] || null;
            this.configPanel.setStep(selectedStep, index);
        }

        this.render();
        this.attachEventListeners();
        this.initializePalette();
        this.initializeConfigPanel();
    }

    /**
     * Handle save action
     */
    private handleSave(): void {
        // Basic validation
        this.state.errors = [];

        if (!this.state.spec.name || this.state.spec.name.trim() === '') {
            this.state.errors.push('Pipeline name is required');
        }

        if (this.state.spec.steps.length === 0) {
            this.state.errors.push('Pipeline must have at least one step');
        }

        if (this.state.errors.length > 0) {
            this.render();
            this.attachEventListeners();
            this.initializePalette();
            this.initializeConfigPanel();
            return;
        }

        // Call save callback
        if (this.onSave) {
            this.onSave(this.state.spec);
            this.state.isDirty = false;
            this.render();
            this.attachEventListeners();
            this.initializePalette();
            this.initializeConfigPanel();
        }
    }

    /**
     * Get current pipeline spec
     */
    getSpec(): PipelineSpec {
        return this.state.spec;
    }
}
