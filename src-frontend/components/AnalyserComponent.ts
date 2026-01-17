import Chart, { ChartConfiguration } from 'chart.js/auto';

import * as api from '../api';
import * as renderers from '../renderers';
import { AppState, ColumnCleanConfig, DatasetVersion, LifecycleStage } from '../types';

import { Component, ComponentActions } from './Component';
import { ExportModal } from './ExportModal';

export class AnalyserComponent extends Component {
  private charts: Map<string, Chart> = new Map();
  private isTransitioning: boolean = false;

  private getCurrentStage(state: AppState): LifecycleStage | null {
    if (!state.currentDataset?.activeVersionId) {
      return null;
    }

    const { activeVersionId, versions } = state.currentDataset;
    const activeVersion = versions.find(v => v.id === activeVersionId);

    return activeVersion?.stage ?? null;
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
      document.getElementById('btn-abort-op')?.addEventListener('click', () => {
        state.isAborting = true;
        this.render(state);
        void api.abortProcessing();
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

    // Special rendering for Validated stage - show summary view
    if (currentStage === 'Validated') {
      const existingWrapper = container.querySelector('.analyser-wrapper');
      if (!existingWrapper) {
        container.innerHTML = `
          <div class="analyser-wrapper">
            <div id="lifecycle-rail-container"></div>
            <div id="analyser-content-container" class="analyser-container">
              ${renderers.renderValidatedSummary(state.analysisResponse, state.currentDataset)}
            </div>
          </div>
        `;
      } else {
        const contentContainer = document.getElementById('analyser-content-container');
        if (contentContainer) {
          contentContainer.innerHTML = renderers.renderValidatedSummary(
            state.analysisResponse,
            state.currentDataset
          );
        }
      }
      this.bindValidatedEvents(state);
      return;
    }

    // Check if this is the first render (container is empty or doesn't have wrapper)
    const existingWrapper = container.querySelector('.analyser-wrapper');
    if (!existingWrapper) {
      // First render: set entire HTML including wrapper structure
      container.innerHTML = renderers.renderAnalyser(
        state.analysisResponse,
        state.expandedRows,
        state.cleaningConfigs,
        currentStage,
        isReadOnly,
        state.selectedColumns,
        state.useOriginalColumnNames
      );
    } else {
      // Subsequent renders: only update content container to preserve lifecycle rail
      const contentContainer = document.getElementById('analyser-content-container');
      if (contentContainer) {
        // Temporarily store the content HTML by re-generating it
        const fullHTML = renderers.renderAnalyser(
          state.analysisResponse,
          state.expandedRows,
          state.cleaningConfigs,
          currentStage,
          isReadOnly,
          state.selectedColumns,
          state.useOriginalColumnNames
        );

        // Extract just the content container portion from the generated HTML
        const tempDiv = document.createElement('div');
        tempDiv.innerHTML = fullHTML;
        const newContentContainer = tempDiv.querySelector('#analyser-content-container');

        if (newContentContainer) {
          contentContainer.innerHTML = newContentContainer.innerHTML;
        }
      }
    }

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

  private bindEmptyAnalyserEvents(_state: AppState): void {
    document.getElementById('btn-open-file')?.addEventListener('click', () => {
      void (async () => {
        const path = await api.openFileDialog();
        if (path) {
          this.actions.runAnalysis(path);
        }
      })();
    });
  }

  override bindEvents(state: AppState): void {
    if (!state.analysisResponse) return;

    // Expand/Collapse rows
    document.querySelectorAll('.analyser-row').forEach(row => {
      row.addEventListener('click', e => {
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
      checkbox.addEventListener('change', e => {
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
      el.addEventListener('change', e => {
        const target = e.target as HTMLInputElement | HTMLSelectElement;
        const colName = target.dataset.col!;
        const prop = target.dataset.prop as keyof ColumnCleanConfig;

        const config = state.cleaningConfigs[colName];
        if (!config) return;

        if (target.type === 'checkbox') {
          const checked = (target as HTMLInputElement).checked;
          if (prop === 'active') config.active = checked;
          else if (prop === 'one_hot_encode') config.one_hot_encode = checked;
          else if (prop === 'clip_outliers') config.clip_outliers = checked;
        } else {
          const value = target.value;
          if (prop === 'rounding') {
            config.rounding = value === 'none' ? null : parseInt(value);
          } else if (prop === 'impute_mode') {
            config.impute_mode = value as ColumnCleanConfig['impute_mode'];
          } else if (prop === 'normalisation') {
            config.normalisation = value as ColumnCleanConfig['normalisation'];
          } else if (prop === 'text_case') {
            config.text_case = value as ColumnCleanConfig['text_case'];
          } else if (prop === 'new_name') {
            config.new_name = value;
          }
        }

        this.actions.onStateChange();
      });
    });

    // Header actions (Bulk changes)
    document.querySelectorAll('.header-action').forEach(el => {
      el.addEventListener('change', e => {
        const target = e.target as HTMLInputElement | HTMLSelectElement;
        const action = target.dataset.action!;

        if (action === 'active-all') {
          const checked = (target as HTMLInputElement).checked;
          state.cleanAllActive = checked;
          Object.values(state.cleaningConfigs).forEach(c => (c.active = checked));
        } else if (action === 'use-original-names') {
          const checked = (target as HTMLInputElement).checked;
          state.useOriginalColumnNames = checked;
          // Update all configs to use either original or standardized names
          if (state.analysisResponse) {
            state.analysisResponse.summary.forEach(s => {
              const config = state.cleaningConfigs[s.name];
              if (config) {
                config.new_name = checked ? s.name : s.standardized_name;
              }
            });
          }
        } else if (action === 'impute-all') {
          const val = target.value;
          Object.values(state.cleaningConfigs).forEach(
            c => (c.impute_mode = val as ColumnCleanConfig['impute_mode'])
          );
        } else if (action === 'round-all') {
          const val = target.value === 'none' ? null : parseInt(target.value);
          Object.values(state.cleaningConfigs).forEach(c => (c.rounding = val));
        } else if (action === 'norm-all') {
          const val = target.value;
          Object.values(state.cleaningConfigs).forEach(
            c => (c.normalisation = val as ColumnCleanConfig['normalisation'])
          );
        } else if (action === 'case-all') {
          const val = target.value;
          Object.values(state.cleaningConfigs).forEach(
            c => (c.text_case = val as ColumnCleanConfig['text_case'])
          );
        } else if (action === 'onehot-all') {
          const checked = (target as HTMLInputElement).checked;
          Object.values(state.cleaningConfigs).forEach(c => (c.one_hot_encode = checked));
        }

        // Re-render to update UI with new config values
        this.render(state);
        this.actions.onStateChange();
      });
    });

    document.querySelectorAll('.header-action-icon').forEach(el => {
      el.addEventListener('click', e => {
        const target = e.currentTarget as HTMLElement;
        const action = target.dataset.action!;
        if (action === 'standardize-all' && state.analysisResponse) {
          state.analysisResponse.summary.forEach(s => {
            const config = state.cleaningConfigs[s.name];
            if (config) {
              config.new_name = s.standardized_name;
            }
          });
          this.actions.showToast('Headers standardized', 'success');
          this.render(state);
          this.actions.onStateChange();
        }
      });
    });

    // Header buttons
    document.getElementById('btn-open-file')?.addEventListener('click', () => {
      void (async () => {
        const path = await api.openFileDialog();
        if (path) {
          this.actions.runAnalysis(path);
        }
      })();
    });

    document.getElementById('btn-reanalyze')?.addEventListener('click', () => {
      if (state.analysisResponse) {
        this.actions.runAnalysis(state.analysisResponse.path);
      }
    });

    document.getElementById('btn-export')?.addEventListener('click', () => {
      void this.handleExport(state);
    });

    document.getElementById('btn-export-analyser')?.addEventListener('click', () => {
      void this.handleExport(state);
    });

    const btnBeginCleaning = document.getElementById('btn-begin-cleaning') as HTMLButtonElement;
    btnBeginCleaning?.addEventListener('click', () => {
      void (async () => {
        if (this.isTransitioning) return;
        btnBeginCleaning.disabled = true;

        // Start timer
        const startTime = Date.now();
        const updateTimer = (): void => {
          const elapsed = Math.floor((Date.now() - startTime) / 1000);
          btnBeginCleaning.innerHTML = `<i class="ph ph-circle-notch ph-spin"></i> Transitioning... ${elapsed}s`;
        };
        updateTimer();
        const timerInterval = setInterval(updateTimer, 1000);

        try {
          await this.handleBeginCleaning(state);
        } finally {
          clearInterval(timerInterval);
          btnBeginCleaning.disabled = false;
          btnBeginCleaning.innerHTML = '<i class="ph ph-broom"></i> Begin Cleaning';
        }
      })();
    });

    const btnContinueAdvanced = document.getElementById(
      'btn-continue-advanced'
    ) as HTMLButtonElement;
    btnContinueAdvanced?.addEventListener('click', () => {
      void (async () => {
        if (this.isTransitioning) return;
        btnContinueAdvanced.disabled = true;

        // Start timer
        const startTime = Date.now();
        const updateTimer = (): void => {
          const elapsed = Math.floor((Date.now() - startTime) / 1000);
          btnContinueAdvanced.innerHTML = `<i class="ph ph-circle-notch ph-spin"></i> Transitioning... ${elapsed}s`;
        };
        updateTimer();
        const timerInterval = setInterval(updateTimer, 1000);

        try {
          await this.handleContinueToAdvanced(state);
        } finally {
          clearInterval(timerInterval);
          btnContinueAdvanced.disabled = false;
          btnContinueAdvanced.innerHTML = '<i class="ph ph-arrow-right"></i> Continue to Advanced';
        }
      })();
    });

    const btnMoveToValidated = document.getElementById(
      'btn-move-to-validated'
    ) as HTMLButtonElement;
    btnMoveToValidated?.addEventListener('click', () => {
      void (async () => {
        if (this.isTransitioning) return;
        btnMoveToValidated.disabled = true;

        // Start timer
        const startTime = Date.now();
        const updateTimer = (): void => {
          const elapsed = Math.floor((Date.now() - startTime) / 1000);
          btnMoveToValidated.innerHTML = `<i class="ph ph-circle-notch ph-spin"></i> Transitioning... ${elapsed}s`;
        };
        updateTimer();
        const timerInterval = setInterval(updateTimer, 1000);

        try {
          await this.handleMoveToValidated(state);
        } finally {
          clearInterval(timerInterval);
          btnMoveToValidated.disabled = false;
          btnMoveToValidated.innerHTML = '<i class="ph ph-check-circle"></i> Move to Validated';
        }
      })();
    });

    // Cleaning info box toggle
    const cleaningInfoHeader = document.querySelector('.cleaning-info-header');
    if (cleaningInfoHeader) {
      cleaningInfoHeader.addEventListener('click', () => {
        const infoBox = document.querySelector('.cleaning-info-box');
        infoBox?.classList.toggle('collapsed');
      });
    }

    // Handle link to reference page in cleaning info box
    const cleaningInfoLink = document.querySelector('.cleaning-info-link');
    if (cleaningInfoLink) {
      cleaningInfoLink.addEventListener('click', e => {
        e.preventDefault();
        this.actions.navigateTo?.('reference');
      });
    }
  }

  private async handleExport(state: AppState): Promise<void> {
    if (!state.analysisResponse) return;

    const modal = new ExportModal('modal-container', this.actions, {
      type: 'Analyser',
      path: state.analysisResponse.path,
    });

    document.getElementById('modal-container')?.classList.add('active');
    await modal.show(state);
    document.getElementById('modal-container')?.classList.remove('active');
  }

  private async handleBeginCleaning(state: AppState): Promise<void> {
    if (!state.currentDataset) {
      this.actions.showToast('No dataset loaded', 'error');
      return;
    }

    this.isTransitioning = true;
    try {
      this.actions.showToast('Transitioning to Cleaning stage...', 'info');

      // Build pipeline with column selection if columns were excluded
      const pipeline: { transforms: unknown[] } = { transforms: [] };

      if (state.selectedColumns.size > 0 && state.analysisResponse) {
        const allColumns = state.analysisResponse.summary.map(c => c.name);
        const selectedCols = Array.from(state.selectedColumns);

        // Only add SelectColumnsTransform if some columns were excluded
        if (selectedCols.length < allColumns.length) {
          pipeline.transforms.push({
            transform_type: 'select_columns',
            parameters: {
              columns: selectedCols,
            },
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
      state.currentDataset.versions = JSON.parse(versionsJson) as DatasetVersion[];
      state.currentDataset.activeVersionId = newVersionId;

      // Re-render to show cleaning controls and update lifecycle rail
      this.actions.onStateChange();
      this.actions.showToast('Cleaning stage unlocked', 'success');
    } catch (err) {
      this.actions.showToast(`Failed to transition: ${String(err)}`, 'error');
    } finally {
      this.isTransitioning = false;
    }
  }

  private async handleContinueToAdvanced(state: AppState): Promise<void> {
    if (!state.currentDataset) {
      this.actions.showToast('No dataset loaded', 'error');
      return;
    }

    this.isTransitioning = true;
    try {
      this.actions.showToast('Transitioning to Advanced stage...', 'info');

      // Build pipeline from current cleaning configs
      const pipeline: { transforms: unknown[] } = { transforms: [] };

      // Add clean transform with current configs
      const activeConfigs = Object.fromEntries(
        Object.entries(state.cleaningConfigs).filter(([_, cfg]) => cfg.active)
      );

      if (Object.keys(activeConfigs).length > 0) {
        pipeline.transforms.push({
          transform_type: 'clean',
          parameters: {
            configs: activeConfigs,
            restricted: true, // Cleaned stage uses restricted mode
          },
        });
      }

      const pipelineJson = JSON.stringify(pipeline);

      // Apply transforms (which will create a new version in Advanced stage)
      const newVersionId = await api.applyTransforms(
        state.currentDataset.id,
        pipelineJson,
        'Advanced'
      );

      // Refresh versions
      const versionsJson = await api.listVersions(state.currentDataset.id);

      // Update state
      state.currentDataset.versions = JSON.parse(versionsJson) as DatasetVersion[];
      state.currentDataset.activeVersionId = newVersionId;

      // Re-render to show advanced controls and update lifecycle rail
      this.actions.onStateChange();
      this.actions.showToast('Advanced stage unlocked - ML preprocessing now available', 'success');
    } catch (err) {
      this.actions.showToast(`Failed to transition: ${String(err)}`, 'error');
    } finally {
      this.isTransitioning = false;
    }
  }

  private async handleMoveToValidated(state: AppState): Promise<void> {
    if (!state.currentDataset) {
      this.actions.showToast('No dataset loaded', 'error');
      return;
    }

    this.isTransitioning = true;
    try {
      this.actions.showToast('Transitioning to Validated stage...', 'info');

      // Build empty pipeline - validation is non-mutating
      const pipeline: { transforms: unknown[] } = { transforms: [] };
      const pipelineJson = JSON.stringify(pipeline);

      // Apply transforms (creates new version in Validated stage)
      const newVersionId = await api.applyTransforms(
        state.currentDataset.id,
        pipelineJson,
        'Validated'
      );

      // Refresh versions
      const versionsJson = await api.listVersions(state.currentDataset.id);

      // Update state
      state.currentDataset.versions = JSON.parse(versionsJson) as DatasetVersion[];
      state.currentDataset.activeVersionId = newVersionId;

      // Re-render to show validation summary
      this.actions.onStateChange();
      this.actions.showToast('Validated stage unlocked - ready for publishing', 'success');
    } catch (err) {
      this.actions.showToast(`Failed to transition: ${String(err)}`, 'error');
    } finally {
      this.isTransitioning = false;
    }
  }

  private bindValidatedEvents(state: AppState): void {
    // Back to Advanced button
    const btnBackToAdvanced = document.getElementById('btn-back-to-advanced');
    btnBackToAdvanced?.addEventListener('click', () => {
      void (async () => {
        if (!state.currentDataset) return;

        // Find the Advanced stage version
        const advancedVersion = state.currentDataset.versions.find(v => v.stage === 'Advanced');
        if (!advancedVersion) {
          this.actions.showToast('Advanced version not found', 'error');
          return;
        }

        try {
          // Set active version back to Advanced
          await api.setActiveVersion(state.currentDataset.id, advancedVersion.id);
          state.currentDataset.activeVersionId = advancedVersion.id;

          // Re-render
          this.actions.onStateChange();
          this.actions.showToast('Returned to Advanced stage', 'success');
        } catch (err) {
          this.actions.showToast(`Failed to switch version: ${String(err)}`, 'error');
        }
      })();
    });

    // Publish Dataset button
    const btnPublish = document.getElementById('btn-publish-dataset');
    btnPublish?.addEventListener('click', () => {
      void (async () => {
        if (!state.currentDataset || !state.analysisResponse) return;

        // Open export modal
        const modal = new ExportModal('modal-container', this.actions, {
          type: 'Analyser',
          path: state.analysisResponse.path,
        });

        document.getElementById('modal-container')?.classList.add('active');
        await modal.show(state);
        document.getElementById('modal-container')?.classList.remove('active');
      })();
    });
  }

  private initCharts(state: AppState): void {
    this.charts.forEach(c => {
      c.destroy();
    });
    this.charts.clear();

    if (!state.analysisResponse) return;

    state.expandedRows.forEach(colName => {
      const col = state.analysisResponse!.summary.find(s => s.name === colName);
      if (!col) return;

      const canvas = document.getElementById(`chart-${colName}`) as HTMLCanvasElement;
      if (!canvas) return;

      const ctx = canvas.getContext('2d');
      if (!ctx) return;

      let chartConfig: ChartConfiguration | null = null;

      if (col.stats.Numeric?.histogram) {
        chartConfig = {
          type: 'bar',
          data: {
            labels: col.stats.Numeric.histogram.map(s => s[0].toFixed(2)),
            datasets: [
              {
                label: 'Frequency',
                data: col.stats.Numeric.histogram.map(d => d[1]),
                backgroundColor: 'rgba(52, 152, 219, 0.5)',
                borderColor: 'rgba(52, 152, 219, 1)',
                borderWidth: 1,
              },
            ],
          },
          options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
              legend: { display: false },
              tooltip: {
                callbacks: {
                  label: context => {
                    return `Value: ${context.label}, Count: ${String(context.parsed.y)}`;
                  },
                },
              },
            },
          },
        };
      } else if (col.stats.Temporal?.histogram) {
        chartConfig = {
          type: 'bar',
          data: {
            labels: col.stats.Temporal.histogram.map(d => new Date(d[0]).toLocaleDateString()),
            datasets: [
              {
                label: 'Frequency',
                data: col.stats.Temporal.histogram.map(d => d[1]),
                backgroundColor: 'rgba(46, 204, 113, 0.5)',
                borderColor: 'rgba(46, 204, 113, 1)',
                borderWidth: 1,
              },
            ],
          },
          options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
              legend: { display: false },
              tooltip: {
                callbacks: {
                  label: context => {
                    return `Value: ${context.label}, Count: ${String(context.parsed.y)}`;
                  },
                },
              },
            },
          },
        };
      } else if (col.stats.Categorical) {
        const entries = Object.entries(col.stats.Categorical)
          .sort((a, b) => b[1] - a[1])
          .slice(0, 10);
        chartConfig = {
          type: 'doughnut',
          data: {
            labels: entries.map(e => e[0]),
            datasets: [
              {
                data: entries.map(e => e[1]),
                backgroundColor: [
                  '#3498db',
                  '#2ecc71',
                  '#e67e22',
                  '#e74c3c',
                  '#9b59b6',
                  '#1abc9c',
                  '#f1c40f',
                  '#34495e',
                  '#95a5a6',
                  '#d35400',
                ],
              },
            ],
          },
          options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
              legend: { display: true },
              tooltip: {
                callbacks: {
                  label: context => {
                    return `Value: ${context.label}, Count: ${String(context.raw)}`;
                  },
                },
              },
            },
          },
        };
      }

      if (chartConfig) {
        this.charts.set(colName, new Chart(ctx, chartConfig));
      }
    });
  }
}
