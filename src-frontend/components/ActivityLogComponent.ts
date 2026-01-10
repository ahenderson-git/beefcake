import { Component, ComponentActions } from "./Component";
import { AppState } from "../types";
import * as renderers from "../renderers";
import * as api from "../api";

export class ActivityLogComponent extends Component {
  constructor(containerId: string, actions: ComponentActions) {
    super(containerId, actions);
  }

  render(state: AppState): void {
    if (!state.config) return;
    this.container.innerHTML = renderers.renderActivityLogView(state.config);
    this.bindEvents(state);
  }

  bindEvents(state: AppState): void {
    document.getElementById('btn-clear-log')?.addEventListener('click', async () => {
      if (state.config) {
        state.config.audit_log = [];
        await api.saveAppConfig(state.config);
        this.actions.onStateChange();
      }
    });
  }
}
