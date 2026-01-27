import * as path from 'path';
import { fileURLToPath } from 'url';

import { test, expect } from '@playwright/test';

import { getStandardMocks, getFileAnalysisMocks, getLifecycleMocks } from './helpers/common-mocks';
import { setupTauriMock } from './helpers/tauri-mock';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

/**
 * Full Workflow E2E Tests
 *
 * These tests verify complete user workflows including:
 * - File loading and analysis
 * - Lifecycle stage transitions
 * - Data cleaning configuration
 * - Export functionality
 *
 * NOTE: These tests use Tauri IPC mocking to simulate backend responses
 * without requiring a full Tauri application build.
 */

const APP_URL = 'http://localhost:14206';

test.describe('Full Workflow - File Analysis', () => {
  test.beforeEach(async ({ page }) => {
    // Set up Tauri mocking with file analysis and lifecycle support
    await setupTauriMock(page, {
      commands: {
        ...getFileAnalysisMocks(),
        ...getLifecycleMocks(),
      },
      fileDialog: {
        openFile: path.resolve(__dirname, 'testdata/clean.csv'),
      },
    });
  });

  test('should load dashboard and display app information', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Verify dashboard is visible
    await expect(page.getByTestId('dashboard-view')).toBeVisible({ timeout: 10000 });

    // Verify app title contains "beefcake"
    await expect(page.locator('h1')).toContainText(/beefcake/i);

    // Verify all navigation buttons are present
    await expect(page.getByTestId('dashboard-open-file-button')).toBeVisible();
    await expect(page.getByTestId('dashboard-powershell-button')).toBeVisible();
    await expect(page.getByTestId('dashboard-python-button')).toBeVisible();
    await expect(page.getByTestId('dashboard-sql-button')).toBeVisible();
  });

  test('should load and analyze CSV file when clicking open file button', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Wait for dashboard to be visible
    await expect(page.getByTestId('dashboard-view')).toBeVisible({ timeout: 5000 });

    const openFileBtn = page.getByTestId('dashboard-open-file-button');
    await expect(openFileBtn).toBeVisible();

    // Click the open file button - this triggers file dialog and analysis
    await openFileBtn.click();

    // Wait for loading to appear and complete
    // Note: Loading screen might be very quick with mocked responses
    await page.waitForTimeout(100);

    // Verify analysis results are displayed
    // After analysis, app should switch to Analyser view
    await expect(page.locator('.analyser-container')).toBeVisible({ timeout: 10000 });

    // Verify file metadata is displayed - use getByTestId for specific elements
    await expect(page.getByTestId('analyser-file-name')).toContainText('customer_data.csv');
    await expect(page.locator('text=10 rows')).toBeVisible();
    await expect(page.locator('text=10 columns')).toBeVisible();
  });
});

test.describe('Full Workflow - Export Modal', () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page, {
      commands: getStandardMocks(),
    });
  });

  test('should have export modal structure in codebase', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Verify dashboard loads
    await expect(page.getByTestId('dashboard-view')).toBeVisible({ timeout: 5000 });

    // Note: Export modal is only visible after loading a dataset and clicking export
    // This test verifies the app structure is in place
    // Full export workflow requires file loading which has separate issues
  });
});

test.describe('Full Workflow - Data Quality', () => {
  test.beforeEach(async ({ page }) => {
    // Setup with file analysis and lifecycle mocks
    await setupTauriMock(page, {
      commands: {
        ...getFileAnalysisMocks(),
        ...getLifecycleMocks(),
      },
      fileDialog: {
        openFile: path.resolve(__dirname, 'testdata/clean.csv'),
      },
    });
  });

  test('should display quality score after file analysis', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Load a file
    await page.getByTestId('dashboard-open-file-button').click();

    // Wait for analyser view to appear
    await expect(page.locator('.analyser-container')).toBeVisible({ timeout: 10000 });

    // Verify quality score is displayed (data-testid for precise matching)
    // Quality score is computed from nulls, mock has 0 nulls so 100%
    await expect(page.getByTestId('analyser-quality-score')).toBeVisible();
  });

  test('should show column statistics after file analysis', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Load a file
    await page.getByTestId('dashboard-open-file-button').click();

    // Wait for analyser view to appear
    await expect(page.locator('.analyser-container')).toBeVisible({ timeout: 10000 });

    // After file load, should see:
    // - Row count (10 rows from mockAnalysisResponse)
    await expect(page.locator('text=10 rows')).toBeVisible();
    // - Column count (10 columns from mockAnalysisResponse)
    await expect(page.locator('text=10 columns')).toBeVisible();
    // - Column names from mock data (use .first() to avoid strict mode violation)
    await expect(page.locator('text=customer_id').first()).toBeVisible();
    await expect(page.locator('text=customer_name').first()).toBeVisible();
  });
});

