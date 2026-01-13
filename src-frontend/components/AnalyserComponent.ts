import { Component, ComponentActions } from "./Component";
import { AppState, ColumnCleanConfig, LifecycleStage } from "../types";
import * as renderers from "../renderers";
import * as api from "../api";
import Chart from 'chart.js/auto';
import { ExportModal } from "./ExportModal";

export class AnalyserComponent extends Component {
  private charts: Map<string, Chart> = new Map();

  private getCurrentStage(state: AppState): LifecycleStage | null {
    if (!state.currentDataset || !state.currentDataset.activeVersionId) {
      return null;
    }
    const activeVersion = state.currentDataset.versions.find(
      v => v.id === state.currentDataset!.activeVersionId
    );
    return activeVersion?.stage || null;
  }

  private isReadOnlyStage(stage: LifecycleStage | null): boolean {
    return stage === 'Profiled' || stage === 'Raw';
  }

  constructor(containerId: string, actions: ComponentActions) {
    super(containerId, actions);
  }

  render(state: AppState): void {
    const container = this.getContainer();
    if (state.isLoading) {
      container.innerHTML = renderers.renderLoading(state.loadingMessage, state.isAborting);
      document.getElementById('btn-abort-op')?.addEventListener('click', async () => {
         state.isAborting = true;
         this.render(state);
         await api.abortProcessing();
      });
      return;
    }

    if (!state.analysisResponse) {
      container.innerHTML = renderers.renderEmptyAnalyser();
      this.bindEmptyAnalyserEvents(state);
      return;
    }

    const currentStage = this.getCurrentStage(state);
    const isReadOnly = this.isReadOnlyStage(currentStage);

    // Initialize selectedColumns if empty (default to all selected)
    if (state.selectedColumns.size === 0 && state.analysisResponse.summary.length > 0) {
      state.analysisResponse.summary.forEach(col => {
        state.selectedColumns.add(col.name);
      });
    }

    container.innerHTML = renderers.renderAnalyser(
      state.analysisResponse,
      state.expandedRows,
      state.cleaningConfigs,
      currentStage,
      isReadOnly,
      state.selectedColumns,
      state.useOriginalColumnNames
    );

    const header = document.getElementById('analyser-header-container');
    if (header) {
      header.innerHTML = renderers.renderAnalyserHeader(
        state.analysisResponse,
        currentStage,
        isReadOnly,
        state.useOriginalColumnNames,
        state.cleanAllActive
      );
    }

    this.bindEvents(state);
    this.initCharts(state);
  }

  private bindEmptyAnalyserEvents(_state: AppState) {
    document.getElementById('btn-open-file')?.addEventListener('click', async () => {
       const path = await api.openFileDialog();
       if (path) {
         this.actions.runAnalysis(path);
       }
    });
  }

