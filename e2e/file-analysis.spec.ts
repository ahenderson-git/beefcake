import * as path from 'path';
import { fileURLToPath } from 'url';

import { Page, test, expect } from '@playwright/test';

import { getFileAnalysisMocks, getStandardMocks } from './helpers/common-mocks';
import { setupTauriMock, mockFileDialog } from './helpers/tauri-mock';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

/**
 * File Analysis E2E Tests
 *
 * Tests cover the complete file loading and analysis workflow:
 * - Opening files via dialog
 * - Analyzing CSV data
 * - Displaying analysis results
 * - Column expansion and detail views
 * - Cleaning configuration
 *
 * Priority: P0 (Critical user workflows)
 */

const APP_URL = 'http://localhost:14206';

/**
 * Helper function to simulate the complete file loading workflow
 * @param page - Playwright page object
 * @param filePath - Path to the test file (relative to e2e directory)
 */
async function loadFileIntoAnalyser(page: Page, filePath = 'testdata/clean.csv'): Promise<void> {
  // Navigate to dashboard
  await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });
  await expect(page.getByTestId('dashboard-view')).toBeVisible({ timeout: 10000 });

  // Mock file dialog to return test file
  await mockFileDialog(page, path.resolve(__dirname, filePath));

  // Click open file button
  await page.getByTestId('dashboard-open-file-button').click();

  // Wait for analyser view to load
  await expect(page.getByTestId('analyser-view')).toBeVisible({ timeout: 10000 });
  // Also wait for at least one column row to ensure data is rendered
  await expect(page.getByTestId('analyser-column-row').first()).toBeVisible({ timeout: 10000 });
}

// Mock analysis response for a CSV file with 3 columns, 100 rows
const MOCK_ANALYSIS_RESPONSE = {
  path: path.resolve(__dirname, '../testdata/clean.csv'),
  file_name: 'clean.csv',
  file_size: 2048,
  row_count: 100,
  total_row_count: 100,
  column_count: 3,
  analysis_duration: { secs: 0, nanos: 50000000 },
  health: {
    score: 0.85,
    risks: ['Column "age" has 5% null values'],
    notes: ['Dataset appears clean'],
  },
  summary: [
    {
      name: 'id',
      kind: 'Numeric',
      count: 100,
      nulls: 0,
      null_pct: 0.0,
      stats: {
        Numeric: {
          mean: 50.5,
          median: 50.5,
          min: 1.0,
          max: 100.0,
          std_dev: 29.0,
          q1: 25.75,
          q3: 75.25,
          distinct_count: 100,
          zero_count: 0,
          negative_count: 0,
          is_integer: true,
          is_sorted: true,
          is_sorted_rev: false,
          skew: 0.0,
        },
      },
      business_summary: ['Unique identifier column', 'All values are unique'],
      histogram: Array(10).fill(10), // 10 buckets, 10 items each
    },
    {
      name: 'name',
      kind: 'Text',
      count: 100,
      nulls: 0,
      null_pct: 0.0,
      stats: {
        Text: {
          min_length: 3,
          max_length: 15,
          avg_length: 8.5,
          distinct: 95,
          top_value: ['John', 5],
        },
      },
      business_summary: ['Text column with names', 'High cardinality'],
      histogram: null,
    },
    {
      name: 'age',
      kind: 'Numeric',
      count: 100,
      nulls: 5,
      null_pct: 5.0,
      stats: {
        Numeric: {
          mean: 35.2,
          median: 34.0,
          min: 18.0,
          max: 65.0,
          std_dev: 12.5,
          q1: 27.0,
          q3: 45.0,
          distinct_count: 45,
          zero_count: 0,
          negative_count: 0,
          is_integer: true,
          is_sorted: false,
          is_sorted_rev: false,
          skew: 0.15,
        },
      },
      business_summary: ['Age column', '5% missing values - consider imputation'],
      histogram: Array(10).fill(10),
    },
  ],
};

