import * as renderers from '../renderers';
import { AppState } from '../types';

import { Component, ComponentActions } from './Component';

export class CliHelpComponent extends Component {
  constructor(containerId: string, actions: ComponentActions) {
    super(containerId, actions);
  }

  render(_state: AppState): void {
    const container = this.getContainer();
    container.innerHTML = renderers.renderCliHelpView();
  }
}
