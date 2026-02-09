/**
 * # Beefcake Frontend Application
 *
 * Main entry point for the Beefcake desktop application frontend.
 * This file orchestrates the component-based architecture and manages
 * application state.
 *
 * ## Architecture Overview
 *
 * ```
 * BeefcakeApp (this file)
 *   │
 *   ├─ state: AppState              (centralized state)
 *   ├─ components: Component[]      (UI components)
 *   └─ lifecycleRail: Component     (dataset lifecycle UI)
 * ```
 *
 * ## Key Patterns
 *
 * ### 1. Component-Based Architecture
 * Each view (Dashboard, Analyser, etc.) is a separate component class
 * that handles its own rendering and event handling.
 *
 * ### 2. Centralized State Management
 * All state lives in `BeefcakeApp.state`. When state changes,
 * `render()` is called to update the UI.
 *
 * ### 3. Event-Driven Updates
 * User actions → Update state → Call render() → Update DOM
 *
 * ### 4. Tauri Bridge
 * All backend communication goes through `api.ts`, which uses
 * Tauri's `invoke()` to call Rust functions.
 *
 * ## TypeScript Concepts Used
 *
 * - **Classes**: Object-oriented state management
 * - **Async/Await**: Handling asynchronous operations
 * - **Interfaces**: Type-safe state and configuration
 * - **Generics**: `Partial<Record<View, Component>>`
 * - **Optional Chaining**: `component?.render()`
 *
 * @module main
 * @see TypeScript Patterns: ../docs/TYPESCRIPT_PATTERNS.md
 * @see Architecture Documentation: ../docs/ARCHITECTURE.md
 */

import '@phosphor-icons/web/regular';
import '@fontsource/fira-code/300.css';
import '@fontsource/fira-code/400.css';
import '@fontsource/fira-code/500.css';

import * as api from './api';
import { ActivityLogComponent } from './components/ActivityLogComponent';
import { AIAssistantComponent } from './components/AIAssistantComponent';
import { AnalyserComponent } from './components/AnalyserComponent';
import { CliHelpComponent } from './components/CliHelpComponent';
import { Component } from './components/Component';
import { DashboardComponent } from './components/DashboardComponent';
import { DictionaryComponent } from './components/DictionaryComponent';
import { IntegrityComponent } from './components/IntegrityComponent';
import { LifecycleComponent } from './components/LifecycleComponent';
import { LifecycleRailComponent } from './components/LifecycleRailComponent';
import { PipelineComponent } from './components/PipelineComponent';
import { PowerShellComponent } from './components/PowerShellComponent';
import { PythonComponent } from './components/PythonComponent';
import { ReferenceComponent } from './components/ReferenceComponent';
import { SettingsComponent } from './components/SettingsComponent';
import { SQLComponent } from './components/SQLComponent';
import { WatcherComponent } from './components/WatcherComponent';
import * as renderers from './renderers';
import { WatcherService } from './services/WatcherService';
import { WizardService } from './services/WizardService';
import {
  View,
  AppState,
  AppConfig,
  getDefaultColumnCleanConfig,
  getDefaultAppConfig,
  DatasetVersion,
} from './types';
import { createLogger } from './utils/logger';

/**
 * Main application controller for Beefcake frontend.
 *
 * Manages application state, component lifecycle, and coordinates
 * communication between UI components and Rust backend via Tauri.
 *
 * ## Lifecycle
 *
 * 1. **Constructor**: Starts async initialization
 * 2. **init()**: Loads config, creates components, sets up navigation
 * 3. **render()**: Updates active component with current state
 * 4. **User Interaction**: Triggers state changes and re-renders
 *
 * ## State Management Pattern
 *
 * ```typescript
 * // State is private - components receive it as read-only
 * private state: AppState = { ... };
 *
 * // State updates trigger re-renders
 * this.state.currentView = 'Analyser';
 * this.render();  // Updates DOM
 * ```
 *
 * @example
 * ```typescript
 * // App initializes automatically on DOM load
 * window.addEventListener('DOMContentLoaded', () => {
 *   new BeefcakeApp();
 * });
 * ```
 */
class BeefcakeApp {
  private logger = createLogger('Lifecycle');

