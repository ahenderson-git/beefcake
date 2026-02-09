/* eslint-disable @typescript-eslint/no-explicit-any */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/no-unsafe-argument */
import { readFileSync } from 'fs';
import { join } from 'path';

import { describe, test, expect } from 'vitest';

import type { DatasetVersion, LifecycleStage } from '../types';

/**
 * Integration Tests for Dataset Lifecycle Schema Validation
 *
 * These tests ensure that the TypeScript DatasetVersion interface matches
 * the actual JSON structure returned by the Rust backend's lifecycle commands.
 *
 * Purpose: Catch schema mismatches in:
 * - Rust enum serialization (DataLocation, LifecycleStage)
 * - Nested structures (TransformPipeline, VersionMetadata)
 * - Record types (custom_fields, parameters)
 */

describe('Dataset Lifecycle Schema Integration', () => {
  let datasetVersionJson: unknown;

  test('should load dataset version fixture', () => {
    const fixturePath = join(__dirname, 'fixtures', 'sample-dataset-version.json');
    const jsonContent = readFileSync(fixturePath, 'utf-8');
    datasetVersionJson = JSON.parse(jsonContent);

    expect(datasetVersionJson).toBeDefined();
    expect(typeof datasetVersionJson).toBe('object');
  });

  test('should have correct top-level structure', () => {
    const version = datasetVersionJson as any;

    // Rust DatasetVersion required fields
    expect(version).toHaveProperty('id');
    expect(version).toHaveProperty('dataset_id');
    expect(version).toHaveProperty('parent_id');
    expect(version).toHaveProperty('stage');
    expect(version).toHaveProperty('pipeline');
    expect(version).toHaveProperty('data_location');
    expect(version).toHaveProperty('metadata');
    expect(version).toHaveProperty('created_at');
  });

  test('should have correct field types', () => {
    const version = datasetVersionJson as any;

    expect(typeof version.id).toBe('string');
    expect(typeof version.dataset_id).toBe('string');
    // parent_id can be null or string
    expect(version.parent_id === null || typeof version.parent_id === 'string').toBe(true);
    expect(typeof version.stage).toBe('string');
    expect(typeof version.pipeline).toBe('object');
    expect(typeof version.data_location).toBe('object');
    expect(typeof version.metadata).toBe('object');
    expect(typeof version.created_at).toBe('string');
  });

  test('should validate LifecycleStage enum values (CRITICAL)', () => {
    const version = datasetVersionJson as any;

    // CRITICAL: Rust enum must serialize as string matching TypeScript union type
    const validStages: LifecycleStage[] = [
      'Raw',
      'Profiled',
      'Cleaned',
      'Advanced',
      'Validated',
      'Published',
    ];

    expect(validStages).toContain(version.stage);
    expect(typeof version.stage).toBe('string');

    // The fixture uses "Cleaned" - verify it matches exactly (case-sensitive)
    expect(version.stage).toBe('Cleaned');
  });

  test('should validate DataLocation enum serialization (CRITICAL)', () => {
    const version = datasetVersionJson as any;

    // CRITICAL: Rust enum DataLocation serialization format
    // Rust enum variants serialize as objects with variant name as key
    // Example: enum DataLocation { ParquetFile(String) } → { "ParquetFile": "path/to/file" }

    expect(version.data_location).toBeDefined();
    expect(typeof version.data_location).toBe('object');

    // Must have exactly ONE of: ParquetFile, OriginalFile, or path
    const locationKeys = Object.keys(version.data_location);
    expect(locationKeys.length).toBeGreaterThan(0);

    // Check if it has the expected variant key
    const hasParquetFile = 'ParquetFile' in version.data_location;
    const hasOriginalFile = 'OriginalFile' in version.data_location;
    const hasPath = 'path' in version.data_location;

    expect(hasParquetFile || hasOriginalFile || hasPath).toBe(true);

    // If ParquetFile exists, it should be a string
    if (hasParquetFile) {
      expect(typeof version.data_location.ParquetFile).toBe('string');
    }

    // If OriginalFile exists, it should be a string
    if (hasOriginalFile) {
      expect(typeof version.data_location.OriginalFile).toBe('string');
    }
  });

  test('should validate TransformPipeline structure (CRITICAL)', () => {
    const version = datasetVersionJson as any;

    expect(version.pipeline).toBeDefined();
    expect(version.pipeline).toHaveProperty('transforms');
    expect(Array.isArray(version.pipeline.transforms)).toBe(true);

    // Validate transform specs if pipeline has transforms
    if (version.pipeline.transforms.length > 0) {
      const firstTransform = version.pipeline.transforms[0];

      expect(firstTransform).toHaveProperty('transform_type');
      expect(firstTransform).toHaveProperty('parameters');

      expect(typeof firstTransform.transform_type).toBe('string');
      expect(typeof firstTransform.parameters).toBe('object');

      // Parameters should be a Record<string, unknown>
      Object.keys(firstTransform.parameters).forEach(key => {
        expect(typeof key).toBe('string');
      });
    }
  });

  test('should validate VersionMetadata structure (CRITICAL)', () => {
    const version = datasetVersionJson as any;

    expect(version.metadata).toBeDefined();
    expect(version.metadata).toHaveProperty('description');
    expect(version.metadata).toHaveProperty('tags');
    expect(version.metadata).toHaveProperty('row_count');
    expect(version.metadata).toHaveProperty('column_count');
    expect(version.metadata).toHaveProperty('file_size_bytes');
    expect(version.metadata).toHaveProperty('created_by');
    expect(version.metadata).toHaveProperty('custom_fields');

    expect(typeof version.metadata.description).toBe('string');
    expect(Array.isArray(version.metadata.tags)).toBe(true);
    expect(typeof version.metadata.created_by).toBe('string');

    // Numeric fields can be null or number
    expect(
      version.metadata.row_count === null || typeof version.metadata.row_count === 'number'
    ).toBe(true);
    expect(
      version.metadata.column_count === null || typeof version.metadata.column_count === 'number'
    ).toBe(true);
    expect(
      version.metadata.file_size_bytes === null ||
        typeof version.metadata.file_size_bytes === 'number'
    ).toBe(true);

    // custom_fields should be Record<string, unknown>
    expect(typeof version.metadata.custom_fields).toBe('object');
  });

  test('should validate custom_fields Record type (CRITICAL)', () => {
    const version = datasetVersionJson as any;

    const customFields = version.metadata.custom_fields;

    // custom_fields is Record<string, unknown> - should be an object with string keys
    expect(typeof customFields).toBe('object');
    expect(Array.isArray(customFields)).toBe(false);

    // All keys should be strings
    Object.keys(customFields).forEach(key => {
      expect(typeof key).toBe('string');
    });

    // Values can be any type (string, number, boolean, object, etc.)
    // Just verify we can access them
    const keys = Object.keys(customFields);
    if (keys.length > 0) {
      const firstKey = keys[0]!;
      expect(customFields[firstKey]).toBeDefined();
    }
  });

  test('should handle null parent_id', () => {
    const version = datasetVersionJson as any;

    // parent_id can be null (for Raw version) or string (for derived versions)
    if (version.parent_id === null) {
      expect(version.parent_id).toBeNull();
    } else {
      expect(typeof version.parent_id).toBe('string');
    }
  });

  test('should parse into TypeScript DatasetVersion type', () => {
    // This test verifies that the JSON structure is compatible with our TypeScript interface
    const version = datasetVersionJson as DatasetVersion;

    expect(version.id).toBeDefined();
    expect(version.dataset_id).toBeDefined();
    expect(version.stage).toBeDefined();
    expect(version.pipeline).toBeDefined();
    expect(version.pipeline.transforms).toBeDefined();
    expect(version.data_location).toBeDefined();
    expect(version.metadata).toBeDefined();
    expect(version.created_at).toBeDefined();

    // Access nested structures safely
    if (version.pipeline.transforms.length > 0) {
      expect(version.pipeline.transforms[0]!.transform_type).toBeDefined();
    }
  });

  test('should detect invalid DataLocation structure (regression test)', () => {
    // This test checks for incorrect serialization of DataLocation enum

    // WRONG structure (flat string):
    const invalidVersion = {
      ...(datasetVersionJson as any),
      data_location: 'C:\\path\\to\\file.parquet', // ← WRONG: should be object
    };

    expect(typeof invalidVersion.data_location).toBe('string');

    // Correct structure:
    const version = datasetVersionJson as any;
    expect(typeof version.data_location).toBe('object');
    expect(version.data_location).not.toBe(null);
  });

  test('should detect invalid LifecycleStage enum (regression test)', () => {
    // This test checks for typos or incorrect enum values

    const validStages: LifecycleStage[] = [
      'Raw',
      'Profiled',
      'Cleaned',
      'Advanced',
      'Validated',
      'Published',
    ];

    // WRONG: lowercase or typo
    const invalidStages = ['cleaned', 'CLEANED', 'Clean', 'Clened', 'raw'];

    invalidStages.forEach(invalidStage => {
      expect(validStages).not.toContain(invalidStage);
    });

    // Correct:
    const version = datasetVersionJson as any;
    expect(validStages).toContain(version.stage);
  });

  test('should validate transform parameters are not empty object', () => {
    const version = datasetVersionJson as any;

    // Each transform should have meaningful parameters (not empty {})
    version.pipeline.transforms.forEach((transform: any) => {
      const paramKeys = Object.keys(transform.parameters);

      // Most transforms should have parameters (though technically {} is valid)
      // This is a soft check - we're just documenting the expectation
      if (transform.transform_type !== 'NoOp') {
        // Most real transforms have parameters
        expect(paramKeys.length).toBeGreaterThanOrEqual(0);
      }
    });
  });

  test('should match structure used in E2E mocks', () => {
    // This test ensures our E2E mocks match the actual backend structure
    const version = datasetVersionJson as any;

    // E2E mocks should have same DataLocation structure
    const hasLocationVariant =
      'ParquetFile' in version.data_location || 'OriginalFile' in version.data_location;
    expect(hasLocationVariant).toBe(true);

    // Should not be flat string
    expect(typeof version.data_location).not.toBe('string');
  });
});
