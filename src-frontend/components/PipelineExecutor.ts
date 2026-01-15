/**
 * Pipeline Executor Component
 *
 * Modal dialog for executing pipelines with progress tracking and result display.
 */

import { PipelineSpec, executePipeline, ExecutionResult } from '../api-pipeline';
import { open } from '@tauri-apps/plugin-dialog';

export type ExecutionState = 'idle' | 'selecting' | 'running' | 'success' | 'error';

export class PipelineExecutor {
    private container: HTMLElement;
    private spec: PipelineSpec;
    private state: ExecutionState = 'idle';
    private inputPath: string | null = null;
    private outputPath: string | null = null;
    private result: ExecutionResult | null = null;
    private error: string | null = null;
    private onClose?: () => void;

    constructor(container: HTMLElement, spec: PipelineSpec) {
        this.container = container;
        this.spec = spec;
    }

    /**
     * Set close callback
     */
    setOnClose(callback: () => void): void {
        this.onClose = callback;
    }

    /**
     * Show executor modal
     */
    async show(): Promise<void> {
        this.state = 'idle';
        this.render();
        this.attachEventListeners();

        // Auto-prompt for input file
        await this.selectInputFile();
    }

    /**
     * Close executor
     */
    private close(): void {
        if (this.onClose) {
            this.onClose();
        }
    }

    /**
     * Render executor UI
     */
    private render(): void {
        this.container.innerHTML = `
            <div class="executor-overlay" id="executor-overlay">
                <div class="executor-modal">
                    <div class="executor-header">
                        <h3>Execute Pipeline: ${this.escapeHtml(this.spec.name)}</h3>
                        <button id="executor-close-btn" class="btn-close">✕</button>
                    </div>
                    <div class="executor-body">
                        ${this.renderContent()}
                    </div>
                    <div class="executor-footer">
                        ${this.renderFooter()}
                    </div>
                </div>
            </div>
        `;
    }

    /**
     * Render content based on state
     */
    private renderContent(): string {
        switch (this.state) {
            case 'idle':
            case 'selecting':
                return this.renderFileSelection();
            case 'running':
                return this.renderProgress();
            case 'success':
                return this.renderSuccess();
            case 'error':
                return this.renderError();
            default:
                return '';
        }
    }

    /**
     * Render file selection UI
     */
    private renderFileSelection(): string {
        return `
            <div class="executor-section">
                <h4>Input Dataset</h4>
                ${this.inputPath ? `
                    <div class="file-selected">
                        <span class="file-icon">📄</span>
                        <span class="file-path">${this.escapeHtml(this.inputPath)}</span>
                    </div>
                ` : `
                    <p class="text-secondary">No file selected</p>
                `}
                <button id="select-input-btn" class="btn-secondary">Select Input File...</button>
            </div>

            <div class="executor-section">
                <h4>Output Location</h4>
                ${this.outputPath ? `
                    <div class="file-selected">
                        <span class="file-icon">💾</span>
                        <span class="file-path">${this.escapeHtml(this.outputPath)}</span>
                    </div>
                ` : `
                    <p class="text-secondary">Use default from pipeline spec</p>
                `}
                <button id="select-output-btn" class="btn-secondary">Specify Output File... (Optional)</button>
            </div>

            <div class="executor-section">
                <h4>Pipeline Steps</h4>
                <p class="text-secondary">${this.spec.steps.length} transformation step(s)</p>
            </div>
        `;
    }

    /**
     * Render progress UI
     */
    private renderProgress(): string {
        return `
            <div class="executor-progress">
                <div class="progress-spinner"></div>
                <h4>Executing Pipeline...</h4>
                <p>Processing ${this.escapeHtml(this.inputPath || '')}...</p>
                <p class="text-secondary">This may take a few moments</p>
            </div>
        `;
    }

    /**
     * Render success UI
     */
    private renderSuccess(): string {
        if (!this.result) return '';

        return `
            <div class="executor-success">
                <div class="success-icon">✓</div>
                <h4>Pipeline Completed Successfully</h4>

                <div class="result-stats">
                    <div class="stat-item">
                        <div class="stat-label">Rows</div>
                        <div class="stat-value">${this.result.rows_before.toLocaleString()} → ${this.result.rows_after.toLocaleString()}</div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-label">Columns</div>
                        <div class="stat-value">${this.result.columns_before} → ${this.result.columns_after}</div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-label">Steps Applied</div>
                        <div class="stat-value">${this.result.steps_applied}</div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-label">Duration</div>
                        <div class="stat-value">${this.result.duration_secs.toFixed(2)}s</div>
                    </div>
                </div>

                ${this.result.warnings.length > 0 ? `
                    <div class="result-warnings">
                        <h5>Warnings:</h5>
                        <ul>
                            ${this.result.warnings.map(w => `<li>${this.escapeHtml(w)}</li>`).join('')}
                        </ul>
                    </div>
                ` : ''}

                <p class="result-summary">${this.escapeHtml(this.result.summary)}</p>
            </div>
        `;
    }

