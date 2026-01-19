import { invoke } from '@tauri-apps/api/core';

import { AppState } from '../types';

import { Component, ComponentActions } from './Component';

interface AIMessage {
  role: 'user' | 'assistant';
  content: string;
  timestamp: Date;
}

interface AIConfig {
  enabled: boolean;
  model: string;
  temperature: number;
  max_tokens: number;
}

export class AIAssistantComponent extends Component {
  private messages: AIMessage[] = [];
  private isEnabled = false;
  private currentContext: string | null = null;

  constructor(containerId: string, actions: ComponentActions) {
    super(containerId, actions);
    void this.loadConfig();
  }

  private async loadConfig(): Promise<void> {
    try {
      const config = await invoke<AIConfig>('ai_get_config');
      const hasKey = await invoke<boolean>('ai_has_api_key');
      // AI is only enabled if both the config is enabled AND an API key is configured
      this.isEnabled = config.enabled && hasKey;
      this.updateUIState();
    } catch (error) {
      console.error('Failed to load AI config:', error);
      this.isEnabled = false;
      this.updateUIState();
    }
  }

  render(state: AppState): void {
    const container = this.getContainer();
    container.innerHTML = `
      <div class="ai-assistant-sidebar" data-testid="ai-assistant">
        <div class="ai-sidebar-header" id="ai-sidebar-header">
          <div class="ai-title">
            <i class="ph ph-robot"></i>
            <span>AI Assistant</span>
          </div>
          <div class="ai-header-controls">
            <div class="ai-status-compact" data-testid="ai-status">
              <span class="status-indicator-small"></span>
            </div>
            <button class="ai-collapse-btn" id="ai-collapse-btn" title="Collapse sidebar">
              <i class="ph ph-caret-right"></i>
            </button>
          </div>
        </div>

        <div class="ai-messages-sidebar" data-testid="ai-messages">
          <div class="welcome-message-sidebar">
            <p>👋 Hi! I'm your AI assistant.</p>
            <p class="sidebar-help-text">I can help with:</p>
            <ul class="sidebar-help-list">
              <li>Data analysis questions</li>
              <li>Statistical explanations</li>
              <li>Transform suggestions</li>
            </ul>
            <p class="config-note-sidebar">Enable in Settings →</p>
          </div>
        </div>

        <div class="ai-input-sidebar" data-testid="ai-input-area">
          <textarea
            class="ai-input-compact"
            data-testid="ai-input"
            placeholder="Ask about your data..."
            rows="2"
            disabled
          ></textarea>
          <div class="ai-controls-sidebar">
            <button
              class="btn-send-sidebar"
              data-testid="ai-send-button"
              title="Send message"
              disabled
            >
              <i class="ph ph-paper-plane-tilt"></i>
            </button>
            <button
              class="btn-clear-sidebar"
              data-testid="ai-clear-button"
              title="Clear chat"
            >
              <i class="ph ph-trash"></i>
            </button>
          </div>
        </div>
      </div>
    `;

    this.bindEvents(state);
    this.updateUIState();
    this.updateContext(state);
  }

  override bindEvents(_state: AppState): void {
    const container = this.getContainer();
    const input = container.querySelector('.ai-input-compact') as HTMLTextAreaElement;
    const sendButton = container.querySelector('.btn-send-sidebar') as HTMLButtonElement;
    const clearButton = container.querySelector('.btn-clear-sidebar') as HTMLButtonElement;

    if (sendButton) {
      sendButton.addEventListener('click', () => {
        void this.sendMessage();
      });
    }

    if (clearButton) {
      clearButton.addEventListener('click', () => {
        this.clearChat();
      });
    }

    if (input) {
      input.addEventListener('keydown', (e: KeyboardEvent) => {
        if (e.key === 'Enter' && !e.shiftKey) {
          e.preventDefault();
          void this.sendMessage();
        }
      });
    }
  }

  private updateUIState(): void {
    const container = this.getContainer();
    const input = container.querySelector('.ai-input-compact') as HTMLTextAreaElement;
    const sendButton = container.querySelector('.btn-send-sidebar') as HTMLButtonElement;
    const statusIndicator = container.querySelector('.status-indicator-small');

    if (this.isEnabled) {
      if (input) input.disabled = false;
      if (sendButton) sendButton.disabled = false;
      if (statusIndicator) statusIndicator.classList.add('active');
    } else {
      if (input) input.disabled = true;
      if (sendButton) sendButton.disabled = true;
      if (statusIndicator) statusIndicator.classList.remove('active');
    }
  }

