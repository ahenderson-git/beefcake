import { AppState, View } from "../types";

export interface ComponentActions {
  switchView: (view: View) => void;
  showToast: (message: string, type?: 'info' | 'error' | 'success') => void;
  onStateChange: () => void;
  runAnalysis: (path: string) => void;
}

export abstract class Component {
  protected container: HTMLElement;
  protected actions: ComponentActions;

  constructor(containerId: string, actions: ComponentActions) {
    const el = document.getElementById(containerId);
    if (!el) throw new Error(`Container with id "${containerId}" not found`);
    this.container = el;
    this.actions = actions;
  }

  abstract render(state: AppState): void;

  // Optional method for components that need to bind their own events after rendering
  // By default it does nothing.
  bindEvents(_state: AppState): void {}
}
