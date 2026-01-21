import { AppConfig } from '../types';
import { escapeHtml } from '../utils';

export function renderSettingsView(config: AppConfig, isAddingConnection: boolean): string {
  return `
    <div class="settings-view">
      <div class="settings-section">
        <h3><i class="ph ph-plug-connect"></i> Database Connections</h3>
        <div class="connections-list">
          ${config.connections.length === 0 ? '<p class="empty-msg">No connections configured.</p>' : ''}
          ${config.connections
            .map(
              conn => `
            <div class="connection-card">
              <div class="conn-info">
                <strong>${escapeHtml(conn.name)}</strong>
                <span>${escapeHtml(conn.settings.host)}:${conn.settings.port} / ${escapeHtml(conn.settings.database)}</span>
              </div>
              <div class="conn-actions">
                <button class="btn-secondary btn-small btn-test-conn" data-id="${conn.id}">Test</button>
                <button class="btn-danger btn-small btn-delete-conn" data-id="${conn.id}"><i class="ph ph-trash"></i></button>
              </div>
            </div>
          `
            )
            .join('')}
        </div>
        
        ${
          isAddingConnection
            ? `
          <div class="connection-form active">
            <h4>Add New Connection</h4>
            <div class="form-grid">
              <input type="text" id="new-conn-name" placeholder="Connection Name (e.g. Production DB)">
              <input type="text" id="new-conn-host" placeholder="Host (localhost)">
              <input type="number" id="new-conn-port" value="5432" placeholder="Port">
              <input type="text" id="new-conn-user" placeholder="User">
              <input type="password" id="new-conn-pass" placeholder="Password">
              <input type="text" id="new-conn-db" placeholder="Database">
              <input type="text" id="new-conn-schema" value="public" placeholder="Schema (public)">
              <input type="text" id="new-conn-table" placeholder="Table">
            </div>
            <div class="form-actions">
              <button id="btn-test-new-conn" class="btn-secondary"><i class="ph ph-plugs-connected"></i> Test</button>
              <button id="btn-save-new-conn" class="btn-primary">Save Connection</button>
              <button id="btn-cancel-new-conn" class="btn-secondary">Cancel</button>
            </div>
          </div>
        `
            : `
          <button id="btn-add-conn" class="btn-secondary"><i class="ph ph-plus"></i> Add Connection</button>
        `
        }
      </div>

      <div class="settings-section">
        <h3><i class="ph ph-gear"></i> Application Preferences</h3>
        <div class="pref-item">
          <label>Default Import Connection</label>
          <select id="select-import-id">
            <option value="">None</option>
            ${config.connections
              .map(
                c => `
              <option value="${c.id}" ${config.active_import_id === c.id ? 'selected' : ''}>${escapeHtml(c.name)}</option>
            `
              )
              .join('')}
          </select>
        </div>
        <div class="pref-item">
          <label>Default Export Connection</label>
          <select id="select-export-id">
            <option value="">None</option>
            ${config.connections
              .map(
                c => `
              <option value="${c.id}" ${config.active_export_id === c.id ? 'selected' : ''}>${escapeHtml(c.name)}</option>
            `
              )
              .join('')}
          </select>
        </div>

        <div class="pref-subsection">
          <h4><i class="ph ph-gauge"></i> Performance & Sampling</h4>
          <p class="subsection-description">Configure how data is sampled and analysed for large files</p>

          <div class="pref-item">
            <label for="analysis-sample-size">
              Statistical Sample Size
              <i class="ph ph-info help-icon" title="Number of rows used for histograms, quantiles, and distribution statistics" aria-label="Help: Number of rows used for histograms, quantiles, and distribution statistics"></i>
            </label>
            <input type="number" id="analysis-sample-size"
              min="1000" max="500000" step="1000"
              value="${config.analysis_sample_size ?? 10000}"
              aria-describedby="sample-size-warning">
            <div id="sample-size-warning" class="sample-size-warning" role="status" aria-live="polite"></div>
          </div>
          <div class="pref-item">
            <label for="sampling-strategy">
              Sampling Strategy for Large Files
              <i class="ph ph-info help-icon" title="How to sample data from billion-row files" aria-label="Help: How to sample data from billion-row files"></i>
            </label>
            <select id="sampling-strategy" aria-describedby="sampling-strategy-info">
              <option value="fast" ${(config.sampling_strategy ?? 'balanced') === 'fast' ? 'selected' : ''}>
                Fast (samples from start - fastest, may be biased)
              </option>
              <option value="balanced" ${(config.sampling_strategy ?? 'balanced') === 'balanced' ? 'selected' : ''}>
                Balanced (stratified sampling - recommended)
              </option>
              <option value="accurate" disabled>
                Accurate (reservoir sampling - Phase 2)
              </option>
            </select>
            <div id="sampling-strategy-info" class="sampling-strategy-info" role="status" aria-live="polite"></div>
          </div>
        </div>

        <div class="pref-item">
          <label>Auto-archive processed files</label>
          <input type="checkbox" checked disabled>
        </div>
        <div class="pref-item">
          <label>Memory Limit (streaming threshold)</label>
          <input type="text" value="2.0 GB" disabled>
        </div>
      </div>

      <div class="settings-section">
        <h3><i class="ph ph-robot"></i> AI Assistant</h3>
        <p class="section-description">Configure AI-powered data analysis support</p>

        <div class="pref-item">
          <label for="ai-enabled">
            Enable AI Assistant
            <i class="ph ph-info help-icon" title="Enable ChatGPT integration for in-app help" aria-label="Help: Enable ChatGPT integration"></i>
          </label>
          <input type="checkbox" id="ai-enabled" ${config.ai_config?.enabled ? 'checked' : ''}>
        </div>

        <div class="pref-item">
          <label for="ai-api-key">
            OpenAI API Key
            <i class="ph ph-info help-icon" title="Your OpenAI API key (stored securely in system keyring)" aria-label="Help: OpenAI API key"></i>
          </label>
          <div class="api-key-input-group">
            <input type="password" id="ai-api-key" placeholder="sk-..." autocomplete="off">
            <button id="btn-toggle-api-key" class="btn-icon" type="button" aria-label="Show/hide API key">
              <i class="ph ph-eye"></i>
            </button>
          </div>
          <div class="api-key-actions">
            <button id="btn-save-api-key" class="btn-secondary btn-small">Save Key</button>
            <button id="btn-test-ai" class="btn-secondary btn-small">Test Connection</button>
            <button id="btn-delete-api-key" class="btn-danger btn-small">Clear Key</button>
          </div>
          <div id="ai-status" class="ai-status-message" role="status" aria-live="polite"></div>
        </div>

        <div class="pref-item">
          <label for="ai-model">
            Model
            <i class="ph ph-info help-icon" title="OpenAI model to use" aria-label="Help: OpenAI model"></i>
          </label>
          <select id="ai-model">
            <option value="gpt-4o-mini" ${(config.ai_config?.model ?? 'gpt-4o-mini') === 'gpt-4o-mini' ? 'selected' : ''}>
              GPT-4o Mini (recommended - fast & affordable)
            </option>
            <option value="gpt-4o" ${(config.ai_config?.model ?? 'gpt-4o-mini') === 'gpt-4o' ? 'selected' : ''}>
              GPT-4o (flagship model)
            </option>
            <option value="gpt-4.1" ${(config.ai_config?.model ?? 'gpt-4o-mini') === 'gpt-4.1' ? 'selected' : ''}>
              GPT-4.1 (latest, best performance)
            </option>
            <option value="gpt-4-turbo" ${(config.ai_config?.model ?? 'gpt-4o-mini') === 'gpt-4-turbo' ? 'selected' : ''}>
              GPT-4 Turbo (legacy)
            </option>
            <option value="gpt-3.5-turbo" ${(config.ai_config?.model ?? 'gpt-4o-mini') === 'gpt-3.5-turbo' ? 'selected' : ''}>
              GPT-3.5 Turbo (legacy, cheapest)
            </option>
          </select>
        </div>

        <div class="pref-item">
          <label for="ai-temperature">
            Temperature
            <i class="ph ph-info help-icon" title="Controls randomness (0 = focused, 2 = creative)" aria-label="Help: Temperature"></i>
          </label>
          <input type="number" id="ai-temperature"
            min="0" max="2" step="0.1"
            value="${config.ai_config?.temperature ?? 0.7}">
        </div>

        <div class="pref-item">
          <label for="ai-max-tokens">
            Max Response Tokens
            <i class="ph ph-info help-icon" title="Maximum length of AI responses" aria-label="Help: Max tokens"></i>
          </label>
          <input type="number" id="ai-max-tokens"
            min="100" max="4000" step="100"
            value="${config.ai_config?.max_tokens ?? 1000}">
        </div>
      </div>
    </div>
  `;
}
