import { Component, ComponentActions } from "./Component";
import { AppState, DataDictionary, SnapshotMetadata, DatasetBusinessMetadata, ColumnBusinessMetadata } from "../types";
import * as renderers from "../renderers";
import * as api from "../api";

/**
 * DictionaryComponent manages the data dictionary viewer/editor.
 *
 * Features:
 * - List all dictionary snapshots
 * - View snapshot details (technical + business metadata)
 * - Edit business metadata (dataset and column level)
 * - Export snapshots to markdown
 */
export class DictionaryComponent extends Component {
  private snapshots: SnapshotMetadata[] = [];
  private currentSnapshot: DataDictionary | null = null;
  private viewMode: 'list' | 'detail' = 'list';

  constructor(containerId: string, actions: ComponentActions) {
    super(containerId, actions);
  }

  async render(_state: AppState): Promise<void> {
    const container = this.getContainer();

    if (this.viewMode === 'list') {
      // Show snapshot list
      container.innerHTML = renderers.renderDictionaryList(this.snapshots);
      this.bindListEvents();
    } else if (this.viewMode === 'detail' && this.currentSnapshot) {
      // Show snapshot detail
      container.innerHTML = renderers.renderDictionaryDetail(this.currentSnapshot);
      this.bindDetailEvents();
    } else {
      container.innerHTML = '<div class="loading">Loading...</div>';
    }
  }

  /**
   * Load all snapshots from backend.
   */
  async loadSnapshots(): Promise<void> {
    try {
      this.snapshots = await api.dictionaryListSnapshots();
      this.viewMode = 'list';
      this.currentSnapshot = null;
    } catch (err) {
      this.actions.showToast(`Failed to load snapshots: ${err}`, 'error');
      this.snapshots = [];
    }
  }

  /**
   * Load a specific snapshot by ID.
   */
  async loadSnapshot(snapshotId: string): Promise<void> {
    try {
      this.currentSnapshot = await api.dictionaryLoadSnapshot(snapshotId);
      this.viewMode = 'detail';
    } catch (err) {
      this.actions.showToast(`Failed to load snapshot: ${err}`, 'error');
      this.viewMode = 'list';
    }
  }

  /**
   * Bind events for list view.
   */
  private bindListEvents(): void {
    // Refresh button
    document.getElementById('btn-refresh-snapshots')?.addEventListener('click', async () => {
      await this.loadSnapshots();
      this.actions.onStateChange();
    });

    // View snapshot buttons
    document.querySelectorAll('.btn-view').forEach(btn => {
      btn.addEventListener('click', async (e) => {
        const snapshotId = (e.currentTarget as HTMLElement).dataset.snapshotId!;
        await this.loadSnapshot(snapshotId);
        this.actions.onStateChange();
      });
    });

    // Export markdown buttons
    document.querySelectorAll('.btn-export-md').forEach(btn => {
      btn.addEventListener('click', async (e) => {
        e.stopPropagation();
        const snapshotId = (e.currentTarget as HTMLElement).dataset.snapshotId!;
        await this.exportMarkdown(snapshotId);
      });
    });

    // Row click to view details
    document.querySelectorAll('.snapshot-row').forEach(row => {
      row.addEventListener('click', async (e) => {
        if ((e.target as HTMLElement).closest('.snapshot-actions')) {
          return; // Don't trigger row click if clicking action buttons
        }
        const snapshotId = (e.currentTarget as HTMLElement).dataset.snapshotId!;
        await this.loadSnapshot(snapshotId);
        this.actions.onStateChange();
      });
    });
  }

  /**
   * Bind events for detail view.
   */
  private bindDetailEvents(): void {
    // Back button
    document.getElementById('btn-back-to-list')?.addEventListener('click', async () => {
      this.viewMode = 'list';
      this.currentSnapshot = null;
      this.actions.onStateChange();
    });

    // Save buttons (top and bottom)
    const saveHandler = async () => {
      await this.saveMetadata();
    };
    document.getElementById('btn-save-metadata')?.addEventListener('click', saveHandler);
    document.getElementById('btn-save-metadata-bottom')?.addEventListener('click', saveHandler);

    // Export markdown button
    document.getElementById('btn-export-markdown')?.addEventListener('click', async () => {
      if (this.currentSnapshot) {
        await this.exportMarkdown(this.currentSnapshot.snapshot_id);
      }
    });
  }

  /**
   * Save metadata changes (creates new snapshot version).
   */
  private async saveMetadata(): Promise<void> {
    if (!this.currentSnapshot) return;

    try {
      // Collect dataset business metadata from form
      const form = document.getElementById('dataset-business-form') as HTMLFormElement;
      const formData = new FormData(form);

      const datasetBusiness: DatasetBusinessMetadata = {
        description: (formData.get('description') as string) || "",
        intended_use: (formData.get('intended_use') as string) || "",
        owner_or_steward: (formData.get('owner_or_steward') as string) || "",
        refresh_expectation: (formData.get('refresh_expectation') as string) || "",
        sensitivity_classification: (formData.get('sensitivity_classification') as string) || "",
        known_limitations: (formData.get('known_limitations') as string) || "",
        tags: []
      };

      // Collect column business metadata
      const columnUpdates: Record<string, ColumnBusinessMetadata> = {};

      this.currentSnapshot.columns.forEach(col => {
        const definition = (document.querySelector(`.column-definition[data-column="${col.current_name}"]`) as HTMLTextAreaElement)?.value;
        const rules = (document.querySelector(`.column-rules[data-column="${col.current_name}"]`) as HTMLTextAreaElement)?.value;
        const sensitivity = (document.querySelector(`.column-sensitivity[data-column="${col.current_name}"]`) as HTMLSelectElement)?.value;
        const notes = (document.querySelector(`.column-notes[data-column="${col.current_name}"]`) as HTMLTextAreaElement)?.value;

        columnUpdates[col.current_name] = {
          business_definition: definition || "",
          business_rules: rules || "",
          sensitivity_tag: sensitivity || "",
          approved_examples: [],
          notes: notes || ""
        };
      });

      // Call API to update business metadata
      const newSnapshotId = await api.dictionaryUpdateBusinessMetadata(
        this.currentSnapshot.snapshot_id,
        datasetBusiness,
        columnUpdates
      );

      this.actions.showToast('Metadata saved successfully! Created new snapshot version.', 'success');

      // Reload the new snapshot
      await this.loadSnapshot(newSnapshotId);
      this.actions.onStateChange();
    } catch (err) {
      this.actions.showToast(`Failed to save metadata: ${err}`, 'error');
    }
  }

  /**
   * Export a snapshot to markdown.
   */
  private async exportMarkdown(snapshotId: string): Promise<void> {
    try {
      // Use Tauri dialog to select output path
      const outputPath = await api.saveFileDialog([{ name: 'Markdown', extensions: ['md'] }]);
      if (!outputPath) return;

      await api.dictionaryExportMarkdown(snapshotId, outputPath);
      this.actions.showToast(`Exported to: ${outputPath}`, 'success');
    } catch (err) {
      this.actions.showToast(`Failed to export markdown: ${err}`, 'error');
    }
  }
}
