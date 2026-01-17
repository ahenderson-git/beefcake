/**
 * Pipeline Component
 *
 * Main component for pipeline management, wrapping the PipelineLibrary
 * and handling pipeline-related events.
 */

import { save } from '@tauri-apps/plugin-dialog';

import { loadPipeline, savePipeline, PipelineSpec } from '../api-pipeline';
import { AppState } from '../types';

import { Component, ComponentActions } from './Component';
import { PipelineEditor } from './PipelineEditor';
import { PipelineExecutor } from './PipelineExecutor';
import { PipelineLibrary } from './PipelineLibrary';

type ViewMode = 'library' | 'editor';

export class PipelineComponent extends Component {
  private pipelineLibrary: PipelineLibrary | null = null;
  private pipelineEditor: PipelineEditor | null = null;
  private viewMode: ViewMode = 'library';
  private currentPipelinePath: string | null = null;
  private templateSpec: PipelineSpec | null = null;

  constructor(containerId: string, actions: ComponentActions) {
    super(containerId, actions);
  }

  render(_state: AppState): void {
    try {
      const container = this.getContainer();

      if (this.viewMode === 'library') {
        // Show library view
        container.innerHTML = `<div id="pipeline-library-container"></div>`;
        const libraryContainer = container.querySelector(
          '#pipeline-library-container'
        ) as HTMLElement;
        if (libraryContainer) {
          void this.initialisePipelineLibrary(libraryContainer);
        }
      } else if (this.viewMode === 'editor') {
        // Show editor view
        container.innerHTML = `<div id="pipeline-editor-container"></div>`;
        const editorContainer = container.querySelector(
          '#pipeline-editor-container'
        ) as HTMLElement;
        if (editorContainer) {
          void this.initialisePipelineEditor(editorContainer);
        }
      }
    } catch (error) {
      console.error('Error rendering pipeline component:', error);
      this.actions.showToast('Error rendering pipeline view', 'error');
      // Try to recover by showing library
      this.viewMode = 'library';
      this.pipelineLibrary = null;
      this.pipelineEditor = null;
      const container = this.getContainer();
      container.innerHTML = `
                <div class="error-recovery">
                    <h3>Something went wrong</h3>
                    <p>There was an error loading the pipeline view.</p>
                    <button id="retry-pipeline-btn" class="btn-primary">Retry</button>
                </div>
            `;
      const retryBtn = container.querySelector('#retry-pipeline-btn');
      retryBtn?.addEventListener('click', () => {
        this.render(_state);
      });
    }
  }

  private async initialisePipelineLibrary(libraryContainer: HTMLElement): Promise<void> {
    try {
      // Always create a fresh library instance
      this.pipelineLibrary = new PipelineLibrary(libraryContainer, this.actions);
      await this.pipelineLibrary.init();

      // Listen for pipeline events
      libraryContainer.addEventListener('pipeline:new', () => {
        this.handleNewPipeline();
      });

      libraryContainer.addEventListener('pipeline:edit', (e: Event) => {
        const { path } = (e as CustomEvent<{ path: string }>).detail;
        void this.handleEditPipeline(path);
      });

      libraryContainer.addEventListener('pipeline:execute', (e: Event) => {
        const { path } = (e as CustomEvent<{ path: string }>).detail;
        void this.handleExecutePipeline(path);
      });

      libraryContainer.addEventListener('pipeline:deleted', () => {
        this.actions.showToast('Pipeline deleted', 'success');
      });

      libraryContainer.addEventListener('pipeline:new-from-template', (e: Event) => {
        const { spec } = (e as CustomEvent<{ spec: PipelineSpec }>).detail;
        this.handleNewPipelineFromTemplate(spec);
      });
    } catch (error) {
      console.error('Error initializing pipeline library:', error);
      this.actions.showToast('Failed to load pipeline library', 'error');
      throw error;
    }
  }

