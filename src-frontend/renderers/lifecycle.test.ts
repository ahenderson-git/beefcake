import { describe, it, expect } from 'vitest';

import { renderLifecycleBanner, renderLifecycleRail } from './lifecycle';

describe('renderLifecycleBanner', () => {
  it('should render a compact banner for Profiled stage', () => {
    const html = renderLifecycleBanner('Profiled', 'Ad-hoc file analysis');

    // Verify testid is present
    expect(html).toContain('data-testid="analyser-stage-banner"');

    // Verify stage label
    expect(html).toContain('Profiled');

    // Verify message
    expect(html).toContain('Ad-hoc file analysis');

    // Verify icon class for Profiled stage
    expect(html).toContain('ph-chart-line');
  });

  it('should render a banner for Raw stage', () => {
    const html = renderLifecycleBanner('Raw', 'Original data');

    expect(html).toContain('data-testid="analyser-stage-banner"');
    expect(html).toContain('Raw');
    expect(html).toContain('Original data');
    expect(html).toContain('ph-file');
  });

  it('should escape HTML in messages', () => {
    const html = renderLifecycleBanner('Profiled', '<script>alert("xss")</script>');

    // Should not contain raw script tags
    expect(html).not.toContain('<script>alert("xss")</script>');

    // Should contain escaped version
    expect(html).toContain('&lt;script&gt;');
  });

  it('should include stage-specific colors via inline styles', () => {
    const html = renderLifecycleBanner('Profiled', 'Test message');

    // Verify inline style for background color
    expect(html).toContain('style="background-color:');
    expect(html).toContain('var(--stage-');
  });
});

describe('renderLifecycleRail', () => {
  it('should render empty placeholder when dataset is null', () => {
    const html = renderLifecycleRail(null);

    expect(html).toContain('lifecycle-rail-empty');
    expect(html).toContain('No dataset loaded');
  });

  it('should render full rail when dataset is provided', () => {
    const mockDataset = {
      id: 'test-dataset-123',
      name: 'Test Dataset',
      activeVersionId: 'version-1',
      rawVersionId: 'version-0',
      versions: [
        {
          id: 'version-0',
          dataset_id: 'test-dataset-123',
          parent_id: null,
          stage: 'Raw' as const,
          created_at: '2024-01-01T00:00:00Z',
          data_location: { OriginalFile: '/test/data.csv' },
          metadata: {
            row_count: 100,
            column_count: 5,
            description: 'Raw version',
            tags: [],
            file_size_bytes: 1024,
            created_by: 'test',
            custom_fields: {},
          },
          pipeline: {
            transforms: [],
          },
        },
        {
          id: 'version-1',
          dataset_id: 'test-dataset-123',
          parent_id: 'version-0',
          stage: 'Profiled' as const,
          created_at: '2024-01-01T01:00:00Z',
          data_location: { ParquetFile: '/test/data.parquet' },
          metadata: {
            row_count: 100,
            column_count: 5,
            description: 'Test version',
            tags: [],
            file_size_bytes: 2048,
            created_by: 'test',
            custom_fields: {},
          },
          pipeline: {
            transforms: [],
          },
        },
      ],
    };

    const html = renderLifecycleRail(mockDataset);

    // Verify rail testid
    expect(html).toContain('data-testid="lifecycle-rail"');

    // Verify dataset name
    expect(html).toContain('Test Dataset');

    // Verify stage count (now 2 versions: Raw and Profiled)
    expect(html).toContain('2/6 stages');

    // Verify stage badges are rendered
    expect(html).toContain('data-testid="lifecycle-stage-raw"');
    expect(html).toContain('data-testid="lifecycle-stage-profiled"');
  });

  it('should mark active stage correctly', () => {
    const mockDataset = {
      id: 'test-dataset-123',
      name: 'Test Dataset',
      activeVersionId: 'version-1',
      rawVersionId: 'version-0',
      versions: [
        {
          id: 'version-0',
          dataset_id: 'test-dataset-123',
          parent_id: null,
          stage: 'Raw' as const,
          created_at: '2024-01-01T00:00:00Z',
          data_location: { OriginalFile: '/test/data.csv' },
          metadata: {
            row_count: 100,
            column_count: 5,
            description: 'Raw version',
            tags: [],
            file_size_bytes: 1024,
            created_by: 'test',
            custom_fields: {},
          },
          pipeline: {
            transforms: [],
          },
        },
        {
          id: 'version-1',
          dataset_id: 'test-dataset-123',
          parent_id: 'version-0',
          stage: 'Profiled' as const,
          created_at: '2024-01-01T01:00:00Z',
          data_location: { ParquetFile: '/test/data.parquet' },
          metadata: {
            row_count: 100,
            column_count: 5,
            description: 'Profiled version',
            tags: [],
            file_size_bytes: 2048,
            created_by: 'test',
            custom_fields: {},
          },
          pipeline: {
            transforms: [],
          },
        },
      ],
    };

    const html = renderLifecycleRail(mockDataset);

    // The active stage should have the stage-active class
    expect(html).toContain('stage-active');

    // Both stages should be marked as completed
    expect(html).toContain('stage-completed');
  });

  it('should display all stage badges even for incomplete stages', () => {
    const mockDataset = {
      id: 'test-dataset-123',
      name: 'Test Dataset',
      activeVersionId: 'version-0',
      rawVersionId: 'version-0',
      versions: [
        {
          id: 'version-0',
          dataset_id: 'test-dataset-123',
          parent_id: null,
          stage: 'Raw' as const,
          created_at: '2024-01-01T00:00:00Z',
          data_location: { OriginalFile: '/test/data.csv' },
          metadata: {
            row_count: 100,
            column_count: 5,
            description: 'Raw version',
            tags: [],
            file_size_bytes: 1024,
            created_by: 'test',
            custom_fields: {},
          },
          pipeline: {
            transforms: [],
          },
        },
      ],
    };

    const html = renderLifecycleRail(mockDataset);

    // All 6 stages should be rendered
    expect(html).toContain('data-testid="lifecycle-stage-raw"');
    expect(html).toContain('data-testid="lifecycle-stage-profiled"');
    expect(html).toContain('data-testid="lifecycle-stage-cleaned"');
    expect(html).toContain('data-testid="lifecycle-stage-advanced"');
    expect(html).toContain('data-testid="lifecycle-stage-validated"');
    expect(html).toContain('data-testid="lifecycle-stage-published"');
  });
});
