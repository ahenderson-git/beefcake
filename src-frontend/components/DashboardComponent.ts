import { Component, ComponentActions } from "./Component";
import { AppState } from "../types";
import * as renderers from "../renderers";
import * as api from "../api";

export class DashboardComponent extends Component {
  constructor(containerId: string, actions: ComponentActions) {
    super(containerId, actions);
  }

  render(state: AppState): void {
    const container = this.getContainer();
    container.innerHTML = renderers.renderDashboardView(state);
    this.bindEvents(state);
  }

  override bindEvents(_state: AppState): void {
    document.getElementById('btn-open-file')?.addEventListener('click', () => {
      this.handleOpenFile();
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

  private async handleOpenFile() {
    try {
      const path = await api.openFileDialog();
      if (path) {
        this.actions.runAnalysis(path);
      }
    } catch (err) {
      this.actions.showToast(`Error opening file: ${err}`, 'error');
    }
  }
}
