# Pipeline Builder - Implementation Guide

This guide provides step-by-step instructions and code examples for implementing the Pipeline Builder UI.

## Prerequisites

- Read `PIPELINE_BUILDER_SPEC.md` for design overview
- Familiarity with TypeScript and Beefcake component architecture
- Understanding of Tauri IPC pattern (see `TYPESCRIPT_PATTERNS.md`)

## Phase 1: Foundation

### Step 1: Create Pipeline API Module

**File**: `src-frontend/api-pipeline.ts`

```typescript
/**
 * Pipeline Management API
 *
 * Wraps Tauri commands for creating, managing, and executing
 * data transformation pipelines.
 *
 * ## Backend Integration
 *
 * These functions call Rust commands defined in `src/tauri_app.rs`:
 * - `list_pipelines`: Get all saved pipelines
 * - `load_pipeline`: Load pipeline from JSON
 * - `save_pipeline`: Save pipeline to JSON
 * - `validate_pipeline`: Check pipeline validity
 * - `execute_pipeline`: Run pipeline on dataset
 *
 * ## Example Usage
 *
 * ```typescript
 * // List all pipelines
 * const pipelines = await listPipelines();
 *
 * // Load specific pipeline
 * const spec = await loadPipeline('pipelines/cleaning.json');
 *
 * // Execute pipeline
 * const result = await executePipeline(spec, 'data.csv', 'out.parquet')
 */
