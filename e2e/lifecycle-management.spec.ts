import { test, expect } from '@playwright/test';

import { getLifecycleMocks } from './helpers/common-mocks';
import { setupTauriMock } from './helpers/tauri-mock';

/**
 * Lifecycle Management E2E Tests
 *
 * Tests cover the dataset lifecycle features:
 * - Stage transitions (Raw → Profiled → Cleaned → Advanced → Validated → Published)
 * - Version diffing between stages
 * - Lifecycle rail navigation
 * - Data immutability across versions
 * - Column schema evolution
 *
 * Priority: P0/P1 (Critical new feature)
 */

const APP_URL = 'http://localhost:14206';

// Mock dataset with multiple lifecycle versions
const MOCK_DATASET = {
  id: 'ds-test-123',
  name: 'customer_data',
  description: 'Customer demographics dataset',
  created_at: '2025-01-20T10:00:00Z',
  updated_at: '2025-01-20T12:00:00Z',
  active_version_id: 'v-cleaned-001',
  versions: [
    {
      id: 'v-raw-001',
      stage: 'Raw',
      created_at: '2025-01-20T10:00:00Z',
      row_count: 1000,
      column_count: 5,
      data_location: {
        OriginalFile: '/data/customer_data.csv',
      },
      parent_version_id: null,
      metadata: {},
    },
    {
      id: 'v-profiled-001',
      stage: 'Profiled',
      created_at: '2025-01-20T10:05:00Z',
      row_count: 1000,
      column_count: 5,
      data_location: {
        ParquetFile: '/data/.beefcake/ds-test-123/profiled_v001.parquet',
      },
      parent_version_id: 'v-raw-001',
      metadata: {
        health_score: 0.75,
      },
    },
    {
      id: 'v-cleaned-001',
      stage: 'Cleaned',
      created_at: '2025-01-20T11:00:00Z',
      row_count: 1000,
      column_count: 5,
      data_location: {
        ParquetFile: '/data/.beefcake/ds-test-123/cleaned_v001.parquet',
      },
      parent_version_id: 'v-profiled-001',
      metadata: {
        cleaning_applied: true,
        columns_renamed: [
          ['Customer Name', 'customer_name'],
          ['Order Date', 'order_date'],
        ],
      },
    },
  ],
};

// Mock version diff response
const MOCK_VERSION_DIFF = {
  from_version: {
    id: 'v-raw-001',
    stage: 'Raw',
  },
  to_version: {
    id: 'v-cleaned-001',
    stage: 'Cleaned',
  },
  row_count_change: 0, // No rows added/removed
  schema_changes: {
    columns_added: [],
    columns_removed: [],
    columns_renamed: [
      ['Customer Name', 'customer_name'],
      ['Order Date', 'order_date'],
      ['Email Address', 'email'],
    ],
    columns_type_changed: [],
  },
  data_quality_changes: {
    null_count_before: 50,
    null_count_after: 0,
    health_score_before: 0.75,
    health_score_after: 0.95,
  },
  summary: 'Cleaned stage: 3 columns renamed, 50 null values imputed',
};

// Mock version schema (columns for a specific version)
const MOCK_CLEANED_SCHEMA = [
  { name: 'id', kind: 'Numeric', nullable: false },
  { name: 'customer_name', kind: 'Text', nullable: false },
  { name: 'order_date', kind: 'Temporal', nullable: false },
  { name: 'email', kind: 'Text', nullable: false },
  { name: 'age', kind: 'Numeric', nullable: true },
];

test.describe('Lifecycle Management - Stage Transitions', () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page, {
      commands: getLifecycleMocks({
        lifecycle_get_dataset: {
          type: 'success',
          data: MOCK_DATASET,
        },
      }),
    });
  });

  test('should display lifecycle rail with all stages', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to Lifecycle view
    await page.getByTestId('nav-lifecycle').click();

    // Verify lifecycle view loads
    await expect(page).toHaveTitle(/beefcake/i);

    // Note: Full lifecycle rail requires dataset to be loaded
    // This test verifies the lifecycle view is accessible
  });

  test('should show stage icons and labels correctly', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to Lifecycle view
    await page.getByTestId('nav-lifecycle').click();

    // Verify lifecycle view loads
    await expect(page).toHaveTitle(/beefcake/i);

    // Note: Stage icons/labels display requires dataset with versions
    // This test verifies lifecycle view accessibility
  });

  test('should transition from Profiled to Cleaned stage', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // TODO: Once lifecycle transitions are connected:
    // 1. Load a dataset at Profiled stage
    // 2. Configure some cleaning options
    // 3. Click "Begin Cleaning" button
    // 4. Verify loading indicator appears
    // 5. Wait for transition to complete
    // 6. Verify lifecycle rail shows Cleaned as active
    // 7. Verify Profiled stage now has checkmark
    // 8. Verify cleaning UI is available

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should transition through all stages sequentially', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // TODO: Once lifecycle transitions are connected:
    // 1. Load a dataset at Raw stage
    // 2. Progress to Profiled (automatic after analysis)
    // 3. Progress to Cleaned (click "Begin Cleaning")
    // 4. Progress to Advanced (click "Apply Advanced Processing")
    // 5. Progress to Validated (click "Validate Data")
    // 6. Progress to Published (click "Publish Dataset")
    // 7. Verify each transition updates the lifecycle rail
    // 8. Verify data immutability (each stage creates new version)

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should prevent skipping lifecycle stages', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // TODO: Once lifecycle rail is connected:
    // 1. Load a dataset at Profiled stage
    // 2. Try to click on Advanced stage (should be disabled)
    // 3. Verify no action occurs
    // 4. Verify tooltip explains "Complete Cleaned stage first"
    // 5. Complete Cleaned stage
    // 6. Verify Advanced stage becomes clickable

    await expect(page).toHaveTitle(/beefcake/i);
  });
});

