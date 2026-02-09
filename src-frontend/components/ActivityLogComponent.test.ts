/* eslint-disable @typescript-eslint/no-explicit-any */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
import { describe, test, expect, vi, beforeEach, afterEach } from 'vitest';

import * as api from '../api';
import type { AppState, AppConfig } from '../types';
import { getDefaultAppConfig } from '../types';

import { ActivityLogComponent } from './ActivityLogComponent';
import { type ComponentActions } from './Component';

// Mock the API module
vi.mock('../api', () => ({
  saveAppConfig: vi.fn(),
  logFrontendEvent: vi.fn(),
}));

// Create mock actions
const createMockActions = (): ComponentActions => ({
  switchView: vi.fn(),
  showToast: vi.fn(),
  onStateChange: vi.fn(),
  runAnalysis: vi.fn(),
  navigateTo: vi.fn(),
});

// Create mock state with audit log entries
const createMockState = (overrides: Partial<AppState> = {}): AppState => ({
  version: '0.2.3',
  currentView: 'ActivityLog',
  analysisResponse: null,
  config: getDefaultAppConfig(),
  expandedRows: new Set<string>(),
  cleaningConfigs: {},
  isAddingConnection: false,
  isLoading: false,
  isAborting: false,
  isCreatingLifecycle: false,
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
  ...overrides,
});

