import { describe, it, expect } from 'vitest';

import { ColumnSummary } from '../../types';

import { getUniqueCount, renderDistribution } from './row';

describe('getUniqueCount', () => {
  it('should return distinct_count for Numeric columns', () => {
    const col: ColumnSummary = {
      name: 'age',
      standardized_name: 'age',
      kind: 'Numeric',
      count: 100,
      nulls: 0,
      stats: {
        Numeric: {
          distinct_count: 45,
          min: 18,
          max: 65,
          p05: 20,
          q1: 27,
          median: 34,
          mean: 35.2,
          trimmed_mean: 35.2,
          q3: 45,
          p95: 60,
          std_dev: 12.5,
          skew: 0.15,
          zero_count: 0,
          negative_count: 0,
          is_integer: true,
          is_sorted: false,
          is_sorted_rev: false,
          bin_width: 5,
          histogram: null,
        },
      },
      interpretation: [],
      ml_advice: [],
      business_summary: [],
      samples: [],
    };

    expect(getUniqueCount(col)).toBe(45);
  });

  it('should return distinct for Text columns', () => {
    const col: ColumnSummary = {
      name: 'name',
      standardized_name: 'name',
      kind: 'Text',
      count: 100,
      nulls: 5,
      stats: {
        Text: {
          distinct: 95,
          top_value: ['John', 5],
          min_length: 3,
          max_length: 15,
          avg_length: 8.5,
        },
      },
      interpretation: [],
      ml_advice: [],
      business_summary: [],
      samples: [],
    };

    expect(getUniqueCount(col)).toBe(95);
  });

  it('should return Object.keys length for Categorical columns', () => {
    const col: ColumnSummary = {
      name: 'category',
      standardized_name: 'category',
      kind: 'Categorical',
      count: 100,
      nulls: 0,
      stats: {
        Categorical: {
          'Category A': 50,
          'Category B': 30,
          'Category C': 20,
        },
      },
      interpretation: [],
      ml_advice: [],
      business_summary: [],
      samples: [],
    };

    expect(getUniqueCount(col)).toBe(3);
  });

  it('should return count of distinct boolean values', () => {
    const col: ColumnSummary = {
      name: 'is_active',
      standardized_name: 'is_active',
      kind: 'Boolean',
      count: 100,
      nulls: 0,
      stats: {
        Boolean: {
          true_count: 60,
          false_count: 40,
        },
      },
      interpretation: [],
      ml_advice: [],
      business_summary: [],
      samples: [],
    };

    expect(getUniqueCount(col)).toBe(2);
  });

  it('should return 1 when Boolean has only true values', () => {
    const col: ColumnSummary = {
      name: 'is_active',
      standardized_name: 'is_active',
      kind: 'Boolean',
      count: 100,
      nulls: 0,
      stats: {
        Boolean: {
          true_count: 100,
          false_count: 0,
        },
      },
      interpretation: [],
      ml_advice: [],
      business_summary: [],
      samples: [],
    };

    expect(getUniqueCount(col)).toBe(1);
  });

  it('should return 0 as fallback for unknown column types', () => {
    const col: ColumnSummary = {
      name: 'unknown',
      standardized_name: 'unknown',
      kind: 'Unknown',
      count: 100,
      nulls: 0,
      stats: {},
      interpretation: [],
      ml_advice: [],
      business_summary: [],
      samples: [],
    };

    expect(getUniqueCount(col)).toBe(0);
  });
});

