import { describe, test, expect, vi, beforeEach } from 'vitest';

import type { AppState } from '../types';

import { Component, type ComponentActions } from './Component';

// Mock implementation of Component for testing
class TestComponent extends Component {
  renderCalled = false;

  render(state: AppState): void {
    this.renderCalled = true;
    // Simple render implementation for testing
    const container = this.getContainer();
    container.innerHTML = `<div>Rendered with ${state.currentView}</div>`;
  }
}

// Create mock actions
const createMockActions = (): ComponentActions => ({
  switchView: vi.fn(),
  showToast: vi.fn(),
  onStateChange: vi.fn(),
  runAnalysis: vi.fn(),
  navigateTo: vi.fn(),
});

// Create mock state
const createMockState = (): AppState => ({
  version: '0.2.0',
  currentView: 'Dashboard',
  analysisResponse: null,
  config: {
    connections: [],
    active_import_id: null,
    active_export_id: null,
    powershell_font_size: 14,
    python_font_size: 14,
    sql_font_size: 14,
    analysis_sample_size: 10000,
    sampling_strategy: 'first',
    ai_config: {
      enabled: false,
      model: 'gpt-4',
      temperature: 0.7,
      max_tokens: 1000,
    },
    audit_log: [],
  },
  expandedRows: new Set<string>(),
  cleaningConfigs: {},
  isAddingConnection: false,
  isLoading: false,
  isAborting: false,
  loadingMessage: '',
  pythonScript: null,
  sqlScript: null,
  pythonSkipCleaning: false,
  sqlSkipCleaning: false,
  currentDataset: null,
  selectedColumns: new Set<string>(),
  useOriginalColumnNames: false,
  cleanAllActive: false,
  advancedProcessingEnabled: false,
  watcherState: {
    enabled: false,
    folder: '',
    state: 'idle',
  },
  watcherActivities: [],
  selectedVersionId: null,
  currentIdeColumns: null,
  previousVersionId: null,
});

describe('Component', () => {
  beforeEach(() => {
    // Setup DOM container
    document.body.innerHTML = '<div id="test-container"></div>';
  });

  describe('constructor', () => {
    test('should find and store container element', () => {
      const actions = createMockActions();
      const component = new TestComponent('test-container', actions);

      expect(component['container']).not.toBeNull();
      expect(component['container']?.id).toBe('test-container');
    });

    test('should handle missing container gracefully', () => {
      const actions = createMockActions();
      // Create component with non-existent container ID
      const component = new TestComponent('non-existent', actions);

      expect(component['container']).toBeNull();
    });

    test('should store actions reference', () => {
      const actions = createMockActions();
      const component = new TestComponent('test-container', actions);

      expect(component['actions']).toBe(actions);
    });
  });

  describe('getContainer', () => {
    test('should return container element if found', () => {
      const actions = createMockActions();
      const component = new TestComponent('test-container', actions);

      const container = component['getContainer']();
      expect(container).toBe(document.getElementById('test-container'));
    });

    test('should throw error if container not found', () => {
      const actions = createMockActions();
      const component = new TestComponent('non-existent', actions);

      expect(() => component['getContainer']()).toThrow('Container not found');
    });
  });

  describe('render', () => {
    test('should be callable on subclass', () => {
      const actions = createMockActions();
      const component = new TestComponent('test-container', actions);
      const state = createMockState();

      component.render(state);

      expect(component.renderCalled).toBe(true);
    });

    test('should update DOM through getContainer', () => {
      const actions = createMockActions();
      const component = new TestComponent('test-container', actions);
      const state = createMockState();
      state.currentView = 'Analyser';

      component.render(state);

      const container = document.getElementById('test-container');
      expect(container?.innerHTML).toContain('Rendered with Analyser');
    });
  });

  describe('bindEvents', () => {
    test('should be callable with default no-op implementation', () => {
      const actions = createMockActions();
      const component = new TestComponent('test-container', actions);
      const state = createMockState();

      // Should not throw
      expect(() => component.bindEvents(state)).not.toThrow();
    });
  });

  describe('integration', () => {
    test('should work with component lifecycle', () => {
      const actions = createMockActions();
      const component = new TestComponent('test-container', actions);
      const state = createMockState();

      // Render component
      component.render(state);
      expect(component.renderCalled).toBe(true);

      // Bind events (no-op in base)
      component.bindEvents(state);

      // Verify container is updated
      const container = component['getContainer']();
      expect(container.innerHTML).toBeTruthy();
    });

    test('should use actions for callbacks', () => {
      const actions = createMockActions();
      const component = new TestComponent('test-container', actions);

      // Test that actions are accessible
      component['actions'].showToast('Test message', 'info');
      expect(actions.showToast).toHaveBeenCalledWith('Test message', 'info');

      component['actions'].switchView('Analyser');
      expect(actions.switchView).toHaveBeenCalledWith('Analyser');
    });
  });
});
