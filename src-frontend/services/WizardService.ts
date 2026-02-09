import * as api from '../api';
import * as renderers from '../renderers';
import { AppState, StandardPaths } from '../types';

export class WizardService {
  private standardPaths: StandardPaths | null = null;
  private wizardWorkspacePath: string | null = null;
  private wizardOpen = false;

  constructor(
    private state: AppState,
    private renderApp: () => void,
    private showToast: (msg: string, type: 'info' | 'error' | 'success') => void
  ) {}

  async maybeShowFirstRunWizard(): Promise<void> {
    if (!this.state.config || this.state.config.settings.first_run_completed || this.wizardOpen) {
      return;
    }
    try {
      this.standardPaths = await api.getStandardPaths();
      this.wizardWorkspacePath = null;
      this.renderFirstRunWizard();
    } catch (err) {
      this.showToast(`Failed to load standard folders: ${String(err)}`, 'error');
    }
  }

  async showWizardOnDemand(): Promise<void> {
    if (this.wizardOpen) {
      return;
    }
    try {
      this.standardPaths = await api.getStandardPaths();
      this.wizardWorkspacePath = null;
      this.renderFirstRunWizard();
    } catch (err) {
      this.showToast(`Failed to load standard folders: ${String(err)}`, 'error');
    }
  }

  private renderFirstRunWizard(): void {
    const modal = document.getElementById('modal-container');
    if (!modal) return;
    this.wizardOpen = true;
    modal.innerHTML = renderers.renderFirstRunWizard(this.standardPaths, this.wizardWorkspacePath);
    modal.classList.add('active');

    modal.querySelectorAll<HTMLButtonElement>('.wizard-folder-btn').forEach(btn => {
      btn.addEventListener('click', () => {
        const path = btn.dataset.path;
        if (path) {
          void api.openPath(path);
        }
      });
    });

    modal.querySelectorAll<HTMLButtonElement>('.wizard-copy-path').forEach(btn => {
      btn.addEventListener('click', () => {
        const path = btn.dataset.copy;
        if (path) {
          const label = btn.title || btn.getAttribute('aria-label') || 'Path';
          void this.copyPath(path, label);
        }
      });
    });

    modal
      .querySelector<HTMLButtonElement>('#btn-wizard-choose-workspace')
      ?.addEventListener('click', () => {
        void (async () => {
          const selected = await api.openFolderDialog();
          if (selected) {
            this.wizardWorkspacePath = selected;
            this.renderFirstRunWizard();
          }
        })();
      });

    modal.querySelector<HTMLButtonElement>('#btn-wizard-skip')?.addEventListener('click', () => {
      this.closeWizard();
    });

    modal.querySelector<HTMLButtonElement>('#btn-wizard-close')?.addEventListener('click', () => {
      this.closeWizard();
    });

    modal.querySelector<HTMLButtonElement>('#btn-wizard-finish')?.addEventListener('click', () => {
      void this.completeWizard();
    });
  }

  private async copyPath(path: string, label?: string): Promise<void> {
    try {
      await navigator.clipboard.writeText(path);
      const msg = label ? `${label} copied` : 'Path copied to clipboard';
      this.showToast(msg, 'success');
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      this.showToast(`Failed to copy path: ${message}`, 'error');
    }
  }

  private closeWizard(): void {
    const modal = document.getElementById('modal-container');
    if (!modal) return;
    modal.classList.remove('active');
    setTimeout(() => {
      if (!modal.classList.contains('active')) {
        modal.innerHTML = '';
      }
    }, 300);
    this.wizardOpen = false;
  }

  private async completeWizard(): Promise<void> {
    if (!this.state.config) {
      this.closeWizard();
      return;
    }

    try {
      if (this.wizardWorkspacePath) {
        await api.addTrustedPath(this.wizardWorkspacePath);
      }
      this.state.config.settings.first_run_completed = true;
      await api.saveAppConfig(this.state.config);
      this.showToast('Setup complete', 'success');
    } catch (err) {
      this.showToast(`Failed to save setup: ${String(err)}`, 'error');
    } finally {
      this.closeWizard();
      this.renderApp();
    }
  }
}