  /**
   * Centralized application state.
   *
   * **Pattern**: Single source of truth for all UI state.
   * When state changes, call `render()` to update the display.
   *
   * **TypeScript Note**: Using `private` prevents external access.
   * Components receive state as a parameter in `render(state)`.
   */
  private state: AppState = {
    version: '0.0.0',
    currentView: 'Dashboard',
    analysisResponse: null,
    expandedRows: new Set(),
    cleaningConfigs: {},
    isAddingConnection: false,
    isLoading: false,
    isAborting: false,
    isCreatingLifecycle: false,
    loadingMessage: '',
    config: null,
    pythonScript: null,
    sqlScript: null,
    pythonSkipCleaning: true,
    sqlSkipCleaning: true,
    currentDataset: null,
    selectedColumns: new Set(),
    useOriginalColumnNames: false,
    cleanAllActive: true,
    advancedProcessingEnabled: false,
    watcherState: null,
    watcherActivities: [],
    selectedVersionId: null,
    currentIdeColumns: null,
    previousVersionId: null,
  };

  private components: Partial<Record<View, Component>> = {};
  private lifecycleRail: LifecycleRailComponent | null = null;
  private aiSidebar: AIAssistantComponent | null = null;
  private watcherService: WatcherService | null = null;
  private wizardService: WizardService | null = null;

  constructor() {
    /* eslint-disable no-console */
    console.log('[BeefcakeApp] Constructor called');
    this.init().catch(err => {
      console.error('[BeefcakeApp] Fatal initialization error:', err);
      // Error is handled inside init, but log it for debugging
    });
  }

  async init(): Promise<void> {
    console.log('[BeefcakeApp] init() started');
    this.renderInitialLayout();
    console.log('[BeefcakeApp] Initial layout rendered');

    // Default values if API calls fail
    this.state.config = getDefaultAppConfig();
    this.state.version = '0.0.0';

    try {
      // Set a short timeout for the initial data load to prevent permanent hang
      const timeoutPromise = new Promise<never>((_, reject) =>
        setTimeout(() => reject(new Error('Initialization timeout')), 2000)
      );

      const dataPromise = (async (): Promise<[AppConfig, string]> => {
        console.log('[BeefcakeApp] Loading config and version...');
        const config = await api.loadAppConfig();
        console.log('[BeefcakeApp] Config loaded:', config);
        const version = await api.getAppVersion();
        console.log('[BeefcakeApp] Version loaded:', version);
        return [config, version];
      })();

      const [config, version] = await Promise.race([dataPromise, timeoutPromise]);

      this.state.config = config || getDefaultAppConfig();
      this.state.version = version || 'unknown';
      console.log('[BeefcakeApp] Config and version set in state');
    } catch (err) {
      console.warn('[BeefcakeApp] API initialization warning (continuing with defaults):', err);
      // We still proceed so the UI renders
      if (err instanceof Error && err.message !== 'Initialization timeout') {
        setTimeout(() => {
          this.showToast('Initialization warning: ' + String(err), 'info');
        }, 1000);
      }
    }

    try {
      this.initComponents();
      console.log('[BeefcakeApp] Setting up navigation...');
      this.setupNavigation();
      console.log('[BeefcakeApp] Navigation setup complete');

      console.log('[BeefcakeApp] Creating WatcherService...');
      this.watcherService = new WatcherService(
        this.state,
        this.components,
        () => this.render(),
        (msg, type) => this.showToast(msg, type)
      );
      console.log('[BeefcakeApp] Initializing WatcherService...');
      void this.watcherService.init();

      console.log('[BeefcakeApp] Creating WizardService...');
      this.wizardService = new WizardService(
        this.state,
        () => this.render(),
        (msg, type) => this.showToast(msg, type)
      );
      console.log('[BeefcakeApp] Starting Polars version check...');
      void this.checkPolarsVersion(); // Non-blocking Polars version check

      console.log('[BeefcakeApp] First render...');
      this.render();
      console.log('[BeefcakeApp] Render complete');

      // Check for first run wizard asynchronously
      void (async () => {
        try {
          console.log('[BeefcakeApp] Checking for first run wizard...');
          await this.wizardService?.maybeShowFirstRunWizard();
        } catch (err) {
          console.warn('[BeefcakeApp] Wizard check failed:', err);
        }
      })();

      console.log('[BeefcakeApp] Initialization complete!');
      /* eslint-enable no-console */
    } catch (err) {
      console.error('[BeefcakeApp] Initialization error:', err);
      this.showToast(`Initialization error: ${String(err)}`, 'error');
    } finally {
      // Hide loading screen after app is ready, even if there was an error
      this.hideLoadingScreen();
    }
  }

