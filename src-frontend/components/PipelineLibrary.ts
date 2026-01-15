/**
 * Pipeline Library Component
 *
 * Displays a list of saved pipelines with search, filter, and
 * action buttons (view, edit, execute, delete).
 *
 * ## State Management
 *
 * - Loads pipeline list on mount
 * - Filters based on search query
 * - Emits events when user selects a pipeline
 *
 * ## Events
 *
 * - `pipeline:new`: User wants to create new pipeline
 * - `pipeline:edit`: User wants to view/edit pipeline
 * - `pipeline:execute`: User wants to execute pipeline
 * - `pipeline:deleted`: User deleted a pipeline
 */

import {
    listPipelines,
    listTemplates,
    loadTemplate,
    type PipelineInfo,
} from '../api-pipeline';

export interface PipelineLibraryState {
    /** All available pipelines */
    pipelines: PipelineInfo[];

    /** All available templates */
    templates: PipelineInfo[];

    /** Current view mode */
    viewMode: 'pipelines' | 'templates';

    /** Search query for filtering */
    searchQuery: string;

    /** Loading state */
    isLoading: boolean;

    /** Error message (if load failed) */
    error: string | null;
}

export class PipelineLibrary {
    private state: PipelineLibraryState = {
        pipelines: [],
        templates: [],
        viewMode: 'pipelines',
        searchQuery: '',
        isLoading: false,
        error: null,
    };

    private container: HTMLElement;

    constructor(container: HTMLElement) {
        this.container = container;
    }

    /**
     * Initialize component and load pipelines.
     */
    async init(): Promise<void> {
        console.log('[PipelineLibrary] Initializing...');
        await Promise.all([
            this.loadPipelines(),
            this.loadTemplates()
        ]);
        console.log('[PipelineLibrary] After load - pipelines:', this.state.pipelines.length, 'templates:', this.state.templates.length);
        this.render();
        this.attachEventListeners();
    }

    /**
     * Load pipelines from backend.
     */
    private async loadPipelines(): Promise<void> {
        try {
            this.state.pipelines = await listPipelines();
        } catch (error) {
            this.state.error = `Failed to load pipelines: ${error}`;
            console.error(this.state.error);
        }
    }

    /**
     * Load templates from backend.
     */
    private async loadTemplates(): Promise<void> {
        try {
            const templates = await listTemplates();
            console.log('Raw templates response:', templates);
            console.log('Templates count:', templates.length);
            console.log('First template:', templates[0]);
            this.state.templates = templates;
        } catch (error) {
            console.error('Failed to load templates:', error);
            console.error('Error details:', JSON.stringify(error));
            this.state.error = `Failed to load templates: ${error}`;
        }
    }

    /**
     * Filter pipelines based on search query.
     */
    private getFilteredPipelines(): PipelineInfo[] {
        if (!this.state.searchQuery) {
            return this.state.pipelines;
        }

        const query = this.state.searchQuery.toLowerCase();
        return this.state.pipelines.filter(p =>
            p.name.toLowerCase().includes(query) ||
            (p.description && p.description.toLowerCase().includes(query))
        );
    }

    /**
     * Render component UI.
     */
    render(): void {
        const items = this.state.viewMode === 'pipelines'
            ? this.getFilteredPipelines()
            : this.state.templates;

        console.log('[PipelineLibrary] Rendering - viewMode:', this.state.viewMode, 'items:', items.length, 'error:', this.state.error);

        this.container.innerHTML = `
            <div class="pipeline-library">
                <div class="library-header">
                    <h2>Pipeline Library</h2>
                    <button id="new-pipeline-btn" class="btn-primary">
                        + New Pipeline
                    </button>
                </div>

                <div class="library-tabs">
                    <button
                        id="tab-pipelines"
                        class="library-tab ${this.state.viewMode === 'pipelines' ? 'active' : ''}"
                    >
                        📁 My Pipelines (${this.state.pipelines.length})
                    </button>
                    <button
                        id="tab-templates"
                        class="library-tab ${this.state.viewMode === 'templates' ? 'active' : ''}"
                    >
                        🎨 Templates (${this.state.templates.length})
                    </button>
                </div>

                ${this.state.viewMode === 'pipelines' ? `
                    <div class="search-bar">
                        <input
                            type="text"
                            id="pipeline-search"
                            placeholder="🔍 Search pipelines..."
                            value="${this.state.searchQuery}"
                        />
                    </div>
                ` : ''}

                ${this.renderContent(items)}
            </div>
        `;
    }

