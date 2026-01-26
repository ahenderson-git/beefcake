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
    if (state.selectedColumns.size === 0 && (state.analysisResponse.summary || []).length > 0) {
      (state.analysisResponse.summary || []).forEach(col => {
        state.selectedColumns.add(col.name);
      });
    }

    // Ensure we have the basic wrapper structure
    const existingWrapper = container.querySelector('.analyser-wrapper');
    if (!existingWrapper) {
      container.innerHTML = `
        <div class="analyser-wrapper">
          <div id="lifecycle-rail-container"></div>
          <div id="analyser-content-container" class="analyser-container-outer"></div>
        </div>
      `;
    }

    const contentContainer = document.getElementById('analyser-content-container');
    if (!contentContainer) {
      // Fallback if something went wrong
      container.innerHTML = `
        <div class="analyser-wrapper">
          <div id="lifecycle-rail-container"></div>
          <div id="analyser-content-container" class="analyser-container-outer"></div>
        </div>
      `;
    }

    const targetContentContainer = document.getElementById('analyser-content-container')!;

    // Generate content based on stage
    let contentHTML: string;
    if (currentStage === 'Validated') {
      contentHTML = renderers.renderValidatedSummary(state.analysisResponse, state.currentDataset);
    } else if (currentStage === 'Published') {
      contentHTML = renderers.renderPublishedView(state.analysisResponse, state.currentDataset);
    } else {
      contentHTML = renderers.renderAnalyser(
        state.analysisResponse,
        state.expandedRows,
        state.cleaningConfigs,
        currentStage,
        isReadOnly,
        state.selectedColumns,
        state.useOriginalColumnNames,
        state.advancedProcessingEnabled
      );
    }

    // Update content
    targetContentContainer.innerHTML = contentHTML;

    // Post-render bindings
    if (currentStage === 'Validated') {
      this.bindValidatedEvents(state);
    } else if (currentStage === 'Published') {
      this.bindPublishedEvents(state);
    } else {
      this.bindEvents(state);
      this.initCharts(state);
    }
  }

  private bindEmptyAnalyserEvents(_state: AppState): void {
    document.getElementById('btn-open-file-empty')?.addEventListener('click', () => {
      void (async () => {
        const path = await api.openFileDialog();
        if (path) {
          this.actions.runAnalysis(path);
        }
      })();
    });
  }

  override bindEvents(state: AppState): void {
    /* eslint-disable no-console */
    console.log('[AnalyserComponent] bindEvents called, hasAnalysis:', !!state.analysisResponse);

    // Always bind these buttons even if no analysis yet
    const btnOpenFile = document.getElementById('btn-open-file');
    if (btnOpenFile) {
      console.log('[AnalyserComponent] Binding btn-open-file');
      btnOpenFile.addEventListener('click', () => {
        console.log('[AnalyserComponent] Open file clicked!');
        void (async () => {
          const path = await api.openFileDialog();
          if (path) {
            this.actions.runAnalysis(path);
          }
        })();
      });
    }

    const btnReanalyze = document.getElementById('btn-reanalyze');
    if (btnReanalyze) {
      console.log('[AnalyserComponent] Binding btn-reanalyze');
      btnReanalyze.addEventListener('click', () => {
        console.log('[AnalyserComponent] Reanalyze clicked!');
        void (async () => {
          const path = await api.openFileDialog();
          if (path) {
            this.actions.runAnalysis(path);
          }
        })();
      });
    }

    // If no analysis response, we're done here
    if (!state.analysisResponse) {
      console.log('[AnalyserComponent] No analysis response, skipping data-specific bindings');
      return;
    }

    console.log('[AnalyserComponent] Binding analysis-specific events...');
    /* eslint-enable no-console */

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
          else if (prop === 'ml_preprocessing') config.ml_preprocessing = checked;
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

        if (action === 'activate-advanced') {
          const checked = (target as HTMLInputElement).checked;
          state.advancedProcessingEnabled = checked;
          // Set ml_preprocessing on all configs to match global toggle
          Object.values(state.cleaningConfigs).forEach(c => (c.ml_preprocessing = checked));
        } else if (action === 'active-all') {
          const checked = (target as HTMLInputElement).checked;
          state.cleanAllActive = checked;
          Object.values(state.cleaningConfigs).forEach(c => (c.active = checked));
        } else if (action === 'use-original-names') {
          const checked = (target as HTMLInputElement).checked;
          state.useOriginalColumnNames = checked;
          // Update all configs to use either original or standardized names
          if (state.analysisResponse) {
            (state.analysisResponse.summary || []).forEach(s => {
              const config = state.cleaningConfigs[s.name];
              if (config) {
                config.new_name = checked ? s.name : s.standardized_name;
              }
            });
          }
        } else if (action === 'impute-all') {
          const val = target.value as ColumnCleanConfig['impute_mode'];

          // Only apply to compatible columns based on column type
          if (state.analysisResponse) {
            (state.analysisResponse.summary || []).forEach(col => {
              const config = state.cleaningConfigs[col.name];
              if (!config) return;

              // Type-aware imputation
              const isNumeric = col.kind === 'Numeric';
              const isTextOrCat = col.kind === 'Text' || col.kind === 'Categorical';
              const isBoolean = col.kind === 'Boolean';

              if (val === 'Mean' || val === 'Median' || val === 'Zero') {
                // Numeric-only operations
                if (isNumeric) config.impute_mode = val;
              } else if (val === 'Mode') {
                // Works on categorical/text/boolean
                if (isTextOrCat || isBoolean) config.impute_mode = val;
              } else {
                // None - applies to all
                config.impute_mode = val;
              }
            });
          }
        } else if (action === 'round-all') {
          const val = target.value === 'none' ? null : parseInt(target.value);
          Object.values(state.cleaningConfigs).forEach(c => (c.rounding = val));
        } else if (action === 'norm-all') {
          const val = target.value as ColumnCleanConfig['normalisation'];

          // Only apply to numeric columns (normalisation requires numeric operations)
          if (state.analysisResponse) {
            (state.analysisResponse.summary || []).forEach(col => {
              const config = state.cleaningConfigs[col.name];
              if (!config) return;

              if (col.kind === 'Numeric') {
                config.normalisation = val;
              }
            });
          }
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
          (state.analysisResponse.summary || []).forEach(s => {
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
        this.actions.navigateTo?.('Reference');
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
        const allColumns = (state.analysisResponse.summary || []).map(c => c.name);
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

      // Update configs to use standardized names for the Cleaning stage
      // (unless user has explicitly chosen to use original names)
      if (state.analysisResponse && !state.useOriginalColumnNames) {
        (state.analysisResponse.summary || []).forEach(col => {
          const config = state.cleaningConfigs[col.name];
          if (config) {
            config.new_name = col.standardized_name || col.name;
          }
        });
      }

      // Add clean transform with current configs to apply column name standardization
      // and basic text cleaning operations
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
      // eslint-disable-next-line @typescript-eslint/require-await
      void (async () => {
        if (!state.currentDataset || !state.analysisResponse) return;

        // Show publish modal to choose View or Snapshot mode
        this.showPublishModal(state);
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
      const col = (state.analysisResponse?.summary ?? []).find(s => s.name === colName);
      if (!col) return;

      const canvas = document.getElementById(`chart-${colName}`) as HTMLCanvasElement;
      if (!canvas) return;

      const ctx = canvas.getContext('2d');
      if (!ctx) return;

      if (col.stats.Numeric?.histogram) {
        const config: ChartConfiguration = {
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
        this.charts.set(colName, new Chart(ctx, config));
      } else if (col.stats.Temporal?.histogram) {
        const config: ChartConfiguration = {
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
        this.charts.set(colName, new Chart(ctx, config));
      } else if (col.stats.Categorical?.top_values) {
        const entries = col.stats.Categorical.top_values.sort((a, b) => b[1] - a[1]).slice(0, 10);
        const config: ChartConfiguration = {
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
        this.charts.set(colName, new Chart(ctx, config));
      }
    });
  }

  private bindPublishedEvents(state: AppState): void {
    // Bind tab switching
    document.querySelectorAll('.command-tab').forEach(tab => {
      tab.addEventListener('click', e => {
        const target = e.currentTarget as HTMLElement;
        const tabName = target.dataset.tab;

        // Update tab active state
        document.querySelectorAll('.command-tab').forEach(t => t.classList.remove('active'));
        target.classList.add('active');

        // Update panel active state
        document.querySelectorAll('.command-panel').forEach(p => p.classList.remove('active'));
        document.querySelector(`.command-panel[data-panel="${tabName}"]`)?.classList.add('active');
      });
    });

    // Bind copy buttons
    document.getElementById('copy-powershell')?.addEventListener('click', () => {
      const code = document.getElementById('powershell-code');
      if (code) {
        void navigator.clipboard.writeText(code.textContent ?? '');
        this.actions.showToast('PowerShell command copied!', 'success');
      }
    });

    document.getElementById('copy-python')?.addEventListener('click', () => {
      const code = document.getElementById('python-code');
      if (code) {
        void navigator.clipboard.writeText(code.textContent ?? '');
        this.actions.showToast('Python script copied!', 'success');
      }
    });

    document.getElementById('copy-json')?.addEventListener('click', () => {
      const code = document.getElementById('json-code');
      if (code) {
        void navigator.clipboard.writeText(code.textContent ?? '');
        this.actions.showToast('Pipeline JSON copied!', 'success');
      }
    });

    // Bind export button
    document.getElementById('btn-export-published')?.addEventListener('click', () => {
      void (async () => {
        if (!state.analysisResponse) return;

        const modal = new ExportModal('modal-container', this.actions, {
          type: 'Analyser',
          path: state.analysisResponse.path,
        });

        document.getElementById('modal-container')?.classList.add('active');
        await modal.show(state);
        document.getElementById('modal-container')?.classList.remove('active');
      })();
    });

    // Bind lifecycle button
    document.getElementById('btn-view-lifecycle')?.addEventListener('click', () => {
      this.actions.switchView('Lifecycle');
    });
  }

  private showPublishModal(state: AppState): void {
    if (!state.currentDataset) return;

    const modalContainer = document.getElementById('modal-container');
    if (!modalContainer) return;

    // Render the publish modal
    modalContainer.innerHTML = renderers.renderPublishModal();
    modalContainer.classList.add('active');

    // Bind close button
    document.getElementById('modal-close')?.addEventListener('click', () => {
      modalContainer.classList.remove('active');
      modalContainer.innerHTML = '';
    });

    // Bind Publish as View button
    document.getElementById('btn-publish-view')?.addEventListener('click', () => {
      void this.handlePublish(state, 'view');
    });

    // Bind Publish as Snapshot button
    document.getElementById('btn-publish-snapshot')?.addEventListener('click', () => {
      void this.handlePublish(state, 'snapshot');
    });
  }

  private async handlePublish(state: AppState, mode: 'view' | 'snapshot'): Promise<void> {
    if (!state.currentDataset) return;

    const modalContainer = document.getElementById('modal-container');
    if (!modalContainer) return;

    try {
      // Close modal
      modalContainer.classList.remove('active');
      modalContainer.innerHTML = '';

      // Show loading state
      state.isLoading = true;
      state.loadingMessage = `Publishing dataset as ${mode}...`;
      this.actions.onStateChange();

      // Create Published version
      const publishedVersionId = await api.publishVersion(
        state.currentDataset.id,
        state.currentDataset.activeVersionId,
        mode
      );

      // Set the new Published version as active
      await api.setActiveVersion(state.currentDataset.id, publishedVersionId);

      // Reload the dataset versions to include the new Published version
      const updatedVersionsJson = await api.listVersions(state.currentDataset.id);
      state.currentDataset.versions = JSON.parse(updatedVersionsJson) as DatasetVersion[];
      state.currentDataset.activeVersionId = publishedVersionId;

      state.isLoading = false;
      this.actions.onStateChange();
      this.actions.showToast(`✓ Dataset published as ${mode}!`, 'success');
    } catch (err) {
      state.isLoading = false;
      this.actions.onStateChange();
      this.actions.showToast(`Failed to publish: ${String(err)}`, 'error');
    }
  }
}