    /**
     * Render error UI
     */
    private renderError(): string {
        return `
            <div class="executor-error">
                <div class="error-icon">⚠</div>
                <h4>Execution Failed</h4>
                <p class="error-message">${this.escapeHtml(this.error || 'Unknown error')}</p>
            </div>
        `;
    }

    /**
     * Render footer buttons
     */
    private renderFooter(): string {
        switch (this.state) {
            case 'idle':
            case 'selecting':
                return `
                    <button id="execute-btn" class="btn-primary" ${!this.inputPath ? 'disabled' : ''}>
                        ▶ Execute Pipeline
                    </button>
                    <button id="cancel-btn" class="btn-secondary">Cancel</button>
                `;
            case 'running':
                return `
                    <button class="btn-secondary" disabled>Executing...</button>
                `;
            case 'success':
                return `
                    <button id="done-btn" class="btn-primary">Done</button>
                `;
            case 'error':
                return `
                    <button id="retry-btn" class="btn-primary">Retry</button>
                    <button id="cancel-btn" class="btn-secondary">Cancel</button>
                `;
            default:
                return '';
        }
    }

    /**
     * Attach event listeners
     */
    private attachEventListeners(): void {
        // Close button
        const closeBtn = this.container.querySelector('#executor-close-btn');
        closeBtn?.addEventListener('click', () => this.close());

        // Overlay click (close on backdrop click)
        const overlay = this.container.querySelector('#executor-overlay');
        overlay?.addEventListener('click', (e) => {
            if (e.target === overlay) {
                this.close();
            }
        });

        // File selection buttons
        const selectInputBtn = this.container.querySelector('#select-input-btn');
        selectInputBtn?.addEventListener('click', () => this.selectInputFile());

        const selectOutputBtn = this.container.querySelector('#select-output-btn');
        selectOutputBtn?.addEventListener('click', () => this.selectOutputFile());

        // Action buttons
        const executeBtn = this.container.querySelector('#execute-btn');
        executeBtn?.addEventListener('click', () => this.execute());

        const cancelBtn = this.container.querySelector('#cancel-btn');
        cancelBtn?.addEventListener('click', () => this.close());

        const doneBtn = this.container.querySelector('#done-btn');
        doneBtn?.addEventListener('click', () => this.close());

        const retryBtn = this.container.querySelector('#retry-btn');
        retryBtn?.addEventListener('click', () => {
            this.state = 'idle';
            this.error = null;
            this.result = null;
            this.render();
            this.attachEventListeners();
        });
    }

    /**
     * Select input file
     */
    private async selectInputFile(): Promise<void> {
        try {
            const selected = await open({
                title: 'Select Input Dataset',
                filters: [{
                    name: 'Data Files',
                    extensions: ['csv', 'json', 'parquet']
                }]
            });

            if (selected) {
                this.inputPath = selected as string;
                this.render();
                this.attachEventListeners();
            }
        } catch (error) {
            console.error('Error selecting input file:', error);
        }
    }

    /**
     * Select output file
     */
    private async selectOutputFile(): Promise<void> {
        try {
            const selected = await open({
                title: 'Select Output Location',
                filters: [{
                    name: 'Data Files',
                    extensions: ['csv', 'json', 'parquet']
                }]
            });

            if (selected) {
                this.outputPath = selected as string;
                this.render();
                this.attachEventListeners();
            }
        } catch (error) {
            console.error('Error selecting output file:', error);
        }
    }

    /**
     * Execute pipeline
     */
    private async execute(): Promise<void> {
        if (!this.inputPath) return;

        this.state = 'running';
        this.render();
        this.attachEventListeners();

        try {
            this.result = await executePipeline(
                this.spec,
                this.inputPath,
                this.outputPath || undefined
            );

            this.state = 'success';
        } catch (error) {
            this.error = String(error);
            this.state = 'error';
        }

        this.render();
        this.attachEventListeners();
    }

    /**
     * Escape HTML
     */
    private escapeHtml(text: string): string {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
}