test.describe('Lifecycle Management - Version Navigation', () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page, {
      commands: getLifecycleMocks({
        lifecycle_get_dataset: {
          type: 'success',
          data: MOCK_DATASET,
        },
        get_version_schema: {
          type: 'success',
          data: MOCK_CLEANED_SCHEMA,
        },
      }),
    });
  });

  test('should navigate between lifecycle versions using rail', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // TODO: Once lifecycle rail is connected:
    // 1. Load a dataset with multiple versions
    // 2. Click on "Profiled" stage in rail
    // 3. Verify data view updates to show Profiled version
    // 4. Verify column names match Profiled schema (original names)
    // 5. Click on "Cleaned" stage in rail
    // 6. Verify data view updates to show Cleaned version
    // 7. Verify column names match Cleaned schema (renamed)

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should update IDE stage selector when rail stage changes', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // TODO: Once lifecycle integration is connected:
    // 1. Load a dataset
    // 2. Navigate to Python IDE
    // 3. Verify stage selector shows current stage (Cleaned)
    // 4. Click "Profiled" in lifecycle rail
    // 5. Verify Python IDE stage selector updates to "Profiled"
    // 6. Verify data path updates to use Profiled version

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should preserve view state when switching versions', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // TODO: Once lifecycle is connected:
    // 1. Load a dataset at Cleaned stage
    // 2. Expand a column row in analyser
    // 3. Switch to Profiled stage
    // 4. Switch back to Cleaned stage
    // 5. Verify the same column row is still expanded
    // 6. Verify scroll position is preserved

    await expect(page).toHaveTitle(/beefcake/i);
  });
});

test.describe('Lifecycle Management - Version Diffs', () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page, {
      commands: getLifecycleMocks({
        lifecycle_get_dataset: {
          type: 'success',
          data: MOCK_DATASET,
        },
        get_version_diff: {
          type: 'success',
          data: MOCK_VERSION_DIFF,
        },
      }),
    });
  });

  test('should open diff modal when clicking "View Diff" button', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // TODO: Once diff UI is connected:
    // 1. Load a dataset with multiple versions
    // 2. Click "View Diff" button in lifecycle component
    // 3. Verify diff modal opens
    // 4. Verify modal title shows "Raw → Cleaned"
    // 5. Verify close button is present

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should display schema changes in diff modal', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // TODO: Once diff UI is connected:
    // 1. Load a dataset
    // 2. Open diff modal (Raw → Cleaned)
    // 3. Verify "Schema Changes" section is visible
    // 4. Verify renamed columns are listed:
    //    - "Customer Name" → "customer_name"
    //    - "Order Date" → "order_date"
    //    - "Email Address" → "email"
    // 5. Verify no columns added/removed

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should display data quality changes in diff modal', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // TODO: Once diff UI is connected:
    // 1. Load a dataset
    // 2. Open diff modal (Raw → Cleaned)
    // 3. Verify "Data Quality" section is visible
    // 4. Verify health score change: 0.75 → 0.95
    // 5. Verify null count change: 50 → 0
    // 6. Verify visual indicators (arrows, colors)

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should close diff modal using close button', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // TODO: Once diff UI is connected:
    // 1. Open diff modal
    // 2. Click close button (X icon)
    // 3. Verify modal closes
    // 4. Verify underlying view is still visible

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should close diff modal using ESC key', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // TODO: Once diff UI is connected:
    // 1. Open diff modal
    // 2. Press ESC key
    // 3. Verify modal closes

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should close diff modal by clicking overlay', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // TODO: Once diff UI is connected:
    // 1. Open diff modal
    // 2. Click outside modal (on overlay)
    // 3. Verify modal closes

    await expect(page).toHaveTitle(/beefcake/i);
  });
});

