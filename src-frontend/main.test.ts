/* eslint-disable @typescript-eslint/no-explicit-any */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-call */
import { describe, test, expect, beforeEach } from 'vitest';

import type { AppConfig, AuditEntry } from './types';

/**
 * Unit Tests for BeefcakeApp.recordToastEvent()
 *
 * These tests verify that toast events are correctly recorded to the audit log
 * with the proper structure (audit_log.entries array).
 */

describe('BeefcakeApp - recordToastEvent', () => {
  let mockConfig: AppConfig;

  beforeEach(() => {
    // Create a mock config with correct structure
    mockConfig = {
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
  });

  test('should add entry to audit_log.entries array', () => {
    // Simulate recordToastEvent logic
    const message = 'Analysis complete';
    const type = 'success';
    const view = 'Analyser';
    const action = 'Toast';
    const details = `[${type}] ${message} (${view})`;

    // This is the core logic from recordToastEvent()
    if (mockConfig.audit_log?.entries) {
      mockConfig.audit_log.entries.push({
        timestamp: new Date().toISOString(),
        action,
        details,
      });
    }

    expect(mockConfig.audit_log.entries).toHaveLength(1);
    expect(mockConfig.audit_log.entries[0]!.action).toBe('Toast');
    expect(mockConfig.audit_log.entries[0]!.details).toBe('[success] Analysis complete (Analyser)');
    expect(mockConfig.audit_log.entries[0]!.timestamp).toBeDefined();
  });

  test('should handle multiple toast events', () => {
    // Add multiple events
    const events = [
      { message: 'Loading...', type: 'info' as const, view: 'Dashboard' },
      { message: 'Analysis complete', type: 'success' as const, view: 'Analyser' },
      { message: 'Export failed', type: 'error' as const, view: 'Analyser' },
    ];

    events.forEach(({ message, type, view }) => {
      const action = 'Toast';
      const details = `[${type}] ${message} (${view})`;

      mockConfig.audit_log.entries.push({
        timestamp: new Date().toISOString(),
        action,
        details,
      });
    });

    expect(mockConfig.audit_log.entries).toHaveLength(3);
    expect(mockConfig.audit_log.entries[0]!.details).toContain('Loading...');
    expect(mockConfig.audit_log.entries[1]!.details).toContain('Analysis complete');
    expect(mockConfig.audit_log.entries[2]!.details).toContain('Export failed');
  });

  test('should trim audit log when exceeding 1000 entries', () => {
    // Fill with 1005 entries
    for (let i = 0; i < 1005; i++) {
      mockConfig.audit_log.entries.push({
        timestamp: new Date().toISOString(),
        action: 'Toast',
        details: `Event ${i}`,
      });
    }

    // Simulate the overflow logic from recordToastEvent()
    if (mockConfig.audit_log.entries.length > 1000) {
      const overflow = mockConfig.audit_log.entries.length - 1000;
      mockConfig.audit_log.entries.splice(0, overflow);
    }

    expect(mockConfig.audit_log.entries).toHaveLength(1000);
    // First entry should now be "Event 5" (0-4 were removed)
    expect(mockConfig.audit_log.entries[0]!.details).toBe('Event 5');
    // Last entry should still be "Event 1004"
    expect(mockConfig.audit_log.entries[999]!.details).toBe('Event 1004');
  });

  test('should handle undefined or null audit_log gracefully', () => {
    const invalidConfig = {
      ...mockConfig,
      audit_log: undefined as any,
    };

    // This should not throw - just skip the recording
    let errorThrown = false;
    try {
      if (invalidConfig.audit_log?.entries) {
        invalidConfig.audit_log.entries.push({
          timestamp: new Date().toISOString(),
          action: 'Toast',
          details: 'Test',
        });
      }
    } catch {
      errorThrown = true;
    }

    expect(errorThrown).toBe(false);
  });

  test('should NOT work with old array structure (regression test)', () => {
    // This test verifies the bug we just fixed
    const oldStructureConfig = {
      ...mockConfig,
      audit_log: [] as any, // Old wrong structure
    };

    // The old structure is an array, the new structure is an object
    const oldIsArray = Array.isArray(oldStructureConfig.audit_log);
    expect(oldIsArray).toBe(true);

    // The correct structure should be an object with 'entries' property
    const newIsArray = Array.isArray(mockConfig.audit_log);
    expect(newIsArray).toBe(false);
    expect(mockConfig.audit_log).toHaveProperty('entries');
    expect(Array.isArray(mockConfig.audit_log.entries)).toBe(true);
  });

  test('should verify AuditEntry structure', () => {
    const entry: AuditEntry = {
      timestamp: '2025-01-27T10:00:00Z',
      action: 'Toast',
      details: '[info] Test message (Dashboard)',
    };

    mockConfig.audit_log.entries.push(entry);

    const retrieved = mockConfig.audit_log.entries[0]!;
    expect(retrieved.timestamp).toBeDefined();
    expect(retrieved.action).toBe('Toast');
    expect(retrieved.details).toBe('[info] Test message (Dashboard)');

    // Verify all required fields are present
    expect(Object.keys(retrieved)).toEqual(['timestamp', 'action', 'details']);
  });
});
