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

class BeefcakeApp {
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
    trimPct: 0.1,
    config: null,
    pythonScript: null,
    sqlScript: null,
    pythonSkipCleaning: true,
    sqlSkipCleaning: true,
    currentDataset: null
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
      'Lifecycle': new LifecycleComponent('view-container', actions)
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
      const response = await api.analyseFile(path, this.state.trimPct);
      this.state.analysisResponse = response;
      
      // Initialize cleaning configs
      this.state.cleaningConfigs = {};
      response.summary.forEach(col => {
        this.state.cleaningConfigs[col.name] = getDefaultColumnCleanConfig(col);
      });

      // Create lifecycle dataset from analysis
      try {
        console.log('Creating lifecycle dataset for:', response.file_name);
        const datasetId = await api.createDataset(response.file_name, path);
        console.log('Dataset created with ID:', datasetId);

        const versionsJson = await api.listVersions(datasetId);
        console.log('Versions JSON:', versionsJson);

        const versions = JSON.parse(versionsJson);
        console.log('Parsed versions:', versions);

        this.state.currentDataset = {
          id: datasetId,
          name: response.file_name,
          versions: versions,
          activeVersionId: versions[0].id, // Raw version
          rawVersionId: versions[0].id
        };

        console.log('Lifecycle dataset created successfully:', this.state.currentDataset);
      } catch (lifecycleErr) {
        console.error('Failed to create lifecycle dataset:', lifecycleErr);
        this.showToast(`Lifecycle creation failed: ${lifecycleErr}`, 'error');
        // Analysis still succeeds, just no lifecycle tracking
      }

      this.state.isLoading = false;
      this.render();
      this.showToast('Analysis complete', 'success');
    } catch (err) {
      this.state.isLoading = false;
      this.render();
      this.showToast(`Analysis failed: ${err}`, 'error');
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
