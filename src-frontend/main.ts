import "@phosphor-icons/web/regular";
import "@fontsource/fira-code/300.css";
import "@fontsource/fira-code/400.css";
import "@fontsource/fira-code/500.css";

import {
  ColumnCleanConfig,
  View,
  AppState
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

class BeefcakeApp {
  private state: AppState = {
    version: '0.0.0',
    currentView: 'Dashboard',
    analysisResponse: null,
    expandedRows: new Set(),
    cleaningConfigs: {},
    isAddingConnection: false,
    trimPct: 0.1,
    config: null,
    pythonScript: null,
    sqlScript: null
  };

  private components: Partial<Record<View, Component>> = {};

  constructor() {
    this.init().catch(err => {
      console.error('Failed to initialize BeefcakeApp:', err);
    });
  }

  async init() {
    [this.state.config, this.state.version] = await Promise.all([
      api.loadAppConfig(),
      api.getAppVersion()
    ]);
    this.renderInitialLayout();
    this.initComponents();
    this.setupNavigation();
    this.render();
    
    // Make app accessible globally for some component callbacks (e.g. re-analysis)
    (window as any).beefcakeApp = this;
  }

  private renderInitialLayout() {
    const app = document.getElementById('app');
    if (app) {
      app.innerHTML = renderers.renderLayout();
    }
  }

  private initComponents() {
    const onStateChange = () => this.render();
    const showToast = (msg: string, type: any) => this.showToast(msg, type);
    const onAnalysis = (path: string) => this.handleAnalysis(path);

    this.components = {
      'Dashboard': new DashboardComponent('view-container', onStateChange, showToast, onAnalysis),
      'Analyser': new AnalyserComponent('view-container', onStateChange, showToast),
      'PowerShell': new PowerShellComponent('view-container', onStateChange, showToast),
      'Python': new PythonComponent('view-container', onStateChange, showToast),
      'SQL': new SQLComponent('view-container', onStateChange, showToast),
      'Settings': new SettingsComponent('view-container', onStateChange, showToast),
      'CLI': new CliHelpComponent('view-container'),
      'ActivityLog': new ActivityLogComponent('view-container', onStateChange)
    };
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
      else title.innerText = view;
    }

    this.render();
  }

  private render() {
    const component = this.components[this.state.currentView];
    if (component) {
      component.render(this.state);
    }
  }

  public async handleAnalysis(path: string) {
    try {
      this.showToast(`Analyzing ${path}...`, 'info');
      const response = await api.analyseFile(path, this.state.trimPct);
      this.state.analysisResponse = response;
      this.state.currentView = 'Analyser';
      
      // Initialize cleaning configs
      this.state.cleaningConfigs = {};
      response.summary.forEach(col => {
        this.state.cleaningConfigs[col.name] = this.getDefaultConfig(col);
      });
      
      this.switchView('Analyser');
      this.showToast('Analysis complete', 'success');
    } catch (err) {
      this.showToast(`Analysis failed: ${err}`, 'error');
    }
  }

  private getDefaultConfig(col: ColumnSummary): ColumnCleanConfig {
    return {
      new_name: col.standardized_name || col.name,
      target_dtype: null,
      active: true,
      advanced_cleaning: false,
      ml_preprocessing: false,
      trim_whitespace: true,
      remove_special_chars: false,
      text_case: "None",
      standardize_nulls: true,
      remove_non_ascii: false,
      regex_find: "",
      regex_replace: "",
      rounding: null,
      extract_numbers: false,
      clip_outliers: false,
      temporal_format: "",
      timezone_utc: false,
      freq_threshold: null,
      normalization: "None",
      one_hot_encode: false,
      impute_mode: "None"
    };
  }

  private showToast(message: string, type: 'info' | 'error' | 'success' = 'info') {
    const container = document.getElementById('toast-container');
    if (!container) return;

    const toast = document.createElement('div');
    toast.innerHTML = renderers.renderToast(message, type);
    const toastEl = toast.firstElementChild!;
    container.appendChild(toastEl);

    setTimeout(() => {
      toastEl.classList.add('fade-out');
      setTimeout(() => toastEl.remove(), 500);
    }, 3000);
  }
}

window.addEventListener('DOMContentLoaded', () => {
  new BeefcakeApp();
});