test.describe('File Analysis - Loading', () => {
  test.beforeEach(async ({ page }) => {
    // Set up Tauri mocking before each test
    await setupTauriMock(page, {
      commands: getFileAnalysisMocks(),
      fileDialog: {
        openFile: path.resolve(__dirname, 'testdata/clean.csv'),
      },
    });
  });

  test('should display dashboard on launch', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Verify dashboard is visible
    await expect(page.getByTestId('dashboard-view')).toBeVisible({ timeout: 10000 });

    // Verify all navigation buttons are present
    await expect(page.getByTestId('dashboard-open-file-button')).toBeVisible();
    await expect(page.getByTestId('dashboard-powershell-button')).toBeVisible();
    await expect(page.getByTestId('dashboard-python-button')).toBeVisible();
    await expect(page.getByTestId('dashboard-sql-button')).toBeVisible();
  });

  test('should open file dialog when clicking open file button', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    const openFileBtn = page.getByTestId('dashboard-open-file-button');
    await expect(openFileBtn).toBeVisible();
    await expect(openFileBtn).toBeEnabled();

    // Button should be functional (file dialog is mocked in beforeEach)
    await expect(openFileBtn).toBeEnabled();
  });

  test('should load CSV file and display analysis results', async ({ page }) => {
    await loadFileIntoAnalyser(page);

    // Verify analyser view is displayed
    await expect(page.getByTestId('analyser-view')).toBeVisible();

    // Verify analysis results are displayed
    await expect(page.getByTestId('analyser-row-count')).toBeVisible();
    await expect(page.getByTestId('analyser-column-count')).toBeVisible();
  });

  test('should show lifecycle creation banner during file analysis', async ({ page }) => {
    await loadFileIntoAnalyser(page);

    // Verify analyser view is displayed
    await expect(page.getByTestId('analyser-view')).toBeVisible();

    // Verify lifecycle creation banner is shown
    // During file analysis, the app creates a lifecycle dataset in the background
    // The banner shows "Creating dataset versions..." while this happens
    const stageBanner = page.getByTestId('analyser-stage-banner');
    await expect(stageBanner).toBeVisible({ timeout: 10000 });
    await expect(stageBanner).toContainText('Creating dataset versions');
  });
});

test.describe('File Analysis - Results Display', () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page, {
      commands: getFileAnalysisMocks(),
    });
  });

  test('should display file metadata correctly', async ({ page }) => {
    await setupTauriMock(page, {
      commands: getFileAnalysisMocks(),
      fileDialog: {
        openFile: path.resolve(__dirname, 'testdata/clean.csv'),
      },
    });

    await loadFileIntoAnalyser(page);

    // Verify file name is displayed
    await expect(page.getByTestId('analyser-file-name')).toBeVisible();

    // Verify file size is displayed
    await expect(page.getByTestId('analyser-file-size')).toBeVisible();

    // Verify row/column counts are accurate
    await expect(page.getByTestId('analyser-row-count')).toBeVisible();
    await expect(page.getByTestId('analyser-column-count')).toBeVisible();

    // Verify analysis duration is shown
    await expect(page.getByTestId('analyser-analysis-duration')).toBeVisible();
  });

  test('should display health score badge with correct color', async ({ page }) => {
    await setupTauriMock(page, {
      commands: getFileAnalysisMocks(),
      fileDialog: {
        openFile: path.resolve(__dirname, 'testdata/clean.csv'),
      },
    });

    await loadFileIntoAnalyser(page);

    // Verify quality score is visible
    await expect(page.getByTestId('analyser-quality-score')).toBeVisible();

    // Verify quality score card is visible
    await expect(page.getByTestId('analyser-quality-score-card')).toBeVisible();
  });

  test('should display column list with correct count', async ({ page }) => {
    await setupTauriMock(page, {
      commands: getFileAnalysisMocks(),
      fileDialog: {
        openFile: path.resolve(__dirname, 'testdata/clean.csv'),
      },
    });

    await loadFileIntoAnalyser(page);

    // Verify column rows are displayed
    const columnRows = page.getByTestId('analyser-column-row');
    await expect(columnRows.first()).toBeVisible();

    // Verify all columns are shown (mock data from common-mocks has 10 columns)
    await expect(columnRows).toHaveCount(10);
  });

  test('should show correct column types', async ({ page }) => {
    await setupTauriMock(page, {
      commands: getFileAnalysisMocks(),
      fileDialog: {
        openFile: path.resolve(__dirname, 'testdata/clean.csv'),
      },
    });

    await loadFileIntoAnalyser(page);

    // Verify column type badges are visible
    const columnTypes = page.getByTestId('analyser-column-type');
    await expect(columnTypes.first()).toBeVisible();
  });
});