  private async initialisePipelineEditor(editorContainer: HTMLElement): Promise<void> {
    // Load pipeline if editing existing
    let spec: PipelineSpec | undefined;
    if (this.currentPipelinePath) {
      try {
        spec = await loadPipeline(this.currentPipelinePath);
      } catch (error) {
        this.actions.showToast(`Failed to load pipeline: ${String(error)}`, 'error');
        this.showLibrary();
        return;
      }
    } else if (this.templateSpec) {
      // Use template spec for new pipeline from template
      spec = this.templateSpec;
      this.templateSpec = null; // Clear after use
    }

    // Create editor
    this.pipelineEditor = new PipelineEditor(editorContainer, spec);

    // Set callbacks
    this.pipelineEditor.setOnBack(() => {
      this.showLibrary();
    });

    this.pipelineEditor.setOnSave((spec: PipelineSpec) => {
      void this.handleSavePipeline(spec);
    });

    this.pipelineEditor.setOnExecute((spec: PipelineSpec) => {
      this.handleExecutePipelineFromSpec(spec);
    });

    // Initialize editor (this handles palette and config panel internally)
    this.pipelineEditor.init();
  }

  private showLibrary(): void {
    this.viewMode = 'library';
    this.currentPipelinePath = null;
    // Reset all editor state
    this.pipelineEditor = null;
    // Also reset library to force re-initialization
    this.pipelineLibrary = null;
    this.render(this.getDefaultState());
  }

  private showEditor(pipelinePath?: string): void {
    this.viewMode = 'editor';
    this.currentPipelinePath = pipelinePath ?? null;
    // Reset editor component
    this.pipelineEditor = null;
    this.render(this.getDefaultState());
  }

  private getDefaultState(): AppState {
    // Return a minimal state object - we don't actually use it
    return {} as AppState;
  }

  private handleNewPipeline(): void {
    this.showEditor();
  }

  private handleNewPipelineFromTemplate(spec: PipelineSpec): void {
    // Store template spec to be used in editor initialization
    this.templateSpec = spec;
    this.showEditor();
  }

  private handleEditPipeline(path: string): void {
    this.showEditor(path);
  }

  private async handleSavePipeline(spec: PipelineSpec): Promise<void> {
    try {
      // Determine save path
      let savePath = this.currentPipelinePath;

      if (!savePath) {
        // New pipeline - prompt for location
        const result = await save({
          title: 'Save Pipeline',
          defaultPath: `${spec.name}.json`,
          filters: [
            {
              name: 'Pipeline Spec',
              extensions: ['json'],
            },
          ],
        });

        if (!result) {
          return; // User cancelled
        }

        savePath = result;
      }

      // Save pipeline
      await savePipeline(savePath, spec);
      this.currentPipelinePath = savePath;
      this.actions.showToast(`Pipeline saved: ${spec.name}`, 'success');
    } catch (error) {
      this.actions.showToast(`Failed to save pipeline: ${String(error)}`, 'error');
      console.error('Error saving pipeline:', error);
    }
  }

  private async handleExecutePipeline(path: string): Promise<void> {
    try {
      // Load the pipeline spec
      const spec = await loadPipeline(path);
      this.handleExecutePipelineFromSpec(spec);
    } catch (error) {
      this.actions.showToast(`Failed to load pipeline: ${String(error)}`, 'error');
      console.error('Error loading pipeline:', error);
    }
  }

  private handleExecutePipelineFromSpec(spec: PipelineSpec): void {
    try {
      // Create executor modal container
      const container = this.getContainer();
      const executorContainer = document.createElement('div');
      executorContainer.id = 'pipeline-executor-modal';
      container.appendChild(executorContainer);

      // Create and show executor
      const executor = new PipelineExecutor(executorContainer, spec);
      executor.setOnClose(() => {
        // Clean up modal when closed
        executorContainer.remove();
      });

      void executor.show();
    } catch (error) {
      this.actions.showToast(`Failed to execute pipeline: ${String(error)}`, 'error');
      console.error('Error executing pipeline:', error);
    }
  }

  cleanup(): void {
    this.pipelineLibrary = null;
  }
}
