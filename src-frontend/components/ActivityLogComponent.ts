import * as api from '../api';
import * as renderers from '../renderers';
import { AppState } from '../types';

import { Component, ComponentActions } from './Component';

export class ActivityLogComponent extends Component {
  constructor(containerId: string, actions: ComponentActions) {
    super(containerId, actions);
  }

  render(state: AppState): void {
    if (!state.config) return;
    const container = this.getContainer();
    container.innerHTML = renderers.renderActivityLogView(state.config);
    this.bindEvents(state);
  }

  override bindEvents(state: AppState): void {
    document.getElementById('btn-clear-log')?.addEventListener('click', () => {
      if (state.config) {
        state.config.audit_log = [];
        void api.saveAppConfig(state.config).then(() => {
          this.actions.onStateChange();
        });
      }
    });
  }
}
