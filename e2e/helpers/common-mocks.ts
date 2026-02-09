/**
 * Common Tauri command mocks for E2E tests
 *
 * This file provides standard mock responses for frequently-used Tauri commands
 * to reduce duplication across test files.
 */

import { MockResponse } from './tauri-mock';

/**
 * Standard logging command mocks to prevent infinite loops
 *
 * The app overrides console.log/error/warn to also call Tauri logging commands.
 * Without these mocks, any console output creates an infinite loop.
 */
export const loggingMocks: Record<string, MockResponse> = {
  log_frontend_event: {
    type: 'success',
    data: null,
  },
  log_frontend_error: {
    type: 'success',
    data: null,
  },
};

/**
 * Standard app initialization mocks
 *
 * These are required for the app to start up properly.
 */
export const appInitMocks: Record<string, MockResponse> = {
  get_app_version: {
    type: 'success',
    data: '0.2.3',
  },
  get_config: {
    type: 'success',
    data: {
      settings: {
        connections: [],
        active_import_id: null,
        active_export_id: null,
        powershell_font_size: 14,
        python_font_size: 14,
        sql_font_size: 14,
        first_run_completed: true,
        trusted_paths: [],
        preview_row_limit: 100,
        security_warning_acknowledged: true,
        skip_full_row_count: false,
        analysis_sample_size: 10000,
        sampling_strategy: 'balanced',
        ai_config: {
          enabled: false,
          model: 'gpt-4o-mini',
          temperature: 0.7,
          max_tokens: 2000,
        },
      },
      audit_log: {
        entries: [], // Correct structure: object with entries array, not flat array
      },
    },
  },
  check_python_environment: {
    type: 'success',
    data: 'Polars 1.0.0 installed',
  },
  watcher_get_state: {
    type: 'success',
    data: {
      is_running: false,
      watched_folder: null,
      activity_history: [],
    },
  },
  // AI Assistant config (optional, but prevents console errors)
  ai_get_config: {
    type: 'success',
    data: {
      enabled: false,
    },
  },
};

/**
 * Mock dataset with lifecycle versions for testing
 */
export const mockDatasetWithVersions = {
  id: 'dataset-test-123',
  name: 'customer_data.csv',
  versions: [
    {
      id: 'v-raw-001',
      dataset_id: 'dataset-test-123',
      stage: 'Raw',
      parent_id: null,
      pipeline: {
        transforms: [],
      },
      data_location: {
        ParquetFile: '/data/raw.parquet',
      },
      metadata: {
        description: 'Raw data version',
        tags: [],
        row_count: 1000,
        column_count: 10,
        file_size_bytes: 102400,
        created_by: 'test-user',
        custom_fields: {},
      },
      created_at: '2025-01-26T10:00:00Z',
    },
    {
      id: 'v-profiled-001',
      dataset_id: 'dataset-test-123',
      stage: 'Profiled',
      parent_id: 'v-raw-001',
      pipeline: {
        transforms: [],
      },
      data_location: {
        ParquetFile: '/data/profiled.parquet',
      },
      metadata: {
        description: 'Profiled data version',
        tags: [],
        row_count: 1000,
        column_count: 10,
        file_size_bytes: 102400,
        created_by: 'test-user',
        custom_fields: {},
      },
      created_at: '2025-01-26T10:05:00Z',
    },
    {
      id: 'v-cleaned-001',
      dataset_id: 'dataset-test-123',
      stage: 'Cleaned',
      parent_id: 'v-profiled-001',
      pipeline: {
        transforms: [],
      },
      data_location: {
        ParquetFile: '/data/cleaned.parquet',
      },
      metadata: {
        description: 'Cleaned data version',
        tags: [],
        row_count: 995,
        column_count: 10,
        file_size_bytes: 101888,
        created_by: 'test-user',
        custom_fields: {},
      },
      created_at: '2025-01-26T10:10:00Z',
    },
  ],
  active_version_id: 'v-cleaned-001',
};

/**
 * Mock version schema (column metadata)
 */
