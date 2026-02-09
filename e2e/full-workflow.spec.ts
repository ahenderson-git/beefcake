import * as path from 'path';
import { fileURLToPath } from 'url';

import { test, expect } from '@playwright/test';

import { getFileAnalysisMocks, getLifecycleMocks } from './helpers/common-mocks';
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
    await expect(page.getByTestId('analyser-row-count')).toContainText('10 rows');
    await expect(page.getByTestId('analyser-column-count')).toContainText('10 columns');
  });
});

test.describe('Full Workflow - Export Modal', () => {
  test.beforeEach(async ({ page }) => {
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

  test('should open export modal after file analysis', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Load a file first
    await page.getByTestId('dashboard-open-file-button').click();
    await expect(page.locator('.analyser-container')).toBeVisible({ timeout: 10000 });

    // Click export button
    await page.getByTestId('analyser-export-button').click();

    // Verify export modal opens
    await expect(page.getByTestId('export-modal-overlay')).toBeVisible({ timeout: 5000 });
    await expect(page.getByTestId('export-modal')).toBeVisible();

    // Verify modal header
    await expect(page.getByTestId('export-modal').locator('h3')).toContainText('Export Data');
  });

  test('should allow selecting export destination', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Load a file first
    await page.getByTestId('dashboard-open-file-button').click();
    await expect(page.locator('.analyser-container')).toBeVisible({ timeout: 10000 });

    // Open export modal
    await page.getByTestId('analyser-export-button').click();
    await expect(page.getByTestId('export-modal')).toBeVisible({ timeout: 5000 });

    // Verify File destination is active by default
    await expect(page.getByTestId('export-dest-file')).toHaveClass(/active/);

    // Click Database destination
    await page.getByTestId('export-dest-database').click();

    // Verify Database destination becomes active
    await expect(page.getByTestId('export-dest-database')).toHaveClass(/active/);
    await expect(page.getByTestId('export-dest-file')).not.toHaveClass(/active/);

    // Verify database config appears
    await expect(page.getByTestId('export-db-connection-select')).toBeVisible();
  });

  test('should have file export UI elements visible', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Load a file
    await page.getByTestId('dashboard-open-file-button').click();
    await expect(page.locator('.analyser-container')).toBeVisible({ timeout: 10000 });

    // Open export modal
    await page.getByTestId('analyser-export-button').click();
    await expect(page.getByTestId('export-modal')).toBeVisible({ timeout: 5000 });

    // Verify File destination is selected
    await expect(page.getByTestId('export-dest-file')).toHaveClass(/active/);

    // Verify file export UI elements
    await expect(page.getByTestId('export-file-path-input')).toBeVisible();
    await expect(page.getByTestId('export-file-browse-button')).toBeVisible();
    await expect(page.getByTestId('export-confirm-button')).toBeVisible();
    await expect(page.getByTestId('export-cancel-button')).toBeVisible();
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
    await expect(page.getByTestId('analyser-row-count')).toContainText('10 rows');
    // - Column count (10 columns from mockAnalysisResponse)
    await expect(page.getByTestId('analyser-column-count')).toContainText('10 columns');
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
  test.beforeEach(async ({ page }) => {
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

  test('should show column details in read-only mode during ad-hoc analysis', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Load a file for ad-hoc analysis (no dataset)
    await page.getByTestId('dashboard-open-file-button').click();
    await expect(page.locator('.analyser-container')).toBeVisible({ timeout: 10000 });

    // Verify we're in Profiled stage (read-only for ad-hoc analysis)
    const progressBar = page.getByTestId('analyser-progress-bar');
    await expect(progressBar.locator('[data-stage="Profiled"]')).toHaveClass(/stage-current/);

    // Expand first column to view details
    const firstColumnRow = page.getByTestId('analyser-column-row').first();
    await firstColumnRow.click();
    await expect(firstColumnRow).toHaveClass(/expanded/);

    // In read-only mode, cleaning config section should NOT be visible
    await expect(firstColumnRow.locator('.details-config')).not.toBeVisible();

    // But statistics should still be visible for inspection
    await expect(firstColumnRow.locator('.details-stats')).toBeVisible();

    // In Profiled stage, checkboxes remain enabled to allow column deselection
    const checkbox = page.getByTestId('analyser-column-checkbox').first();
    await expect(checkbox).toBeEnabled();
  });

  test('should hide editing controls in read-only mode', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Load a file for ad-hoc analysis (no dataset)
    await page.getByTestId('dashboard-open-file-button').click();
    await expect(page.locator('.analyser-container')).toBeVisible({ timeout: 10000 });

    // Verify we're in Profiled stage (read-only)
    const progressBar = page.getByTestId('analyser-progress-bar');
    await expect(progressBar.locator('[data-stage="Profiled"]')).toHaveClass(/stage-current/);

    // Bulk editing controls like "Clean All" should NOT be visible in read-only mode
    await expect(page.locator('#btn-clean-all')).not.toBeVisible();

    // But export button should still be available
    await expect(page.getByTestId('analyser-export-button')).toBeVisible();

    // In Profiled stage, checkboxes remain enabled to allow column deselection
    const firstCheckbox = page.getByTestId('analyser-column-checkbox').first();
    await expect(firstCheckbox).toBeEnabled();
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

  test('should show lifecycle creation banner after file load', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Load a file to trigger lifecycle creation
    await page.getByTestId('dashboard-open-file-button').click();

    // Wait for analyser view to appear
    await expect(page.locator('.analyser-container')).toBeVisible({ timeout: 10000 });

    // Verify lifecycle creation banner is shown
    // The app creates a lifecycle dataset in the background after file analysis
    const stageBanner = page.getByTestId('analyser-stage-banner');
    await expect(stageBanner).toBeVisible({ timeout: 5000 });

    // Banner should show "Creating dataset versions..." during the lifecycle creation process
    await expect(stageBanner).toContainText('Creating dataset versions');
  });

  test('should show lifecycle progress bar after file analysis', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Load a file
    await page.getByTestId('dashboard-open-file-button').click();
    await expect(page.locator('.analyser-container')).toBeVisible({ timeout: 10000 });

    // Verify progress bar is visible
    await expect(page.getByTestId('analyser-progress-bar')).toBeVisible();

    // Verify at least Raw and Profiled stages are shown
    const progressBar = page.getByTestId('analyser-progress-bar');
    await expect(progressBar.locator('[data-stage="Raw"]')).toBeVisible();
    await expect(progressBar.locator('[data-stage="Profiled"]')).toBeVisible();

    // Verify Profiled stage has current styling
    await expect(progressBar.locator('[data-stage="Profiled"]')).toHaveClass(/stage-current/);
  });

  test('should display all lifecycle stages in progress bar', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Load a file
    await page.getByTestId('dashboard-open-file-button').click();
    await expect(page.locator('.analyser-container')).toBeVisible({ timeout: 10000 });

    // Verify all 6 lifecycle stages are present
    const progressBar = page.getByTestId('analyser-progress-bar');
    const stages = ['Raw', 'Profiled', 'Cleaned', 'Advanced', 'Validated', 'Published'];

    for (const stage of stages) {
      await expect(progressBar.locator(`[data-stage="${stage}"]`)).toBeVisible();
    }

    // Verify locked stages have lock icon
    await expect(progressBar.locator('[data-stage="Cleaned"].stage-locked')).toBeVisible();
    await expect(progressBar.locator('[data-stage="Advanced"].stage-locked')).toBeVisible();
  });
});

