import { Component, ComponentActions } from "./Component";
import { AppState } from "../types";
import * as renderers from "../renderers";

export class ReferenceComponent extends Component {
  constructor(containerId: string, actions: ComponentActions) {
    super(containerId, actions);
  }

  render(state: AppState): void {
    this.container.innerHTML = renderers.renderReferenceView();
    this.bindEvents(state);
  }

  bindEvents(_state: AppState): void {
    // No specific events for now, just static content with links
  }
}