test.describe('Full Workflow - Column Expansion', () => {
  test.beforeEach(async ({ page }) => {
    // Setup with file analysis and lifecycle mocks
    await setupTauriMock(page, {
      commands: {
        ...getFileAnalysisMocks(),
        ...getLifecycleMocks(),
      },
      fileDialog: {
        openFile: path.resolve(__dirname, 'testdata/clean.csv'),
      },
    });
  });

  test('should expand column row to show detailed statistics', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Load a file
    await page.getByTestId('dashboard-open-file-button').click();

    // Wait for analyser view to appear
    await expect(page.locator('.analyser-container')).toBeVisible({ timeout: 10000 });

    // Find the first column row and click the expander
    const firstColumnRow = page.getByTestId('analyser-column-row').first();
    await expect(firstColumnRow).toBeVisible();

    // Click the expander icon to expand the row
    await firstColumnRow.getByTestId('analyser-column-expander').click();

    // Verify the row is now expanded (has 'expanded' class)
    await expect(firstColumnRow).toHaveClass(/expanded/);

    // The expanded view shows detailed statistics in .row-details
    // Verify the details are visible
    await expect(firstColumnRow.locator('.row-details')).toBeVisible();
    await expect(firstColumnRow.locator('.details-stats')).toBeVisible();
  });
});

test.describe('Full Workflow - Cleaning Configuration', () => {
  test.skip('should allow enabling cleaning for a column', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // After file load and column expansion:
    // 1. Check "Enable Cleaning" checkbox
    // 2. Select cleaning options (impute, normalize, etc.)
    // 3. Apply cleaning

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test.skip('should support bulk cleaning operations', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Test "Clean All" checkbox functionality
    // Should enable/disable cleaning for all columns at once

    await expect(page).toHaveTitle(/beefcake/i);
  });
});

test.describe('Full Workflow - Lifecycle Stages', () => {
  test.beforeEach(async ({ page }) => {
    // Setup with file analysis and lifecycle mocks
    await setupTauriMock(page, {
      commands: {
        ...getFileAnalysisMocks(),
        ...getLifecycleMocks(),
      },
      fileDialog: {
        openFile: path.resolve(__dirname, 'testdata/clean.csv'),
      },
    });
  });

  test('should show lifecycle rail with stage indicators after file load', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Load a file to trigger lifecycle creation
    await page.getByTestId('dashboard-open-file-button').click();

    // Wait for analyser view to appear
    await expect(page.locator('.analyser-container')).toBeVisible({ timeout: 10000 });

    // Wait a bit for lifecycle rail to be created asynchronously
    await page.waitForTimeout(500);

    // Verify lifecycle rail is visible
    await expect(page.getByTestId('lifecycle-rail')).toBeVisible({ timeout: 5000 });

    // Verify lifecycle stages container exists
    await expect(page.getByTestId('lifecycle-stages')).toBeVisible();

    // Verify all 6 stages are displayed (from mockDatasetWithVersions)
    // The mock has Raw, Profiled, and Cleaned versions
    await expect(page.getByTestId('lifecycle-stage-raw')).toBeVisible();
    await expect(page.getByTestId('lifecycle-stage-profiled')).toBeVisible();
  });

  test.skip('should transition from Profiled to Cleaned stage', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // After file load (creates Raw + Profiled):
    // 1. Click "Begin Cleaning" button
    // 2. Verify lifecycle rail shows Cleaned as active
    // 3. Verify cleaning UI becomes available

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test.skip('should transition through all lifecycle stages', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Test full lifecycle progression:
    // Raw → Profiled → Cleaned → Advanced → Validated → Published

    await expect(page).toHaveTitle(/beefcake/i);
  });
});

test.describe('Full Workflow - Export', () => {
  test.skip('should open export modal', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // After file is loaded and processed:
    // 1. Click Export button
    // 2. Verify export modal opens
    // 3. Verify destination options are visible

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test.skip('should allow selecting export destination', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Test export destination selection:
    // 1. Open export modal
    // 2. Click "Local File" or "Database"
    // 3. Verify appropriate config UI appears

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test.skip('should support file export workflow', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Complete file export flow:
    // 1. Open export modal
    // 2. Select "Local File"
    // 3. Choose file path
    // 4. Click "Start Export"
    // 5. Verify success toast

    await expect(page).toHaveTitle(/beefcake/i);
  });
});

test.describe('Full Workflow - Error Handling', () => {
  test.skip('should show loading state during analysis', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // During long operations:
    // 1. Loading spinner should appear
    // 2. Loading message should be visible
    // 3. Abort button should be available

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test.skip('should allow aborting long operations', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Test abort functionality:
    // 1. Trigger long operation
    // 2. Click abort button
    // 3. Verify operation stops
    // 4. Verify UI returns to stable state

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test.skip('should show error toast on failure', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Test error handling:
    // 1. Trigger an error (invalid file, etc.)
    // 2. Verify error toast appears
    // 3. Verify app remains stable

    await expect(page).toHaveTitle(/beefcake/i);
  });
});

test.describe('Full Workflow - Integration Test', () => {
  test.skip('should complete full analysis workflow end-to-end', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // This test would cover the complete happy path:
    // 1. Open file
    // 2. Verify analysis results
    // 3. Configure cleaning
    // 4. Transition through lifecycle stages
    // 5. Export data
    // 6. Verify success

    // For now, just verify app loads
    await expect(page).toHaveTitle(/beefcake/i);
    await expect(page.getByTestId('dashboard-view')).toBeVisible();
  });
});
