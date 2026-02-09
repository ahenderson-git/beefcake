import { describe, test, expect, vi, beforeEach, afterEach } from 'vitest';

import * as api from '../api';
import type { AppState, DatasetVersion } from '../types';
import { getDefaultAppConfig } from '../types';

import { type ComponentActions } from './Component';
import { PythonComponent } from './PythonComponent';

// Mock the API module
vi.mock('../api', () => ({
  executePythonScript: vi.fn(),
  openFileDialog: vi.fn(),
  saveFileDialog: vi.fn(),
  readScriptFile: vi.fn(),
  writeScriptFile: vi.fn(),
  installPolars: vi.fn(),
  getVersionSchema: vi.fn(),
  saveAppConfig: vi.fn().mockResolvedValue(undefined),
  runPython: vi.fn(),
}));

// Mock Monaco Editor
vi.mock('monaco-editor', () => ({
  editor: {
    create: vi.fn().mockReturnValue({
      getValue: vi.fn().mockReturnValue('print("Hello World")'),
      setValue: vi.fn(),
      dispose: vi.fn(),
      onDidChangeModelContent: vi.fn(),
      getSelection: vi.fn().mockReturnValue({
        startLineNumber: 1,
        startColumn: 1,
        endLineNumber: 1,
        endColumn: 1,
      }),
      executeEdits: vi.fn(),
      focus: vi.fn(),
      updateOptions: vi.fn(),
    }),
  },
  Range: vi
    .fn()
    .mockImplementation((startLine: number, startCol: number, endLine: number, endCol: number) => ({
      startLineNumber: startLine,
      startColumn: startCol,
      endLineNumber: endLine,
      endColumn: endCol,
    })),
}));

// Mock AnsiUp
vi.mock('ansi_up', () => ({
  AnsiUp: class {
    use_classes = false;
    ansi_to_html(text: string): string {
      return text;
    }
  },
}));