  private async sendMessage(): Promise<void> {
    const container = this.getContainer();
    const input = container.querySelector('.ai-input-compact') as HTMLTextAreaElement;
    const query = input.value.trim();

    if (!query) return;

    // Add user message
    this.addMessage('user', query);
    input.value = '';

    // Show loading state
    this.showLoading(true);

    try {
      // Send query to backend with current context
      const response = await invoke<string>('ai_send_query', {
        query,
        context: this.currentContext ?? undefined,
      });

      // Add assistant response
      this.addMessage('assistant', response);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      this.addMessage(
        'assistant',
        `❌ Error: ${errorMessage}\n\nPlease check your API key and configuration in Settings.`
      );
    } finally {
      this.showLoading(false);
    }
  }

  private addMessage(role: 'user' | 'assistant', content: string): void {
    const message: AIMessage = {
      role,
      content,
      timestamp: new Date(),
    };

    this.messages.push(message);
    this.renderMessages();
  }

  private renderMessages(): void {
    const container = this.getContainer();
    const messagesContainer = container.querySelector('.ai-messages-sidebar');
    if (!messagesContainer) return;

    // Clear welcome message if this is the first real message
    if (this.messages.length > 0) {
      const welcomeMessage = messagesContainer.querySelector('.welcome-message-sidebar');
      if (welcomeMessage) {
        welcomeMessage.remove();
      }
    }

    // Render all messages
    messagesContainer.innerHTML = this.messages
      .map(
        msg => `
      <div class="ai-message-sidebar ${msg.role}" data-testid="ai-message-${msg.role}">
        <div class="message-header-sidebar">
          <span class="message-role-sidebar">${msg.role === 'user' ? '👤' : '🤖'}</span>
          <span class="message-time-sidebar">${this.formatTime(msg.timestamp)}</span>
        </div>
        <div class="message-content-sidebar">${this.formatContent(msg.content)}</div>
      </div>
    `
      )
      .join('');

    // Scroll to bottom
    messagesContainer.scrollTop = messagesContainer.scrollHeight;
  }

  private formatContent(content: string): string {
    // Basic markdown support
    return content
      .replace(/```(\w+)?\n([\s\S]*?)```/g, '<pre><code>$2</code></pre>')
      .replace(/`([^`]+)`/g, '<code>$1</code>')
      .replace(
        /\[([^[\]]+)\]\(([^)]+)\)/g,
        (match, p1, p2) => `<a href="${p2}" target="_blank" rel="noopener noreferrer">${p1}</a>`
      )
      .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
      .replace(/\*([^*]+)\*/g, '<em>$1</em>')
      .replace(/\n/g, '<br>');
  }

  private formatTime(date: Date): string {
    return date.toLocaleTimeString('en-US', {
      hour: '2-digit',
      minute: '2-digit',
    });
  }

  private showLoading(show: boolean): void {
    const container = this.getContainer();
    const sendButton = container.querySelector('.btn-send-sidebar') as HTMLButtonElement;

    if (sendButton) {
      sendButton.disabled = show;
      sendButton.innerHTML = show
        ? '<i class="ph ph-spinner"></i>'
        : '<i class="ph ph-paper-plane-tilt"></i>';
    }

    if (show) {
      const messagesContainer = container.querySelector('.ai-messages-sidebar');
      if (messagesContainer) {
        const loadingDiv = document.createElement('div');
        loadingDiv.className = 'ai-message-sidebar assistant loading';
        loadingDiv.innerHTML = `
          <div class="message-header-sidebar">
            <span class="message-role-sidebar">🤖</span>
          </div>
          <div class="message-content-sidebar">
            <span class="loading-dots">●●●</span>
          </div>
        `;
        messagesContainer.appendChild(loadingDiv);
        messagesContainer.scrollTop = messagesContainer.scrollHeight;
      }
    } else {
      const loadingMessage = container.querySelector('.ai-message-sidebar.loading');
      if (loadingMessage) {
        loadingMessage.remove();
      }
    }
  }

  private clearChat(): void {
    this.messages = [];
    this.render({} as AppState);
    this.updateUIState();
  }

  public updateContext(state: AppState): void {
    if (!state.analysisResponse) {
      this.currentContext = null;
      return;
    }

    // Build context from current analysis
    const context = {
      fileName: state.analysisResponse.file_name,
      rowCount: state.analysisResponse.row_count,
      columnCount: state.analysisResponse.column_count,
      columns: state.analysisResponse.summary.slice(0, 20).map(col => ({
        name: col.name,
        type: col.kind,
        nullCount: col.nulls,
        nullPercent: ((col.nulls / state.analysisResponse!.row_count) * 100).toFixed(1),
      })),
    };

    this.currentContext = JSON.stringify(context, null, 2);
  }

  public enable(): void {
    this.isEnabled = true;
    this.updateUIState();
  }

  public disable(): void {
    this.isEnabled = false;
    this.updateUIState();
  }
}
