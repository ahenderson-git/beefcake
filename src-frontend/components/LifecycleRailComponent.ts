import * as api from '../api';
import * as renderers from '../renderers';
import { AppState, LifecycleStage } from '../types';

import { Component, ComponentActions } from './Component';

export class LifecycleRailComponent extends Component {
  constructor(containerId: string, actions: ComponentActions) {
    super(containerId, actions);
  }

  render(state: AppState): void {
    // This component renders into a specific rail container, not the main view
    const railContainer = document.getElementById('lifecycle-rail-container');
    if (!railContainer) {
      // Don't warn here, it's expected in views other than Analyser
      return;
    }

    // Render stage indicator: full rail with dataset, or compact banner without
    if (state.isCreatingLifecycle) {
      // Show loading banner while creating dataset versions
      const bannerStage: LifecycleStage = 'Raw';
      const bannerMessage = 'Creating dataset versions...';
      railContainer.innerHTML = renderers.renderLifecycleBanner(bannerStage, bannerMessage);
      // No events to bind for banner
    } else if (state.currentDataset) {
      // Full lifecycle rail when dataset exists
      railContainer.innerHTML = renderers.renderLifecycleRail(state.currentDataset);
      this.bindEvents(state);
    } else if (state.analysisResponse) {
      // This should only happen if dataset creation failed
      const bannerStage: LifecycleStage = 'Profiled';
      const bannerMessage = 'Dataset creation failed - analysis only mode';
      railContainer.innerHTML = renderers.renderLifecycleBanner(bannerStage, bannerMessage);
      // No events to bind for banner
    } else {
      // No analysis yet - empty placeholder
      railContainer.innerHTML = renderers.renderLifecycleRail(null);
    }
  }

  override bindEvents(state: AppState): void {
    // Stage click handlers - for switching active version
    document.querySelectorAll('.lifecycle-stage').forEach(stageEl => {
      stageEl.addEventListener('click', e => {
        void (async () => {
          const target = e.currentTarget as HTMLElement;
          const stage = target.dataset.stage as LifecycleStage;

          if (target.classList.contains('stage-locked')) {
            this.actions.showToast(
              `Cannot switch to ${stage} stage yet - prerequisites not met`,
              'error'
            );
            return;
          }

          if (!state.currentDataset) return;

          // Find version with this stage
          const targetVersion = state.currentDataset.versions.find(v => v.stage === stage);
          if (targetVersion && targetVersion.id !== state.currentDataset.activeVersionId) {
            try {
              await api.setActiveVersion(state.currentDataset.id, targetVersion.id);
              state.currentDataset.activeVersionId = targetVersion.id;
              this.actions.showToast(`Switched to ${stage} version`, 'success');
              this.actions.onStateChange();
            } catch (err) {
              this.actions.showToast(`Failed to switch version: ${String(err)}`, 'error');
            }
          }
        })();
      });
    });
  }
}