export const mockVersionSchema = [
  { name: 'customer_id', dtype: 'Int64', null_count: 0 },
  { name: 'customer_name', dtype: 'String', null_count: 5 },
  { name: 'email', dtype: 'String', null_count: 3 },
  { name: 'age', dtype: 'Int64', null_count: 10 },
  { name: 'order_date', dtype: 'Date', null_count: 0 },
  { name: 'total_spent', dtype: 'Float64', null_count: 2 },
  { name: 'loyalty_tier', dtype: 'String', null_count: 0 },
  { name: 'region', dtype: 'String', null_count: 0 },
  { name: 'signup_date', dtype: 'Date', null_count: 1 },
  { name: 'is_active', dtype: 'Boolean', null_count: 0 },
];

/**
 * Create stateful lifecycle mocks
 *
 * Creates fresh mock functions for each test to avoid state pollution.
 * Call this function in your test's beforeEach to get clean mocks.
 */
export function createLifecycleMocks(): Record<
  string,
  MockResponse | ((args: unknown) => MockResponse | Promise<MockResponse>)
> {
  // State to track which versions have been created (per test)
  let listVersionsCallCount = 0;

  return {
    lifecycle_get_dataset: {
      type: 'success',
      data: mockDatasetWithVersions,
    },
    lifecycle_list_versions: () => {
      listVersionsCallCount++;

      // First call: return only Raw version (after create_dataset)
      if (listVersionsCallCount === 1) {
        return {
          type: 'success',
          data: JSON.stringify([mockDatasetWithVersions.versions[0]]),
        };
      }

      // Second call: return Raw + Profiled versions (after apply_transforms for Profiled)
      if (listVersionsCallCount === 2) {
        return {
          type: 'success',
          data: JSON.stringify([
            mockDatasetWithVersions.versions[0],
            mockDatasetWithVersions.versions[1],
          ]),
        };
      }

      // Subsequent calls: return all versions
      return {
        type: 'success',
        data: JSON.stringify(mockDatasetWithVersions.versions),
      };
    },
    lifecycle_get_version: {
      type: 'success',
      data: mockDatasetWithVersions.versions[2], // Return Cleaned version by default
    },
    lifecycle_get_version_schema: {
      type: 'success',
      data: mockVersionSchema,
    },
    lifecycle_apply_transforms: (args: unknown) => {
      // Return appropriate version ID based on the target stage
      const request = (args as { request?: { next_stage?: string } })?.request;
      const stage = request?.next_stage;

      if (stage === 'Profiled') {
        return {
          type: 'success',
          data: 'v-profiled-001',
        };
      } else if (stage === 'Cleaned') {
        return {
          type: 'success',
          data: 'v-cleaned-001',
        };
      } else if (stage === 'Advanced') {
        return {
          type: 'success',
          data: 'v-advanced-001',
        };
      }

      // Default fallback
      return {
        type: 'success',
        data: 'v-unknown-001',
      };
    },
    lifecycle_set_active_version: {
      type: 'success',
      data: null,
    },
    lifecycle_publish_version: {
      type: 'success',
      data: {
        version_id: 'v-published-001',
        stage: 'Published',
      },
    },
    lifecycle_get_version_diff: {
      type: 'success',
      data: {
        schema_changes: {
          added_columns: ['normalized_age', 'encoded_tier'],
          removed_columns: [],
          renamed_columns: {},
          type_changes: {},
        },
        quality_changes: {
          null_count_improvements: {
            customer_name: { before: 5, after: 0 },
            email: { before: 3, after: 0 },
          },
          row_count_change: 0,
        },
      },
    },
    lifecycle_create_dataset: {
      type: 'success',
      data: mockDatasetWithVersions.id, // Return dataset ID string, not full object
    },
  };
}

/**
 * Lifecycle command mocks (legacy - use createLifecycleMocks() for stateful behavior)
 */
export const lifecycleMocks = createLifecycleMocks();

/**
 * Combine standard mocks with custom ones
 *
 * @param customMocks - Additional mocks specific to the test
 * @returns Combined mock configuration
 */
export function getStandardMocks(
  customMocks: Record<
    string,
    MockResponse | ((args: unknown) => MockResponse | Promise<MockResponse>)
  > = {}
): Record<string, MockResponse | ((args: unknown) => MockResponse | Promise<MockResponse>)> {
  return {
    ...appInitMocks,
    ...loggingMocks,
    ...customMocks,
  };
}

