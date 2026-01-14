import { Component, ComponentActions } from "./Component";
import { AppState, ExportOptions, ExportSource } from "../types";
import * as renderers from "../renderers";
import * as api from "../api";

export interface ExportModalActions extends ComponentActions {
  onExport: (options: ExportOptions) => Promise<void>;
}

export class ExportModal extends Component {
  private source: ExportSource;
  private resolve?: (value: boolean) => void;
  private currentDestType: 'File' | 'Database' = 'File';
  private isExporting: boolean = false;
  private isAborting: boolean = false;

  constructor(containerId: string, actions: ComponentActions, source: ExportSource) {
    super(containerId, actions);
    this.source = source;
  }

  render(state: AppState): void {
    const activeExportId = state.config?.active_export_id;
    const connections = state.config?.connections || [];
    
    const container = this.getContainer();
    container.innerHTML = renderers.renderExportModal(
      this.source, 
      connections, 
      activeExportId, 
      this.currentDestType,
      this.isExporting,
      this.isAborting
    );
    container.classList.add('active');
    this.bindEvents(state);
  }

  private updateConfigSection(state: AppState): void {
    const activeExportId = state.config?.active_export_id;
    const connections = state.config?.connections || [];

    const configContainer = document.getElementById('export-config-container');
    if (configContainer) {
      configContainer.innerHTML = renderers.renderExportConfig(
        this.currentDestType,
        connections,
        activeExportId
      );
    }

    // Update toggle button active states
    document.querySelectorAll('.toggle-btn').forEach(btn => {
      const btnElement = btn as HTMLElement;
      if (btnElement.dataset.dest === this.currentDestType) {
        btnElement.classList.add('active');
      } else {
        btnElement.classList.remove('active');
      }
    });

    // Re-bind events for the new config section
    this.bindConfigEvents();
    this.bindExportButton(state);
  }

  private bindConfigEvents(): void {
    document.getElementById('btn-browse-export')?.addEventListener('click', async () => {
      const path = await api.saveFileDialog();
      if (path) {
        const input = document.getElementById('export-file-path') as HTMLInputElement;
        if (input) input.value = path;
      }
    });
  }

  private bindExportButton(state: AppState): void {
    document.getElementById('btn-start-export')?.addEventListener('click', () => {
      if (!this.isExporting) this.handleExport(state);
    });
  }

  override bindEvents(state: AppState): void {
    const modal = document.getElementById('export-modal');
    if (!modal) return;

    // Close on overlay click (only if not exporting)
    modal.addEventListener('click', (e) => {
      if (e.target === modal && !this.isExporting) this.close(false);
    });

    document.querySelectorAll('.btn-close-modal').forEach(btn => {
      btn.addEventListener('click', () => {
        if (!this.isExporting) this.close(false);
      });
    });

    // Toggle destination type
    document.querySelectorAll('.toggle-btn').forEach(btn => {
      btn.addEventListener('click', (e) => {
        if (!this.isExporting) {
          const target = e.currentTarget as HTMLElement;
          this.currentDestType = target.dataset.dest as 'File' | 'Database';
          this.updateConfigSection(state);
        }
      });
    });

    this.bindExportButton(state);

    document.getElementById('btn-abort-export')?.addEventListener('click', async () => {
      this.isAborting = true;
      this.render(state);
      await api.abortProcessing();
    });

    this.bindConfigEvents();
  }

  private async handleExport(state: AppState) {
    let target = '';
    let format: 'csv' | 'json' | 'parquet' | undefined;

    if (this.currentDestType === 'File') {
      const input = document.getElementById('export-file-path') as HTMLInputElement;
      if (!input || !input.value) {
        this.actions.showToast('Please select a destination file', 'error');
        return;
      }
      target = input.value;
      const ext = target.split('.').pop()?.toLowerCase();
      if (ext === 'csv' || ext === 'json' || ext === 'parquet') {
        format = ext as any;
      } else {
        format = 'parquet'; // Default
      }
    } else {
      const connSelect = document.getElementById('export-connection-id') as HTMLSelectElement;
      if (!connSelect) return;

      target = connSelect.value;
      if (!target) {
        this.actions.showToast('Please select a database connection', 'error');
        return;
      }
    }

    // Check if dictionary creation is enabled (only for file exports)
    let createDictionary = true; // Default
    if (this.currentDestType === 'File') {
      const dictCheckbox = document.getElementById('export-create-dictionary') as HTMLInputElement;
      if (dictCheckbox) {
        createDictionary = dictCheckbox.checked;
      }
    }

    const options: ExportOptions = {
      source: this.source,
      configs: state.cleaningConfigs,
      destination: {
        type: this.currentDestType,
        target,
        ...(format && { format })
      },
      create_dictionary: createDictionary
    };

    try {
      this.isExporting = true;
      this.render(state);
      
      this.actions.showToast(`Exporting to ${this.currentDestType}...`, 'info');
      await api.exportData(options);
      
      this.actions.showToast('Export successful!', 'success');
      this.close(true);
    } catch (err) {
      this.isExporting = false;
      this.render(state);
      console.error('Export failed:', err);
      this.actions.showToast(`Export failed: ${err}`, 'error');
    }
  }

  private close(success: boolean) {
    const container = this.getContainer();
    container.classList.remove('active');
    container.innerHTML = '';
    if (this.resolve) this.resolve(success);
  }

  public show(state: AppState): Promise<boolean> {
    return new Promise((resolve) => {
      this.resolve = resolve;
      this.render(state);
    });
  }
}