  private hideLoadingScreen(): void {
    const loadingScreen = document.getElementById('loading-screen');
    if (loadingScreen) {
      loadingScreen.classList.add('loading-screen-fade-out');
      // Remove from DOM after transition completes
      setTimeout(() => {
        loadingScreen.remove();
      }, 300);
    }
  }

  private renderInitialLayout(): void {
    const app = document.getElementById('app');
    if (app) {
      app.innerHTML = renderers.renderLayout();
    }
  }

  private async checkPolarsVersion(): Promise<void> {
    try {
      const result = await api.checkPythonEnvironment();
      // Parse version from response like "Polars 1.18.0 installed"
      const match = result.match(/Polars\s+(\d+\.\d+\.\d+)\s+installed/);
      if (match?.[1]) {
        this.state.polarsVersion = match[1];
        this.render(); // Update UI to show version
      }
    } catch (err) {
      // Silently fail - version will remain "unknown"
      /* eslint-disable-next-line no-console */
      console.debug('Failed to detect Polars version:', err);
    }
  }

  private async showWizardOnDemand(): Promise<void> {
    if (this.wizardService) {
      await this.wizardService.showWizardOnDemand();
    }
  }

  private initComponents(): void {
    const actions = {
      onStateChange: () => this.render(),
      showToast: (msg: string, type?: 'info' | 'error' | 'success') => this.showToast(msg, type),
      runAnalysis: (path: string) => {
        void this.handleAnalysis(path);
      },
      switchView: (view: View) => {
        void this.switchView(view);
      },
      navigateTo: async (view: string, datasetId?: string) => {
        if (view === 'analyser' && datasetId) {
          // Load dataset and switch to analyser
          await this.loadDatasetById(datasetId);
        } else {
          await this.switchView(view as View);
        }
      },
      showFirstRunWizard: () => {
        void this.showWizardOnDemand();
      },
    };

    this.components = {
      Dashboard: new DashboardComponent('view-container', actions),
      Analyser: new AnalyserComponent('view-container', actions),
      PowerShell: new PowerShellComponent('view-container', actions),
      Python: new PythonComponent('view-container', actions),
      SQL: new SQLComponent('view-container', actions),
      Settings: new SettingsComponent('view-container', actions),
      CLI: new CliHelpComponent('view-container', actions),
      ActivityLog: new ActivityLogComponent('view-container', actions),
      Reference: new ReferenceComponent('view-container', actions),
      Lifecycle: new LifecycleComponent('view-container', actions),
      Pipeline: new PipelineComponent('view-container', actions),
      Watcher: new WatcherComponent('view-container', actions),
      Dictionary: new DictionaryComponent('view-container', actions),
      Integrity: new IntegrityComponent('view-container', actions),
    };

    // Initialize lifecycle rail component
    this.lifecycleRail = new LifecycleRailComponent('lifecycle-rail-container', actions);

    // Initialize AI sidebar as persistent component
    this.aiSidebar = new AIAssistantComponent('ai-sidebar-container', actions);
    this.aiSidebar.render(this.state);

    // Setup AI sidebar toggle
    this.setupAISidebarToggle();
  }

  private setupNavigation(): void {
    document.querySelectorAll('.nav-item').forEach(item => {
      item.addEventListener('click', e => {
        const view = (e.currentTarget as HTMLElement).dataset.view as View;
        void this.switchView(view);
      });
    });
  }