/**
 * Mock analysis response for file analysis tests
 * Matches the AnalysisResponse interface from types/analysis.ts
 */
export const mockAnalysisResponse = {
  file_name: 'customer_data.csv',
  path: 'C:\\Users\\test\\data\\customer_data.csv',
  file_size: 1024,
  row_count: 10,
  total_row_count: 10,
  column_count: 10,
  analysis_duration: { secs: 0, nanos: 150000000 },
  health: {
    score: 0.85,
    risks: ['Some columns have missing values'],
    notes: [],
  },
  correlation_matrix: null,
  summary: [
    {
      name: 'customer_id',
      standardized_name: 'customer_id',
      kind: 'Numeric',
      count: 10,
      nulls: 0,
      stats: {
        Numeric: {
          mean: 5.5,
          median: 5.5,
          min: 1.0,
          max: 10.0,
          std_dev: 3.03,
          q1: 3.0,
          q3: 8.0,
          distinct_count: 10,
          zero_count: 0,
          negative_count: 0,
          is_integer: true,
          is_sorted: true,
          is_sorted_rev: false,
          skew: 0.0,
        },
      },
      interpretation: [],
      ml_advice: [],
      business_summary: ['Unique identifier column'],
      samples: ['1', '2', '3', '4', '5'],
    },
    {
      name: 'customer_name',
      standardized_name: 'customer_name',
      kind: 'Text',
      count: 10,
      nulls: 0,
      stats: {
        Text: {
          min_length: 8,
          max_length: 15,
          avg_length: 11.5,
          distinct: 10,
          top_value: ['John Smith', 1],
        },
      },
      interpretation: [],
      ml_advice: [],
      business_summary: ['Text column with names'],
      samples: ['John Smith', 'Jane Doe', 'Bob Johnson', 'Alice Williams', 'Charlie Brown'],
    },
    {
      name: 'email',
      standardized_name: 'email',
      kind: 'Text',
      count: 10,
      nulls: 0,
      stats: {
        Text: {
          min_length: 15,
          max_length: 20,
          avg_length: 17.5,
          distinct: 10,
          top_value: ['john@example.com', 1],
        },
      },
      interpretation: [],
      ml_advice: [],
      business_summary: ['Email addresses'],
      samples: ['john@example.com', 'jane@example.com', 'bob@example.com'],
    },
    {
      name: 'age',
      standardized_name: 'age',
      kind: 'Numeric',
      count: 10,
      nulls: 0,
      stats: {
        Numeric: {
          mean: 38.6,
          median: 36.5,
          min: 28.0,
          max: 55.0,
          std_dev: 8.5,
          q1: 32.0,
          q3: 45.0,
          distinct_count: 10,
          zero_count: 0,
          negative_count: 0,
          is_integer: true,
          is_sorted: false,
          is_sorted_rev: false,
          skew: 0.2,
        },
      },
      interpretation: [],
      ml_advice: [],
      business_summary: ['Age column'],
      samples: ['35', '28', '42', '31', '55'],
    },
    {
      name: 'order_date',
      standardized_name: 'order_date',
      kind: 'Temporal',
      count: 10,
      nulls: 0,
      stats: {
        Temporal: {
          min: '2025-01-15',
          max: '2025-01-25',
          distinct_count: 10,
        },
      },
      interpretation: [],
      ml_advice: [],
      business_summary: ['Order dates'],
      samples: ['2025-01-15', '2025-01-16', '2025-01-17'],
    },
    {
      name: 'total_spent',
      standardized_name: 'total_spent',
      kind: 'Numeric',
      count: 10,
      nulls: 0,
      stats: {
        Numeric: {
          mean: 1598.25,
          median: 1350.375,
          min: 650.75,
          max: 3200.0,
          std_dev: 845.2,
          q1: 950.0,
          q3: 2200.0,
          distinct_count: 10,
          zero_count: 0,
          negative_count: 0,
          is_integer: false,
          is_sorted: false,
          is_sorted_rev: false,
          skew: 0.3,
        },
      },
      interpretation: [],
      ml_advice: [],
      business_summary: ['Total amount spent'],
      samples: ['1250.50', '890.25', '2100.00'],
    },
    {
      name: 'loyalty_tier',
      standardized_name: 'loyalty_tier',
      kind: 'Categorical',
      count: 10,
      nulls: 0,
      stats: {
        Categorical: {
          distinct_count: 4,
          most_common: 'Gold',
          value_counts: [
            ['Gold', 3],
            ['Silver', 3],
            ['Platinum', 2],
            ['Bronze', 2],
          ],
        },
      },
      interpretation: [],
      ml_advice: [],
      business_summary: ['Customer loyalty tier'],
      samples: ['Gold', 'Silver', 'Platinum', 'Bronze'],
    },
    {
      name: 'region',
      standardized_name: 'region',
      kind: 'Categorical',
      count: 10,
      nulls: 0,
      stats: {
        Categorical: {
          distinct_count: 4,
          most_common: 'North',
          value_counts: [
            ['North', 3],
            ['South', 3],
            ['East', 2],
            ['West', 2],
          ],
        },
      },
      interpretation: [],
      ml_advice: [],
      business_summary: ['Geographic region'],
      samples: ['North', 'South', 'East', 'West'],
    },
    {
      name: 'signup_date',
      standardized_name: 'signup_date',
      kind: 'Temporal',
      count: 10,
      nulls: 0,
      stats: {
        Temporal: {
          min: '2023-12-01',
          max: '2024-02-15',
          distinct_count: 10,
        },
      },
      interpretation: [],
      ml_advice: [],
      business_summary: ['Customer signup date'],
      samples: ['2024-01-10', '2024-02-15', '2023-12-01'],
    },
    {
      name: 'is_active',
      standardized_name: 'is_active',
      kind: 'Categorical',
      count: 10,
      nulls: 0,
      stats: {
        Categorical: {
          distinct_count: 1,
          most_common: 'true',
          value_counts: [['true', 10]],
        },
      },
      interpretation: [],
      ml_advice: [],
      business_summary: ['Active status'],
      samples: ['true'],
    },
  ],
};

