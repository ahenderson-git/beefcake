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
    
    this.container.innerHTML = renderers.renderExportModal(
      this.source, 
      connections, 
      activeExportId, 
      this.currentDestType,
      this.isExporting,
      this.isAborting
    );
    this.container.classList.add('active');
    this.bindEvents(state);
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
          this.render(state);
        }
      });
    });

    document.getElementById('btn-start-export')?.addEventListener('click', () => {
      if (!this.isExporting) this.handleExport(state);
    });

    document.getElementById('btn-abort-export')?.addEventListener('click', async () => {
      this.isAborting = true;
      this.render(state);
      await api.abortProcessing();
    });

    document.getElementById('btn-browse-export')?.addEventListener('click', async () => {
      const path = await api.saveFileDialog();
      if (path) {
        const input = document.getElementById('export-file-path') as HTMLInputElement;
        if (input) input.value = path;
      }
    });
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

    const options: ExportOptions = {
      source: this.source,
      configs: state.cleaningConfigs,
      destination: {
        type: this.currentDestType,
        target,
        ...(format && { format })
      }
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
    this.container.classList.remove('active');
    this.container.innerHTML = '';
    if (this.resolve) this.resolve(success);
  }

  public show(state: AppState): Promise<boolean> {
    return new Promise((resolve) => {
      this.resolve = resolve;
      this.render(state);
    });
  }
}