```

import { invoke } from '@tauri-apps/api/core';

/**
 * Information about a saved pipeline file.
 */
export interface PipelineInfo {
    /** Pipeline display name */
    name: string;

    /** Full path to pipeline JSON file */
    path: string;

    /** ISO 8601 creation timestamp */
    created: string;

    /** ISO 8601 last modified timestamp */
    modified: string;

    /** Number of transformation steps */
    stepCount: number;

    /** Optional last execution timestamp */
    lastRun?: string;
}

/**
 * Complete pipeline specification.
 *
 * This is the main data structure for a pipeline. It's serialized
 * to JSON and can be saved, loaded, and executed.
 */
export interface PipelineSpec {
    /** Pipeline name (used for display and file naming) */
    name: string;

    /** Optional description of pipeline purpose */
    description?: string;

    /** Ordered list of transformation steps */
    steps: PipelineStep[];

    /** Metadata (creation date, author, etc.) */
    metadata?: Record<string, unknown>;
}

/**
 * A single transformation step in a pipeline.
 *
 * Each step has a `type` (e.g., "filter_rows", "select_columns")
 * and `params` specific to that transformation type.
 */
export interface PipelineStep {
    /** Step type identifier (e.g., "filter_rows") */
    type: string;

    /** Step-specific parameters */
    params: Record<string, unknown>;

    /** Optional step description */
    description?: string;
}

/**
 * Result of pipeline execution.
 */
export interface ExecutionResult {
    /** Whether execution completed successfully */
    success: boolean;

    /** Path to output dataset (if successful) */
    outputPath?: string;

    /** Number of rows in output */
    rowCount?: number;

    /** Number of columns in output */
    columnCount?: number;

    /** Error message (if failed) */
    error?: string;

    /** Results for individual steps */
    stepResults?: StepResult[];

    /** Total execution time in seconds */
    duration?: number;
}

/**
 * Result of executing a single pipeline step.
 */
export interface StepResult {
    /** Zero-based step index */
    stepIndex: number;

    /** Whether step succeeded */
    success: boolean;

    /** Step execution time in seconds */
    duration: number;

    /** Optional message (success or error) */
    message?: string;

    /** Rows affected by this step */
    rowsAffected?: number;
}

/**
 * Pipeline validation result.
 */
export interface ValidationResult {
    /** Whether pipeline is valid */
    valid: boolean;

    /** List of validation errors (if any) */
    errors?: string[];

    /** List of validation warnings */
    warnings?: string[];
}

/**
 * Lists all saved pipeline specifications.
 *
 * **Backend**: Calls `list_pipelines` in `src/tauri_app.rs`
 *
 * Scans the pipelines directory and returns metadata for each
 * pipeline JSON file found.
 *
 * @returns Promise resolving to array of pipeline info
 * @throws Error string if pipelines directory cannot be accessed
 */
export async function listPipelines(): Promise<PipelineInfo[]> {
    try {
        return await invoke<PipelineInfo[]>('list_pipelines');
    } catch (error) {
        console.error('Failed to list pipelines:', error);
        throw error;
    }
}

/**
 * Loads a pipeline specification from file.
 *
 * **Backend**: Calls `load_pipeline` in `src/tauri_app.rs`
 *
 * Reads and parses a pipeline JSON file into a PipelineSpec object.
 *
 * @param path - Absolute or relative path to pipeline JSON file
 * @returns Promise resolving to pipeline specification
 * @throws Error string if file not found or invalid JSON
 *
 * @example
 * ```typescript
 * const spec = await loadPipeline('pipelines/data_cleaning.json');
 * console.log(`Pipeline has ${spec.steps.length} steps`);
 * ```
 */
export async function loadPipeline(path: string): Promise<PipelineSpec> {
    try {
        return await invoke<PipelineSpec>('load_pipeline', { path });
    } catch (error) {
        console.error(`Failed to load pipeline from ${path}:`, error);
        throw error;
    }
}

/**
 * Saves a pipeline specification to file.
 *
 * **Backend**: Calls `save_pipeline` in `src/tauri_app.rs`
 *
 * Serializes pipeline to JSON and writes to specified path.
 * Creates parent directories if needed.
 *
 * @param path - Path where to save pipeline (relative or absolute)
 * @param spec - Pipeline specification to save
 * @returns Promise resolving to saved file path
 * @throws Error string if write fails or path invalid
 *
 * @example
 * ```typescript
 * const spec: PipelineSpec = {
 *     name: 'My Pipeline',
 *     steps: [
 *         { type: 'filter_rows', params: { column: 'status', value: 'active' } }
 *     ]
 * };
 * await savePipeline('pipelines/my_pipeline.json', spec);
 * ```
 */
export async function savePipeline(
    path: string,
    spec: PipelineSpec
): Promise<string> {
    try {
        return await invoke<string>('save_pipeline', { path, spec });
    } catch (error) {
        console.error(`Failed to save pipeline to ${path}:`, error);
        throw error;
    }
}

/**
 * Validates a pipeline specification.
 *
 * **Backend**: Calls `validate_pipeline` in `src/tauri_app.rs`
 *
 * Checks pipeline for:
 * - Valid step types
 * - Required parameters for each step
 * - Compatible data types
 * - Logical errors (e.g., filtering before selecting columns)
 *
 * @param spec - Pipeline specification to validate
 * @returns Promise resolving to validation result
 *
 * @example
 * ```typescript
 * const result = await validatePipeline(spec);
 * if (!result.valid) {
 *     console.error('Pipeline errors:', result.errors);
 * }
 * ```
 */
export async function validatePipeline(
    spec: PipelineSpec
): Promise<ValidationResult> {
    try {
        return await invoke<ValidationResult>('validate_pipeline', { spec });
    } catch (error) {
        console.error('Failed to validate pipeline:', error);
        throw error;
    }
}

/**
 * Executes a pipeline on a dataset.
 *
 * **Backend**: Calls `execute_pipeline` in `src/tauri_app.rs`
 *
 * Runs each pipeline step in sequence, passing output from one
 * step as input to the next. Can optionally save result to file.
 *
 * ## Progress Tracking
 *
 * Pipeline execution can be long-running. Consider listening to
 * progress events (if backend supports them) or polling status.
 *
 * @param spec - Pipeline specification to execute
 * @param inputPath - Path to input dataset (CSV, JSON, or Parquet)
 * @param outputPath - Optional path for output (creates new version if omitted)
 * @returns Promise resolving to execution result
 * @throws Error string if execution fails
 *
 * @example
 * ```typescript
 * try {
 *     const result = await executePipeline(
 *         spec,
 *         'data/sales.csv',
 *         'data/sales_cleaned.csv'
 *     );
 *
 *     if (result.success) {
 *         console.log(`Processed ${result.rowCount} rows in ${result.duration}s`);
 *     }
 * } catch (error) {
 *     console.error('Pipeline failed:', error);
 * }
 * ```
 */
export async function executePipeline(
    spec: PipelineSpec,
    inputPath: string,
    outputPath?: string
): Promise<ExecutionResult> {
    try {
        return await invoke<ExecutionResult>('execute_pipeline', {
            spec,
            inputPath,
            outputPath,
        });
    } catch (error) {
        console.error('Failed to execute pipeline:', error);
        throw error;
    }
}
```

### Step 2: Create PipelineLibrary Component

**File**: `src-frontend/components/PipelineLibrary.ts`