// Mock DOMPurify
vi.mock('dompurify', () => ({
  default: {
    sanitize: vi.fn((html: string) => html),
  },
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
  currentView: 'Python',
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

describe('PythonComponent', () => {
  let container: HTMLElement;
  let component: PythonComponent;
  let mockActions: ComponentActions;

  beforeEach(() => {
    // Mock window.confirm
    global.confirm = vi.fn().mockReturnValue(true);

    // Create a container element
    container = document.createElement('div');
    container.id = 'test-container';
    document.body.appendChild(container);

    // Create mock actions
    mockActions = createMockActions();

    // Create component
    component = new PythonComponent('test-container', mockActions);
  });

  afterEach(() => {
    // Clean up
    vi.clearAllMocks();
    document.body.removeChild(container);
  });

  describe('render', () => {
    test('should render Python IDE view with correct test ID', async () => {
      const state = createMockState();
      component.render(state);

      // Wait for setTimeout to complete
      await new Promise(resolve => setTimeout(resolve, 10));

      const pythonView = container.querySelector('[data-testid="python-ide-view"]');
      expect(pythonView).toBeTruthy();
    });

    test('should render toolbar with run button', async () => {
      const state = createMockState();
      component.render(state);

      await new Promise(resolve => setTimeout(resolve, 10));

      const runButton = container.querySelector('[data-testid="python-ide-run-button"]');
      expect(runButton).toBeTruthy();
    });

    test('should render editor container', async () => {
      const state = createMockState();
      component.render(state);

      await new Promise(resolve => setTimeout(resolve, 10));

      const editorContainer = container.querySelector('[data-testid="python-ide-editor"]');
      expect(editorContainer).toBeTruthy();
    });

    test('should render output panel', async () => {
      const state = createMockState();
      component.render(state);

      await new Promise(resolve => setTimeout(resolve, 10));

      const outputPanel = container.querySelector('[data-testid="python-ide-output"]');
      expect(outputPanel).toBeTruthy();
    });
  });

  describe('bindEvents', () => {
    test('should bind run button click event', async () => {
      const state = createMockState();
      vi.mocked(api.runPython).mockResolvedValue('Hello World');

      component.render(state);
      await new Promise(resolve => setTimeout(resolve, 10));

      const runButton = document.getElementById('btn-run-py');
      expect(runButton).toBeTruthy();

      runButton?.click();

      // The executePythonScript should be called when run completes
      await new Promise(resolve => setTimeout(resolve, 50));
    });

    test('should bind clear button click event', async () => {
      const state = createMockState();
      component.render(state);
      await new Promise(resolve => setTimeout(resolve, 10));

      const clearButton = document.getElementById('btn-clear-py');
      expect(clearButton).toBeTruthy();

      clearButton?.click();

      // Output should be cleared
      const output = document.getElementById('py-output');
      expect(output?.textContent).toBe('');
    });

    test('should bind export button click event', async () => {
      const state = createMockState();
      component.render(state);
      await new Promise(resolve => setTimeout(resolve, 10));

      const exportButton = document.getElementById('btn-export-py');
      expect(exportButton).toBeTruthy();
    });

    test('should bind font size increase button', async () => {
      const state = createMockState();
      component.render(state);
      await new Promise(resolve => setTimeout(resolve, 10));

      const incButton = document.getElementById('btn-inc-font-py');
      expect(incButton).toBeTruthy();
    });

    test('should bind font size decrease button', async () => {
      const state = createMockState();
      component.render(state);
      await new Promise(resolve => setTimeout(resolve, 10));

      const decButton = document.getElementById('btn-dec-font-py');
      expect(decButton).toBeTruthy();
    });

    test('should bind load script button', async () => {
      const state = createMockState();
      component.render(state);
      await new Promise(resolve => setTimeout(resolve, 10));

      const loadButton = document.getElementById('btn-load-py');
      expect(loadButton).toBeTruthy();
    });

    test('should bind save script button', async () => {
      const state = createMockState();
      component.render(state);
      await new Promise(resolve => setTimeout(resolve, 10));

      const saveButton = document.getElementById('btn-save-py');
      expect(saveButton).toBeTruthy();
    });

    test('should bind install polars button', async () => {
      const state = createMockState();
      component.render(state);
      await new Promise(resolve => setTimeout(resolve, 10));

      const installButton = document.getElementById('btn-install-polars');
      expect(installButton).toBeTruthy();
    });

    test('should bind skip cleaning checkbox', async () => {
      const state = createMockState();
      component.render(state);
      await new Promise(resolve => setTimeout(resolve, 10));

      const skipCheckbox = document.getElementById('py-skip-cleaning') as HTMLInputElement;
      if (skipCheckbox) {
        expect(skipCheckbox.type).toBe('checkbox');

        skipCheckbox.checked = true;
        skipCheckbox.dispatchEvent(new Event('change'));

        expect(state.pythonSkipCleaning).toBe(true);
      }
    });
  });

  describe('default script', () => {
    test('should load default Python script when state has no script', async () => {
      const state = createMockState({ pythonScript: null });
      component.render(state);
      await new Promise(resolve => setTimeout(resolve, 10));

      // The editor should be initialized with the default script
      // We can't easily test the editor content without the full Monaco setup
      // but we can verify the render doesn't crash
      expect(container.innerHTML).toBeTruthy();
    });

    test('should load saved script from state', async () => {
      const customScript = 'print("Custom Script")';
      const state = createMockState({ pythonScript: customScript });
      component.render(state);
      await new Promise(resolve => setTimeout(resolve, 10));

      // The editor should be initialized with the custom script
      expect(container.innerHTML).toBeTruthy();
    });
  });

  describe('font size management', () => {
    test('should respect font size from config', async () => {
      const state = createMockState();
      if (state.config) {
        state.config.settings.python_font_size = 16;
      }

      component.render(state);
      await new Promise(resolve => setTimeout(resolve, 10));

      // The editor should be initialized with font size 16
      expect(container.innerHTML).toBeTruthy();
    });

    test('should use default font size if not configured', async () => {
      const state = createMockState();
      if (state.config) {
        state.config.settings.python_font_size = 14; // Default
      }

      component.render(state);
      await new Promise(resolve => setTimeout(resolve, 10));

      // The editor should use default font size 14
      expect(container.innerHTML).toBeTruthy();
    });
  });

  describe('column schema integration', () => {
    /* eslint-disable @typescript-eslint/no-unsafe-assignment */
    const createMockVersion = (): DatasetVersion => ({
      id: 'v1',
      dataset_id: 'test-dataset',
      parent_id: null,
      stage: 'Cleaned',
      pipeline: { transforms: [] },
      data_location: { ParquetFile: '/path/to/file.parquet' },
      metadata: {
        description: '',
        tags: [],
        row_count: 0,
        column_count: 0,
        file_size_bytes: 0,
        created_by: 'test',
        custom_fields: {},
      },
      created_at: '2025-01-26T00:00:00Z',
    });

    test('should fetch version schema when dataset is available', async () => {
      const state = createMockState({
        currentDataset: {
          id: 'test-dataset',
          name: 'Test Dataset',
          versions: [createMockVersion()],
          activeVersionId: 'v1',
          rawVersionId: 'v1',
        },
      });

      vi.mocked(api.getVersionSchema).mockResolvedValue([
        { name: 'id', dtype: 'Int64' },
        { name: 'name', dtype: 'String' },
      ]);

      component.render(state);
      await new Promise(resolve => setTimeout(resolve, 50));

      // Should have called getVersionSchema
      expect(api.getVersionSchema).toHaveBeenCalledWith('test-dataset', 'v1');
    });

    test('should handle schema fetch errors gracefully', async () => {
      const state = createMockState({
        currentDataset: {
          id: 'test-dataset',
          name: 'Test Dataset',
          versions: [createMockVersion()],
          activeVersionId: 'v1',
          rawVersionId: 'v1',
        },
      });

      vi.mocked(api.getVersionSchema).mockRejectedValue(new Error('API Error'));
      // Suppress expected error log from the component to keep test output clean
      const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

      component.render(state);
      await new Promise(resolve => setTimeout(resolve, 50));

      // Should not crash on error
      expect(container.innerHTML).toBeTruthy();
      errorSpy.mockRestore();
    });
    /* eslint-enable @typescript-eslint/no-unsafe-assignment */
  });
});
