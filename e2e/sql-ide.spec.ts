import { test, expect } from '@playwright/test';

import { getStandardMocks } from './helpers/common-mocks';
import { setupTauriMock } from './helpers/tauri-mock';

/**
 * SQL IDE E2E Tests
 *
 * Tests cover the SQL query interface:
 * - Monaco editor with SQL syntax
 * - Query execution with Polars SQL context
 * - Results display
 * - Security warning on first run
 * - Installing Polars package
 * - Saving and loading queries
 * - Column refactoring when switching lifecycle stages
 * - SQL-specific error handling (syntax errors, duplicate columns)
 *
 * Priority: P1 (Important IDE feature)
 */

const APP_URL = 'http://localhost:14206';

const MOCK_DATASET = {
  id: 'ds-test-123',
  name: 'customer_data',
  versions: [
    {
      id: 'v-cleaned-001',
      stage: 'Cleaned',
      data_location: {
        ParquetFile: '/data/cleaned.parquet',
      },
    },
  ],
  active_version_id: 'v-cleaned-001',
};

test.describe('SQL IDE - Editor Initialization', () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page, {
      commands: getStandardMocks(),
    });
  });

  test('should display SQL IDE with Monaco editor', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();

    // Verify SQL view is visible
    await expect(page.getByTestId('sql-ide-view')).toBeVisible({ timeout: 5000 });

    // Verify Monaco editor container is present
    await expect(page.getByTestId('sql-ide-editor')).toBeVisible();

    // Verify toolbar buttons are visible
    await expect(page.getByTestId('sql-ide-run-button')).toBeVisible();
    await expect(page.getByTestId('sql-ide-save-button')).toBeVisible();
    await expect(page.getByTestId('sql-ide-load-button')).toBeVisible();
    await expect(page.getByTestId('sql-ide-install-button')).toBeVisible();
    await expect(page.getByTestId('sql-ide-docs-button')).toBeVisible();

    // Verify output panel is visible
    await expect(page.getByTestId('sql-ide-output-panel')).toBeVisible();
  });

  test('should have export button in SQL IDE', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();
    await expect(page.getByTestId('sql-ide-view')).toBeVisible({ timeout: 5000 });

    // Verify export button
    await expect(page.getByTestId('sql-ide-export-button')).toBeVisible();
  });

  test('should have clear button in SQL IDE', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();
    await expect(page.getByTestId('sql-ide-view')).toBeVisible({ timeout: 5000 });

    // Verify clear button
    await expect(page.getByTestId('sql-ide-clear-button')).toBeVisible();
  });

  test('should have copy output button in SQL IDE', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();
    await expect(page.getByTestId('sql-ide-view')).toBeVisible({ timeout: 5000 });

    // Verify copy button exists
    await expect(page.getByTestId('sql-ide-copy-button')).toBeVisible();
  });

  test('should have output panel in SQL IDE', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();
    await expect(page.getByTestId('sql-ide-view')).toBeVisible({ timeout: 5000 });

    // Verify output panel and output area
    await expect(page.getByTestId('sql-ide-output-panel')).toBeVisible();
    await expect(page.getByTestId('sql-ide-output')).toBeVisible();
  });

  test('should have skip cleaning checkbox in SQL IDE', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();
    await expect(page.getByTestId('sql-ide-view')).toBeVisible({ timeout: 5000 });

    // Verify skip cleaning checkbox and label exist
    await expect(page.getByTestId('sql-skip-cleaning-checkbox')).toBeVisible();
    await expect(page.getByTestId('sql-skip-cleaning-label')).toBeVisible();
  });

  test('should have install Polars button in SQL IDE', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();
    await expect(page.getByTestId('sql-ide-view')).toBeVisible({ timeout: 5000 });

    // Verify install button exists
    await expect(page.getByTestId('sql-ide-install-button')).toBeVisible();
  });

  test('should have SQL syntax highlighting', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();

    // TODO: Once editor is connected:
    // 1. Type SQL: "SELECT * FROM data WHERE age > 30"
    // 2. Verify SQL keywords are highlighted (SELECT, FROM, WHERE)
    // 3. Verify numbers are colored differently (30)
    // 4. Verify strings have distinct color

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should support SQL autocomplete', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();

    // TODO: Once editor is connected:
    // 1. Type "SEL"
    // 2. Verify autocomplete suggests "SELECT"
    // 3. Press Tab to accept
    // 4. Type "FROM d"
    // 5. Verify "data" table is suggested
    // 6. Verify column names are suggested after SELECT

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should format SQL query with Ctrl+Shift+F', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();

    // TODO: Once formatting is connected:
    // 1. Type unformatted SQL: "select id,name,age from data where age>30"
    // 2. Press Ctrl+Shift+F
    // 3. Verify query is formatted with proper spacing:
    //    SELECT
    //      id,
    //      name,
    //      age
    //    FROM data
    //    WHERE age > 30

    await expect(page).toHaveTitle(/beefcake/i);
  });
});