describe('renderDistribution', () => {
  it('should render numeric histogram with 2-tuple format [bin_centre, count]', () => {
    const col: ColumnSummary = {
      name: 'age',
      standardized_name: 'age',
      kind: 'Numeric',
      count: 100,
      nulls: 0,
      stats: {
        Numeric: {
          distinct_count: 45,
          min: 18,
          max: 65,
          p05: 20,
          q1: 27,
          median: 34,
          mean: 35.2,
          trimmed_mean: 35.2,
          q3: 45,
          p95: 60,
          std_dev: 12.5,
          skew: 0.15,
          zero_count: 0,
          negative_count: 0,
          is_integer: true,
          is_sorted: false,
          is_sorted_rev: false,
          bin_width: 5,
          histogram: [
            [25.5, 10], // [bin_centre, count]
            [35.5, 20],
            [45.5, 15],
          ],
        },
      },
      interpretation: [],
      ml_advice: [],
      business_summary: [],
      samples: [],
    };

    const result = renderDistribution(col);

    // Verify structure
    expect(result).toContain('distribution-chart');
    expect(result).toContain('Distribution');
    expect(result).toContain('histogram');

    // Verify counts are rendered (max count is 20)
    expect(result).toContain('20');

    // Verify bin centres are rendered
    expect(result).toContain('25.50'); // binCentre.toFixed(2)
    expect(result).toContain('35.50');
    expect(result).toContain('45.50');

    // Verify no undefined in output
    expect(result).not.toContain('undefined');
    expect(result).not.toContain('NaN');
  });

  it('should handle empty histogram gracefully', () => {
    const col: ColumnSummary = {
      name: 'age',
      standardized_name: 'age',
      kind: 'Numeric',
      count: 100,
      nulls: 0,
      stats: {
        Numeric: {
          distinct_count: 0,
          min: null,
          max: null,
          p05: null,
          q1: null,
          median: null,
          mean: null,
          trimmed_mean: null,
          q3: null,
          p95: null,
          std_dev: null,
          skew: null,
          zero_count: 0,
          negative_count: 0,
          is_integer: true,
          is_sorted: false,
          is_sorted_rev: false,
          bin_width: 0,
          histogram: [],
        },
      },
      interpretation: [],
      ml_advice: [],
      business_summary: [],
      samples: [],
    };

    const result = renderDistribution(col);

    // Should return empty or valid HTML
    expect(result).toBeDefined();
    expect(result).toContain('distribution-chart');
  });

  it('should render categorical distribution correctly', () => {
    const col: ColumnSummary = {
      name: 'category',
      standardized_name: 'category',
      kind: 'Categorical',
      count: 100,
      nulls: 0,
      stats: {
        Categorical: {
          'Category A': 50,
          'Category B': 30,
          'Category C': 20,
        },
      },
      interpretation: [],
      ml_advice: [],
      business_summary: [],
      samples: [],
    };

    const result = renderDistribution(col);

    expect(result).toContain('distribution-chart');
    expect(result).toContain('Top Categories');
    expect(result).toContain('Category A');
    expect(result).toContain('50');
    expect(result).not.toContain('undefined');
  });

  it('should return empty string for categorical with no values', () => {
    const col: ColumnSummary = {
      name: 'category',
      standardized_name: 'category',
      kind: 'Categorical',
      count: 0,
      nulls: 0,
      stats: {
        Categorical: {},
      },
      interpretation: [],
      ml_advice: [],
      business_summary: [],
      samples: [],
    };

    const result = renderDistribution(col);

    expect(result).toBe('');
  });

  it('should return empty string for text columns', () => {
    const col: ColumnSummary = {
      name: 'name',
      standardized_name: 'name',
      kind: 'Text',
      count: 100,
      nulls: 0,
      stats: {
        Text: {
          distinct: 95,
          top_value: ['John', 5],
          min_length: 3,
          max_length: 15,
          avg_length: 8.5,
        },
      },
      interpretation: [],
      ml_advice: [],
      business_summary: [],
      samples: [],
    };

    const result = renderDistribution(col);

    expect(result).toBe('');
  });

  it('should calculate histogram bar heights correctly', () => {
    const col: ColumnSummary = {
      name: 'age',
      standardized_name: 'age',
      kind: 'Numeric',
      count: 100,
      nulls: 0,
      stats: {
        Numeric: {
          distinct_count: 45,
          min: 0,
          max: 100,
          p05: null,
          q1: null,
          median: null,
          mean: null,
          trimmed_mean: null,
          q3: null,
          p95: null,
          std_dev: null,
          skew: null,
          zero_count: 0,
          negative_count: 0,
          is_integer: true,
          is_sorted: false,
          is_sorted_rev: false,
          bin_width: 10,
          histogram: [
            [10, 5], // 5/20 = 25% height
            [20, 20], // 20/20 = 100% height
            [30, 10], // 10/20 = 50% height
          ],
        },
      },
      interpretation: [],
      ml_advice: [],
      business_summary: [],
      samples: [],
    };

    const result = renderDistribution(col);

    // Check that height percentages are calculated correctly
    // Max count is 20, so heights should be: 25%, 100%, 50%
    expect(result).toContain('height: 25%');
    expect(result).toContain('height: 100%');
    expect(result).toContain('height: 50%');
  });
});
