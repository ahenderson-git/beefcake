import { Component, ComponentActions } from "./Component";
import { AppState } from "../types";
import * as renderers from "../renderers";

export class CliHelpComponent extends Component {
  constructor(containerId: string, actions: ComponentActions) {
    super(containerId, actions);
  }

  render(_state: AppState): void {
    const container = this.getContainer();
    container.innerHTML = renderers.renderCliHelpView();
  }
}