    /**
     * Render main content (loading, error, or pipeline list).
     */
    private renderContent(items: PipelineInfo[]): string {
        if (this.state.isLoading) {
            return '<div class="loading">Loading...</div>';
        }

        if (this.state.error) {
            return `
                <div class="error">
                    ${this.state.error}
                    <button id="retry-btn">Retry</button>
                </div>
            `;
        }

        if (items.length === 0) {
            return this.renderEmptyState();
        }

        if (this.state.viewMode === 'templates') {
            return `
                <div class="template-grid">
                    ${items.map(t => this.renderTemplateCard(t)).join('')}
                </div>
            `;
        }

        return `
            <div class="pipeline-list">
                ${items.map(p => this.renderPipelineCard(p)).join('')}
            </div>
        `;
    }

    /**
     * Render empty state when no pipelines found.
     */
    private renderEmptyState(): string {
        if (this.state.searchQuery) {
            return `
                <div class="empty-state">
                    <p>No pipelines match "${this.state.searchQuery}"</p>
                </div>
            `;
        }

        return `
            <div class="empty-state">
                <h3>No pipelines yet</h3>
                <p>Create your first pipeline to automate data transformations</p>
                <button id="new-pipeline-empty-btn" class="btn-primary">
                    Create Pipeline
                </button>
            </div>
        `;
    }

    /**
     * Render individual pipeline card.
     */
    private renderPipelineCard(pipeline: PipelineInfo): string {
        const created = pipeline.created
            ? new Date(pipeline.created).toLocaleDateString()
            : 'Unknown';
        const modified = pipeline.modified
            ? new Date(pipeline.modified).toLocaleDateString()
            : 'Unknown';

        return `
            <div class="pipeline-card" data-path="${this.escapeHtml(pipeline.path)}">
                <div class="card-header">
                    <h3>📄 ${this.escapeHtml(pipeline.name)}</h3>
                    <div class="card-actions">
                        <button class="btn-icon edit-btn" title="Edit" data-path="${this.escapeHtml(pipeline.path)}">✏️</button>
                        <button class="btn-icon execute-btn" title="Execute" data-path="${this.escapeHtml(pipeline.path)}">▶️</button>
                        <button class="btn-icon delete-btn" title="Delete" data-path="${this.escapeHtml(pipeline.path)}">🗑️</button>
                    </div>
                </div>
                <div class="card-body">
                    ${pipeline.description ? `<p class="card-description">${this.escapeHtml(pipeline.description)}</p>` : ''}
                    <p class="card-meta">
                        Created: ${created}
                    </p>
                    <p class="card-meta">
                        Modified: ${modified} | Steps: ${pipeline.step_count}
                    </p>
                </div>
            </div>
        `;
    }

    /**
     * Render individual template card.
     */
    private renderTemplateCard(template: PipelineInfo): string {
        // Extract template metadata from the file path
        const templateName = template.name;
        const icon = this.getTemplateIcon(templateName);
        const category = this.getTemplateCategory(templateName);

        return `
            <div class="template-card" data-template="${this.escapeHtml(templateName)}">
                <div class="template-icon">${icon}</div>
                <div class="template-content">
                    <h3>${this.escapeHtml(templateName)}</h3>
                    ${template.description ? `<p class="template-description">${this.escapeHtml(template.description)}</p>` : ''}
                    <div class="template-meta">
                        <span class="template-category">${category}</span>
                        <span class="template-steps">${template.step_count} steps</span>
                    </div>
                </div>
                <button class="btn-primary use-template-btn" data-template="${this.escapeHtml(templateName)}">
                    Use Template
                </button>
            </div>
        `;
    }

    /**
     * Get icon for template based on name
     */
    private getTemplateIcon(name: string): string {
        const iconMap: Record<string, string> = {
            'Data Cleaning': '🧹',
            'ML Preprocessing': '🤖',
            'Date Normalisation': '📅',
            'Text Processing': '📝',
            'Outlier Handling': '📊',
            'Column Selection': '🗂️',
            'Missing Data Handling': '🔧',
            'Type Conversion': '🔄'
        };
        return iconMap[name] || '📋';
    }

    /**
     * Get category for template based on name
     */
    private getTemplateCategory(name: string): string {
        if (name.includes('ML') || name.includes('Preprocessing')) return 'Machine Learning';
        if (name.includes('Clean') || name.includes('Missing')) return 'Data Cleaning';
        if (name.includes('Date') || name.includes('Type') || name.includes('Text')) return 'Transformation';
        if (name.includes('Outlier') || name.includes('Column')) return 'Analysis';
        return 'General';
    }

