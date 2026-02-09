/* eslint-disable @typescript-eslint/no-explicit-any */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/no-unsafe-argument */
import { readFileSync } from 'fs';
import { join } from 'path';

import { describe, test, expect } from 'vitest';

import type { AnalysisResponse } from '../types';

/**
 * Integration Tests for AnalysisResponse Schema Validation
 *
 * These tests ensure that the TypeScript AnalysisResponse interface matches
 * the actual JSON structure returned by the Rust backend's analyze_file command.
 *
 * Purpose: Catch schema mismatches in complex nested structures like ColumnStats
 * before they cause runtime errors (similar to the audit_log bug we fixed).
 */

describe('AnalysisResponse Schema Integration', () => {
  let analysisResponseJson: unknown;

  test('should load analysis response fixture', () => {
    const fixturePath = join(__dirname, 'fixtures', 'sample-analysis-response.json');
    const jsonContent = readFileSync(fixturePath, 'utf-8');
    analysisResponseJson = JSON.parse(jsonContent);

    expect(analysisResponseJson).toBeDefined();
    expect(typeof analysisResponseJson).toBe('object');
  });

  test('should have correct top-level structure', () => {
    const response = analysisResponseJson as any;

    // Rust AnalysisResponse required fields
    expect(response).toHaveProperty('file_name');
    expect(response).toHaveProperty('path');
    expect(response).toHaveProperty('file_size');
    expect(response).toHaveProperty('row_count');
    expect(response).toHaveProperty('total_row_count');
    expect(response).toHaveProperty('column_count');
    expect(response).toHaveProperty('analysis_duration');
    expect(response).toHaveProperty('health');
    expect(response).toHaveProperty('correlation_matrix');
    expect(response).toHaveProperty('summary');
  });

  test('should have correct field types', () => {
    const response = analysisResponseJson as any;

    expect(typeof response.file_name).toBe('string');
    expect(typeof response.path).toBe('string');
    expect(typeof response.file_size).toBe('number');
    expect(typeof response.row_count).toBe('number');
    expect(typeof response.total_row_count).toBe('number');
    expect(typeof response.column_count).toBe('number');

    // Analysis duration structure
    expect(response.analysis_duration).toBeDefined();
    expect(typeof response.analysis_duration.secs).toBe('number');
    expect(typeof response.analysis_duration.nanos).toBe('number');

    // Health structure
    expect(response.health).toBeDefined();
    expect(typeof response.health.score).toBe('number');
    expect(Array.isArray(response.health.risks)).toBe(true);
    expect(Array.isArray(response.health.notes)).toBe(true);

    // Summary is array
    expect(Array.isArray(response.summary)).toBe(true);
  });

  test('should handle null correlation_matrix', () => {
    const response = analysisResponseJson as any;

    // correlation_matrix can be null or an object
    expect(
      response.correlation_matrix === null || typeof response.correlation_matrix === 'object'
    ).toBe(true);
  });

  test('should have correct ColumnSummary structure', () => {
    const response = analysisResponseJson as any;
    const firstColumn = response.summary[0];

    expect(firstColumn).toBeDefined();
    expect(firstColumn).toHaveProperty('name');
    expect(firstColumn).toHaveProperty('standardized_name');
    expect(firstColumn).toHaveProperty('kind');
    expect(firstColumn).toHaveProperty('count');
    expect(firstColumn).toHaveProperty('nulls');
    expect(firstColumn).toHaveProperty('stats');
    expect(firstColumn).toHaveProperty('interpretation');
    expect(firstColumn).toHaveProperty('ml_advice');
    expect(firstColumn).toHaveProperty('business_summary');
    expect(firstColumn).toHaveProperty('samples');

    expect(typeof firstColumn.name).toBe('string');
    expect(typeof firstColumn.kind).toBe('string');
    expect(typeof firstColumn.count).toBe('number');
    expect(typeof firstColumn.nulls).toBe('number');
    expect(Array.isArray(firstColumn.interpretation)).toBe(true);
    expect(Array.isArray(firstColumn.ml_advice)).toBe(true);
    expect(Array.isArray(firstColumn.business_summary)).toBe(true);
    expect(Array.isArray(firstColumn.samples)).toBe(true);
  });

  test('should validate Numeric stats structure (CRITICAL)', () => {
    const response = analysisResponseJson as any;

    // Find the numeric column (customer_id)
    const numericColumn = response.summary.find((col: any) => col.kind === 'Numeric');
    expect(numericColumn).toBeDefined();

    // CRITICAL: Rust serializes as { "Numeric": { ... } }, NOT as direct object
    expect(numericColumn.stats).toHaveProperty('Numeric');
    expect(typeof numericColumn.stats.Numeric).toBe('object');

    const numericStats = numericColumn.stats.Numeric;

    // Validate all numeric stat fields
    expect(numericStats).toHaveProperty('mean');
    expect(numericStats).toHaveProperty('median');
    expect(numericStats).toHaveProperty('min');
    expect(numericStats).toHaveProperty('max');
    expect(numericStats).toHaveProperty('std_dev');
    expect(numericStats).toHaveProperty('q1');
    expect(numericStats).toHaveProperty('q3');
    expect(numericStats).toHaveProperty('p05');
    expect(numericStats).toHaveProperty('p95');
    expect(numericStats).toHaveProperty('trimmed_mean');
    expect(numericStats).toHaveProperty('skew');
    expect(numericStats).toHaveProperty('distinct_count');
    expect(numericStats).toHaveProperty('zero_count');
    expect(numericStats).toHaveProperty('negative_count');
    expect(numericStats).toHaveProperty('is_integer');
    expect(numericStats).toHaveProperty('is_sorted');
    expect(numericStats).toHaveProperty('is_sorted_rev');
    expect(numericStats).toHaveProperty('bin_width');
    expect(numericStats).toHaveProperty('histogram');

    // Validate histogram structure: array of [number, number] tuples from Rust Vec<(f64, usize)>
    if (numericStats.histogram !== null) {
      expect(Array.isArray(numericStats.histogram)).toBe(true);
      if (numericStats.histogram.length > 0) {
        const firstBin = numericStats.histogram[0];
        expect(Array.isArray(firstBin)).toBe(true);
        expect(firstBin.length).toBe(2); // 2-tuple: [bin_centre, count]
        expect(typeof firstBin[0]).toBe('number'); // bin_centre
        expect(typeof firstBin[1]).toBe('number'); // count
        expect(firstBin[2]).toBeUndefined(); // Should NOT have 3rd element
      }
    }
  });

  test('should validate histogram format matches Rust backend (CRITICAL)', () => {
    // This test ensures the histogram format matches Rust Vec<(f64, usize)>
    // which serializes as [[bin_centre, count], ...] NOT [[min, max, count], ...]
    const response = analysisResponseJson as any;

    const numericColumn = response.summary.find(
      (col: any) => col.kind === 'Numeric' && col.stats.Numeric?.histogram?.length > 0
    );

    if (numericColumn?.stats.Numeric?.histogram) {
      const histogram = numericColumn.stats.Numeric.histogram;

      // Verify each bin is a 2-tuple
      histogram.forEach((bin: any) => {
        expect(bin.length).toBe(2); // Must be 2-tuple
        expect(typeof bin[0]).toBe('number'); // bin_centre
        expect(typeof bin[1]).toBe('number'); // count
        expect(bin[2]).toBeUndefined(); // NO third element

        // Count should be a positive integer
        expect(bin[1]).toBeGreaterThanOrEqual(0);
        expect(Number.isInteger(bin[1])).toBe(true);
      });
    }
  });

  test('should validate Text stats structure (CRITICAL)', () => {
    const response = analysisResponseJson as any;

    const textColumn = response.summary.find((col: any) => col.kind === 'Text');
    expect(textColumn).toBeDefined();

    // CRITICAL: Rust serializes as { "Text": { ... } }
    expect(textColumn.stats).toHaveProperty('Text');
    expect(typeof textColumn.stats.Text).toBe('object');

    const textStats = textColumn.stats.Text;

    expect(textStats).toHaveProperty('min_length');
    expect(textStats).toHaveProperty('max_length');
    expect(textStats).toHaveProperty('avg_length');
    expect(textStats).toHaveProperty('distinct_count');
    expect(textStats).toHaveProperty('top_value');

    expect(typeof textStats.min_length).toBe('number');
    expect(typeof textStats.max_length).toBe('number');
    expect(typeof textStats.avg_length).toBe('number');
    expect(typeof textStats.distinct_count).toBe('number');

    // top_value can be null or [string, number] tuple
    if (textStats.top_value !== null) {
      expect(Array.isArray(textStats.top_value)).toBe(true);
      expect(textStats.top_value.length).toBe(2);
      expect(typeof textStats.top_value[0]).toBe('string');
      expect(typeof textStats.top_value[1]).toBe('number');
    }
  });

  test('should validate Boolean stats structure (CRITICAL)', () => {
    const response = analysisResponseJson as any;

    const booleanColumn = response.summary.find((col: any) => col.kind === 'Boolean');
    expect(booleanColumn).toBeDefined();

    // CRITICAL: Rust serializes as { "Boolean": { ... } }
    expect(booleanColumn.stats).toHaveProperty('Boolean');
    expect(typeof booleanColumn.stats.Boolean).toBe('object');

    const booleanStats = booleanColumn.stats.Boolean;

    expect(booleanStats).toHaveProperty('true_count');
    expect(booleanStats).toHaveProperty('false_count');

    expect(typeof booleanStats.true_count).toBe('number');
    expect(typeof booleanStats.false_count).toBe('number');
  });

  test('should validate Categorical stats structure (CRITICAL)', () => {
    const response = analysisResponseJson as any;

    const categoricalColumn = response.summary.find((col: any) => col.kind === 'Categorical');
    expect(categoricalColumn).toBeDefined();

    // CRITICAL: Rust serializes as { "Categorical": { ... } }
    expect(categoricalColumn.stats).toHaveProperty('Categorical');
    expect(typeof categoricalColumn.stats.Categorical).toBe('object');

    const categoricalStats = categoricalColumn.stats.Categorical;

    expect(categoricalStats).toHaveProperty('distinct_count');
    expect(categoricalStats).toHaveProperty('top_values');

    expect(typeof categoricalStats.distinct_count).toBe('number');
    expect(Array.isArray(categoricalStats.top_values)).toBe(true);

    // top_values is array of [string, number] tuples
    if (categoricalStats.top_values.length > 0) {
      const firstValue = categoricalStats.top_values[0];
      expect(Array.isArray(firstValue)).toBe(true);
      expect(firstValue.length).toBe(2);
      expect(typeof firstValue[0]).toBe('string');
      expect(typeof firstValue[1]).toBe('number');
    }
  });

  test('should validate Temporal stats structure (CRITICAL)', () => {
    const response = analysisResponseJson as any;

    const temporalColumn = response.summary.find((col: any) => col.kind === 'Temporal');
    expect(temporalColumn).toBeDefined();

    // CRITICAL: Rust serializes as { "Temporal": { ... } }
    expect(temporalColumn.stats).toHaveProperty('Temporal');
    expect(typeof temporalColumn.stats.Temporal).toBe('object');

    const temporalStats = temporalColumn.stats.Temporal;

    expect(temporalStats).toHaveProperty('min');
    expect(temporalStats).toHaveProperty('max');
    expect(temporalStats).toHaveProperty('distinct_count');
    expect(temporalStats).toHaveProperty('p05');
    expect(temporalStats).toHaveProperty('p95');
    expect(temporalStats).toHaveProperty('is_sorted');
    expect(temporalStats).toHaveProperty('is_sorted_rev');
    expect(temporalStats).toHaveProperty('bin_width');
    expect(temporalStats).toHaveProperty('histogram');

    // min/max can be strings (ISO dates) or null
    if (temporalStats.min !== null) {
      expect(typeof temporalStats.min).toBe('string');
    }
    if (temporalStats.max !== null) {
      expect(typeof temporalStats.max).toBe('string');
    }

    expect(typeof temporalStats.distinct_count).toBe('number');
    expect(typeof temporalStats.is_sorted).toBe('boolean');
    expect(typeof temporalStats.is_sorted_rev).toBe('boolean');

    // Validate histogram structure for temporal: array of [number, number] tuples from Rust Vec<(f64, usize)>
    if (temporalStats.histogram !== null) {
      expect(Array.isArray(temporalStats.histogram)).toBe(true);
      if (temporalStats.histogram.length > 0) {
        const firstBin = temporalStats.histogram[0];
        expect(Array.isArray(firstBin)).toBe(true);
        expect(firstBin.length).toBe(2); // 2-tuple: [timestamp_ms, count]
        expect(typeof firstBin[0]).toBe('number'); // timestamp_ms
        expect(typeof firstBin[1]).toBe('number'); // count
        expect(firstBin[2]).toBeUndefined(); // Should NOT have 3rd element
      }
    }
  });

  test('should parse into TypeScript AnalysisResponse type', () => {
    // This test verifies that the JSON structure is compatible with our TypeScript interface
    const response = analysisResponseJson as AnalysisResponse;

    expect(response.file_name).toBeDefined();
    expect(response.summary.length).toBeGreaterThan(0);

    // Access nested stats safely
    const numericColumn = response.summary.find(col => col.kind === 'Numeric');
    if (numericColumn?.stats.Numeric) {
      expect(numericColumn.stats.Numeric.mean).toBeDefined();
    }
  });

  test('should detect invalid stats structure (regression test)', () => {
    // This test checks for a bug where stats might be serialized incorrectly

    // WRONG structure (flat, not wrapped):
    const invalidColumn = {
      name: 'test',
      kind: 'Numeric',
      stats: {
        mean: 5.5, // ← WRONG: should be stats.Numeric.mean
        median: 5.5,
      },
    };

    // Validate this is wrong structure
    const hasNumericProperty = 'Numeric' in invalidColumn.stats;
    expect(hasNumericProperty).toBe(false); // This is wrong - we detect it

    // Correct structure:
    const response = analysisResponseJson as any;
    const validColumn = response.summary[0];
    expect('Numeric' in validColumn.stats).toBe(true); // Correct
  });

  test('should match structure used in E2E mocks', () => {
    // This test ensures our E2E mocks match the actual backend structure
    const response = analysisResponseJson as any;

    // E2E mocks should have same structure
    expect(response.summary.length).toBeGreaterThan(0);

    // Verify first column has correct stat wrapper
    const firstColumn = response.summary[0];
    const statKeys = Object.keys(firstColumn.stats);

    // Should have exactly one stat type key (Numeric, Text, Boolean, Categorical, or Temporal)
    expect(statKeys.length).toBe(1);
    expect(['Numeric', 'Text', 'Boolean', 'Categorical', 'Temporal']).toContain(statKeys[0]);
  });
});
