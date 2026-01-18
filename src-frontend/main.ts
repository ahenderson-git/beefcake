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
import {
  View,
  AppState,
  AppConfig,
  getDefaultColumnCleanConfig,
  WatcherState,
  WatcherActivity,
  DatasetVersion,
} from './types';

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
    watcherState: null,
    watcherActivities: [],
  };

  private components: Partial<Record<View, Component>> = {};
  private lifecycleRail: LifecycleRailComponent | null = null;
  private aiSidebar: AIAssistantComponent | null = null;

  constructor() {
    this.init().catch(() => {
      // Error is handled inside init
    });
  }

  async init(): Promise<void> {
    this.renderInitialLayout();

    try {
      // Set a timeout for the initial data load to prevent permanent hang
      const timeoutPromise = new Promise<never>((_, reject) =>
        setTimeout(() => reject(new Error('Initialization timeout')), 5000)
      );

      const dataPromise = (async (): Promise<[AppConfig, string]> => {
        const config = await api.loadAppConfig();
        const version = await api.getAppVersion();
        return [config, version];
      })();

      const [config, version] = await Promise.race([dataPromise, timeoutPromise]);

      this.state.config = config;
      this.state.version = version;
    } catch (err) {
      // We still proceed so the UI renders, but we show a toast
      setTimeout(() => {
        this.showToast('Initialization error: ' + String(err), 'error');
      }, 1000);
    }

    try {
      this.initComponents();
      this.setupNavigation();
      void this.setupWatcherEvents();
      this.render();
    } catch (err) {
      this.showToast(`Initialization error: ${String(err)}`, 'error');
    } finally {
      // Hide loading screen after app is ready, even if there was an error
      this.hideLoadingScreen();
    }
  }

  private hideLoadingScreen(): void {
    const loadingScreen = document.getElementById('loading-screen');
    if (loadingScreen) {
      loadingScreen.classList.add('hidden');
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
    } catch (err) {
      this.showToast(`Render error: ${String(err)}`, 'error');
    }
  }

  public async handleAnalysis(path: string): Promise<void> {
    try {
      this.state.isLoading = true;
      this.state.isAborting = false;
      this.state.loadingMessage = `Analyzing...`;
      await this.switchView('Analyser');

      this.showToast(`Analyzing ${path}...`, 'info');
      const response = await api.analyseFile(path);
      this.state.analysisResponse = response;

      // Initialize cleaning configs
      this.state.cleaningConfigs = {};
      response.summary.forEach(col => {
        this.state.cleaningConfigs[col.name] = getDefaultColumnCleanConfig(col);
      });

      // Immediately show analysis results without waiting for lifecycle
      this.state.isLoading = false;
      this.render();
      this.showToast('Analysis complete', 'success');

      // Create lifecycle dataset asynchronously in background
      // This avoids blocking the UI for large file operations
      void this.createLifecycleDatasetAsync(response.file_name, path);
    } catch (err) {
      this.state.isLoading = false;
      this.render();
      this.showToast(`Analysis failed: ${String(err)}`, 'error');
    }
  }

  private async createLifecycleDatasetAsync(fileName: string, path: string): Promise<void> {
    try {
      const datasetId = await api.createDataset(fileName, path);

      const versionsJson = await api.listVersions(datasetId);

      const versions = JSON.parse(versionsJson) as DatasetVersion[];

      if (versions.length > 0 && versions[0]) {
        this.state.currentDataset = {
          id: datasetId,
          name: fileName,
          versions: versions,
          activeVersionId: versions[0].id, // Raw version
          rawVersionId: versions[0].id,
        };
      }

      // Re-render to show lifecycle rail with Raw stage
      this.render();

      // Automatically create Profiled version since we already ran analysis
      // Profiled stage just captures analysis metadata without transforming data
      try {
        const emptyPipeline = { transforms: [] };
        const profiledVersionId = await api.applyTransforms(
          datasetId,
          JSON.stringify(emptyPipeline),
          'Profiled'
        );

        // Refresh versions list
        const updatedVersionsJson = await api.listVersions(datasetId);
        const updatedVersions = JSON.parse(updatedVersionsJson) as DatasetVersion[];

        // Update state with new versions
        if (this.state.currentDataset) {
          this.state.currentDataset.versions = updatedVersions;
          this.state.currentDataset.activeVersionId = profiledVersionId;
        }

        // Re-render to show Profiled stage completed
        this.render();
      } catch (profileErr) {
        // Not critical - user can still use Raw version
      }
    } catch (lifecycleErr) {
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

    const duration = type === 'error' ? 10000 : 3000;
    setTimeout(() => {
      toastEl.classList.add('fade-out');
      setTimeout(() => toastEl.remove(), 500);
    }, duration);
  }

  private async setupWatcherEvents(): Promise<void> {
    try {
      const { listen } = await import('@tauri-apps/api/event');

      // Listen for watcher status updates
      void listen<WatcherState>('watcher:status', event => {
        this.state.watcherState = event.payload;
        this.render();
      });

      // Listen for file detected
      void listen<WatcherActivity>('watcher:file_detected', event => {
        const watcherComp = this.components['Watcher'] as unknown as WatcherComponent;
        if (watcherComp) {
          watcherComp.handleWatcherEvent(this.state, 'watcher:file_detected', event.payload);
        }
      });

      // Listen for file ready
      void listen<WatcherActivity>('watcher:file_ready', event => {
        const watcherComp = this.components['Watcher'] as unknown as WatcherComponent;
        if (watcherComp) {
          watcherComp.handleWatcherEvent(this.state, 'watcher:file_ready', event.payload);
        }
      });

      // Listen for ingest started
      void listen<WatcherActivity>('watcher:ingest_started', event => {
        const watcherComp = this.components['Watcher'] as unknown as WatcherComponent;
        if (watcherComp) {
          watcherComp.handleWatcherEvent(this.state, 'watcher:ingest_started', event.payload);
        }
      });

      // Listen for ingest succeeded
      void listen<WatcherActivity>('watcher:ingest_succeeded', event => {
        const watcherComp = this.components['Watcher'] as unknown as WatcherComponent;
        if (watcherComp) {
          watcherComp.handleWatcherEvent(this.state, 'watcher:ingest_succeeded', event.payload);
        }
      });

      // Listen for ingest failed
      void listen<WatcherActivity>('watcher:ingest_failed', event => {
        const watcherComp = this.components['Watcher'] as unknown as WatcherComponent;
        if (watcherComp) {
          watcherComp.handleWatcherEvent(this.state, 'watcher:ingest_failed', event.payload);
        }
      });

      // Load initial watcher state
      try {
        this.state.watcherState = await api.watcherGetState();
      } catch (err) {
        this.showToast(`Failed to load watcher state: ${String(err)}`, 'error');
      }
    } catch (err) {
      this.showToast(`Failed to setup watcher events: ${String(err)}`, 'error');
    }
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

window.addEventListener('DOMContentLoaded', () => {
  void new BeefcakeApp();
});