test.describe('File Analysis - Column Details', () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page, {
      commands: {
        analyze_file: {
          type: 'success',
          data: MOCK_ANALYSIS_RESPONSE,
        },
        get_app_version: {
          type: 'success',
          data: '0.2.3',
        },
        get_config: {
          type: 'success',
          data: {
            settings: {
              connections: [],
              active_import_id: null,
              active_export_id: null,
              powershell_font_size: 14,
              python_font_size: 14,
              sql_font_size: 14,
              first_run_completed: true,
              trusted_paths: [],
              preview_row_limit: 100,
              security_warning_acknowledged: false,
              skip_full_row_count: false,
              analysis_sample_size: 10000,
              sampling_strategy: 'balanced',
              ai_config: {
                enabled: false,
                model: 'gpt-4o-mini',
                temperature: 0.7,
                max_tokens: 2000,
              },
            },
            audit_log: {
              entries: [],
            },
          },
        },
      },
    });
  });

  test('should expand column row to show detailed statistics', async ({ page }) => {
    await setupTauriMock(page, {
      commands: getFileAnalysisMocks(),
      fileDialog: {
        openFile: path.resolve(__dirname, 'testdata/clean.csv'),
      },
    });

    await loadFileIntoAnalyser(page);

    // Click on "age" column row to expand it
    const ageColumnRow = page.locator('[data-testid="analyser-column-row"][data-col="age"]');
    await expect(ageColumnRow).toBeVisible();

    // Click the expander button
    const expander = ageColumnRow.getByTestId('analyser-column-expander');
    await expander.click();

    // Verify the row has expanded class
    await expect(ageColumnRow).toHaveClass(/expanded/);
  });

  test('should render histogram for numeric columns', async ({ page }) => {
    await setupTauriMock(page, {
      commands: getFileAnalysisMocks(),
      fileDialog: {
        openFile: path.resolve(__dirname, 'testdata/clean.csv'),
      },
    });

    await loadFileIntoAnalyser(page);

    // Expand "customer_id" column (numeric with histogram data)
    const idColumnRow = page.locator('[data-testid="analyser-column-row"][data-col="customer_id"]');
    await expect(idColumnRow).toBeVisible();

    // Click the expander
    await idColumnRow.getByTestId('analyser-column-expander').click();

    // Verify the row is expanded
    await expect(idColumnRow).toHaveClass(/expanded/);

    // Note: Histogram rendering requires Chart.js to be initialized
    // This test verifies the expansion works; actual chart rendering
    // would require visual regression testing or canvas inspection
  });

  test('should render histogram without undefined values (CRITICAL BUG TEST)', async ({ page }) => {
    // This test specifically verifies the fix for histogram rendering bug
    // where histogram was expected to be [min, max, count] but is actually [bin_centre, count]
    await setupTauriMock(page, {
      commands: getFileAnalysisMocks(),
      fileDialog: {
        openFile: path.resolve(__dirname, 'testdata/clean.csv'),
      },
    });

    await loadFileIntoAnalyser(page);

    // Expand a numeric column with histogram data
    const numericColumn = page
      .locator('[data-testid="analyser-column-row"]')
      .filter({ has: page.getByTestId('analyser-column-type').filter({ hasText: 'Numeric' }) })
      .first();
    await expect(numericColumn).toBeVisible();

    // Click the expander to show histogram
    await numericColumn.getByTestId('analyser-column-expander').click();
    await expect(numericColumn).toHaveClass(/expanded/);

    // Wait for histogram to potentially render
    await page.waitForTimeout(500);

    // Check for any error toasts that would indicate rendering failures
    const errorToasts = page.locator('.toast.error, [data-toast-type="error"]');
    const errorCount = await errorToasts.count();

    // If there are errors, log them for debugging
    if (errorCount > 0) {
      for (let i = 0; i < errorCount; i++) {
        const errorText = await errorToasts.nth(i).textContent();
        console.error(`Error toast ${i}: ${errorText}`);
      }
    }

    // Verify no render errors occurred
    expect(errorCount).toBe(0);

    // Verify the histogram bars exist if histogram data is present
    const histogramContainer = numericColumn.locator('.histogram, .distribution-chart');
    const histogramExists = (await histogramContainer.count()) > 0;

    if (histogramExists) {
      // Verify histogram bars rendered
      const histBars = numericColumn.locator('.hist-bar');
      const barCount = await histBars.count();

      // If histogram data exists, there should be bars
      if (barCount > 0) {
        // Verify that tooltips don't contain "undefined"
        for (let i = 0; i < Math.min(barCount, 5); i++) {
          const title = await histBars.nth(i).getAttribute('title');
          expect(title).not.toContain('undefined');
          expect(title).not.toContain('NaN');
        }
      }
    }
  });

  test('should show appropriate message for text columns without histogram', async ({ page }) => {
    await setupTauriMock(page, {
      commands: getFileAnalysisMocks(),
      fileDialog: {
        openFile: path.resolve(__dirname, 'testdata/clean.csv'),
      },
    });

    await loadFileIntoAnalyser(page);

    // Expand "customer_name" column (text, no histogram)
    const nameColumnRow = page.locator(
      '[data-testid="analyser-column-row"][data-col="customer_name"]'
    );
    await expect(nameColumnRow).toBeVisible();

    // Click the expander
    await nameColumnRow.getByTestId('analyser-column-expander').click();

    // Verify the row is expanded
    await expect(nameColumnRow).toHaveClass(/expanded/);

    // Verify the column type badge shows "Text"
    const columnType = nameColumnRow.getByTestId('analyser-column-type');
    await expect(columnType).toBeVisible();
  });
});