  override bindEvents(state: AppState): void {
    if (!state.analysisResponse) return;

    // Expand/Collapse rows
    document.querySelectorAll('.analyser-row').forEach(row => {
      row.addEventListener('click', (e) => {
        if ((e.target as HTMLElement).closest('.row-action')) return;
        if ((e.target as HTMLElement).closest('.col-select-checkbox')) return;
        const colName = (e.currentTarget as HTMLElement).dataset.col!;
        if (state.expandedRows.has(colName)) {
          state.expandedRows.delete(colName);
        } else {
          state.expandedRows.add(colName);
        }
        this.render(state);
      });
    });

    // Column selection checkboxes
    document.querySelectorAll('.col-select-checkbox').forEach(checkbox => {
      checkbox.addEventListener('change', (e) => {
        const colName = (e.target as HTMLInputElement).dataset.col!;
        const isChecked = (e.target as HTMLInputElement).checked;

        if (isChecked) {
          state.selectedColumns.add(colName);
        } else {
          state.selectedColumns.delete(colName);
        }
        // No need to re-render, just track the state
      });
    });

    // Row actions (Active, Impute, Round, etc.)
    document.querySelectorAll('.row-action').forEach(el => {
      el.addEventListener('change', (e) => {
        const target = e.target as HTMLInputElement | HTMLSelectElement;
        const colName = target.dataset.col!;
        const prop = target.dataset.prop as keyof ColumnCleanConfig;
        
        if (!state.cleaningConfigs[colName]) return;
        
        if (target.type === 'checkbox') {
          (state.cleaningConfigs[colName] as any)[prop] = (target as HTMLInputElement).checked;
        } else {
          let value: any = target.value;
          if (prop === 'rounding') {
            value = value === 'none' ? null : parseInt(value);
          }
          (state.cleaningConfigs[colName] as any)[prop] = value;
        }
        
        this.actions.onStateChange();
      });
    });

    // Header actions (Bulk changes)
    document.querySelectorAll('.header-action').forEach(el => {
      el.addEventListener('change', (e) => {
        const target = e.target as HTMLInputElement | HTMLSelectElement;
        const action = target.dataset.action!;
        
        if (action === 'active-all') {
          const checked = (target as HTMLInputElement).checked;
          state.cleanAllActive = checked;
          Object.values(state.cleaningConfigs).forEach(c => c.active = checked);
        } else if (action === 'use-original-names') {
          const checked = (target as HTMLInputElement).checked;
          state.useOriginalColumnNames = checked;
          // Update all configs to use either original or standardized names
          if (state.analysisResponse) {
            state.analysisResponse.summary.forEach((s) => {
              const config = state.cleaningConfigs[s.name];
              if (config) {
                config.new_name = checked ? s.name : s.standardized_name;
              }
            });
          }
        } else if (action === 'impute-all') {
          const val = target.value;
          Object.values(state.cleaningConfigs).forEach(c => c.impute_mode = val as any);
        } else if (action === 'round-all') {
          const val = target.value === 'none' ? null : parseInt(target.value);
          Object.values(state.cleaningConfigs).forEach(c => c.rounding = val);
        } else if (action === 'norm-all') {
          const val = target.value;
          Object.values(state.cleaningConfigs).forEach(c => c.normalization = val as any);
        } else if (action === 'case-all') {
          const val = target.value;
          Object.values(state.cleaningConfigs).forEach(c => c.text_case = val as any);
        } else if (action === 'onehot-all') {
          const checked = (target as HTMLInputElement).checked;
          Object.values(state.cleaningConfigs).forEach(c => c.one_hot_encode = checked);
        }

        // Re-render to update UI with new config values
        this.render(state);
        this.actions.onStateChange();
      });
    });

    document.querySelectorAll('.header-action-icon').forEach(el => {
      el.addEventListener('click', async (e) => {
        const target = e.currentTarget as HTMLElement;
        const action = target.dataset.action!;
        if (action === 'standardize-all' && state.analysisResponse) {
          state.analysisResponse.summary.forEach((s) => {
            const config = state.cleaningConfigs[s.name];
            if (config) {
              config.new_name = s.standardized_name;
            }
          });
          this.actions.showToast("Headers standardized", "success");
          this.render(state);
          this.actions.onStateChange();
        }
      });
    });

    // Header buttons
    document.getElementById('btn-open-file')?.addEventListener('click', async () => {
      const path = await api.openFileDialog();
      if (path) {
        this.actions.runAnalysis(path);
      }
    });

    document.getElementById('btn-reanalyze')?.addEventListener('click', () => {
      if (state.analysisResponse) {
        this.actions.runAnalysis(state.analysisResponse.path);
      }
    });

    document.getElementById('btn-export')?.addEventListener('click', () => {
      this.handleExport(state);
    });

    document.getElementById('btn-export-analyser')?.addEventListener('click', () => {
      this.handleExport(state);
    });

    document.getElementById('btn-begin-cleaning')?.addEventListener('click', async () => {
      await this.handleBeginCleaning(state);
    });
  }

  private async handleExport(state: AppState) {
    if (!state.analysisResponse) return;

    const modal = new ExportModal('modal-container', this.actions, {
      type: 'Analyser',
      path: state.analysisResponse.path
    });

    document.getElementById('modal-container')?.classList.add('active');
    await modal.show(state);
    document.getElementById('modal-container')?.classList.remove('active');
  }

