// Note: This file has non-standard import order due to vi.mock() hoisting requirements
/* eslint-disable import/order */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
import { describe, test, expect, vi, beforeEach } from 'vitest';
import type { AnalysisResponse, AppConfig, ColumnCleanConfig } from './types';

// Mock @tauri-apps/api/core
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

// Mock @tauri-apps/plugin-dialog
vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
  save: vi.fn(),
}));

// Import after mocking
import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import * as api from './api';
/* eslint-enable import/order */

describe('API', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('analyseFile', () => {
    test('should call invoke with correct command and parameters', async () => {
      const mockResponse: AnalysisResponse = {
        file_name: 'test.csv',
        path: '/path/to/test.csv',
        file_size: 1024,
        row_count: 100,
        total_row_count: 100,
        column_count: 5,
        summary: [],
        health: { score: 0.95, risks: [], notes: [] },
        analysis_duration: { secs: 1, nanos: 0 },
        correlation_matrix: null,
      };

      vi.mocked(invoke).mockResolvedValue(mockResponse);

      // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
      const result = await api.analyseFile('/path/to/file.csv');

      expect(invoke).toHaveBeenCalledWith('analyze_file', { path: '/path/to/file.csv' });
      expect(result).toEqual(mockResponse);
    });

    test('should handle errors from backend', async () => {
      vi.mocked(invoke).mockRejectedValue('File not found');

      await expect(api.analyseFile('/invalid/path.csv')).rejects.toBe('File not found');
    });
  });

  describe('getAppVersion', () => {
    test('should return version string', async () => {
      vi.mocked(invoke).mockResolvedValue('0.2.0');

      const version = await api.getAppVersion();

      expect(invoke).toHaveBeenCalledWith('get_app_version');
      expect(version).toBe('0.2.0');
    });
  });

  describe('loadAppConfig', () => {
    test('should load configuration from backend', async () => {
      const mockConfig: AppConfig = {
        settings: {
          connections: [],
          active_import_id: null,
          active_export_id: null,
          powershell_font_size: 14,
          python_font_size: 14,
          sql_font_size: 14,
          first_run_completed: false,
          trusted_paths: [],
          preview_row_limit: 100,
          security_warning_acknowledged: false,
          skip_full_row_count: false,
          analysis_sample_size: 10000,
          sampling_strategy: 'first',
          ai_config: {
            enabled: true,
            model: 'gpt-4',
            temperature: 0.7,
            max_tokens: 1000,
          },
        },
        audit_log: {
          entries: [],
        },
      };

      vi.mocked(invoke).mockResolvedValue(mockConfig);

      const config = await api.loadAppConfig();

      expect(invoke).toHaveBeenCalledWith('get_config');
      expect(config).toEqual(mockConfig);
    });
  });

  describe('saveAppConfig', () => {
    test('should save configuration to backend', async () => {
      const mockConfig: AppConfig = {
        settings: {
          connections: [],
          active_import_id: null,
          active_export_id: null,
          powershell_font_size: 12,
          python_font_size: 12,
          sql_font_size: 12,
          first_run_completed: false,
          trusted_paths: [],
          preview_row_limit: 100,
          security_warning_acknowledged: false,
          skip_full_row_count: false,
          analysis_sample_size: 10000,
          sampling_strategy: 'balanced',
          ai_config: {
            enabled: true,
            model: 'gpt-4o',
            temperature: 0.7,
            max_tokens: 2000,
          },
        },
        audit_log: {
          entries: [],
        },
      };

      vi.mocked(invoke).mockResolvedValue(undefined);

      await api.saveAppConfig(mockConfig);

      expect(invoke).toHaveBeenCalledWith('save_config', { config: mockConfig });
    });
  });

  describe('runPowerShell', () => {
    test('should execute PowerShell script', async () => {
      vi.mocked(invoke).mockResolvedValue('Script output');

      const result = await api.runPowerShell('Get-Process');

      expect(invoke).toHaveBeenCalledWith('run_powershell', { script: 'Get-Process' });
      expect(result).toBe('Script output');
    });
  });

  describe('runPython', () => {
    test('should execute Python script without data', async () => {
      vi.mocked(invoke).mockResolvedValue('Hello from Python');

      const result = await api.runPython('print("Hello from Python")');

      expect(invoke).toHaveBeenCalledWith('run_python', {
        script: 'print("Hello from Python")',
        dataPath: undefined,
        configs: undefined,
      });
      expect(result).toBe('Hello from Python');
    });

    test('should execute Python script with data path', async () => {
      vi.mocked(invoke).mockResolvedValue('Data processed');

      const result = await api.runPython('df.head()', '/path/to/data.csv');

      expect(invoke).toHaveBeenCalledWith('run_python', {
        script: 'df.head()',
        dataPath: '/path/to/data.csv',
        configs: undefined,
      });
      expect(result).toBe('Data processed');
    });

    test('should execute Python script with configs', async () => {
      const configs: Record<string, ColumnCleanConfig> = {
        age: {
          new_name: 'age',
          active: true,
          trim_whitespace: true,
          standardise_nulls: true,
          target_dtype: null,
          rounding: null,
          freq_threshold: null,
          regex_find: '',
          regex_replace: '',
          temporal_format: '',
          normalisation: 'None',
          impute_mode: 'None',
          text_case: 'None',
          extract_numbers: false,
          clip_outliers: false,
          timezone_utc: false,
          one_hot_encode: false,
          advanced_cleaning: false,
          ml_preprocessing: false,
          remove_special_chars: false,
          remove_non_ascii: false,
        },
      };

      vi.mocked(invoke).mockResolvedValue('Processed with configs');

      await api.runPython('df.head()', '/path/to/data.csv', configs);

      expect(invoke).toHaveBeenCalledWith('run_python', {
        script: 'df.head()',
        dataPath: '/path/to/data.csv',
        configs,
      });
    });
  });

  describe('runSql', () => {
    test('should execute SQL query', async () => {
      vi.mocked(invoke).mockResolvedValue('Query results');

      //noinspection SqlNoDataSourceInspection,SqlDialectInspection
      const result = await api.runSql('SELECT * FROM data LIMIT 10');

      //noinspection SqlNoDataSourceInspection,SqlDialectInspection
      expect(invoke).toHaveBeenCalledWith('run_sql', {
        query: 'SELECT * FROM data LIMIT 10',
        dataPath: undefined,
        configs: undefined,
      });
      expect(result).toBe('Query results');
    });
  });

  describe('exportData', () => {
    test('should export data with options', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      const options: import('./types').ExportOptions = {
        source: {
          type: 'Analyser',
          path: '/path/to/data.csv',
        },
        configs: {},
        destination: {
          type: 'File',
          target: '/path/to/output.parquet',
          format: 'parquet',
        },
      };

      await api.exportData(options);

      expect(invoke).toHaveBeenCalledWith('export_data', { options });
    });
  });

  describe('abortProcessing', () => {
    test('should abort current processing', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      await api.abortProcessing();

      expect(invoke).toHaveBeenCalledWith('abort_processing');
    });
  });

  describe('resetAbortSignal', () => {
    test('should reset abort signal', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      await api.resetAbortSignal();

      expect(invoke).toHaveBeenCalledWith('reset_abort_signal');
    });
  });

  describe('openFileDialog', () => {
    test('should open file dialog and return selected path', async () => {
      vi.mocked(open).mockResolvedValue('/selected/file.csv');

      const result = await api.openFileDialog();

      expect(open).toHaveBeenCalledWith({
        multiple: false,
        filters: [
          {
            name: 'Data Files',
            extensions: ['csv', 'json', 'parquet'],
          },
        ],
      });
      expect(result).toBe('/selected/file.csv');
    });

    test('should handle cancellation', async () => {
      vi.mocked(open).mockResolvedValue(null);

      const result = await api.openFileDialog();

      expect(result).toBeNull();
    });

    test('should handle array return (multiple=false edge case)', async () => {
      vi.mocked(open).mockResolvedValue(['/file.csv'] as unknown as string);

      const result = await api.openFileDialog();

      // Should return null for non-string types
      expect(result).toBeNull();
    });

    test('should accept custom filters', async () => {
      const customFilters = [{ name: 'CSV Files', extensions: ['csv'] }];

      vi.mocked(open).mockResolvedValue('/file.csv');

      await api.openFileDialog(customFilters);

      expect(open).toHaveBeenCalledWith({
        multiple: false,
        filters: customFilters,
      });
    });
  });

  describe('saveFileDialog', () => {
    test('should open save dialog and return path', async () => {
      vi.mocked(save).mockResolvedValue('/save/location.csv');

      const result = await api.saveFileDialog();

      expect(save).toHaveBeenCalledWith({
        filters: [
          {
            name: 'Data Files',
            extensions: ['csv', 'json', 'parquet'],
          },
        ],
      });
      expect(result).toBe('/save/location.csv');
    });

    test('should handle cancellation', async () => {
      vi.mocked(save).mockResolvedValue(null);

      const result = await api.saveFileDialog();

      expect(result).toBeNull();
    });

    test('should accept custom filters', async () => {
      const customFilters = [{ name: 'Parquet Files', extensions: ['parquet'] }];

      vi.mocked(save).mockResolvedValue('/file.parquet');

      await api.saveFileDialog(customFilters);

      expect(save).toHaveBeenCalledWith({
        filters: customFilters,
      });
    });
  });

  describe('createDataset', () => {
    test('should create dataset and return ID', async () => {
      vi.mocked(invoke).mockResolvedValue('dataset-123');

      const datasetId = await api.createDataset('My Dataset', '/path/to/data.csv');

      expect(invoke).toHaveBeenCalledWith('lifecycle_create_dataset', {
        request: { name: 'My Dataset', source_path: '/path/to/data.csv' },
      });
      expect(datasetId).toBe('dataset-123');
    });
  });

  describe('sanitizeHeaders', () => {
    test('should sanitize column names', async () => {
      vi.mocked(invoke).mockResolvedValue(['user_name', 'email_address', 'age']);

      const result = await api.sanitizeHeaders(['User Name', 'Email-Address', 'age']);

      expect(invoke).toHaveBeenCalledWith('sanitize_headers', {
        names: ['User Name', 'Email-Address', 'age'],
      });
      expect(result).toEqual(['user_name', 'email_address', 'age']);
    });
  });

  describe('applyTransforms', () => {
    test('should apply transforms to dataset', async () => {
      vi.mocked(invoke).mockResolvedValue('version-456');

      const versionId = await api.applyTransforms('dataset-123', '{"steps":[]}', 'Cleaned');

      expect(invoke).toHaveBeenCalledWith('lifecycle_apply_transforms', {
        request: {
          dataset_id: 'dataset-123',
          transforms: [],
          next_stage: 'Cleaned',
        },
      });
      expect(versionId).toBe('version-456');
    });
  });

  describe('setActiveVersion', () => {
    test('should set active version for dataset', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      await api.setActiveVersion('dataset-123', 'version-456');

      expect(invoke).toHaveBeenCalledWith('lifecycle_set_active_version', {
        request: { dataset_id: 'dataset-123', version_id: 'version-456' },
      });
    });
  });

  describe('publishVersion', () => {
    test('should publish version as view', async () => {
      vi.mocked(invoke).mockResolvedValue('published-789');

      const result = await api.publishVersion('dataset-123', 'version-456', 'view');

      expect(invoke).toHaveBeenCalledWith('lifecycle_publish_version', {
        request: { dataset_id: 'dataset-123', version_id: 'version-456', mode: 'view' },
      });
      expect(result).toBe('published-789');
    });

    test('should publish version as snapshot', async () => {
      vi.mocked(invoke).mockResolvedValue('published-789');

      const result = await api.publishVersion('dataset-123', 'version-456', 'snapshot');

      expect(invoke).toHaveBeenCalledWith('lifecycle_publish_version', {
        request: { dataset_id: 'dataset-123', version_id: 'version-456', mode: 'snapshot' },
      });
      expect(result).toBe('published-789');
    });
  });

  describe('getVersionDiff', () => {
    test('should get diff between two versions', async () => {
      const mockDiff = {
        schema_changes: { added: ['new_col'], removed: ['old_col'], renamed: {} },
        row_count_change: { before: 100, after: 95, delta: -5 },
        column_stats_changes: {},
      };

      vi.mocked(invoke).mockResolvedValue(mockDiff);

      const result = await api.getVersionDiff('dataset-123', 'v1', 'v2');

      expect(invoke).toHaveBeenCalledWith('lifecycle_get_version_diff', {
        request: { dataset_id: 'dataset-123', from_version_id: 'v1', to_version_id: 'v2' },
      });
      expect(result).toEqual(mockDiff);
    });
  });

  describe('listVersions', () => {
    test('should list all versions for dataset', async () => {
      vi.mocked(invoke).mockResolvedValue('["v1","v2","v3"]');

      const result = await api.listVersions('dataset-123');

      expect(invoke).toHaveBeenCalledWith('lifecycle_list_versions', {
        request: { dataset_id: 'dataset-123' },
      });
      expect(result).toBe('["v1","v2","v3"]');
    });
  });

  describe('savePipelineSpec', () => {
    test('should save pipeline to file', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      await api.savePipelineSpec('{"name":"test"}', '/path/to/pipeline.json');

      expect(invoke).toHaveBeenCalledWith('save_pipeline_spec', {
        specJson: '{"name":"test"}',
        path: '/path/to/pipeline.json',
      });
    });
  });

  describe('loadPipelineSpec', () => {
    test('should load pipeline from file', async () => {
      vi.mocked(invoke).mockResolvedValue('{"name":"loaded"}');

      const result = await api.loadPipelineSpec('/path/to/pipeline.json');

      expect(invoke).toHaveBeenCalledWith('load_pipeline_spec', {
        path: '/path/to/pipeline.json',
      });
      expect(result).toBe('{"name":"loaded"}');
    });
  });

  describe('validatePipelineSpec', () => {
    test('should validate pipeline with no errors', async () => {
      vi.mocked(invoke).mockResolvedValue([]);

      const errors = await api.validatePipelineSpec('{"steps":[]}', '/input.csv');

      expect(invoke).toHaveBeenCalledWith('validate_pipeline_spec', {
        specJson: '{"steps":[]}',
        inputPath: '/input.csv',
      });
      expect(errors).toEqual([]);
    });

    test('should return validation errors', async () => {
      vi.mocked(invoke).mockResolvedValue(['Column "age" not found', 'Invalid step type']);

      const errors = await api.validatePipelineSpec('{"steps":[]}', '/input.csv');

      expect(errors).toHaveLength(2);
      expect(errors[0]).toBe('Column "age" not found');
    });
  });

  describe('generatePowerShell', () => {
    test('should generate PowerShell script', async () => {
      vi.mocked(invoke).mockResolvedValue('# PowerShell script content');

      const result = await api.generatePowerShell('{"steps":[]}', '/output.csv');

      expect(invoke).toHaveBeenCalledWith('generate_powershell', {
        specJson: '{"steps":[]}',
        outputPath: '/output.csv',
      });
      expect(result).toContain('PowerShell');
    });
  });

  describe('pipelineFromConfigs', () => {
    test('should create pipeline from clean configs', async () => {
      vi.mocked(invoke).mockResolvedValue('{"name":"auto_pipeline"}');

      const result = await api.pipelineFromConfigs(
        'Auto Pipeline',
        '{"age":{}}',
        'csv',
        '/output.csv'
      );

      expect(invoke).toHaveBeenCalledWith('pipeline_from_configs', {
        name: 'Auto Pipeline',
        configsJson: '{"age":{}}',
        inputFormat: 'csv',
        outputPath: '/output.csv',
      });
      expect(result).toBe('{"name":"auto_pipeline"}');
    });
  });

  describe('watcher API', () => {
    test('should get watcher state', async () => {
      const mockState = { enabled: true, folder: '/watch', last_event: null };
      vi.mocked(invoke).mockResolvedValue(mockState);

      const result = await api.watcherGetState();

      expect(invoke).toHaveBeenCalledWith('watcher_get_state');
      expect(result).toEqual(mockState);
    });

    test('should start watcher', async () => {
      const mockState = { enabled: true, folder: '/watch', last_event: null };
      vi.mocked(invoke).mockResolvedValue(mockState);

      const result = await api.watcherStart('/watch');

      expect(invoke).toHaveBeenCalledWith('watcher_start', { folder: '/watch' });
      expect(result).toEqual(mockState);
    });

    test('should stop watcher', async () => {
      const mockState = { enabled: false, folder: '', last_event: null };
      vi.mocked(invoke).mockResolvedValue(mockState);

      const result = await api.watcherStop();

      expect(invoke).toHaveBeenCalledWith('watcher_stop');
      expect(result).toEqual(mockState);
    });

    test('should set watcher folder', async () => {
      const mockState = { enabled: true, folder: '/new-watch', last_event: null };
      vi.mocked(invoke).mockResolvedValue(mockState);

      const result = await api.watcherSetFolder('/new-watch');

      expect(invoke).toHaveBeenCalledWith('watcher_set_folder', { folder: '/new-watch' });
      expect(result).toEqual(mockState);
    });

    test('should ingest file manually', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      await api.watcherIngestNow('/file.csv');

      expect(invoke).toHaveBeenCalledWith('watcher_ingest_now', { path: '/file.csv' });
    });
  });

  describe('dictionary API', () => {
    test('should load snapshot', async () => {
      const mockDict = { snapshot_id: 's1', columns: [] };
      vi.mocked(invoke).mockResolvedValue(mockDict);

      const result = await api.dictionaryLoadSnapshot('s1');

      expect(invoke).toHaveBeenCalledWith('dictionary_load_snapshot', { snapshotId: 's1' });
      expect(result).toEqual(mockDict);
    });

    test('should list snapshots without filter', async () => {
      const mockSnapshots = [{ id: 's1', timestamp: '2025-01-01' }];
      vi.mocked(invoke).mockResolvedValue(mockSnapshots);

      const result = await api.dictionaryListSnapshots();

      expect(invoke).toHaveBeenCalledWith('dictionary_list_snapshots', { datasetHash: undefined });
      expect(result).toEqual(mockSnapshots);
    });

    test('should list snapshots with filter', async () => {
      const mockSnapshots = [{ id: 's1', timestamp: '2025-01-01' }];
      vi.mocked(invoke).mockResolvedValue(mockSnapshots);

      const result = await api.dictionaryListSnapshots('hash123');

      expect(invoke).toHaveBeenCalledWith('dictionary_list_snapshots', { datasetHash: 'hash123' });
      expect(result).toEqual(mockSnapshots);
    });

    test('should update business metadata', async () => {
      vi.mocked(invoke).mockResolvedValue('Updated successfully');

      const datasetMeta: import('./types').DatasetBusinessMetadata = {
        description: 'Q1 Sales',
        tags: [],
      };
      const columnMeta = {
        age: {
          business_definition: 'Customer age',
          approved_examples: [],
        },
      };

      const result = await api.dictionaryUpdateBusinessMetadata('s1', datasetMeta, columnMeta);

      expect(invoke).toHaveBeenCalledWith('dictionary_update_business_metadata', {
        request: {
          snapshot_id: 's1',
          dataset_business: datasetMeta,
          column_business_updates: columnMeta,
        },
      });
      expect(result).toBe('Updated successfully');
    });

    test('should export markdown', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      await api.dictionaryExportMarkdown('s1', '/output.md');

      expect(invoke).toHaveBeenCalledWith('dictionary_export_markdown', {
        snapshotId: 's1',
        outputPath: '/output.md',
      });
    });
  });

  describe('documentation API', () => {
    test('should list documentation files', async () => {
      const mockFiles = [
        { path: 'README.md', name: 'README', size: 1024 },
        { path: 'GUIDE.md', name: 'GUIDE', size: 2048 },
      ];
      vi.mocked(invoke).mockResolvedValue(mockFiles);

      const result = await api.listDocumentationFiles();

      expect(invoke).toHaveBeenCalledWith('list_documentation_files');
      expect(result).toEqual(mockFiles);
    });

    test('should read documentation file', async () => {
      vi.mocked(invoke).mockResolvedValue('# Documentation\n\nContent here...');

      const result = await api.readDocumentationFile('README.md');

      expect(invoke).toHaveBeenCalledWith('read_documentation_file', { docPath: 'README.md' });
      expect(result).toContain('Documentation');
    });
  });

  describe('readTextFile', () => {
    test('should read text file', async () => {
      vi.mocked(invoke).mockResolvedValue('File contents here');

      const result = await api.readTextFile('/path/to/file.txt');

      expect(invoke).toHaveBeenCalledWith('read_text_file', { path: '/path/to/file.txt' });
      expect(result).toBe('File contents here');
    });
  });

  describe('writeTextFile', () => {
    test('should write text file', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      await api.writeTextFile('/path/to/file.txt', 'New contents');

      expect(invoke).toHaveBeenCalledWith('write_text_file', {
        path: '/path/to/file.txt',
        contents: 'New contents',
      });
    });
  });

  describe('installPythonPackage', () => {
    test('should install Python package', async () => {
      vi.mocked(invoke).mockResolvedValue('Successfully installed pandas');

      const result = await api.installPythonPackage('pandas');

      expect(invoke).toHaveBeenCalledWith('install_python_package', { package: 'pandas' });
      expect(result).toContain('pandas');
    });
  });

  describe('pushToDb', () => {
    test('should push data to database', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      const configs: Record<string, import('./types').ColumnCleanConfig> = {
        age: {
          new_name: 'age',
          target_dtype: null,
          active: true,
          advanced_cleaning: false,
          ml_preprocessing: false,
          trim_whitespace: false,
          remove_special_chars: false,
          text_case: 'None',
          standardise_nulls: false,
          remove_non_ascii: false,
          regex_find: '',
          regex_replace: '',
          rounding: null,
          extract_numbers: false,
          clip_outliers: false,
          temporal_format: '',
          timezone_utc: false,
          freq_threshold: null,
          normalisation: 'None',
          one_hot_encode: false,
          impute_mode: 'None',
        },
      };

      await api.pushToDb('/data.csv', 'conn-123', configs);

      expect(invoke).toHaveBeenCalledWith('push_to_db', {
        path: '/data.csv',
        connectionId: 'conn-123',
        configs,
      });
    });
  });

  describe('testConnection', () => {
    test('should test database connection', async () => {
      vi.mocked(invoke).mockResolvedValue('Connection successful');

      const settings = {
        db_type: 'postgres',
        host: 'localhost',
        port: '5432',
        user: 'testuser',
        password: 'testpass',
        database: 'test',
        schema: 'public',
        table: 'data',
      };

      const result = await api.testConnection(settings);

      expect(invoke).toHaveBeenCalledWith('test_connection', {
        settings,
        connectionId: undefined,
      });
      expect(result).toBe('Connection successful');
    });

    test('should test connection with connection ID', async () => {
      vi.mocked(invoke).mockResolvedValue('Connection successful');

      const settings = {
        db_type: 'postgres',
        host: 'localhost',
        port: '5432',
        user: 'testuser',
        password: 'testpass',
        database: 'test',
        schema: 'public',
        table: 'data',
      };

      await api.testConnection(settings, 'conn-123');

      expect(invoke).toHaveBeenCalledWith('test_connection', {
        settings,
        connectionId: 'conn-123',
      });
    });
  });

  describe('deleteConnection', () => {
    test('should delete database connection', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      await api.deleteConnection('conn-123');

      expect(invoke).toHaveBeenCalledWith('delete_connection', { id: 'conn-123' });
    });
  });

  describe('error handling', () => {
    test('should propagate errors from backend', async () => {
      vi.mocked(invoke).mockRejectedValue('Database connection failed');

      await expect(api.loadAppConfig()).rejects.toBe('Database connection failed');
    });

    test('should handle network timeouts', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Timeout'));

      await expect(api.analyseFile('/file.csv')).rejects.toThrow('Timeout');
    });
  });
});