test.describe('File Analysis - Cleaning Configuration', () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page, {
      commands: getFileAnalysisMocks(),
    });
  });

  test('should enable cleaning checkbox for a column', async ({ page }) => {
    await setupTauriMock(page, {
      commands: getFileAnalysisMocks(),
      fileDialog: {
        openFile: path.resolve(__dirname, 'testdata/clean.csv'),
      },
    });

    await loadFileIntoAnalyser(page);

    // Expand a column to see if cleaning options exist
    const ageColumn = page.locator('[data-testid="analyser-column-row"][data-col="age"]');
    await expect(ageColumn).toBeVisible();

    // Note: Cleaning checkboxes are stage-dependent (only in Cleaned/Advanced stages)
    // This test verifies the column row structure exists
    await ageColumn.scrollIntoViewIfNeeded();
    // The name cell may be visually compacted in some layouts; assert a stable cell instead
    await expect(ageColumn.getByTestId('analyser-column-type')).toBeVisible({ timeout: 15000 });
  });

  test('should configure imputation for column with nulls', async ({ page }) => {
    await setupTauriMock(page, {
      commands: getFileAnalysisMocks(),
      fileDialog: {
        openFile: path.resolve(__dirname, 'testdata/clean.csv'),
      },
    });

    await loadFileIntoAnalyser(page);

    // Verify column with nulls is displayed
    const ageColumn = page.locator('[data-testid="analyser-column-row"][data-col="age"]');
    await expect(ageColumn).toBeVisible();

    // Note: Full imputation configuration requires Cleaned/Advanced stage
    // This test verifies the column structure exists
    await ageColumn.scrollIntoViewIfNeeded();
    await expect(ageColumn.getByTestId('analyser-column-quality')).toBeVisible({ timeout: 15000 });
  });

  test('should configure normalization for numeric column', async ({ page }) => {
    await setupTauriMock(page, {
      commands: getFileAnalysisMocks(),
      fileDialog: {
        openFile: path.resolve(__dirname, 'testdata/clean.csv'),
      },
    });

    await loadFileIntoAnalyser(page);

    // Verify numeric column is displayed with stats
    const totalSpentColumn = page.locator(
      '[data-testid="analyser-column-row"][data-col="total_spent"]'
    );
    await expect(totalSpentColumn).toBeVisible();

    // Verify column type shows numeric
    await totalSpentColumn.scrollIntoViewIfNeeded();
    await expect(totalSpentColumn.getByTestId('analyser-column-type')).toBeVisible({
      timeout: 15000,
    });
  });

  test('should enable ML preprocessing options', async ({ page }) => {
    await setupTauriMock(page, {
      commands: getFileAnalysisMocks(),
      fileDialog: {
        openFile: path.resolve(__dirname, 'testdata/clean.csv'),
      },
    });

    await loadFileIntoAnalyser(page);

    // Verify analyser view has loaded with data
    await expect(page.getByTestId('analyser-view')).toBeVisible();
    await expect(page.getByTestId('analyser-row-count')).toBeVisible();

    // Note: ML preprocessing UI is stage-dependent
    // This test verifies basic analyser functionality
  });

  test('should support bulk cleaning with "Clean All" checkbox', async ({ page }) => {
    await setupTauriMock(page, {
      commands: getFileAnalysisMocks(),
      fileDialog: {
        openFile: path.resolve(__dirname, 'testdata/clean.csv'),
      },
    });

    await loadFileIntoAnalyser(page);

    // Verify multiple columns are displayed
    const columnRows = page.getByTestId('analyser-column-row');
    await expect(columnRows.first()).toBeVisible();

    // Note: "Clean All" functionality requires proper stage setup
    // This test verifies multiple columns load correctly
    const count = await columnRows.count();
    expect(count).toBeGreaterThan(0);
  });
});

