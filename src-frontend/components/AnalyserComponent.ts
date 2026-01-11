import { Component, ComponentActions } from "./Component";
import { AppState, ColumnCleanConfig } from "../types";
import * as renderers from "../renderers";
import * as api from "../api";
import Chart from 'chart.js/auto';
import { ExportModal } from "./ExportModal";

export class AnalyserComponent extends Component {
  private charts: Map<string, Chart> = new Map();

  constructor(containerId: string, actions: ComponentActions) {
    super(containerId, actions);
  }

  render(state: AppState): void {
    if (state.isLoading) {
      this.container.innerHTML = renderers.renderLoading(state.loadingMessage, state.isAborting);
      document.getElementById('btn-abort-op')?.addEventListener('click', async () => {
         state.isAborting = true;
         this.render(state);
         await api.abortProcessing();
      });
      return;
    }

    if (!state.analysisResponse) {
      this.container.innerHTML = renderers.renderEmptyAnalyser();
      this.bindEmptyAnalyserEvents(state);
      return;
    }

    this.container.innerHTML = renderers.renderAnalyser(
      state.analysisResponse,
      state.expandedRows,
      state.cleaningConfigs
    );

    const header = document.getElementById('analyser-header-container');
    if (header) {
      header.innerHTML = renderers.renderAnalyserHeader(state.analysisResponse, state.trimPct);
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
        const colName = (e.currentTarget as HTMLElement).dataset.col!;
        if (state.expandedRows.has(colName)) {
          state.expandedRows.delete(colName);
        } else {
          state.expandedRows.add(colName);
        }
        this.render(state);
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
          Object.values(state.cleaningConfigs).forEach(c => c.active = checked);
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

    // Trim control
    document.getElementById('trim-range')?.addEventListener('input', (e) => {
      const val = parseFloat((e.target as HTMLInputElement).value);
      state.trimPct = val;
      // Update the span next to it
      const span = (e.target as HTMLElement).nextElementSibling;
      if (span) span.textContent = `${Math.round(val * 100)}%`;
    });

    document.getElementById('trim-range')?.addEventListener('change', () => {
      this.actions.runAnalysis(state.analysisResponse!.path);
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
