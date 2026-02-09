import * as api from '../api';
import * as renderers from '../renderers';
import { AppState, VerificationResult } from '../types';

import { Component, ComponentActions } from './Component';

export class IntegrityComponent extends Component {
  private verificationResult: VerificationResult | null = null;
  private isVerifying = false;

  constructor(containerId: string, actions: ComponentActions) {
    super(containerId, actions);
  }

  render(state: AppState): void {
    const container = this.getContainer();
    container.innerHTML = renderers.renderIntegrityView(this.verificationResult, this.isVerifying);
    this.bindEvents(state);
  }

  override bindEvents(_state: AppState): void {
    document.getElementById('btn-select-receipt')?.addEventListener('click', () => {
      void this.handleSelectReceipt();
    });
  }

  private async handleSelectReceipt(): Promise<void> {
    try {
      const path = await api.openFileDialog();
      if (!path) return;

      if (!path.endsWith('.receipt.json')) {
        this.actions.showToast('Please select a .receipt.json file', 'error');
        return;
      }

      this.isVerifying = true;
      this.verificationResult = null;
      this.actions.onStateChange();

      const result = await api.verifyReceipt(path);
      this.verificationResult = result;
      this.isVerifying = false;
      this.actions.onStateChange();

      if (result.passed) {
        this.actions.showToast('✓ Verification passed!', 'success');
      } else {
        this.actions.showToast('✗ Verification failed', 'error');
      }
    } catch (err) {
      this.isVerifying = false;
      this.verificationResult = null;
      this.actions.onStateChange();
      this.actions.showToast(`Verification error: ${String(err)}`, 'error');
    }
  }
}