describe('ActivityLogComponent', () => {
  let container: HTMLDivElement;
  let component: ActivityLogComponent;
  let mockActions: ComponentActions;

  beforeEach(() => {
    // Set up DOM using Vitest's built-in environment
    container = document.createElement('div');
    container.id = 'test-container';
    document.body.appendChild(container);

    mockActions = createMockActions();
    component = new ActivityLogComponent('test-container', mockActions);

    // Reset mocks
    vi.clearAllMocks();
  });

  afterEach(() => {
    // Clean up DOM
    document.body.removeChild(container);
  });

  describe('render', () => {
    test('should render activity log view with correct structure', () => {
      const state = createMockState();
      component.render(state);

      const activityView = container.querySelector('.activity-view');
      expect(activityView).toBeTruthy();

      const header = container.querySelector('.activity-header');
      expect(header).toBeTruthy();

      const clearButton = container.querySelector('#btn-clear-log');
      expect(clearButton).toBeTruthy();
      expect(clearButton?.textContent).toContain('Clear History');

      const activityList = container.querySelector('.activity-list');
      expect(activityList).toBeTruthy();
    });

    test('should display empty message when no audit log entries', () => {
      const state = createMockState();
      component.render(state);

      const emptyMsg = container.querySelector('.empty-msg');
      expect(emptyMsg).toBeTruthy();
      expect(emptyMsg?.textContent).toContain('No recent activity');
    });

    test('should render audit log entries', () => {
      const config: AppConfig = {
        ...getDefaultAppConfig(),
        audit_log: {
          entries: [
            {
              timestamp: '2025-01-27T10:00:00Z',
              action: 'Toast',
              details: '[success] Analysis complete (Analyser)',
            },
            {
              timestamp: '2025-01-27T10:05:00Z',
              action: 'Database',
              details: 'Connected to PostgreSQL',
            },
            {
              timestamp: '2025-01-27T10:10:00Z',
              action: 'Export',
              details: 'Exported data to CSV',
            },
          ],
        },
      };

      const state = createMockState({ config });
      component.render(state);

      const entries = container.querySelectorAll('.activity-entry');
      expect(entries).toHaveLength(3);

      // Entries should be in reverse order (newest first)
      const firstEntry = entries[0]!;
      expect(firstEntry.textContent).toContain('Export');
      expect(firstEntry.textContent).toContain('Exported data to CSV');

      const lastEntry = entries[2]!;
      expect(lastEntry.textContent).toContain('Toast');
      expect(lastEntry.textContent).toContain('Analysis complete');
    });

    test('should display correct icons for different actions', () => {
      const config: AppConfig = {
        ...getDefaultAppConfig(),
        audit_log: {
          entries: [
            { timestamp: '2025-01-27T10:00:00Z', action: 'Database', details: 'DB action' },
            { timestamp: '2025-01-27T10:01:00Z', action: 'Export', details: 'Export action' },
            { timestamp: '2025-01-27T10:02:00Z', action: 'Toast', details: 'Toast action' },
          ],
        },
      };

      const state = createMockState({ config });
      component.render(state);

      const icons = container.querySelectorAll('.entry-icon i');

      // Reverse order: Toast, Export, Database
      expect(icons[0]?.className).toContain('ph-info'); // Toast
      expect(icons[1]?.className).toContain('ph-export'); // Export
      expect(icons[2]?.className).toContain('ph-database'); // Database
    });

    test('should display action and timestamp metadata', () => {
      const config: AppConfig = {
        ...getDefaultAppConfig(),
        audit_log: {
          entries: [
            {
              timestamp: '2025-01-27T15:30:00Z',
              action: 'Analysis',
              details: 'File analyzed successfully',
            },
          ],
        },
      };

      const state = createMockState({ config });
      component.render(state);

      const action = container.querySelector('.entry-action');
      expect(action?.textContent).toBe('Analysis');

      const time = container.querySelector('.entry-time');
      expect(time?.textContent).toBe('2025-01-27T15:30:00Z');
    });
  });

  describe('bindEvents', () => {
    test('should clear audit log when clear button clicked', async () => {
      const config: AppConfig = {
        ...getDefaultAppConfig(),
        audit_log: {
          entries: [{ timestamp: '2025-01-27T10:00:00Z', action: 'Toast', details: 'Test entry' }],
        },
      };

      const state = createMockState({ config });
      component.render(state);

      vi.mocked(api.saveAppConfig).mockResolvedValue(undefined);

      const clearButton = container.querySelector('#btn-clear-log') as HTMLButtonElement;
      expect(clearButton).toBeTruthy();

      clearButton?.click();

      // Wait for async operations
      await vi.waitFor(() => {
        expect(api.saveAppConfig).toHaveBeenCalledWith(
          expect.objectContaining({
            audit_log: {
              entries: [],
            },
          })
        );
      });

      expect(mockActions.onStateChange).toHaveBeenCalled();
    });

    test('should handle clearing already empty audit log', async () => {
      const state = createMockState();
      component.render(state);

      vi.mocked(api.saveAppConfig).mockResolvedValue(undefined);

      const clearButton = container.querySelector('#btn-clear-log') as HTMLButtonElement;
      clearButton?.click();

      await vi.waitFor(() => {
        expect(api.saveAppConfig).toHaveBeenCalledWith(
          expect.objectContaining({
            audit_log: {
              entries: [],
            },
          })
        );
      });
    });

    test('should not throw if config is undefined', () => {
      const state = createMockState({ config: undefined as any });

      expect(() => {
        component.render(state);
      }).not.toThrow();

      const clearButton = container.querySelector('#btn-clear-log') as HTMLButtonElement;

      expect(() => {
        clearButton?.click();
      }).not.toThrow();

      expect(api.saveAppConfig).not.toHaveBeenCalled();
    });
  });

  describe('audit_log structure validation', () => {
    test('should work with correct audit_log.entries structure', () => {
      const config: AppConfig = {
        ...getDefaultAppConfig(),
        audit_log: {
          entries: [{ timestamp: '2025-01-27T10:00:00Z', action: 'Test', details: 'Test details' }],
        },
      };

      const state = createMockState({ config });

      expect(() => {
        component.render(state);
      }).not.toThrow();

      const entries = container.querySelectorAll('.activity-entry');
      expect(entries).toHaveLength(1);
    });

    test('should handle missing entries property gracefully', () => {
      // Simulate config with wrong structure (for backward compatibility check)
      const config = {
        ...getDefaultAppConfig(),
        audit_log: {} as any, // Missing entries
      };

      const state = createMockState({ config });

      expect(() => {
        component.render(state);
      }).not.toThrow();

      const emptyMsg = container.querySelector('.empty-msg');
      expect(emptyMsg?.textContent).toContain('No recent activity');
    });
  });
});