    /**
     * Escape HTML to prevent XSS.
     */
    private escapeHtml(text: string): string {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    /**
     * Attach event listeners to UI elements.
     */
    private attachEventListeners(): void {
        // Tab switching
        const pipelinesTab = this.container.querySelector('#tab-pipelines');
        pipelinesTab?.addEventListener('click', () => {
            this.state.viewMode = 'pipelines';
            this.state.searchQuery = '';
            this.render();
            this.attachEventListeners();
        });

        const templatesTab = this.container.querySelector('#tab-templates');
        templatesTab?.addEventListener('click', () => {
            this.state.viewMode = 'templates';
            this.render();
            this.attachEventListeners();
        });

        // Search input
        const searchInput = this.container.querySelector<HTMLInputElement>('#pipeline-search');
        searchInput?.addEventListener('input', (e) => {
            this.state.searchQuery = (e.target as HTMLInputElement).value;
            this.render();
            this.attachEventListeners(); // Re-attach after render
        });

        // New pipeline buttons
        const newBtn = this.container.querySelector('#new-pipeline-btn');
        const newEmptyBtn = this.container.querySelector('#new-pipeline-empty-btn');
        [newBtn, newEmptyBtn].forEach(btn => {
            btn?.addEventListener('click', () => this.handleNewPipeline());
        });

        // Retry button
        const retryBtn = this.container.querySelector('#retry-btn');
        retryBtn?.addEventListener('click', () => this.loadPipelines());

        // Pipeline card actions
        this.container.querySelectorAll('.edit-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const path = (e.currentTarget as HTMLElement).getAttribute('data-path');
                if (path) this.handleEditPipeline(path);
            });
        });

        this.container.querySelectorAll('.execute-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const path = (e.currentTarget as HTMLElement).getAttribute('data-path');
                if (path) this.handleExecutePipeline(path);
            });
        });

        this.container.querySelectorAll('.delete-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const path = (e.currentTarget as HTMLElement).getAttribute('data-path');
                if (path) this.handleDeletePipeline(path);
            });
        });

        // Template "Use Template" buttons
        this.container.querySelectorAll('.use-template-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const templateName = (e.currentTarget as HTMLElement).getAttribute('data-template');
                if (templateName) this.handleUseTemplate(templateName);
            });
        });
    }

    /**
     * Handle "New Pipeline" action.
     */
    private handleNewPipeline(): void {
        const event = new CustomEvent('pipeline:new');
        this.container.dispatchEvent(event);
    }

    /**
     * Handle "Edit Pipeline" action.
     */
    private handleEditPipeline(path: string): void {
        const event = new CustomEvent('pipeline:edit', {
            detail: { path },
        });
        this.container.dispatchEvent(event);
    }

    /**
     * Handle "Execute Pipeline" action.
     */
    private handleExecutePipeline(path: string): void {
        const event = new CustomEvent('pipeline:execute', {
            detail: { path },
        });
        this.container.dispatchEvent(event);
    }

    /**
     * Handle "Delete Pipeline" action.
     */
    private async handleDeletePipeline(path: string): Promise<void> {
        const pipeline = this.state.pipelines.find(p => p.path === path);
        if (!pipeline) return;

        const confirmed = confirm(
            `Delete pipeline "${pipeline.name}"? This cannot be undone.`
        );

        if (confirmed) {
            // TODO: Call delete API when available
            // For now, just remove from local state and refresh
            this.state.pipelines = this.state.pipelines.filter(
                p => p.path !== path
            );
            this.render();
            this.attachEventListeners();

            const event = new CustomEvent('pipeline:deleted', {
                detail: { path },
            });
            this.container.dispatchEvent(event);
        }
    }

    /**
     * Handle "Use Template" action.
     */
    private async handleUseTemplate(templateName: string): Promise<void> {
        try {
            // Load the template spec
            const spec = await loadTemplate(templateName);

            // Emit event to create new pipeline from template
            const event = new CustomEvent('pipeline:new-from-template', {
                detail: { spec },
            });
            this.container.dispatchEvent(event);
        } catch (error) {
            console.error(`Failed to load template "${templateName}":`, error);
            alert(`Failed to load template: ${error}`);
        }
    }

    /**
     * Refresh pipeline list.
     */
    async refresh(): Promise<void> {
        await this.loadPipelines();
    }
}
