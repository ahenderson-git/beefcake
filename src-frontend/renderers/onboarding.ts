import { StandardPaths } from '../types';
import { escapeHtml } from '../utils';

export function renderFirstRunWizard(
  standardPaths: StandardPaths | null,
  workspacePath: string | null
): string {
  const folders = standardPaths;
  const workspace = workspacePath ? escapeHtml(workspacePath) : '';
  const workspaceLabel = workspacePath
    ? `<div class="wizard-workspace-path">Selected: ${workspace}</div>`
    : '<div class="wizard-workspace-path">No workspace selected yet.</div>';

  return `
    <div class="modal-overlay" data-testid="first-run-modal-overlay">
      <div class="modal-content modal-onboarding" data-testid="first-run-modal">
        <div class="modal-header">
          <h3>Welcome to Beefcake</h3>
          <button class="modal-close" id="btn-wizard-close" aria-label="Close">
            <i class="ph ph-x"></i>
          </button>
        </div>

        <div class="modal-body">
          <p class="modal-description">
            Beefcake uses a standard folder layout to keep your files organized. You can also add
            a workspace folder to trust for read/write access.
          </p>

          <div class="wizard-section">
            <h4><i class="ph ph-folders"></i> Standard Folders</h4>
            <div class="folder-grid">
              <div class="folder-item">
                <button class="btn-secondary btn-small wizard-folder-btn" data-path="${folders ? escapeHtml(folders.input_dir) : ''}" ${folders ? '' : 'disabled'}>
                  <i class="ph ph-tray-arrow-down"></i> Open Input
                </button>
                <button class="btn-secondary btn-small btn-copy-path wizard-copy-path" title="Copy input path" aria-label="Copy input path" data-copy="${folders ? escapeHtml(folders.input_dir) : ''}" ${folders ? '' : 'disabled'}>
                  <i class="ph ph-copy"></i>
                </button>
              </div>
              <div class="folder-item">
                <button class="btn-secondary btn-small wizard-folder-btn" data-path="${folders ? escapeHtml(folders.output_dir) : ''}" ${folders ? '' : 'disabled'}>
                  <i class="ph ph-tray-arrow-up"></i> Open Output
                </button>
                <button class="btn-secondary btn-small btn-copy-path wizard-copy-path" title="Copy output path" aria-label="Copy output path" data-copy="${folders ? escapeHtml(folders.output_dir) : ''}" ${folders ? '' : 'disabled'}>
                  <i class="ph ph-copy"></i>
                </button>
              </div>
              <div class="folder-item">
                <button class="btn-secondary btn-small wizard-folder-btn" data-path="${folders ? escapeHtml(folders.scripts_dir) : ''}" ${folders ? '' : 'disabled'}>
                  <i class="ph ph-code"></i> Open Scripts
                </button>
                <button class="btn-secondary btn-small btn-copy-path wizard-copy-path" title="Copy scripts path" aria-label="Copy scripts path" data-copy="${folders ? escapeHtml(folders.scripts_dir) : ''}" ${folders ? '' : 'disabled'}>
                  <i class="ph ph-copy"></i>
                </button>
              </div>
              <div class="folder-item">
                <button class="btn-secondary btn-small wizard-folder-btn" data-path="${folders ? escapeHtml(folders.logs_dir) : ''}" ${folders ? '' : 'disabled'}>
                  <i class="ph ph-file-text"></i> Open Logs
                </button>
                <button class="btn-secondary btn-small btn-copy-path wizard-copy-path" title="Copy logs path" aria-label="Copy logs path" data-copy="${folders ? escapeHtml(folders.logs_dir) : ''}" ${folders ? '' : 'disabled'}>
                  <i class="ph ph-copy"></i>
                </button>
              </div>
              <div class="folder-item">
                <button class="btn-secondary btn-small wizard-folder-btn" data-path="${folders ? escapeHtml(folders.templates_dir) : ''}" ${folders ? '' : 'disabled'}>
                  <i class="ph ph-grid-four"></i> Open Templates
                </button>
                <button class="btn-secondary btn-small btn-copy-path wizard-copy-path" title="Copy templates path" aria-label="Copy templates path" data-copy="${folders ? escapeHtml(folders.templates_dir) : ''}" ${folders ? '' : 'disabled'}>
                  <i class="ph ph-copy"></i>
                </button>
              </div>
            </div>
            <div class="folder-actions">
              <button class="btn-secondary btn-small wizard-folder-btn" data-path="${folders ? escapeHtml(folders.base_dir) : ''}" ${folders ? '' : 'disabled'}>
                <i class="ph ph-house"></i> Open Base Folder
              </button>
              <button class="btn-secondary btn-small btn-copy-path wizard-copy-path" title="Copy base path" aria-label="Copy base path" data-copy="${folders ? escapeHtml(folders.base_dir) : ''}" ${folders ? '' : 'disabled'}>
                <i class="ph ph-copy"></i> Copy Base Path
              </button>
            </div>
            ${
              folders
                ? `<p class="folder-path-hint">Base: ${escapeHtml(folders.base_dir)}</p>`
                : '<p class="folder-path-hint">Loading standard folders…</p>'
            }
          </div>

          <div class="wizard-section">
            <h4><i class="ph ph-briefcase"></i> Workspace Folder</h4>
            <p class="wizard-subtext">
              Add a workspace directory if you want to read/write files outside the app folders.
            </p>
            <button id="btn-wizard-choose-workspace" class="btn-secondary btn-small">
              <i class="ph ph-folder-open"></i> Choose Workspace
            </button>
            ${workspaceLabel}
          </div>

          <div class="wizard-actions">
            <button id="btn-wizard-skip" class="btn-secondary">Remind Me Later</button>
            <button id="btn-wizard-finish" class="btn-primary">Finish Setup</button>
          </div>
        </div>
      </div>
    </div>
  `;
}
