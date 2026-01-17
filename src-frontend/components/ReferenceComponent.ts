import * as renderers from '../renderers';
import { AppState } from '../types';

import { Component, ComponentActions } from './Component';

export class ReferenceComponent extends Component {
  constructor(containerId: string, actions: ComponentActions) {
    super(containerId, actions);
  }

  render(state: AppState): void {
    const container = this.getContainer();
    container.innerHTML = renderers.renderReferenceView();
    this.bindEvents(state);
  }

  override bindEvents(_state: AppState): void {
    // No specific events for now, just static content with links
  }
}
