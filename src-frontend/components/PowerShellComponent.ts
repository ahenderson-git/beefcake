import * as monaco from 'monaco-editor';

import * as api from '../api';
import * as renderers from '../renderers';
import { AppState } from '../types';

import { Component, ComponentActions } from './Component';

export class PowerShellComponent extends Component {
  private editor: monaco.editor.IStandaloneCodeEditor | null = null;

  constructor(containerId: string, actions: ComponentActions) {
    super(containerId, actions);
  }

  render(state: AppState): void {
    const container = this.getContainer();
    container.innerHTML = renderers.renderPowerShellView(
      state.config?.settings.powershell_font_size ?? 14
    );
    this.initMonaco(state);
    this.bindEvents(state);
  }

  private initMonaco(state: AppState): void {
    const editorContainer = document.getElementById('ps-editor');
    if (editorContainer) {
      this.editor = monaco.editor.create(editorContainer, {
        value: '# PowerShell script\nWrite-Host "Hello from Beefcake!"',
        language: 'powershell',
        theme: 'vs-dark',
        automaticLayout: true,
        fontSize: state.config?.settings.powershell_font_size ?? 14,
        fontFamily: "'Fira Code', monospace",
        fontLigatures: true,
        minimap: { enabled: false },
      });
    }
  }

  override bindEvents(state: AppState): void {
    document.getElementById('btn-run-ps')?.addEventListener('click', () => {
      void this.runPowerShell(state);
    });
    document.getElementById('btn-clear-ps')?.addEventListener('click', () => {
      const output = document.getElementById('ps-output');
      if (output) output.textContent = '';
    });
    document.getElementById('btn-inc-font')?.addEventListener('click', () => {
      void this.updateFontSize(state, 1);
    });
    document.getElementById('btn-dec-font')?.addEventListener('click', () => {
      void this.updateFontSize(state, -1);
    });
    document.getElementById('btn-load-ps')?.addEventListener('click', () => {
      void this.handleLoadScript();
    });
    document.getElementById('btn-save-ps')?.addEventListener('click', () => {
      void this.handleSaveScript();
    });
  }

  private async runPowerShell(state: AppState): Promise<void> {
    if (!this.editor) return;
    if (!(await this.ensureSecurityAcknowledged(state))) return;
    const script = this.editor.getValue();
    const output = document.getElementById('ps-output');
    if (!output) return;

    output.textContent = 'Running...';
    try {
      output.textContent = await api.runPowerShell(script);
    } catch (err) {
      output.textContent = `Error: ${String(err)}`;
    }
  }

  private async updateFontSize(state: AppState, delta: number): Promise<void> {
    if (state.config) {
      state.config.settings.powershell_font_size = Math.max(
        8,
        Math.min(32, state.config.settings.powershell_font_size + delta)
      );
      this.editor?.updateOptions({ fontSize: state.config.settings.powershell_font_size });

      const label = document.getElementById('ps-font-size-label');
      if (label) label.textContent = state.config.settings.powershell_font_size.toString();

      await api.saveAppConfig(state.config);
      this.actions.onStateChange();
    }
  }

  private async ensureSecurityAcknowledged(state: AppState): Promise<boolean> {
    if (!state.config) {
      this.actions.showToast('App configuration missing', 'error');
      return false;
    }

    if (state.config.settings.security_warning_acknowledged) {
      return true;
    }

    const confirmed = confirm(
      'Running scripts can execute arbitrary code on your machine. Do you want to continue?'
    );
    if (!confirmed) {
      return false;
    }

    state.config.settings.security_warning_acknowledged = true;
    await api.saveAppConfig(state.config);
    this.actions.showToast('Security warning acknowledged', 'info');
    return true;
  }

  private async handleLoadScript(): Promise<void> {
    try {
      const path = await api.openFileDialog([{ name: 'PowerShell', extensions: ['ps1'] }]);
      if (path) {
        const content = await api.readTextFile(path);
        this.editor?.setValue(content);
        this.actions.showToast('Script loaded', 'success');
      }
    } catch (err) {
      this.actions.showToast(`Error loading script: ${String(err)}`, 'error');
    }
  }

  private async handleSaveScript(): Promise<void> {
    try {
      const path = await api.saveFileDialog([{ name: 'PowerShell', extensions: ['ps1'] }]);
      if (path) {
        const content = this.editor?.getValue() ?? '';
        await api.writeTextFile(path, content);
        this.actions.showToast('Script saved', 'success');
      }
    } catch (err) {
      this.actions.showToast(`Error saving script: ${String(err)}`, 'error');
    }
  }
}
