import { describe, test, expect } from 'vitest';

import { getDefaultColumnCleanConfig, type ColumnSummary } from './types';

describe('types', () => {
  describe('getDefaultColumnCleanConfig', () => {
    test('should create default config with standardized name', () => {
      const col: ColumnSummary = {
        name: 'User Name',
        standardized_name: 'user_name',
        kind: 'Text',
        count: 100,
        nulls: 0,
        stats: {},
        interpretation: [],
        ml_advice: [],
        business_summary: [],
        samples: [],
      };

      const config = getDefaultColumnCleanConfig(col);

      expect(config.new_name).toBe('user_name');
      expect(config.active).toBe(true);
      expect(config.trim_whitespace).toBe(true);
      expect(config.standardise_nulls).toBe(true);
      expect(config.advanced_cleaning).toBe(false);
      expect(config.ml_preprocessing).toBe(false);
    });

    test('should use original name as fallback if no standardized name', () => {
      const col: ColumnSummary = {
        name: 'Email',
        standardized_name: '',
        kind: 'Text',
        count: 100,
        nulls: 0,
        stats: {},
        interpretation: [],
        ml_advice: [],
        business_summary: [],
        samples: [],
      };

      const config = getDefaultColumnCleanConfig(col);

      expect(config.new_name).toBe('Email');
    });

    test('should initialize with safe default values', () => {
      const col: ColumnSummary = {
        name: 'age',
        standardized_name: 'age',
        kind: 'Numeric',
        count: 100,
        nulls: 5,
        stats: {},
        interpretation: [],
        ml_advice: [],
        business_summary: [],
        samples: [],
      };

      const config = getDefaultColumnCleanConfig(col);

      expect(config.target_dtype).toBeNull();
      expect(config.rounding).toBeNull();
      expect(config.freq_threshold).toBeNull();
      expect(config.regex_find).toBe('');
      expect(config.regex_replace).toBe('');
      expect(config.temporal_format).toBe('');
      expect(config.normalisation).toBe('None');
      expect(config.impute_mode).toBe('None');
      expect(config.text_case).toBe('None');
    });

    test('should not enable advanced features by default', () => {
      const col: ColumnSummary = {
        name: 'salary',
        standardized_name: 'salary',
        kind: 'Numeric',
        count: 100,
        nulls: 0,
        stats: {},
        interpretation: [],
        ml_advice: [],
        business_summary: [],
        samples: [],
      };

      const config = getDefaultColumnCleanConfig(col);

      expect(config.extract_numbers).toBe(false);
      expect(config.clip_outliers).toBe(false);
      expect(config.timezone_utc).toBe(false);
      expect(config.one_hot_encode).toBe(false);
      expect(config.remove_special_chars).toBe(false);
      expect(config.remove_non_ascii).toBe(false);
    });
  });
});