  private async switchView(view: View): Promise<void> {
    this.state.currentView = view;
    this.state.isAddingConnection = false;

    document.querySelectorAll('.nav-item').forEach(item => {
      item.classList.toggle('active', (item as HTMLElement).dataset.view === view);
    });

    const title = document.getElementById('view-title');
    if (title) {
      if (view === 'CLI') title.innerText = 'CLI Help';
      else if (view === 'ActivityLog') title.innerText = 'Activity Log';
      else if (view === 'Python') title.innerText = 'Python IDE';
      else if (view === 'SQL') title.innerText = 'SQL IDE';
      else if (view === 'Reference') title.innerText = 'Reference Material';
      else if (view === 'Lifecycle') title.innerText = 'Dataset Lifecycle';
      else if (view === 'Dictionary') title.innerText = 'Data Dictionary';
      else title.innerText = view;
    }

    // Load snapshots when switching to Dictionary view
    if (view === 'Dictionary') {
      const dictionaryComponent = this.components['Dictionary'] as DictionaryComponent;
      if (dictionaryComponent) {
        await dictionaryComponent.loadSnapshots();
      }
    }

    this.render();
  }

  private render(): void {
    try {
      const component = this.components[this.state.currentView];
      if (component) {
        component.render(this.state);
      }

      // Always render lifecycle rail if dataset is loaded and we're in analyser view
      if (this.lifecycleRail && this.state.currentView === 'Analyser') {
        this.lifecycleRail.render(this.state);
      }

      // Always update AI sidebar with current state
      if (this.aiSidebar) {
        this.aiSidebar.updateContext(this.state);
      }

      // Setup IDE sidebar toggle when in Python or SQL view
      if (this.state.currentView === 'Python' || this.state.currentView === 'SQL') {
        // Use setTimeout to ensure DOM is ready
        setTimeout(() => {
          this.setupIDESidebarToggle();
        }, 0);
      }
    } catch (err) {
      this.showToast(`Render error: ${String(err)}`, 'error');
    }
  }

  public async handleAnalysis(path: string): Promise<void> {
    try {
      this.state.isLoading = true;
      this.state.isAborting = false;
      this.state.loadingMessage = `Analysing...`;
      await this.switchView('Analyser');

      this.showToast(`Analysing ${path}...`, 'info');
      const response = await api.analyseFile(path);
      this.state.analysisResponse = response;

      // Initialize cleaning configs
      this.state.cleaningConfigs = {};
      (response.summary || []).forEach(col => {
        this.state.cleaningConfigs[col.name] = getDefaultColumnCleanConfig(col);
      });

      // Show analysis results and begin creating lifecycle dataset
      this.state.isLoading = false;
      this.state.isCreatingLifecycle = true;
      this.render();
      this.showToast('Analysis complete', 'success');

      // Create lifecycle dataset (now awaited to track progress)
      await this.createLifecycleDatasetAsync(response.file_name, path);
      this.state.isCreatingLifecycle = false;
      // render() is already called inside createLifecycleDatasetAsync
    } catch (err) {
      this.state.isLoading = false;
      this.state.isCreatingLifecycle = false;
      this.render();
      this.showToast(`Analysis failed: ${String(err)}`, 'error');
    }
  }

  private async createLifecycleDatasetAsync(fileName: string, path: string): Promise<void> {
    try {
      this.logger.info('Creating dataset:', { fileName, path });
      const datasetId = await api.createDataset(fileName, path);
      this.logger.info('Dataset created with ID:', datasetId);

      this.logger.info('Listing versions for dataset:', datasetId);
      const versionsJson = await api.listVersions(datasetId);
      this.logger.debug('Versions JSON:', versionsJson);

      const versions = JSON.parse(versionsJson) as DatasetVersion[];
      this.logger.info('Parsed versions:', versions);

      if (versions.length > 0 && versions[0]) {
        this.state.currentDataset = {
          id: datasetId,
          name: fileName,
          versions: versions,
          activeVersionId: versions[0].id, // Raw version
          rawVersionId: versions[0].id,
        };
        this.logger.info('currentDataset set:', this.state.currentDataset);
      } else {
        this.logger.error('No versions found in dataset');
      }

      // Re-render to show lifecycle rail with Raw stage
      this.render();

      // Automatically create Profiled version since we already ran analysis
      // Profiled stage just captures analysis metadata without transforming data
      try {
        this.logger.info('Creating Profiled version...');
        const emptyPipeline = { transforms: [] };
        const profiledVersionId = await api.applyTransforms(
          datasetId,
          JSON.stringify(emptyPipeline),
          'Profiled'
        );
        this.logger.info('Profiled version created:', profiledVersionId);

        // Refresh versions list
        const updatedVersionsJson = await api.listVersions(datasetId);
        const updatedVersions = JSON.parse(updatedVersionsJson) as DatasetVersion[];
        this.logger.info('Updated versions:', updatedVersions);

        // Update state with new versions
        if (this.state.currentDataset) {
          this.state.currentDataset.versions = updatedVersions;
          this.state.currentDataset.activeVersionId = profiledVersionId;
        }

        // Re-render to show Profiled stage completed
        this.render();
      } catch (profileErr) {
        console.error('[Lifecycle] Profiled version creation failed:', profileErr);
        this.showToast('Warning: Could not create Profiled stage. Using Raw stage.', 'error');
      }
    } catch (lifecycleErr) {
      console.error('[Lifecycle] Dataset creation FAILED:', lifecycleErr);
      this.showToast(`Lifecycle creation failed: ${String(lifecycleErr)}`, 'error');
      // Analysis still succeeds, just no lifecycle tracking
    }
  }

