import { Component, ComponentActions } from "./Component";
import { AppState, DbConnection } from "../types";
import * as renderers from "../renderers";
import * as api from "../api";

export class SettingsComponent extends Component {
  constructor(containerId: string, actions: ComponentActions) {
    super(containerId, actions);
  }

  render(state: AppState): void {
    if (!state.config) return;
    const container = this.getContainer();
    container.innerHTML = renderers.renderSettingsView(state.config, state.isAddingConnection);
    this.bindEvents(state);
  }

  override bindEvents(state: AppState): void {
    if (state.isAddingConnection) {
      document.getElementById('btn-save-new-conn')?.addEventListener('click', () => {
        this.saveNewConnection(state);
      });

      document.getElementById('btn-cancel-new-conn')?.addEventListener('click', () => {
        state.isAddingConnection = false;
        this.actions.onStateChange();
      });

      document.getElementById('btn-test-new-conn')?.addEventListener('click', (e) => {
        this.testNewConnection(e.currentTarget as HTMLButtonElement);
      });
    } else {
      document.getElementById('btn-add-conn')?.addEventListener('click', () => {
        state.isAddingConnection = true;
        this.actions.onStateChange();
      });
    }

    document.getElementById('select-import-id')?.addEventListener('change', async (e) => {
      if (state.config) {
        state.config.active_import_id = (e.target as HTMLSelectElement).value || null;
        await api.saveAppConfig(state.config);
      }
    });

    document.getElementById('select-export-id')?.addEventListener('change', async (e) => {
      if (state.config) {
        state.config.active_export_id = (e.target as HTMLSelectElement).value || null;
        await api.saveAppConfig(state.config);
      }
    });

    document.querySelectorAll('.btn-test-conn').forEach(btn => {
      btn.addEventListener('click', (e) => {
        const id = (e.currentTarget as HTMLElement).dataset.id!;
        this.handleTestConnection(state, id, e.currentTarget as HTMLButtonElement);
      });
    });

    document.querySelectorAll('.btn-delete-conn').forEach(btn => {
      btn.addEventListener('click', (e) => {
        const id = (e.currentTarget as HTMLElement).dataset.id!;
        this.deleteConnection(state, id);
      });
    });
  }

  private getNewConnectionSettings() {
    const host = (document.getElementById('new-conn-host') as HTMLInputElement)?.value;
    const port = (document.getElementById('new-conn-port') as HTMLInputElement)?.value;
    const user = (document.getElementById('new-conn-user') as HTMLInputElement)?.value;
    const password = (document.getElementById('new-conn-pass') as HTMLInputElement)?.value;
    const database = (document.getElementById('new-conn-db') as HTMLInputElement)?.value;
    const schema = (document.getElementById('new-conn-schema') as HTMLInputElement)?.value || 'public';
    const table = (document.getElementById('new-conn-table') as HTMLInputElement)?.value || '';

    return {
      db_type: 'postgres',
      host,
      port,
      user,
      password,
      database,
      schema,
      table
    };
  }

  private async saveNewConnection(state: AppState) {
    const name = (document.getElementById('new-conn-name') as HTMLInputElement)?.value;
    if (!name) {
      this.actions.showToast('Connection name is required', 'error');
      return;
    }

    const newConn: DbConnection = {
      id: crypto.randomUUID(),
      name,
      settings: this.getNewConnectionSettings() as any
    };

    if (state.config) {
      state.config.connections.push(newConn);
      await api.saveAppConfig(state.config);
      state.isAddingConnection = false;
      this.actions.onStateChange();
      this.actions.showToast('Connection added', 'success');
    }
  }

  private async testNewConnection(btn: HTMLButtonElement) {
    const settings = this.getNewConnectionSettings();
    const icon = btn.querySelector('i');
    
    if (icon) {
      btn.disabled = true;
      icon.className = 'ph ph-circle-notch loading-spin';
    }

    try {
      this.actions.showToast('Testing connection...', 'info');
      const result = await api.testConnection(settings);
      this.actions.showToast(result, 'success');
    } catch (err) {
      this.actions.showToast(String(err), 'error');
    } finally {
      if (icon) {
        btn.disabled = false;
        icon.className = 'ph ph-plugs-connected';
      }
    }
  }

  private async handleTestConnection(state: AppState, id: string, btn: HTMLButtonElement) {
    const conn = state.config?.connections.find(c => c.id === id);
    if (!conn) return;

    const icon = btn.querySelector('i');
    if (icon) {
      btn.disabled = true;
      icon.className = 'ph ph-circle-notch loading-spin';
    }

    try {
      this.actions.showToast(`Testing connection "${conn.name}"...`, 'info');
      const result = await api.testConnection(conn.settings, id);
      this.actions.showToast(result, 'success');
    } catch (err) {
      this.actions.showToast(String(err), 'error');
    } finally {
      if (icon) {
        btn.disabled = false;
        icon.className = 'ph ph-plugs-connected';
      }
    }
  }

  private async deleteConnection(state: AppState, id: string) {
    if (state.config) {
      if (!confirm('Are you sure you want to delete this connection?')) return;
      state.config.connections = state.config.connections.filter(c => c.id !== id);
      if (state.config.active_import_id === id) state.config.active_import_id = null;
      if (state.config.active_export_id === id) state.config.active_export_id = null;
      
      await api.deleteConnection(id);
      await api.saveAppConfig(state.config);
      this.actions.onStateChange();
      this.actions.showToast('Connection deleted', 'success');
    }
  }
}