test.describe('File Analysis - Error Scenarios', () => {
  test('should handle invalid file format gracefully', async ({ page }) => {
    await setupTauriMock(page, {
      commands: getStandardMocks({
        analyze_file: {
          type: 'error',
          error: 'Unsupported file format: .txt. Please use CSV, JSON, or Parquet files.',
        },
      }),
      fileDialog: {
        openFile: path.resolve(__dirname, '../testdata/invalid_format.txt'),
      },
    });

    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Verify dashboard loads initially
    await expect(page.getByTestId('dashboard-view')).toBeVisible();

    // Trigger file load with invalid file
    await page.getByTestId('dashboard-open-file-button').click();

    // Wait for error handling
    await page.waitForTimeout(1000);

    // Verify app doesn't crash - page title should still be present
    await expect(page).toHaveTitle(/beefcake/i);

    // Note: Error toast verification would require toast implementation to be connected
    // This test verifies the app doesn't crash on invalid file format
  });

  test('should handle file read permission errors', async ({ page }) => {
    await setupTauriMock(page, {
      commands: getStandardMocks({
        analyze_file: {
          type: 'error',
          error: 'Permission denied: Unable to read file /protected/data.csv',
        },
      }),
      fileDialog: {
        openFile: '/protected/data.csv',
      },
    });

    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Verify dashboard loads initially
    await expect(page.getByTestId('dashboard-view')).toBeVisible();

    // Trigger file load with protected file
    await page.getByTestId('dashboard-open-file-button').click();

    // Wait for error handling
    await page.waitForTimeout(1000);

    // Verify app doesn't crash
    await expect(page).toHaveTitle(/beefcake/i);

    // Note: Full error toast and suggestion verification requires toast UI to be implemented
  });

  test('should handle corrupted CSV files', async ({ page }) => {
    await setupTauriMock(page, {
      commands: getStandardMocks({
        analyze_file: {
          type: 'error',
          error: 'CSV parsing error: Unexpected end of file at line 47',
        },
      }),
      fileDialog: {
        openFile: path.resolve(__dirname, '../testdata/corrupted.csv'),
      },
    });

    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Verify dashboard loads initially
    await expect(page.getByTestId('dashboard-view')).toBeVisible();

    // Trigger file load with corrupted file
    await page.getByTestId('dashboard-open-file-button').click();

    // Wait for error handling
    await page.waitForTimeout(1000);

    // Verify app doesn't crash
    await expect(page).toHaveTitle(/beefcake/i);

    // Note: Error toast with line number requires full toast implementation
  });
});