  private showToast(message: string, type: 'info' | 'error' | 'success' = 'info'): void {
    const container = document.getElementById('toast-container');
    if (!container) return;

    const toast = document.createElement('div');
    toast.innerHTML = renderers.renderToast(message, type);
    const toastEl = toast.firstElementChild!;
    container.appendChild(toastEl);

    this.recordToastEvent(message, type);

    const duration = type === 'error' ? 10000 : 3000;
    setTimeout(() => {
      toastEl.classList.add('fade-out');
      setTimeout(() => toastEl.remove(), 500);
    }, duration);
  }

  private recordToastEvent(message: string, type: 'info' | 'error' | 'success'): void {
    const view = this.state.currentView;
    const action = 'Toast';
    const details = `[${type}] ${message} (${view})`;

    if (this.state.config?.audit_log?.entries) {
      this.state.config.audit_log.entries.push({
        timestamp: new Date().toISOString(),
        action,
        details,
      });

      if (this.state.config.audit_log.entries.length > 1000) {
        const overflow = this.state.config.audit_log.entries.length - 1000;
        this.state.config.audit_log.entries.splice(0, overflow);
      }
    }

    const level = type === 'error' ? 'error' : 'info';
    void api.logFrontendEvent(level, action, details, { view, type }).catch(() => {});
  }

  private setupAISidebarToggle(): void {
    const aiSidebar = document.getElementById('ai-sidebar');
    if (!aiSidebar) return;

    // Load saved collapsed state
    const isCollapsed = localStorage.getItem('ai-sidebar-collapsed') === 'true';
    if (!isCollapsed) {
      aiSidebar.classList.remove('collapsed');
    }

    const toggleSidebar = (): void => {
      aiSidebar.classList.toggle('collapsed');
      const collapsed = aiSidebar.classList.contains('collapsed');
      localStorage.setItem('ai-sidebar-collapsed', collapsed.toString());
    };

    // Handle collapse button click (created dynamically by AIAssistantComponent)
    // Use event delegation since the button is created after this runs
    aiSidebar.addEventListener('click', e => {
      const target = e.target as HTMLElement;
      if (target.closest('#ai-collapse-btn') ?? target.closest('#ai-collapsed-tab')) {
        toggleSidebar();
      }
    });

    // Handle double-click on header to toggle
    aiSidebar.addEventListener('dblclick', e => {
      const target = e.target as HTMLElement;
      if (target.closest('#ai-sidebar-header')) {
        toggleSidebar();
      }
    });
  }

  private setupIDESidebarToggle(): void {
    const ideSidebar = document.getElementById('ide-sidebar');
    if (!ideSidebar) return;

    // Load saved collapsed state
    const isCollapsed = localStorage.getItem('ide-sidebar-collapsed') === 'true';
    if (isCollapsed) {
      ideSidebar.classList.add('collapsed');
    } else {
      ideSidebar.classList.remove('collapsed');
    }

    const toggleSidebar = (): void => {
      ideSidebar.classList.toggle('collapsed');
      const collapsed = ideSidebar.classList.contains('collapsed');
      localStorage.setItem('ide-sidebar-collapsed', collapsed.toString());
    };

    // Handle collapse button click (created dynamically by IDE renderers)
    // Use event delegation since the button is created after this runs
    ideSidebar.addEventListener('click', e => {
      const target = e.target as HTMLElement;
      if (target.closest('#ide-collapse-btn') ?? target.closest('#ide-collapsed-tab')) {
        toggleSidebar();
      }
    });

    // Handle double-click on header to toggle
    ideSidebar.addEventListener('dblclick', e => {
      const target = e.target as HTMLElement;
      if (target.closest('#ide-sidebar-header')) {
        toggleSidebar();
      }
    });
  }