```typescript
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
 * - `pipeline:selected`: User wants to view/edit pipeline
 * - `pipeline:execute`: User wants to execute pipeline
 * - `pipeline:deleted`: User deleted a pipeline
 */

import {
    listPipelines,
    type PipelineInfo,
} from '../api-pipeline';

export interface PipelineLibraryState {
    /** All available pipelines */
    pipelines: PipelineInfo[];

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
        await this.loadPipelines();
        this.render();
        this.attachEventListeners();
    }

    /**
     * Load pipelines from backend.
     */
    private async loadPipelines(): Promise<void> {
        this.state.isLoading = true;
        this.state.error = null;
        this.render();

        try {
            this.state.pipelines = await listPipelines();
        } catch (error) {
            this.state.error = `Failed to load pipelines: ${error}`;
            console.error(this.state.error);
        } finally {
            this.state.isLoading = false;
            this.render();
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
            p.name.toLowerCase().includes(query)
        );
    }

    /**
     * Render component UI.
     */
    render(): void {
        const pipelines = this.getFilteredPipelines();

        this.container.innerHTML = `
            <div class="pipeline-library">
                <div class="library-header">
                    <h2>Pipeline Library</h2>
                    <button id="new-pipeline-btn" class="btn-primary">
                        + New Pipeline
                    </button>
                </div>

                <div class="search-bar">
                    <input
                        type="text"
                        id="pipeline-search"
                        placeholder="🔍 Search pipelines..."
                        value="${this.state.searchQuery}"
                    />
                </div>

                ${this.renderContent(pipelines)}
            </div>
        `;
    }

    /**
     * Render main content (loading, error, or pipeline list).
     */
    private renderContent(pipelines: PipelineInfo[]): string {
        if (this.state.isLoading) {
            return '<div class="loading">Loading pipelines...</div>';
        }

        if (this.state.error) {
            return `
                <div class="error">
                    ${this.state.error}
                    <button id="retry-btn">Retry</button>
                </div>
            `;
        }

        if (pipelines.length === 0) {
            return this.renderEmptyState();
        }

        return `
            <div class="pipeline-list">
                ${pipelines.map(p => this.renderPipelineCard(p)).join('')}
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
        const lastRun = pipeline.lastRun
            ? new Date(pipeline.lastRun).toLocaleDateString()
            : 'Never';

        return `
            <div class="pipeline-card" data-path="${pipeline.path}">
                <div class="card-header">
                    <h3>📄 ${pipeline.name}</h3>
                    <div class="card-actions">
                        <button class="btn-icon edit-btn" title="Edit">✏️</button>
                        <button class="btn-icon execute-btn" title="Execute">▶️</button>
                        <button class="btn-icon delete-btn" title="Delete">🗑️</button>
                    </div>
                </div>
                <div class="card-body">
                    <p class="card-meta">
                        Created: ${new Date(pipeline.created).toLocaleDateString()}
                    </p>
                    <p class="card-meta">
                        Steps: ${pipeline.stepCount} | Last run: ${lastRun}
                    </p>
                </div>
            </div>
        `;
    }

    /**
     * Attach event listeners to UI elements.
     */
    private attachEventListeners(): void {
        // Search input
        const searchInput = this.container.querySelector('#pipeline-search');
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
        this.container.querySelectorAll('.pipeline-card').forEach(card => {
            const path = card.getAttribute('data-path');
            if (!path) return;

            card.querySelector('.edit-btn')?.addEventListener('click', () => {
                this.handleEditPipeline(path);
            });

            card.querySelector('.execute-btn')?.addEventListener('click', () => {
                this.handleExecutePipeline(path);
            });

            card.querySelector('.delete-btn')?.addEventListener('click', () => {
                this.handleDeletePipeline(path);
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
            // TODO: Call delete API
            // For now, just remove from local state
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
     * Refresh pipeline list.
     */
    async refresh(): Promise<void> {
        await this.loadPipelines();
    }
}
```

### Step 3: Create Basic CSS Styles

**File**: `src-frontend/styles/pipeline.css`

```css
:root {
    --error-color: #ff0000;
    --primary-color: #007bff;
    --primary-color-dark: #0056b3;
    --border-color: #ddd;
    --card-bg: #fff;
    --text-secondary: #6c757d;
}

/* Pipeline Library Styles */

.pipeline-library {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 1rem;
}

.library-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
}

.library-header h2 {
    margin: 0;
}

.search-bar {
    margin-bottom: 1rem;
}

.search-bar input {
    width: 100%;
    padding: 0.5rem 1rem;
    font-size: 1rem;
    border: 1px solid var(--border-color);
    border-radius: 4px;
}

.pipeline-list {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 1rem;
    overflow-y: auto;
}

.pipeline-card {
    background: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 1rem;
    transition: box-shadow 0.2s;
}

.pipeline-card:hover {
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.card-header {
    display: flex;
    justify-content: space-between;
    align-items: start;
    margin-bottom: 0.5rem;
}

.card-header h3 {
    margin: 0;
    font-size: 1.1rem;
}

.card-actions {
    display: flex;
    gap: 0.5rem;
}

.btn-icon {
    background: none;
    border: none;
    font-size: 1.2rem;
    cursor: pointer;
    padding: 0.25rem;
    opacity: 0.6;
    transition: opacity 0.2s;
}

.btn-icon:hover {
    opacity: 1;
}

.card-body {
    color: var(--text-secondary);
}

.card-meta {
    margin: 0.25rem 0;
    font-size: 0.9rem;
}

.empty-state {
    text-align: center;
    padding: 3rem 1rem;
    color: var(--text-secondary);
}

.empty-state h3 {
    margin-bottom: 0.5rem;
}

.empty-state p {
    margin-bottom: 1rem;
}

.loading,
.error {
    text-align: center;
    padding: 2rem;
}

.error {
    color: #ff0000; /* var(--error-color) */
}

.btn-primary {
    background: #007bff; /* var(--primary-color) */
    color: white;
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    cursor: pointer;
    font-size: 1rem;
    transition: background 0.2s;
}

.btn-primary:hover {
    background: #0056b3; /* var(--primary-color-dark) */
}
```

### Step 4: Integrate into Main App

**File**: `src-frontend/main.ts` (modifications)

Add pipeline component to navigation and main app:

```typescript
import { PipelineLibrary } from './components/PipelineLibrary';

class BeefcakeApp {
    // ... existing code ...

    private pipelineLibrary?: PipelineLibrary;

    private async initComponents(): Promise<void> {
        // ... existing components ...

        // Initialize Pipeline Library
        const pipelineContainer = document.getElementById('pipeline-container');
        if (pipelineContainer) {
            this.pipelineLibrary = new PipelineLibrary(pipelineContainer);

            // Listen for pipeline events
            pipelineContainer.addEventListener('pipeline:new', () => {
                this.showView('pipeline-editor');
            });

            pipelineContainer.addEventListener('pipeline:edit', (e: Event) => {
                const { path } = (e as CustomEvent).detail;
                this.editPipeline(path);
            });

            pipelineContainer.addEventListener('pipeline:execute', (e: Event) => {
                const { path } = (e as CustomEvent).detail;
                this.executePipeline(path);
            });
        }
    }

    private async editPipeline(path: string): Promise<void> {
        // TODO: Load pipeline and show editor
        console.log('Edit pipeline:', path);
    }

    private async executePipeline(path: string): Promise<void> {
        // TODO: Load pipeline and show executor
        console.log('Execute pipeline:', path);
    }

    // ... existing code ...
}
```

**File**: `index.html` (add container)

```html
<div id="pipeline-container" class="view" style="display: none;"></div>
```

## Testing Phase 1

### Manual Testing Checklist

- [ ] Pipeline library loads without errors
- [ ] Pipeline list displays correctly
- [ ] Search filters pipelines
- [ ] "New Pipeline" button triggers event
- [ ] Pipeline card actions trigger correct events
- [ ] Empty state shows when no pipelines
- [ ] Error handling works (disconnect backend)
- [ ] Refresh reloads pipeline list

### Unit Test Example

```typescript
// tests/components/PipelineLibrary.test.ts

import { PipelineLibrary } from '../../src-frontend/components/PipelineLibrary';

describe('PipelineLibrary', () => {
    let container: HTMLElement;
    let library: PipelineLibrary;

    beforeEach(() => {
        container = document.createElement('div');
        library = new PipelineLibrary(container);
    });

    it('should render empty state when no pipelines', () => {
        library.render();
        expect(container.textContent).toContain('No pipelines yet');
    });

    it('should filter pipelines by search query', () => {
        // TODO: Mock pipeline data and test filtering
    });

    it('should emit events when actions are clicked', () => {
        // TODO: Test event emission
    });
});
```

## Next Steps

After completing Phase 1:

1. **Verify Backend Commands**: Test that all 5 pipeline commands work correctly
2. **Create Sample Pipelines**: Add example pipeline JSON files for testing
3. **Proceed to Phase 2**: Implement PipelineEditor component
4. **Add Progress Tracking**: Enhance backend to emit execution progress

## Troubleshooting

### Common Issues

**Problem**: Pipelines don't load
- Check backend `list_pipelines` command is working
- Verify pipelines directory exists
- Check console for error messages

**Problem**: Events not firing
- Ensure event listeners are re-attached after render
- Check event names match between component and app
- Verify container element exists

**Problem**: Styling looks wrong
- Import pipeline.css in main.ts or index.html
- Check CSS variable definitions exist
- Verify card grid is responsive

---

## Phase 5: Optional Enhancements (COMPLETED)

### Phase 5A: Drag-and-Drop & Templates

#### ✅ Enhancement 1: Drag-and-Drop Step Reordering

**Implementation Status**: Complete

**Files Modified**:
- `src-frontend/components/PipelineEditor.ts` (+80 lines)
- `src-frontend/style.css` (+28 lines)

**Key Features**:
1. Visual drag handle (⋮⋮) on each step card
2. HTML5 Drag-and-Drop API integration
3. Visual feedback: opacity during drag, border highlight on drop targets
4. Smart selection tracking (selection follows dragged step)
5. Fallback: Up/Down arrow buttons remain for keyboard accessibility

**Technical Implementation**:
```typescript
// In PipelineEditor class
class PipelineEditor {
  private draggedStepIndex: number | null = null;

  // Render step card with draggable attribute
  private renderStepCard(step: Step, index: number): string {
    return `
      <div class="step-card" data-index="${index}" draggable="true">
        <span class="step-drag-handle" title="Drag to reorder">⋮⋮</span>
        ...
      </div>
    `;
  }

  // Event handlers
  private attachDragHandlers(cardElement: HTMLElement, index: number): void {
    cardElement.addEventListener('dragstart', (e: DragEvent) => {
      this.draggedStepIndex = index;
      cardElement.classList.add('dragging');
    });

    cardElement.addEventListener('drop', (e: DragEvent) => {
      const dropIndex = parseInt(cardElement.getAttribute('data-index') || '0');
      if (this.draggedStepIndex !== null) {
        this.moveStepToIndex(this.draggedStepIndex, dropIndex);
      }
    });
  }

  // Reorder method with smart selection tracking
  private moveStepToIndex(fromIndex: number, toIndex: number): void {
    const steps = this.state.pipeline?.steps || [];
    const movedStep = steps[fromIndex];
    steps.splice(fromIndex, 1);
    steps.splice(toIndex, 0, movedStep);

    // Update selected index if necessary
    if (this.state.selectedStepIndex === fromIndex) {
      this.state.selectedStepIndex = toIndex;
    }
  }
}
```

**CSS Styling**:
```css
.step-card.dragging {
  opacity: 0.4;
  cursor: move !important;
}

.step-card.drag-over {
  border: 2px dashed #007bff; /* var(--primary-color) */
  background: rgba(52, 152, 219, 0.05);
  transform: scale(1.02);
}

.step-drag-handle {
  cursor: move;
  color: #999;
  font-size: 1.2rem;
  opacity: 0.6;
  transition: opacity 0.2s;
}

.step-card:hover .step-drag-handle {
  opacity: 1;
  color: #007bff; /* var(--primary-color) */
}
```

---

#### ✅ Enhancement 2: Pipeline Templates Library

**Implementation Status**: Complete

**Files Created**:
- 8 template JSON files in `data/pipelines/templates/`
- Templates: Data Cleaning, ML Preprocessing, Date Normalization, Text Processing, Outlier Handling, Column Selection, Missing Data Handling, Type Conversion

**Files Modified**:
- `src/tauri_app.rs` (+60 lines): Backend commands
- `src-frontend/api-pipeline.ts` (+52 lines): API wrappers
- `src-frontend/components/PipelineLibrary.ts` (+150 lines): Templates UI
- `src-frontend/components/PipelineComponent.ts` (+15 lines): Integration
- `src-frontend/style.css` (+125 lines): Template styling

**Backend Commands**:
```rust
// src/tauri_app.rs

#[tauri::command]
pub async fn list_pipeline_templates() -> Result<String, String> {
    let templates_dir = PathBuf::from("data")
        .join("pipelines")
        .join("templates");

    // Scan directory, return template metadata as JSON
}

#[tauri::command]
pub async fn load_pipeline_template(template_name: String) -> Result<String, String> {
    let template_path = PathBuf::from("data")
        .join("pipelines")
        .join("templates")
        .join(format!("{}.json", template_name.to_lowercase().replace(' ', "-")));

    let spec = PipelineSpec::from_file(&template_path)?;
    spec.to_json()
}
```

**Frontend API Wrappers**:
```typescript
// src-frontend/api-pipeline.ts

export async function listTemplates(): Promise<PipelineInfo[]> {
    const json = await invoke<string>('list_pipeline_templates');
    return JSON.parse(json);
}

export async function loadTemplate(templateName: string): Promise<PipelineSpec> {
    const json = await invoke<string>('load_pipeline_template', { templateName });
    return JSON.parse(json);
}
```

**UI Components**:
```typescript
// PipelineLibrary - Template rendering
class PipelineLibrary {
  private renderTemplateCard(template: PipelineInfo): string {
    const icon = this.getTemplateIcon(template.name);
    const category = this.getTemplateCategory(template.name);

    return `
      <div class="template-card" data-template="${template.name}">
        <div class="template-icon">${icon}</div>
        <div class="template-content">
          <h3>${template.name}</h3>
          <p class="template-description">${template.description}</p>
          <div class="template-meta">
            <span class="template-category">${category}</span>
            <span class="template-steps">${template.step_count} steps</span>
          </div>
        </div>
        <button class="btn-primary use-template-btn">Use Template</button>
      </div>
    `;
  }

  // Event handler for "Use Template" button
  private async handleUseTemplate(templateName: string): Promise<void> {
    const spec = await loadTemplate(templateName);
    const event = new CustomEvent('pipeline:new-from-template', {
      detail: { spec }
    });
    this.container.dispatchEvent(event);
  }
}
```

**Tab Switcher UI**:
```html
<div class="library-tabs">
    <button id="tab-pipelines" class="library-tab active">
        📁 My Pipelines (${this.state.pipelines.length})
    </button>
    <button id="tab-templates" class="library-tab">
        🎨 Templates (${this.state.templates.length})
    </button>
</div>
```

**Template Example** (`data/pipelines/templates/data-cleaning.json`):
```json
{
  "name": "Data Cleaning",
  "version": "0.1",
  "description": "Basic data cleaning workflow: trim whitespace, drop unwanted columns, and handle missing values",
  "category": "cleaning",
  "icon": "🧹",
  "steps": [
    { "op": "trim_whitespace", "columns": [] },
    { "op": "drop_columns", "columns": [] },
    { "op": "impute", "strategy": "mean", "columns": [] }
  ],
  "input": { "format": "csv" },
  "output": { "format": "parquet", "path": "" }
}
```

---

### Testing Phase 5A

**Drag-and-Drop Tests**:
1. Create pipeline with 5+ steps
2. Hover over step cards to see drag handle
3. Drag first step to last position - verify it moves
4. Drag middle step up and down - verify smooth reordering
5. Drag selected step - confirm selection follows
6. Check visual feedback: opacity during drag, border on drop target

**Template Tests**:
1. Navigate to Pipeline Library
2. Click "🎨 Templates" tab
3. Verify 8 templates displayed with icons
4. Click "Use Template" on "Data Cleaning" template
5. Verify Pipeline Editor opens with 3 pre-configured steps
6. Customize column parameters
7. Save with new name
8. Execute pipeline and verify it works

**Integration Tests**:
1. Load template → customize → save → execute
2. Create manual pipeline → reorder steps via drag → execute
3. Switch between My Pipelines and Templates tabs
4. Search pipelines (templates should not be searchable)

---

### Phase 5A Statistics

| Metric | Value |
|--------|-------|
| Total Files Modified | 6 |
| Total Files Created | 8 |
| Total Lines Added | ~500 |
| Backend Commands | 2 new |
| Frontend Components Updated | 3 |
| CSS Rules Added | ~125 |
| Build Time | 13.28s |
| Bundle Size Increase | +4KB |

---

## Reference

- See `PIPELINE_BUILDER_SPEC.md` for complete design
- See `TYPESCRIPT_PATTERNS.md` for Tauri bridge patterns
- See `src/tauri_app.rs` for backend command signatures
- See existing components (Dashboard, Analyser) for examples
- See `ENHANCEMENTS.md` for detailed Phase 5A implementation notes
