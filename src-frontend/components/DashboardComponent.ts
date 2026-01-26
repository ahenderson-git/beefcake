import * as api from '../api';
import * as renderers from '../renderers';
import { AppState } from '../types';

import { Component, ComponentActions } from './Component';

export class DashboardComponent extends Component {
  constructor(containerId: string, actions: ComponentActions) {
    super(containerId, actions);
  }

  render(state: AppState): void {
    const container = this.getContainer();
    container.innerHTML = renderers.renderDashboardView(state);
    /* eslint-disable no-console */
    console.log('[DashboardComponent] Rendered, calling bindEvents...');
    this.bindEvents(state);
    console.log('[DashboardComponent] bindEvents complete');
  }

  override bindEvents(_state: AppState): void {
    console.log('[DashboardComponent] bindEvents called');

    const btnOpenFile = document.getElementById('btn-open-file');
    console.log('[DashboardComponent] btn-open-file element:', btnOpenFile);

    btnOpenFile?.addEventListener('click', () => {
      console.log('[DashboardComponent] Open file button clicked!');
      /* eslint-enable no-console */
      void this.handleOpenFile();
    });

    document.getElementById('btn-powershell')?.addEventListener('click', () => {
      this.actions.switchView('PowerShell');
    });

    document.getElementById('btn-python')?.addEventListener('click', () => {
      this.actions.switchView('Python');
    });

    document.getElementById('btn-sql')?.addEventListener('click', () => {
      this.actions.switchView('SQL');
    });
  }

  private async handleOpenFile(): Promise<void> {
    try {
      const path = await api.openFileDialog();
      if (path) {
        this.actions.runAnalysis(path);
      }
    } catch (err) {
      this.actions.showToast(`Error opening file: ${String(err)}`, 'error');
    }
  }
}
