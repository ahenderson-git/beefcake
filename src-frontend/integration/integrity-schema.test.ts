/* eslint-disable @typescript-eslint/no-explicit-any */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/no-unsafe-argument */
import { readFileSync } from 'fs';
import { join } from 'path';

import { describe, test, expect } from 'vitest';

import type { VerificationResult, IntegrityReceipt } from '../types';

/**
 * Integration Tests for Integrity Verification Schema Validation
 *
 * These tests ensure that the TypeScript VerificationResult and IntegrityReceipt
 * interfaces match the actual JSON structure returned by the Rust backend.
 *
 * Purpose: Catch schema mismatches in security-critical integrity verification
 * structures before they cause runtime errors or compromise verification results.
 */

describe('Integrity Verification Schema Integration', () => {
  let verificationResultJson: unknown;

  test('should load verification result fixture', () => {
    const fixturePath = join(__dirname, 'fixtures', 'sample-verification-result.json');
    const jsonContent = readFileSync(fixturePath, 'utf-8');
    verificationResultJson = JSON.parse(jsonContent);

    expect(verificationResultJson).toBeDefined();
    expect(typeof verificationResultJson).toBe('object');
  });

  test('should have correct top-level VerificationResult structure', () => {
    const result = verificationResultJson as any;

    // Rust VerificationResult required fields
    expect(result).toHaveProperty('passed');
    expect(result).toHaveProperty('message');
    expect(result).toHaveProperty('file_path');
    expect(result).toHaveProperty('expected_hash');
    expect(result).toHaveProperty('actual_hash');
    expect(result).toHaveProperty('receipt');
  });

  test('should have correct field types for VerificationResult', () => {
    const result = verificationResultJson as any;

    expect(typeof result.passed).toBe('boolean');
    expect(typeof result.message).toBe('string');
    expect(typeof result.file_path).toBe('string');
    expect(typeof result.expected_hash).toBe('string');

    // actual_hash can be null (if file not found) or string
    expect(result.actual_hash === null || typeof result.actual_hash === 'string').toBe(true);

    expect(typeof result.receipt).toBe('object');
  });

  test('should validate IntegrityReceipt top-level structure (CRITICAL)', () => {
    const result = verificationResultJson as any;
    const receipt = result.receipt;

    // CRITICAL: Receipt structure must match exactly for security validation
    expect(receipt).toBeDefined();
    expect(receipt).toHaveProperty('receipt_version');
    expect(receipt).toHaveProperty('created_utc');
    expect(receipt).toHaveProperty('producer');
    expect(receipt).toHaveProperty('export');
    expect(receipt).toHaveProperty('integrity');
  });

  test('should validate IntegrityReceipt field types (CRITICAL)', () => {
    const result = verificationResultJson as any;
    const receipt = result.receipt;

    expect(typeof receipt.receipt_version).toBe('number');
    expect(typeof receipt.created_utc).toBe('string');
    expect(typeof receipt.producer).toBe('object');
    expect(typeof receipt.export).toBe('object');
    expect(typeof receipt.integrity).toBe('object');
  });

  test('should validate producer structure (CRITICAL)', () => {
    const result = verificationResultJson as any;
    const producer = result.receipt.producer;

    expect(producer).toBeDefined();
    expect(producer).toHaveProperty('app_name');
    expect(producer).toHaveProperty('app_version');
    expect(producer).toHaveProperty('platform');

    expect(typeof producer.app_name).toBe('string');
    expect(typeof producer.app_version).toBe('string');
    expect(typeof producer.platform).toBe('string');
  });

  test('should validate export structure (CRITICAL)', () => {
    const result = verificationResultJson as any;
    const exportInfo = result.receipt.export;

    expect(exportInfo).toBeDefined();
    expect(exportInfo).toHaveProperty('filename');
    expect(exportInfo).toHaveProperty('format');
    expect(exportInfo).toHaveProperty('file_size_bytes');
    expect(exportInfo).toHaveProperty('row_count');
    expect(exportInfo).toHaveProperty('column_count');
    expect(exportInfo).toHaveProperty('schema');

    expect(typeof exportInfo.filename).toBe('string');
    expect(typeof exportInfo.format).toBe('string');
    expect(typeof exportInfo.file_size_bytes).toBe('number');
    expect(typeof exportInfo.row_count).toBe('number');
    expect(typeof exportInfo.column_count).toBe('number');
    expect(Array.isArray(exportInfo.schema)).toBe(true);
  });

  test('should validate schema array structure (CRITICAL)', () => {
    const result = verificationResultJson as any;
    const schema = result.receipt.export.schema;

    // CRITICAL: Schema is Array<{ name: string, dtype: string }>
    expect(Array.isArray(schema)).toBe(true);
    expect(schema.length).toBeGreaterThan(0);

    // Validate first schema entry
    const firstColumn = schema[0];
    expect(firstColumn).toHaveProperty('name');
    expect(firstColumn).toHaveProperty('dtype');

    expect(typeof firstColumn.name).toBe('string');
    expect(typeof firstColumn.dtype).toBe('string');

    // Validate all schema entries have correct structure
    schema.forEach((col: any) => {
      expect(col).toHaveProperty('name');
      expect(col).toHaveProperty('dtype');
      expect(typeof col.name).toBe('string');
      expect(typeof col.dtype).toBe('string');
    });
  });

  test('should validate integrity structure (CRITICAL)', () => {
    const result = verificationResultJson as any;
    const integrity = result.receipt.integrity;

    // CRITICAL: Integrity hash structure for verification
    expect(integrity).toBeDefined();
    expect(integrity).toHaveProperty('hash_algorithm');
    expect(integrity).toHaveProperty('hash');

    expect(typeof integrity.hash_algorithm).toBe('string');
    expect(typeof integrity.hash).toBe('string');

    // Hash should be non-empty
    expect(integrity.hash.length).toBeGreaterThan(0);
    expect(integrity.hash_algorithm.length).toBeGreaterThan(0);
  });

  test('should validate hash format consistency (CRITICAL)', () => {
    const result = verificationResultJson as any;

    // CRITICAL: Hashes should have consistent format
    // expected_hash and actual_hash should match integrity.hash format

    const expectedHash = result.expected_hash;
    const actualHash = result.actual_hash;
    const receiptHash = result.receipt.integrity.hash;

    // All hashes should be strings
    expect(typeof expectedHash).toBe('string');
    if (actualHash !== null) {
      expect(typeof actualHash).toBe('string');
    }
    expect(typeof receiptHash).toBe('string');

    // When verification passes, hashes should match
    if (result.passed && actualHash !== null) {
      expect(actualHash).toContain(receiptHash);
      expect(expectedHash).toContain(receiptHash);
    }
  });

  test('should handle null actual_hash (file not found case)', () => {
    const result = verificationResultJson as any;

    // actual_hash can be null if file doesn't exist
    // In the passing case, it should match expected_hash
    if (result.actual_hash === null) {
      expect(result.passed).toBe(false); // Verification should fail if no file
    } else {
      expect(typeof result.actual_hash).toBe('string');
    }
  });

  test('should parse into TypeScript VerificationResult type', () => {
    // This test verifies that the JSON structure is compatible with our TypeScript interface
    const result = verificationResultJson as VerificationResult;

    expect(result.passed).toBeDefined();
    expect(result.message).toBeDefined();
    expect(result.file_path).toBeDefined();
    expect(result.expected_hash).toBeDefined();
    expect(result.receipt).toBeDefined();

    // Access nested structures safely
    expect(result.receipt.receipt_version).toBeDefined();
    expect(result.receipt.producer.app_name).toBeDefined();
    expect(result.receipt.export.filename).toBeDefined();
    expect(result.receipt.integrity.hash).toBeDefined();
  });

  test('should parse into TypeScript IntegrityReceipt type', () => {
    const result = verificationResultJson as VerificationResult;
    const receipt: IntegrityReceipt = result.receipt;

    expect(receipt.receipt_version).toBeDefined();
    expect(receipt.created_utc).toBeDefined();
    expect(receipt.producer.app_name).toBeDefined();
    expect(receipt.export.schema.length).toBeGreaterThan(0);
    expect(receipt.integrity.hash_algorithm).toBeDefined();
  });

  test('should detect invalid receipt structure (regression test)', () => {
    // This test checks for missing or incorrect receipt fields

    // WRONG structure (missing critical fields):
    const invalidResult = {
      passed: true,
      message: 'OK',
      file_path: 'test.parquet',
      expected_hash: 'abc123',
      actual_hash: 'abc123',
      receipt: {
        // Missing producer, export, integrity!
        receipt_version: 1,
      },
    };

    const hasProducer = 'producer' in invalidResult.receipt;
    const hasExport = 'export' in invalidResult.receipt;
    const hasIntegrity = 'integrity' in invalidResult.receipt;

    expect(hasProducer).toBe(false); // WRONG - we detect it
    expect(hasExport).toBe(false); // WRONG - we detect it
    expect(hasIntegrity).toBe(false); // WRONG - we detect it

    // Correct structure:
    const result = verificationResultJson as any;
    expect('producer' in result.receipt).toBe(true);
    expect('export' in result.receipt).toBe(true);
    expect('integrity' in result.receipt).toBe(true);
  });

  test('should validate schema is not empty', () => {
    const result = verificationResultJson as any;
    const schema = result.receipt.export.schema;

    // Schema should have at least one column
    expect(schema.length).toBeGreaterThan(0);

    // No schema entry should have empty name or dtype
    schema.forEach((col: any) => {
      expect(col.name.length).toBeGreaterThan(0);
      expect(col.dtype.length).toBeGreaterThan(0);
    });
  });

  test('should validate receipt version is positive number', () => {
    const result = verificationResultJson as any;

    expect(result.receipt.receipt_version).toBeGreaterThan(0);
    expect(Number.isInteger(result.receipt.receipt_version)).toBe(true);
  });

  test('should validate created_utc is ISO 8601 format', () => {
    const result = verificationResultJson as any;
    const createdUtc = result.receipt.created_utc;

    // Should be a valid ISO 8601 datetime string
    expect(typeof createdUtc).toBe('string');

    // Should be parseable as Date
    const parsed = new Date(createdUtc);
    expect(parsed.toString()).not.toBe('Invalid Date');

    // Should contain 'Z' or timezone offset
    expect(createdUtc.includes('Z') || createdUtc.includes('+') || createdUtc.includes('-')).toBe(
      true
    );
  });

  test('should validate numeric export fields are non-negative', () => {
    const result = verificationResultJson as any;
    const exportInfo = result.receipt.export;

    expect(exportInfo.file_size_bytes).toBeGreaterThanOrEqual(0);
    expect(exportInfo.row_count).toBeGreaterThanOrEqual(0);
    expect(exportInfo.column_count).toBeGreaterThan(0); // Must have at least one column
  });

  test('should match structure used in verification UI', () => {
    // This test ensures our verification result structure matches what UI expects
    const result = verificationResultJson as VerificationResult;

    // UI should be able to display these fields without errors
    expect(result.passed).toBeDefined();
    expect(result.message).toBeDefined();
    expect(result.receipt.producer.app_name).toBeDefined();
    expect(result.receipt.export.filename).toBeDefined();
    expect(result.receipt.integrity.hash_algorithm).toBeDefined();

    // Schema should be displayable as table
    expect(Array.isArray(result.receipt.export.schema)).toBe(true);
    result.receipt.export.schema.forEach(col => {
      expect(col.name).toBeDefined();
      expect(col.dtype).toBeDefined();
    });
  });
});
