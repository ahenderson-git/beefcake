/**
 * Step Palette Component
 *
 * Displays available transformation steps organized by category.
 * Users can click to add steps to their pipeline.
 */

import { PipelineStep } from '../api-pipeline';

export interface StepDefinition {
    id: string;
    name: string;
    category: string;
    description: string;
    icon: string;
    createStep: () => PipelineStep;
}

export class StepPalette {
    private container: HTMLElement;
    private searchQuery: string = '';
    private expandedCategories: Set<string> = new Set(['Data Cleaning']);
    private onStepAdd?: (step: PipelineStep) => void;

    // Available step definitions
    private readonly stepDefinitions: StepDefinition[] = [
        // Data Cleaning
        {
            id: 'trim_whitespace',
            name: 'Trim Whitespace',
            category: 'Data Cleaning',
            description: 'Remove leading and trailing spaces from text columns',
            icon: '✂️',
            createStep: () => ({
                op: 'trim_whitespace',
                columns: []
            } as unknown as PipelineStep)
        },
        {
            id: 'drop_columns',
            name: 'Drop Columns',
            category: 'Data Cleaning',
            description: 'Remove unwanted columns from the dataset',
            icon: '🗑️',
            createStep: () => ({
                op: 'drop_columns',
                columns: []
            } as unknown as PipelineStep)
        },
        {
            id: 'rename_columns',
            name: 'Rename Columns',
            category: 'Data Cleaning',
            description: 'Rename columns with better names',
            icon: '✏️',
            createStep: () => ({
                op: 'rename_columns',
                mapping: {}
            } as unknown as PipelineStep)
        },

        // Type Conversion
        {
            id: 'cast_types',
            name: 'Cast Types',
            category: 'Type Conversion',
            description: 'Convert column data types',
            icon: '🔄',
            createStep: () => ({
                op: 'cast_types',
                columns: {}
            } as unknown as PipelineStep)
        },
        {
            id: 'parse_dates',
            name: 'Parse Dates',
            category: 'Type Conversion',
            description: 'Parse text columns as dates with custom formats',
            icon: '📅',
            createStep: () => ({
                op: 'parse_dates',
                columns: {}
            } as unknown as PipelineStep)
        },

        // Missing Data
        {
            id: 'impute',
            name: 'Impute Missing',
            category: 'Missing Data',
            description: 'Fill missing values using statistical methods',
            icon: '🔧',
            createStep: () => ({
                op: 'impute',
                strategy: 'mean',
                columns: []
            } as unknown as PipelineStep)
        },

        // Normalisation
        {
            id: 'normalise',
            name: 'Normalise',
            category: 'Normalisation',
            description: 'Scale numeric columns using z-score or min-max',
            icon: '📊',
            createStep: () => ({
                op: 'normalise_columns',
                method: 'z_score',
                columns: []
            } as unknown as PipelineStep)
        },
        {
            id: 'clip_outliers',
            name: 'Clip Outliers',
            category: 'Normalisation',
            description: 'Cap extreme values using quantiles',
            icon: '📐',
            createStep: () => ({
                op: 'clip_outliers',
                columns: [],
                lower_quantile: 0.05,
                upper_quantile: 0.95
            } as unknown as PipelineStep)
        },

        // Feature Engineering
        {
            id: 'one_hot_encode',
            name: 'One-Hot Encode',
            category: 'Feature Engineering',
            description: 'Convert categorical columns to binary columns',
            icon: '🎯',
            createStep: () => ({
                op: 'one_hot_encode',
                columns: [],
                drop_original: true
            } as unknown as PipelineStep)
        },
        {
            id: 'extract_numbers',
            name: 'Extract Numbers',
            category: 'Feature Engineering',
            description: 'Extract numeric values from text using regex',
            icon: '🔢',
            createStep: () => ({
                op: 'extract_numbers',
                columns: []
            } as unknown as PipelineStep)
        },
        {
            id: 'regex_replace',
            name: 'Regex Replace',
            category: 'Feature Engineering',
            description: 'Find and replace text using patterns',
            icon: '🔍',
            createStep: () => ({
                op: 'regex_replace',
                columns: [],
                pattern: '',
                replacement: ''
            } as unknown as PipelineStep)
        },
    ];

    constructor(container: HTMLElement) {
        this.container = container;
    }

