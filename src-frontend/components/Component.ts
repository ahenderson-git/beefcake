import { AppState, View } from '../types';

export interface ComponentActions {
  switchView: (view: View) => void;
  showToast: (message: string, type?: 'info' | 'error' | 'success') => void;
  onStateChange: () => void;
  runAnalysis: (path: string) => void;
  navigateTo?: (view: string, datasetId?: string) => void;
  showFirstRunWizard?: () => void;
}

export abstract class Component {
  protected container: HTMLElement | null;
  protected actions: ComponentActions;

  constructor(
    private containerId: string,
    actions: ComponentActions
  ) {
    this.container = document.getElementById(containerId);
    if (!this.container) {
      console.warn(
        `[Component] Container ${containerId} not found during construction. Will retry during getContainer().`
      );
    }
    this.actions = actions;
  }

  abstract render(state: AppState): void;

  protected getContainer(): HTMLElement {
    if (!this.container) {
      // Re-attempt to find container in case it was added to DOM later
      this.container = document.getElementById(this.containerId);

      if (!this.container) {
        throw new Error(`Container ${this.containerId} not found for component`);
      }
    }
    return this.container;
  }

  // Optional method for components that need to bind their own events after rendering
  // By default it does nothing.
  bindEvents(_state: AppState): void {}
}