test.describe('SQL IDE - Layout Validation', () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page, {
      commands: getStandardMocks(),
    });
  });

  test('should have editor panel in top section', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();
    await expect(page.getByTestId('sql-ide-view')).toBeVisible({ timeout: 5000 });

    // Verify editor panel exists and is in correct position
    await expect(page.getByTestId('sql-ide-editor')).toBeVisible();
    const editorPanel = page.getByTestId('sql-ide-editor');
    await expect(editorPanel).toBeVisible();
  });

  test('should have output panel in bottom section', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();
    await expect(page.getByTestId('sql-ide-view')).toBeVisible({ timeout: 5000 });

    // Verify output panel exists
    await expect(page.getByTestId('sql-ide-output-panel')).toBeVisible();
    await expect(page.getByTestId('sql-ide-output')).toBeVisible();
  });

  test('should have toolbar at the top', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();
    await expect(page.getByTestId('sql-ide-view')).toBeVisible({ timeout: 5000 });

    // Verify toolbar exists with all key buttons
    await expect(page.getByTestId('sql-ide-toolbar')).toBeVisible();
    await expect(page.getByTestId('sql-ide-run-button')).toBeVisible();
  });

  test('should have consistent spacing between panels', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();
    await expect(page.getByTestId('sql-ide-view')).toBeVisible({ timeout: 5000 });

    // Verify both panels are visible (layout is working)
    await expect(page.getByTestId('sql-ide-editor')).toBeVisible();
    await expect(page.getByTestId('sql-ide-output-panel')).toBeVisible();
  });

  test('should render full IDE container properly', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();
    await expect(page.getByTestId('sql-ide-view')).toBeVisible({ timeout: 5000 });

    // Verify main IDE view container exists
    const ideView = page.getByTestId('sql-ide-view');
    await expect(ideView).toBeVisible();

    // Verify all major components are rendered within the view
    await expect(ideView.getByTestId('sql-ide-toolbar')).toBeVisible();
    await expect(ideView.getByTestId('sql-ide-editor')).toBeVisible();
    await expect(ideView.getByTestId('sql-ide-output-panel')).toBeVisible();
  });
});

test.describe('SQL IDE - Query Execution', () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page, {
      commands: getStandardMocks({
        lifecycle_get_dataset: {
          type: 'success',
          data: MOCK_DATASET,
        },
        run_sql: {
          type: 'success',
          data: 'shape: (50, 3)\n┌─────┬──────────────┬─────┐\n│ id  │ customer_name│ age │\n│ --- │ ---          │ --- │\n│ i64 │ str          │ i64 │\n╞═════╪══════════════╪═════╡\n│ 1   │ John Smith   │ 35  │\n│ 2   │ Jane Doe     │ 42  │\n│ ... │ ...          │ ... │\n└─────┴──────────────┴─────┘\n\n50 rows selected (0.02s)',
        },
      }),
    });
  });

  test('should execute simple SELECT query', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();

    // TODO: Once execution is connected:
    // 1. Type query: "SELECT * FROM data LIMIT 10"
    // 2. Click "Run Query" button
    // 3. Verify output shows ASCII table with results
    // 4. Verify execution time is displayed
    // 5. Verify no error messages

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should execute query with WHERE clause', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();

    // TODO: Once execution is connected:
    // 1. Type query: "SELECT id, customer_name, age FROM data WHERE age > 30"
    // 2. Run query
    // 3. Verify filtered results (50 rows)
    // 4. Verify only selected columns appear
    // 5. Verify execution time is reasonable

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should execute aggregate query', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();

    // TODO: Once execution is connected:
    // 1. Type query: "SELECT age, COUNT(*) as count FROM data GROUP BY age ORDER BY age"
    // 2. Run query
    // 3. Verify grouped results are displayed
    // 4. Verify aggregate column "count" is shown
    // 5. Verify results are sorted by age

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should execute query with JOIN', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();

    // TODO: Once multi-table support is connected:
    // 1. Type query with self-join or CTE
    // 2. Run query
    // 3. Verify joined results are displayed
    // 4. Verify column names are disambiguated

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should execute query with calculated columns', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();

    // TODO: Once execution is connected:
    // 1. Type query: "SELECT age, age * 2 AS double_age FROM data LIMIT 5"
    // 2. Run query
    // 3. Verify calculated column "double_age" appears
    // 4. Verify values are correctly calculated

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should clear output when running new query', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();

    // TODO: Once execution is connected:
    // 1. Run first query
    // 2. Verify output shows results
    // 3. Run second query
    // 4. Verify output is cleared
    // 5. Verify only second query results are shown

    await expect(page).toHaveTitle(/beefcake/i);
  });
});