    /**
     * Initialize the palette
     */
    init(): void {
        this.render();
        this.attachEventListeners();
    }

    /**
     * Set callback for when step is added
     */
    setOnStepAdd(callback: (step: PipelineStep) => void): void {
        this.onStepAdd = callback;
    }

    /**
     * Render the palette
     */
    render(): void {
        const categories = this.getCategories();
        const filteredSteps = this.getFilteredSteps();

        this.container.innerHTML = `
            <div class="step-palette">
                <div class="palette-header">
                    <h3>Step Palette</h3>
                </div>

                <div class="palette-search">
                    <input
                        type="text"
                        id="palette-search-input"
                        placeholder="🔍 Search steps..."
                        value="${this.escapeHtml(this.searchQuery)}"
                    />
                </div>

                <div class="palette-categories">
                    ${categories.map(category => this.renderCategory(category, filteredSteps)).join('')}
                </div>
            </div>
        `;
    }

    /**
     * Get unique categories
     */
    private getCategories(): string[] {
        const categories = new Set(this.stepDefinitions.map(s => s.category));
        return Array.from(categories);
    }

    /**
     * Filter steps by search query
     */
    private getFilteredSteps(): StepDefinition[] {
        if (!this.searchQuery) {
            return this.stepDefinitions;
        }

        const query = this.searchQuery.toLowerCase();
        return this.stepDefinitions.filter(step =>
            step.name.toLowerCase().includes(query) ||
            step.description.toLowerCase().includes(query) ||
            step.category.toLowerCase().includes(query)
        );
    }

    /**
     * Render a category section
     */
    private renderCategory(category: string, filteredSteps: StepDefinition[]): string {
        const categorySteps = filteredSteps.filter(s => s.category === category);

        if (categorySteps.length === 0) {
            return '';
        }

        const isExpanded = this.expandedCategories.has(category);

        return `
            <div class="palette-category">
                <div class="category-header" data-category="${this.escapeHtml(category)}">
                    <span class="category-toggle">${isExpanded ? '▼' : '▶'}</span>
                    <span class="category-name">${this.escapeHtml(category)}</span>
                    <span class="category-count">${categorySteps.length}</span>
                </div>
                ${isExpanded ? `
                    <div class="category-steps">
                        ${categorySteps.map(step => this.renderStep(step)).join('')}
                    </div>
                ` : ''}
            </div>
        `;
    }

    /**
     * Render a single step
     */
    private renderStep(step: StepDefinition): string {
        return `
            <div class="palette-step" data-step-id="${step.id}" title="${this.escapeHtml(step.description)}">
                <span class="step-icon">${step.icon}</span>
                <div class="step-info">
                    <div class="step-name">${this.escapeHtml(step.name)}</div>
                    <div class="step-description">${this.escapeHtml(step.description)}</div>
                </div>
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
        // Search input
        const searchInput = this.container.querySelector<HTMLInputElement>('#palette-search-input');
        searchInput?.addEventListener('input', (e) => {
            this.searchQuery = (e.target as HTMLInputElement).value;
            this.render();
            this.attachEventListeners();
        });

        // Category toggle
        this.container.querySelectorAll('.category-header').forEach(header => {
            header.addEventListener('click', (e) => {
                const category = (e.currentTarget as HTMLElement).getAttribute('data-category');
                if (category) {
                    this.toggleCategory(category);
                }
            });
        });

        // Step click
        this.container.querySelectorAll('.palette-step').forEach(stepEl => {
            stepEl.addEventListener('click', (e) => {
                const stepId = (e.currentTarget as HTMLElement).getAttribute('data-step-id');
                if (stepId) {
                    this.handleStepClick(stepId);
                }
            });
        });
    }

    /**
     * Toggle category expansion
     */
    private toggleCategory(category: string): void {
        if (this.expandedCategories.has(category)) {
            this.expandedCategories.delete(category);
        } else {
            this.expandedCategories.add(category);
        }
        this.render();
        this.attachEventListeners();
    }

    /**
     * Handle step click
     */
    private handleStepClick(stepId: string): void {
        const stepDef = this.stepDefinitions.find(s => s.id === stepId);
        if (stepDef && this.onStepAdd) {
            const step = stepDef.createStep();
            this.onStepAdd(step);
        }
    }
}