  private async handleBeginCleaning(state: AppState) {
    if (!state.currentDataset) {
      this.actions.showToast('No dataset loaded', 'error');
      return;
    }

    try {
      this.actions.showToast('Transitioning to Cleaning stage...', 'info');

      // Build pipeline with column selection if columns were excluded
      const pipeline: { transforms: any[] } = { transforms: [] };

      if (state.selectedColumns.size > 0 && state.analysisResponse) {
        const allColumns = state.analysisResponse.summary.map(c => c.name);
        const selectedCols = Array.from(state.selectedColumns);

        // Only add SelectColumnsTransform if some columns were excluded
        if (selectedCols.length < allColumns.length) {
          pipeline.transforms.push({
            transform_type: 'select_columns',
            parameters: {
              columns: selectedCols
            }
          });
        }
      }

      const pipelineJson = JSON.stringify(pipeline);

      // Apply transforms (which will create a new version in Cleaned stage)
      const newVersionId = await api.applyTransforms(
        state.currentDataset.id,
        pipelineJson,
        'Cleaned'
      );

      // Refresh versions
      const versionsJson = await api.listVersions(state.currentDataset.id);

      // Update state
      state.currentDataset.versions = JSON.parse(versionsJson);
      state.currentDataset.activeVersionId = newVersionId;

      // Re-render to show cleaning controls and update lifecycle rail
      this.actions.onStateChange();
      this.actions.showToast('Cleaning stage unlocked', 'success');
    } catch (err) {
      this.actions.showToast(`Failed to transition: ${err}`, 'error');
    }
  }

  private initCharts(state: AppState) {
    this.charts.forEach(c => c.destroy());
    this.charts.clear();

    if (!state.analysisResponse) return;

    state.expandedRows.forEach(colName => {
      const col = state.analysisResponse!.summary.find(s => s.name === colName);
      if (!col) return;

      const canvas = document.getElementById(`chart-${colName}`) as HTMLCanvasElement;
      if (!canvas) return;

      const ctx = canvas.getContext('2d');
      if (!ctx) return;

      let chartConfig: any = null;

      if (col.stats.Numeric?.histogram) {
        chartConfig = {
          type: 'bar',
          data: {
            labels: col.stats.Numeric.histogram.map(s => s[0].toFixed(2)),
            datasets: [{
              label: 'Frequency',
              data: col.stats.Numeric.histogram.map(d => d[1]),
              backgroundColor: 'rgba(52, 152, 219, 0.5)',
              borderColor: 'rgba(52, 152, 219, 1)',
              borderWidth: 1
            }]
          }
        };
      } else if (col.stats.Temporal?.histogram) {
        chartConfig = {
          type: 'bar',
          data: {
            labels: col.stats.Temporal.histogram.map(d => new Date(d[0]).toLocaleDateString()),
            datasets: [{
              label: 'Frequency',
              data: col.stats.Temporal.histogram.map(d => d[1]),
              backgroundColor: 'rgba(46, 204, 113, 0.5)',
              borderColor: 'rgba(46, 204, 113, 1)',
              borderWidth: 1
            }]
          }
        };
      } else if (col.stats.Categorical) {
        const entries = Object.entries(col.stats.Categorical)
          .sort((a, b) => b[1] - a[1])
          .slice(0, 10);
        chartConfig = {
          type: 'doughnut',
          data: {
            labels: entries.map(e => e[0]),
            datasets: [{
              data: entries.map(e => e[1]),
              backgroundColor: [
                '#3498db', '#2ecc71', '#e67e22', '#e74c3c', '#9b59b6',
                '#1abc9c', '#f1c40f', '#34495e', '#95a5a6', '#d35400'
              ]
            }]
          }
        };
      }

      if (chartConfig) {
        chartConfig.options = {
          responsive: true,
          maintainAspectRatio: false,
          plugins: {
            legend: { display: col.stats.Categorical !== undefined },
            tooltip: {
              callbacks: {
                label: (context: any) => `Value: ${context.label}, Count: ${context.parsed.y ?? context.parsed}`
              }
            }
          }
        };
        this.charts.set(colName, new Chart(ctx, chartConfig));
      }
    });
  }
}