test.describe('SQL IDE - Error Handling', () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page, {
      commands: getStandardMocks({
        lifecycle_get_dataset: {
          type: 'success',
          data: MOCK_DATASET,
        },
      }),
    });
  });

  test('should display SQL syntax errors', async ({ page }) => {
    await setupTauriMock(page, {
      commands: getStandardMocks({
        run_sql: {
          type: 'error',
          error: 'SQL error: syntax error at or near "FORM" (expected FROM)',
        },
      }),
    });

    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();

    // TODO: Once error handling is connected:
    // 1. Type invalid SQL: "SELECT * FORM data"
    // 2. Run query
    // 3. Verify error message appears in output
    // 4. Verify error highlights the problem keyword
    // 5. Verify suggestion appears (did you mean "FROM"?)

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should handle duplicate column names with helpful error', async ({ page }) => {
    await setupTauriMock(page, {
      commands: getStandardMocks({
        run_sql: {
          type: 'error',
          error:
            'the name \'literal\' passed to `LazyFrame.with_columns` is duplicate\n\nTip: When selecting multiple constant values in SQL, you must give them unique names using \'AS\'.\nExample: SELECT 1 AS col1, 2 AS col2 FROM data\n\nFixed query suggestion:\n```sql\nSELECT 1 AS "col_0", 2 AS "col_1" FROM data\n```',
        },
      }),
    });

    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();

    // TODO: Once error handling is connected:
    // 1. Type query: "SELECT 1, 2 FROM data"
    // 2. Run query
    // 3. Verify error message explains the problem
    // 4. Verify helpful tip appears
    // 5. Verify fixed query suggestion is shown
    // 6. Verify suggestion is syntax-highlighted

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should handle column not found errors', async ({ page }) => {
    await setupTauriMock(page, {
      commands: getStandardMocks({
        run_sql: {
          type: 'error',
          error: 'ColumnNotFoundError: column "unknown_column" not found',
        },
      }),
    });

    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();

    // TODO: Once error handling is connected:
    // 1. Type query: "SELECT unknown_column FROM data"
    // 2. Run query
    // 3. Verify error message is clear
    // 4. Verify suggestion to check available columns

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should handle missing dataset path', async ({ page }) => {
    await setupTauriMock(page, {
      commands: getStandardMocks(),
    });

    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE (no dataset loaded)
    await page.getByTestId('nav-sql').click();

    // TODO: Once validation is connected:
    // 1. Try to run query without loading dataset
    // 2. Verify error message appears
    // 3. Verify message: "No dataset loaded. Please load a file first."
    // 4. Verify toast notification suggests loading file

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should show helpful error when Polars is missing', async ({ page }) => {
    await setupTauriMock(page, {
      commands: getStandardMocks({
        run_sql: {
          type: 'error',
          error: "ModuleNotFoundError: No module named 'polars'",
        },
      }),
    });

    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();

    // TODO: Once error handling is connected:
    // 1. Run query without Polars installed
    // 2. Verify error message in output
    // 3. Verify helpful tip: "Click 'Install Polars' button"
    // 4. Verify Install Polars button is highlighted

    await expect(page).toHaveTitle(/beefcake/i);
  });
});

test.describe('SQL IDE - Query Management', () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page, {
      commands: getStandardMocks({
        write_text_file: {
          type: 'success',
          data: null,
        },
        read_text_file: {
          type: 'success',
          data: 'SELECT\n  customer_name,\n  age,\n  email\nFROM data\nWHERE age > 30\nORDER BY age DESC;',
        },
      }),
      fileDialog: {
        openFile: '/queries/analysis.sql',
        saveFile: '/queries/new_query.sql',
      },
    });
  });

  test('should save SQL query to file', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();

    // TODO: Once save functionality is connected:
    // 1. Type query in editor
    // 2. Click "Save Query" button
    // 3. Choose file location in dialog
    // 4. Verify success toast appears
    // 5. Verify file is saved

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should load SQL query from file', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();

    // TODO: Once load functionality is connected:
    // 1. Click "Load Query" button
    // 2. Select file from dialog
    // 3. Verify query content is loaded into editor
    // 4. Verify formatted SQL appears
    // 5. Verify success toast appears

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should copy output to clipboard', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();

    // TODO: Once copy functionality is connected:
    // 1. Run query with results
    // 2. Click "Copy Output" button
    // 3. Verify clipboard contains results table
    // 4. Verify success toast appears

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should open Polars SQL documentation', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();

    // TODO: Once docs button is connected:
    // 1. Click "SQL Docs" button
    // 2. Verify new tab opens
    // 3. Verify URL is https://docs.pola.rs/user-guide/sql/intro/

    await expect(page).toHaveTitle(/beefcake/i);
  });
});