/**
 * File analysis command mocks
 */
export const fileAnalysisMocks: Record<string, MockResponse> = {
  analyze_file: {
    type: 'success',
    data: mockAnalysisResponse,
  },
};

/**
 * Get mocks with lifecycle support enabled
 *
 * @param customMocks - Additional mocks specific to the test
 * @returns Combined mock configuration with lifecycle mocks
 */
export function getLifecycleMocks(
  customMocks: Record<
    string,
    MockResponse | ((args: unknown) => MockResponse | Promise<MockResponse>)
  > = {}
): Record<string, MockResponse | ((args: unknown) => MockResponse | Promise<MockResponse>)> {
  return {
    ...appInitMocks,
    ...loggingMocks,
    ...createLifecycleMocks(), // Use function to get fresh state per test
    ...customMocks,
  };
}

/**
 * Get mocks with file analysis support enabled
 *
 * @param customMocks - Additional mocks specific to the test
 * @returns Combined mock configuration with file analysis mocks
 */
export function getFileAnalysisMocks(
  customMocks: Record<
    string,
    MockResponse | ((args: unknown) => MockResponse | Promise<MockResponse>)
  > = {}
): Record<string, MockResponse | ((args: unknown) => MockResponse | Promise<MockResponse>)> {
  return {
    ...appInitMocks,
    ...loggingMocks,
    ...fileAnalysisMocks,
    ...createLifecycleMocks(), // Use function to get fresh state per test
    ...customMocks,
  };
}

/**
 * Helper to create a default cleaning config for a column
 * Matches the structure created by getDefaultColumnCleanConfig() in types/config.ts
 */
export function createMockCleaningConfig(columnName: string) {
  return {
    new_name: columnName,
    target_dtype: null,
    active: true,
    advanced_cleaning: false,
    ml_preprocessing: false,
    trim_whitespace: true,
    remove_special_chars: false,
    text_case: 'None',
    standardise_nulls: true,
    remove_non_ascii: false,
    extract_numbers: false,
    regex_find: '',
    regex_replace: '',
    temporal_format: '',
    rounding: null,
    clip_outliers: false,
    normalisation: 'None',
    one_hot_encode: false,
    freq_threshold: null,
    impute_mode: 'None',
    timezone_utc: false,
  };
}