  private async loadDatasetById(_datasetId: string): Promise<void> {
    try {
      // This would need to be implemented to load a dataset by ID
      // For now, just switch to analyser view
      await this.switchView('Analyser');
      this.showToast('Dataset loaded', 'success');
    } catch (err) {
      this.showToast(`Failed to load dataset: ${String(err)}`, 'error');
    }
  }
}

/* eslint-disable no-console */
console.log('[Beefcake] Script loaded, waiting for DOMContentLoaded...');

// Set up global error handlers to catch all errors and log to file
window.addEventListener('error', event => {
  const errorMessage = `Uncaught error: ${event.message}`;
  const errorObj = event.error as Error | undefined;
  const context = {
    filename: event.filename,
    lineno: event.lineno,
    colno: event.colno,
    error: errorObj?.stack ?? String(event.error),
  };
  console.error(errorMessage, context);
  void api.logFrontendError('error', errorMessage, context);
});

// Capture resource loading failures (e.g. CSS, scripts, images)
window.addEventListener(
  'error',
  event => {
    const target = event.target;
    if (!target || target === window) return;

    const el = target as HTMLElement;
    const tag = (el.tagName || '').toLowerCase();
    if (!['link', 'style', 'script', 'img'].includes(tag)) return;

    const url =
      (el as HTMLLinkElement).href ||
      (el as HTMLScriptElement).src ||
      (el as HTMLImageElement).src ||
      undefined;

    const details = `Failed to load ${tag}: ${url ?? 'unknown'}`;
    void api.logFrontendEvent('error', 'Resource', details, { tag, url }).catch(() => {});
  },
  true
);

window.addEventListener('unhandledrejection', event => {
  const errorMessage = `Unhandled promise rejection: ${String(event.reason)}`;
  const reasonObj = event.reason as Error | undefined;
  const context = {
    reason: reasonObj?.stack ?? String(event.reason),
    promise: String(event.promise),
  };
  console.error(errorMessage, context);
  void api.logFrontendError('error', errorMessage, context);
});

// Override console.error to also log to file
const originalConsoleError = console.error;
console.error = (...args: unknown[]) => {
  originalConsoleError.apply(console, args);
  const message = args.map(arg => String(arg)).join(' ');
  void api.logFrontendError('error', message, { args });
};

// Override console.warn to also log to file
const originalConsoleWarn = console.warn;
console.warn = (...args: unknown[]) => {
  originalConsoleWarn.apply(console, args);
  const message = args.map(arg => String(arg)).join(' ');
  void api.logFrontendError('warn', message, { args });
};

// Override console.log to also log to file
const originalConsoleLog = console.log;
console.log = (...args: unknown[]) => {
  originalConsoleLog.apply(console, args);
  const message = args.map(arg => String(arg)).join(' ');
  void api.logFrontendEvent('info', 'Console', message, { args }).catch(() => {});
};

// Override console.info to also log to file
const originalConsoleInfo = console.info;
console.info = (...args: unknown[]) => {
  originalConsoleInfo.apply(console, args);
  const message = args.map(arg => String(arg)).join(' ');
  void api.logFrontendEvent('info', 'Console', message, { args }).catch(() => {});
};

window.addEventListener('DOMContentLoaded', () => {
  console.log('[Beefcake] DOMContentLoaded fired, creating BeefcakeApp...');
  try {
    void new BeefcakeApp();
  } catch (err) {
    console.error('[Beefcake] Failed to create BeefcakeApp:', err);
    void api.logFrontendError('error', 'Failed to create BeefcakeApp', {
      error: err instanceof Error ? err.stack : String(err),
    });
  }
});

console.log('[Beefcake] Event listener registered');
