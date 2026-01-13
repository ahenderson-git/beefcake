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

import "@phosphor-icons/web/regular";
import "@fontsource/fira-code/300.css";
import "@fontsource/fira-code/400.css";
import "@fontsource/fira-code/500.css";

import {
  View,
  AppState,
  AppConfig,
  getDefaultColumnCleanConfig
} from './types';

import * as api from './api';
import * as renderers from './renderers';

import { Component } from './components/Component';
import { DashboardComponent } from './components/DashboardComponent';
import { AnalyserComponent } from './components/AnalyserComponent';
import { PowerShellComponent } from './components/PowerShellComponent';
import { PythonComponent } from './components/PythonComponent';
import { SQLComponent } from './components/SQLComponent';
import { SettingsComponent } from './components/SettingsComponent';
import { CliHelpComponent } from './components/CliHelpComponent';
import { ActivityLogComponent } from './components/ActivityLogComponent';
import { ReferenceComponent } from './components/ReferenceComponent';
import { LifecycleComponent } from './components/LifecycleComponent';
import { LifecycleRailComponent } from './components/LifecycleRailComponent';
import { PipelineComponent } from './components/PipelineComponent';

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
    cleanAllActive: true
  };

  private components: Partial<Record<View, Component>> = {};
  private lifecycleRail: LifecycleRailComponent | null = null;

  constructor() {
    this.init().catch(err => {
      console.error('Failed to initialize BeefcakeApp:', err);
    });
  }

  async init() {
    console.log('BeefcakeApp: Initializing...');
    this.renderInitialLayout();

    try {
      console.log('BeefcakeApp: Loading initial data...');
      // Set a timeout for the initial data load to prevent permanent hang
      const timeoutPromise = new Promise((_, reject) =>
        setTimeout(() => reject(new Error('Initialization timeout')), 5000)
      );

      const dataPromise = (async () => {
        const config = await api.loadAppConfig();
        console.log('BeefcakeApp: Config loaded');
        const version = await api.getAppVersion();
        console.log('BeefcakeApp: Version loaded');
        return [config, version];
      })();

      const [config, version] = await Promise.race([dataPromise, timeoutPromise]) as [AppConfig, string];

      this.state.config = config;
      this.state.version = version;
      console.log('BeefcakeApp: Data loaded successfully');
    } catch (err) {
      console.error('BeefcakeApp: Failed to load initial app data:', err);
      // We still proceed so the UI renders, but we show a toast
      setTimeout(() => {
        this.showToast('Initialization error: ' + err, 'error');
      }, 1000);
    }

    try {
      console.log('BeefcakeApp: Initializing components...');
      this.initComponents();
      console.log('BeefcakeApp: Setting up navigation...');
      this.setupNavigation();
      console.log('BeefcakeApp: Rendering...');
      this.render();
      console.log('BeefcakeApp: Initialization complete');
    } catch (err) {
      console.error('BeefcakeApp: Error during component initialization:', err);
    } finally {
      // Hide loading screen after app is ready, even if there was an error
      console.log('BeefcakeApp: Hiding loading screen');
      this.hideLoadingScreen();
    }
  }

  private hideLoadingScreen() {
    const loadingScreen = document.getElementById('loading-screen');
    if (loadingScreen) {
      loadingScreen.classList.add('hidden');
      // Remove from DOM after transition completes
      setTimeout(() => {
        loadingScreen.remove();
      }, 300);
    }
  }

  private renderInitialLayout() {
    const app = document.getElementById('app');
    if (app) {
      app.innerHTML = renderers.renderLayout();
    }
  }

  private initComponents() {
    const actions = {
      onStateChange: () => this.render(),
      showToast: (msg: string, type?: any) => this.showToast(msg, type),
      runAnalysis: (path: string) => this.handleAnalysis(path),
      switchView: (view: View) => this.switchView(view)
    };

    this.components = {
      'Dashboard': new DashboardComponent('view-container', actions),
      'Analyser': new AnalyserComponent('view-container', actions),
      'PowerShell': new PowerShellComponent('view-container', actions),
      'Python': new PythonComponent('view-container', actions),
      'SQL': new SQLComponent('view-container', actions),
      'Settings': new SettingsComponent('view-container', actions),
      'CLI': new CliHelpComponent('view-container', actions),
      'ActivityLog': new ActivityLogComponent('view-container', actions),
      'Reference': new ReferenceComponent('view-container', actions),
      'Lifecycle': new LifecycleComponent('view-container', actions),
      'Pipeline': new PipelineComponent('view-container', actions)
    };

    // Initialize lifecycle rail component
    this.lifecycleRail = new LifecycleRailComponent('lifecycle-rail-container', actions);
  }

  private setupNavigation() {
    document.querySelectorAll('.nav-item').forEach(item => {
      item.addEventListener('click', (e) => {
        const view = (e.currentTarget as HTMLElement).dataset.view as View;
        this.switchView(view);
      });
    });
  }

  private switchView(view: View) {
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
      else title.innerText = view;
    }

    this.render();
  }

  private render() {
    try {
      const component = this.components[this.state.currentView];
      if (component) {
        component.render(this.state);
      }

      // Always render lifecycle rail if dataset is loaded and we're in analyser view
      if (this.lifecycleRail && this.state.currentView === 'Analyser') {
        console.log('Rendering lifecycle rail. currentDataset:', this.state.currentDataset);
        this.lifecycleRail.render(this.state);
      }
    } catch (err) {
      console.error('BeefcakeApp: Error during render:', err);
    }
  }

  public async handleAnalysis(path: string) {
    try {
      this.state.isLoading = true;
      this.state.isAborting = false;
      this.state.loadingMessage = `Analyzing ${path}...`;
      this.switchView('Analyser');

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
      this.createLifecycleDatasetAsync(response.file_name, path);
    } catch (err) {
      this.state.isLoading = false;
      this.render();
      this.showToast(`Analysis failed: ${err}`, 'error');
    }
  }

  private async createLifecycleDatasetAsync(fileName: string, path: string) {
    try {
      console.log('Creating lifecycle dataset for:', fileName);
      const datasetId = await api.createDataset(fileName, path);
      console.log('Dataset created with ID:', datasetId);

      let versionsJson = await api.listVersions(datasetId);
      console.log('Versions JSON:', versionsJson);

      let versions = JSON.parse(versionsJson);
      console.log('Parsed versions:', versions);

      this.state.currentDataset = {
        id: datasetId,
        name: fileName,
        versions: versions,
        activeVersionId: versions[0].id, // Raw version
        rawVersionId: versions[0].id
      };

      console.log('Lifecycle dataset created successfully:', this.state.currentDataset);

      // Re-render to show lifecycle rail with Raw stage
      this.render();

      // Automatically create Profiled version since we already ran analysis
      // Profiled stage just captures analysis metadata without transforming data
      try {
        console.log('Creating Profiled version...');
        const emptyPipeline = { transforms: [] };
        const profiledVersionId = await api.applyTransforms(
          datasetId,
          JSON.stringify(emptyPipeline),
          'Profiled'
        );
        console.log('Profiled version created:', profiledVersionId);

        // Refresh versions list
        versionsJson = await api.listVersions(datasetId);
        versions = JSON.parse(versionsJson);

        // Update state with new versions
        this.state.currentDataset.versions = versions;
        this.state.currentDataset.activeVersionId = profiledVersionId;

        // Re-render to show Profiled stage completed
        this.render();
      } catch (profileErr) {
        console.error('Failed to create Profiled version:', profileErr);
        // Not critical - user can still use Raw version
      }
    } catch (lifecycleErr) {
      console.error('Failed to create lifecycle dataset:', lifecycleErr);
      this.showToast(`Lifecycle creation failed: ${lifecycleErr}`, 'error');
      // Analysis still succeeds, just no lifecycle tracking
    }
  }

  private showToast(message: string, type: 'info' | 'error' | 'success' = 'info') {
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
}

window.addEventListener('DOMContentLoaded', () => {
  new BeefcakeApp();
});