// Export tests moved to "Full Workflow - Export Modal" section above

// Note: Error handling tests (loading, abort, error toasts) are comprehensively
// covered in error-handling.spec.ts and are not duplicated here.

test.describe('Full Workflow - Integration Test', () => {
  test('should complete full analysis workflow end-to-end', async ({ page }) => {
    // Setup comprehensive mocks
    await setupTauriMock(page, {
      commands: {
        ...getFileAnalysisMocks(),
        ...getLifecycleMocks(),
      },
      fileDialog: {
        openFile: path.resolve(__dirname, 'testdata/clean.csv'),
      },
    });

    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // 1. Open file from dashboard
    await expect(page.getByTestId('dashboard-view')).toBeVisible({ timeout: 5000 });
    await page.getByTestId('dashboard-open-file-button').click();

    // 2. Verify analysis results
    await expect(page.locator('.analyser-container')).toBeVisible({ timeout: 10000 });
    await expect(page.getByTestId('analyser-row-count')).toContainText('10 rows');
    await expect(page.getByTestId('analyser-column-count')).toContainText('10 columns');
    await expect(page.getByTestId('analyser-quality-score')).toBeVisible();

    // 3. Verify we're in Profiled stage (read-only for ad-hoc analysis)
    const progressBar = page.getByTestId('analyser-progress-bar');
    await expect(progressBar).toBeVisible();
    await expect(progressBar.locator('[data-stage="Profiled"]')).toHaveClass(/stage-current/);

    // 4. Expand column to view statistics (read-only)
    const firstRow = page.getByTestId('analyser-column-row').first();
    await firstRow.click();
    await expect(firstRow).toHaveClass(/expanded/);

    // In read-only mode, config controls should not be visible
    await expect(firstRow.locator('.details-config')).not.toBeVisible();

    // But statistics should be visible
    await expect(firstRow.locator('.details-stats')).toBeVisible();

    // 5. Open export modal
    await page.getByTestId('analyser-export-button').click();
    await expect(page.getByTestId('export-modal')).toBeVisible();

    // 6. Verify export destinations work
    await expect(page.getByTestId('export-dest-file')).toHaveClass(/active/);
    await page.getByTestId('export-dest-database').click();
    await expect(page.getByTestId('export-dest-database')).toHaveClass(/active/);

    // Close modal and verify app remains stable
    await page.getByTestId('export-modal-close').click();
    await expect(page.locator('.analyser-container')).toBeVisible();
  });
});