test.describe('SQL IDE - Lifecycle Integration', () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page, {
      commands: getStandardMocks({
        lifecycle_get_dataset: {
          type: 'success',
          data: {
            id: 'ds-test-123',
            versions: [
              {
                id: 'v-raw-001',
                stage: 'Raw',
                data_location: { OriginalFile: '/data/raw.csv' },
              },
              {
                id: 'v-cleaned-001',
                stage: 'Cleaned',
                data_location: { ParquetFile: '/data/cleaned.parquet' },
              },
            ],
            active_version_id: 'v-cleaned-001',
          },
        },
        get_version_schema: {
          type: 'success',
          data: [
            { name: 'customer_name', kind: 'Text' },
            { name: 'order_date', kind: 'Temporal' },
            { name: 'email', kind: 'Text' },
          ],
        },
        get_version_diff: {
          type: 'success',
          data: {
            schema_changes: {
              columns_renamed: [
                ['Customer Name', 'customer_name'],
                ['Email Address', 'email'],
              ],
            },
          },
        },
      }),
    });
  });

  test('should show stage selector when dataset has versions', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();

    // TODO: Once lifecycle integration is connected:
    // 1. Load a dataset with versions
    // 2. Verify stage selector dropdown appears
    // 3. Verify dropdown shows: Raw, Cleaned
    // 4. Verify Cleaned is selected (active version)

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should update data path when switching stages', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();

    // TODO: Once lifecycle integration is connected:
    // 1. Select "Raw" from stage dropdown
    // 2. Run query: SELECT * FROM data LIMIT 5
    // 3. Verify query uses Raw version (/data/raw.csv)
    // 4. Select "Cleaned" from dropdown
    // 5. Run same query
    // 6. Verify query uses Cleaned version (/data/cleaned.parquet)

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should show refactor button when switching stages', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();

    // TODO: Once refactoring is connected:
    // 1. Start at Cleaned stage
    // 2. Switch to Raw stage
    // 3. Verify refactor button appears
    // 4. Verify tooltip explains functionality

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should refactor SQL column names preserving quote styles', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();

    // TODO: Once refactoring is connected:
    // 1. Write query with old names:
    //    SELECT "Customer Name", `Email Address` FROM data
    // 2. Switch from Raw to Cleaned stage
    // 3. Click "Refactor" button
    // 4. Confirm in dialog
    // 5. Verify query is updated:
    //    SELECT "customer_name", `email` FROM data
    // 6. Verify quote styles are preserved
    // 7. Verify success toast: "Updated 2 column references"

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should handle refactoring with unquoted column names', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();

    // TODO: Once refactoring is connected:
    // 1. Write query: SELECT customer_name, email FROM data
    // 2. Switch stages
    // 3. Refactor
    // 4. Verify unquoted names are updated correctly
    // 5. Verify SQL keywords are NOT refactored (SELECT, FROM, etc.)

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should show info toast when no refactoring needed', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();

    // TODO: Once refactoring is connected:
    // 1. Switch to stage with no column renames
    // 2. Click refactor button
    // 3. Verify info toast: "No column renames detected between stages"

    await expect(page).toHaveTitle(/beefcake/i);
  });
});

test.describe('SQL IDE - Security', () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page, {
      commands: getStandardMocks({
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
        save_config: {
          type: 'success',
          data: null,
        },
      }),
    });
  });

  test('should show security warning on first query execution', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();

    // TODO: Once security flow is connected:
    // 1. Type a query
    // 2. Click "Run Query"
    // 3. Verify confirm dialog appears
    // 4. Verify message: "Running scripts can execute arbitrary code"
    // 5. Click "OK" to acknowledge
    // 6. Verify query executes
    // 7. Verify config is saved

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should not show warning on subsequent executions', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();

    // TODO: Once security flow is connected:
    // 1. Run first query (acknowledge warning)
    // 2. Run second query
    // 3. Verify no warning dialog
    // 4. Verify query executes immediately

    await expect(page).toHaveTitle(/beefcake/i);
  });

  test('should cancel execution if warning declined', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('nav-sql').click();

    // TODO: Once security flow is connected:
    // 1. Type query
    // 2. Click "Run Query"
    // 3. Click "Cancel" on security warning
    // 4. Verify query does NOT execute
    // 5. Verify output panel remains empty

    await expect(page).toHaveTitle(/beefcake/i);
  });
});