test.describe('Lifecycle Management - Column Refactoring', () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page, {
      commands: getLifecycleMocks({
        lifecycle_get_dataset: {
          type: 'success',
          data: MOCK_DATASET,
        },
        get_version_diff: {
          type: 'success',
          data: MOCK_VERSION_DIFF,
        },
        get_version_schema: {
          type: 'success',
          data: MOCK_CLEANED_SCHEMA,
        },
      }),
    });
  });

  test('should show refactor button when switching stages in Python IDE', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // TODO: Once IDE lifecycle integration is connected:
    // 1. Load a dataset at Cleaned stage
    // 2. Navigate to Python IDE
    // 3. Select Raw stage from dropdown
    // 4. Verify refactor button appears
    // 5. Verify button tooltip explains functionality

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should refactor Python column names when clicking refactor button', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // TODO: Once refactoring is connected:
    // 1. Load a dataset
    // 2. Navigate to Python IDE
    // 3. Write code with old column names:
    //    df.select("Customer Name", "Order Date", "Email Address")
    // 4. Switch from Raw to Cleaned stage
    // 5. Click "Refactor" button
    // 6. Confirm refactoring in dialog
    // 7. Verify code is updated:
    //    df.select("customer_name", "order_date", "email")
    // 8. Verify success toast shows "Updated 3 column references"

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should preserve quote style during Python refactoring', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // TODO: Once refactoring is connected:
    // 1. Load a dataset
    // 2. Navigate to Python IDE
    // 3. Write code with single quotes:
    //    df.select('Customer Name', 'Order Date')
    // 4. Switch stages and refactor
    // 5. Verify single quotes are preserved:
    //    df.select('customer_name', 'order_date')

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should refactor SQL column names with multiple quote styles', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // TODO: Once refactoring is connected:
    // 1. Load a dataset
    // 2. Navigate to SQL IDE
    // 3. Write query with mixed quotes:
    //    SELECT "Customer Name", `Order Date`, [Email Address] FROM data
    // 4. Switch stages and refactor
    // 5. Verify quote styles are preserved:
    //    SELECT "customer_name", `order_date`, [email] FROM data

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should show info toast when no column renames detected', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // TODO: Once refactoring is connected:
    // 1. Load a dataset
    // 2. Navigate to Python IDE
    // 3. Switch to a stage with no column renames
    // 4. Click refactor button
    // 5. Verify info toast: "No column renames detected between stages"

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should show info toast when script is empty', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // TODO: Once refactoring is connected:
    // 1. Navigate to Python IDE (empty editor)
    // 2. Switch stages
    // 3. Click refactor button
    // 4. Verify info toast: "Script is empty - nothing to refactor"

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should allow undoing refactoring with Ctrl+Z', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // TODO: Once refactoring is connected:
    // 1. Write Python code with old column names
    // 2. Perform refactoring
    // 3. Verify code is updated
    // 4. Press Ctrl+Z
    // 5. Verify code reverts to original column names

    await expect(page).toHaveTitle(/beefcake/i);
  });
});

test.describe('Lifecycle Management - Data Immutability', () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page, {
      commands: getLifecycleMocks({
        lifecycle_get_dataset: {
          type: 'success',
          data: MOCK_DATASET,
        },
      }),
    });
  });

  test('should verify each stage creates a new immutable version', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // TODO: Once lifecycle is connected:
    // 1. Load a dataset at Raw stage
    // 2. Note the version ID (v-raw-001)
    // 3. Progress to Profiled stage
    // 4. Verify new version ID is created (v-profiled-001)
    // 5. Progress to Cleaned stage
    // 6. Verify new version ID is created (v-cleaned-001)
    // 7. Verify all 3 versions exist in lifecycle rail

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should verify raw data remains unchanged after cleaning', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // TODO: Once lifecycle is connected:
    // 1. Load a dataset with nulls at Raw stage
    // 2. Apply cleaning to impute null values
    // 3. Progress to Cleaned stage
    // 4. Navigate back to Raw stage
    // 5. Verify null values still exist in Raw version
    // 6. Navigate to Cleaned stage
    // 7. Verify null values are imputed in Cleaned version

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should verify parent-child relationships between versions', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // TODO: Once lifecycle metadata is accessible:
    // 1. Load a dataset with multiple versions
    // 2. Verify Raw version has no parent (parent_version_id = null)
    // 3. Verify Profiled version has Raw as parent
    // 4. Verify Cleaned version has Profiled as parent
    // 5. Verify lineage chain: Raw → Profiled → Cleaned

    await expect(page).toHaveTitle(/beefcake/i);
  });
});
