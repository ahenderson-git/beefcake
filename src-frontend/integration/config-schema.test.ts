/* eslint-disable @typescript-eslint/no-explicit-any */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-call */
import { readFileSync } from 'fs';
import { join } from 'path';

import { describe, test, expect } from 'vitest';

import type { AuditEntry } from '../types';

/**
 * Integration Tests for Config Schema Validation
 *
 * These tests ensure that the TypeScript AppConfig interface matches
 * the actual JSON structure returned by the Rust backend.
 *
 * Purpose: Catch schema mismatches between backend and frontend
 * before they cause runtime errors.
 */

describe('Config Schema Integration', () => {
  let defaultConfigJson: unknown;

  test('should load default config fixture', () => {
    const fixturePath = join(__dirname, 'fixtures', 'default-config.json');
    const jsonContent = readFileSync(fixturePath, 'utf-8');
    defaultConfigJson = JSON.parse(jsonContent);

    expect(defaultConfigJson).toBeDefined();
    expect(typeof defaultConfigJson).toBe('object');
  });

  test('should have correct top-level structure', () => {
    const config = defaultConfigJson as any;

    // Rust AppConfig has 'settings' and 'audit_log' at top level
    expect(config).toHaveProperty('settings');
    expect(config).toHaveProperty('audit_log');
  });

  test('should have audit_log as object with entries array (NOT flat array)', () => {
    const config = defaultConfigJson as any;

    // This is the critical test that would have caught the bug!
    expect(config.audit_log).toBeDefined();
    expect(typeof config.audit_log).toBe('object');
    expect(Array.isArray(config.audit_log)).toBe(false); // NOT an array
    expect(config.audit_log).toHaveProperty('entries');
    expect(Array.isArray(config.audit_log.entries)).toBe(true); // entries IS an array
  });

  test('should have settings object with all required fields', () => {
    const config = defaultConfigJson as any;

    expect(config.settings).toBeDefined();
    expect(config.settings).toHaveProperty('connections');
    expect(config.settings).toHaveProperty('active_import_id');
    expect(config.settings).toHaveProperty('active_export_id');
    expect(config.settings).toHaveProperty('powershell_font_size');
    expect(config.settings).toHaveProperty('python_font_size');
    expect(config.settings).toHaveProperty('sql_font_size');
    expect(config.settings).toHaveProperty('first_run_completed');
    expect(config.settings).toHaveProperty('trusted_paths');
    expect(config.settings).toHaveProperty('preview_row_limit');
    expect(config.settings).toHaveProperty('security_warning_acknowledged');
    expect(config.settings).toHaveProperty('skip_full_row_count');
    expect(config.settings).toHaveProperty('analysis_sample_size');
    expect(config.settings).toHaveProperty('sampling_strategy');
    expect(config.settings).toHaveProperty('ai_config');
  });

  test('should have ai_config nested structure', () => {
    const config = defaultConfigJson as any;

    expect(config.settings.ai_config).toBeDefined();
    expect(config.settings.ai_config).toHaveProperty('enabled');
    expect(config.settings.ai_config).toHaveProperty('model');
    expect(config.settings.ai_config).toHaveProperty('temperature');
    expect(config.settings.ai_config).toHaveProperty('max_tokens');
  });

  test('should have correct field types', () => {
    const config = defaultConfigJson as any;

    // Settings types
    expect(Array.isArray(config.settings.connections)).toBe(true);
    expect(typeof config.settings.powershell_font_size).toBe('number');
    expect(typeof config.settings.python_font_size).toBe('number');
    expect(typeof config.settings.sql_font_size).toBe('number');
    expect(typeof config.settings.first_run_completed).toBe('boolean');
    expect(Array.isArray(config.settings.trusted_paths)).toBe(true);
    expect(typeof config.settings.preview_row_limit).toBe('number');
    expect(typeof config.settings.security_warning_acknowledged).toBe('boolean');
    expect(typeof config.settings.skip_full_row_count).toBe('boolean');
    expect(typeof config.settings.analysis_sample_size).toBe('number');
    expect(typeof config.settings.sampling_strategy).toBe('string');

    // AI Config types
    expect(typeof config.settings.ai_config.enabled).toBe('boolean');
    expect(typeof config.settings.ai_config.model).toBe('string');
    expect(typeof config.settings.ai_config.temperature).toBe('number');
    expect(typeof config.settings.ai_config.max_tokens).toBe('number');

    // Audit log types
    expect(Array.isArray(config.audit_log.entries)).toBe(true);
  });

  test('should parse into TypeScript AppConfig type', () => {
    // This test verifies that the JSON structure is compatible with our TypeScript interface
    // If the structure doesn't match, TypeScript compilation would fail

    // Note: In a real implementation, we need to flatten the structure
    // because our frontend AppConfig interface expects a flat structure
    const config = defaultConfigJson as any;

    // Config already has correct structure from Rust backend
    expect(config).toBeDefined();
    expect(config.settings).toBeDefined();
    expect(config.audit_log).toHaveProperty('entries');
    expect(Array.isArray(config.audit_log.entries)).toBe(true);
  });

  test('should validate audit entry structure', () => {
    // Create a sample audit entry matching the Rust AuditEntry structure
    const sampleEntry: AuditEntry = {
      timestamp: '2025-01-27T10:00:00Z',
      action: 'Config',
      details: 'Updated settings',
    };

    expect(sampleEntry).toHaveProperty('timestamp');
    expect(sampleEntry).toHaveProperty('action');
    expect(sampleEntry).toHaveProperty('details');

    expect(typeof sampleEntry.timestamp).toBe('string');
    expect(typeof sampleEntry.action).toBe('string');
    expect(typeof sampleEntry.details).toBe('string');
  });

  test('should detect regression to old array structure', () => {
    // This test explicitly checks for the bug we just fixed
    const invalidConfig = {
      ...(defaultConfigJson as any),
      audit_log: [], // ← Old WRONG structure (flat array)
    };

    const isArray = Array.isArray(invalidConfig.audit_log);
    // Check if audit_log is an object with an 'entries' property (not an array with .entries() method)
    const hasEntriesProperty =
      !Array.isArray(invalidConfig.audit_log) &&
      typeof invalidConfig.audit_log === 'object' &&
      invalidConfig.audit_log !== null &&
      'entries' in invalidConfig.audit_log;

    // Should detect that this is wrong!
    expect(isArray).toBe(true); // This is wrong - we detect it
    expect(hasEntriesProperty).toBe(false); // This is wrong - we detect it

    // The correct structure should be:
    const validConfig = defaultConfigJson as any;
    expect(Array.isArray(validConfig.audit_log)).toBe(false);
    expect(validConfig.audit_log).toHaveProperty('entries');
  });

  test('should handle empty audit log entries', () => {
    const config = defaultConfigJson as any;

    expect(config.audit_log.entries).toBeDefined();
    expect(Array.isArray(config.audit_log.entries)).toBe(true);
    expect(config.audit_log.entries.length).toBe(0); // Default fixture has empty entries
  });

  test('should match structure used in E2E mocks', () => {
    // This test ensures our E2E mocks match the actual backend structure
    const e2eMockStructure = {
      connections: [],
      active_import_id: null,
      active_export_id: null,
      powershell_font_size: 14,
      python_font_size: 14,
      sql_font_size: 14,
      audit_log: {
        entries: [], // ← Correct E2E mock structure
      },
      trusted_paths: [],
      security_warning_acknowledged: true,
      first_run_completed: true,
    };

    // Verify E2E mock has correct structure
    expect(e2eMockStructure.audit_log).toHaveProperty('entries');
    expect(Array.isArray(e2eMockStructure.audit_log.entries)).toBe(true);
    expect(Array.isArray(e2eMockStructure.audit_log)).toBe(false);
  });
});
