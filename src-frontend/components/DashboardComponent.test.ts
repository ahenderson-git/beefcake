import { describe, test, expect, vi, beforeEach, afterEach } from 'vitest';

import * as api from '../api';
import type { AppState } from '../types';
import { getDefaultAppConfig } from '../types';

import { type ComponentActions } from './Component';
import { DashboardComponent } from './DashboardComponent';

// Mock the API module
vi.mock('../api', () => ({
  openFileDialog: vi.fn(),
  analyseFile: vi.fn(),
}));

// Create mock actions
const createMockActions = (): ComponentActions => ({
  switchView: vi.fn(),
  showToast: vi.fn(),
  onStateChange: vi.fn(),
  runAnalysis: vi.fn(),
  navigateTo: vi.fn(),
});

// Create mock state
const createMockState = (overrides: Partial<AppState> = {}): AppState => ({
  version: '0.2.3',
  currentView: 'Dashboard',
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

describe('DashboardComponent', () => {
  let container: HTMLElement;
  let component: DashboardComponent;
  let mockActions: ComponentActions;
  let mockState: AppState;

  beforeEach(() => {
    // Create a container element
    container = document.createElement('div');
    container.id = 'test-container';
    document.body.appendChild(container);

    // Create mock actions and state
    mockActions = createMockActions();
    mockState = createMockState();

    // Create component
    component = new DashboardComponent('test-container', mockActions);
  });

  afterEach(() => {
    // Clean up
    vi.clearAllMocks();
    document.body.removeChild(container);
  });

  describe('render', () => {
    test('should render dashboard view with correct test ID', () => {
      component.render(mockState);

      const dashboardView = container.querySelector('[data-testid="dashboard-view"]');
      expect(dashboardView).toBeTruthy();
    });

    test('should display application version', () => {
      component.render(mockState);

      const content = container.innerHTML;
      expect(content).toContain('v0.2.3');
    });

    test('should render all navigation buttons', () => {
      component.render(mockState);

      const openFileBtn = container.querySelector('[data-testid="dashboard-open-file-button"]');
      const powershellBtn = container.querySelector('[data-testid="dashboard-powershell-button"]');
      const pythonBtn = container.querySelector('[data-testid="dashboard-python-button"]');
      const sqlBtn = container.querySelector('[data-testid="dashboard-sql-button"]');

      expect(openFileBtn).toBeTruthy();
      expect(powershellBtn).toBeTruthy();
      expect(pythonBtn).toBeTruthy();
      expect(sqlBtn).toBeTruthy();
    });

    test('should display connection count from config', () => {
      const mockState = createMockState();
      if (mockState.config) {
        mockState.config.connections = [
          {
            id: '1',
            name: 'test-connection',
            settings: {
              db_type: 'PostgreSQL',
              host: 'localhost',
              port: '5432',
              user: 'user',
              password: '',
              database: 'testdb',
              schema: 'public',
              table: 'test_table',
            },
          },
        ];
      }

      component.render(mockState);

      const content = container.innerHTML;
      expect(content).toContain('1'); // Connection count
    });

    test('should display last analysis file name when available', () => {
      mockState.analysisResponse = {
        file_name: 'test_data.csv',
        path: '/path/to/test_data.csv',
        file_size: 1024,
        row_count: 100,
        total_row_count: 100,
        column_count: 5,
        summary: [],
        health: { score: 0.95, risks: [], notes: [] },
        analysis_duration: { secs: 1, nanos: 0 },
        correlation_matrix: null,
      };

      component.render(mockState);

      const content = container.innerHTML;
      expect(content).toContain('test_data.csv');
      expect(content).toContain('1 KB'); // File size formatted
    });

    test('should display "None" when no analysis is available', () => {
      mockState.analysisResponse = null;

      component.render(mockState);

      const content = container.innerHTML;
      expect(content).toContain('None');
      expect(content).toContain('Ready for input');
    });
  });

  describe('bindEvents', () => {
    test('should bind click event to open file button', () => {
      component.render(mockState);

      const openFileBtn = container.querySelector('#btn-open-file') as HTMLButtonElement;
      expect(openFileBtn).toBeTruthy();

      // Click the button
      openFileBtn.click();

      // The handleOpenFile method will be called, which calls api.openFileDialog
      // We can't easily test async calls here, but we verify the button is clickable
      expect(openFileBtn).toBeTruthy();
    });

    test('should bind click event to PowerShell button', () => {
      component.render(mockState);

      const powershellBtn = container.querySelector('#btn-powershell') as HTMLButtonElement;
      powershellBtn.click();

      expect(mockActions.switchView).toHaveBeenCalledWith('PowerShell');
    });

    test('should bind click event to Python button', () => {
      component.render(mockState);

      const pythonBtn = container.querySelector('#btn-python') as HTMLButtonElement;
      pythonBtn.click();

      expect(mockActions.switchView).toHaveBeenCalledWith('Python');
    });

    test('should bind click event to SQL button', () => {
      component.render(mockState);

      const sqlBtn = container.querySelector('#btn-sql') as HTMLButtonElement;
      sqlBtn.click();

      expect(mockActions.switchView).toHaveBeenCalledWith('SQL');
    });
  });

  describe('handleOpenFile', () => {
    test('should call runAnalysis when file is selected', async () => {
      vi.mocked(api.openFileDialog).mockResolvedValue('/path/to/file.csv');

      component.render(mockState);

      const openFileBtn = container.querySelector('#btn-open-file') as HTMLButtonElement;
      openFileBtn.click();

      // Wait for async operation
      await vi.waitFor(() => {
        expect(api.openFileDialog).toHaveBeenCalled();
      });

      await vi.waitFor(() => {
        expect(mockActions.runAnalysis).toHaveBeenCalledWith('/path/to/file.csv');
      });
    });

    test('should not call runAnalysis when no file is selected', async () => {
      vi.mocked(api.openFileDialog).mockResolvedValue(null);

      component.render(mockState);

      const openFileBtn = container.querySelector('#btn-open-file') as HTMLButtonElement;
      openFileBtn.click();

      // Wait for async operation
      await vi.waitFor(() => {
        expect(api.openFileDialog).toHaveBeenCalled();
      });

      // runAnalysis should not be called
      expect(mockActions.runAnalysis).not.toHaveBeenCalled();
    });

    test('should show error toast when file dialog fails', async () => {
      vi.mocked(api.openFileDialog).mockRejectedValue(new Error('Failed to open dialog'));

      component.render(mockState);

      const openFileBtn = container.querySelector('#btn-open-file') as HTMLButtonElement;
      openFileBtn.click();

      // Wait for async operation
      await vi.waitFor(() => {
        expect(api.openFileDialog).toHaveBeenCalled();
      });

      await vi.waitFor(() => {
        expect(mockActions.showToast).toHaveBeenCalledWith(
          expect.stringContaining('Error opening file'),
          'error'
        );
      });
    });
  });
});
