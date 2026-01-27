import { test, expect } from '@playwright/test';

import { getStandardMocks } from './helpers/common-mocks';
import { setupTauriMock } from './helpers/tauri-mock';

/**
 * Example E2E test for Beefcake GUI
 *
 * NOTE: These tests require:
 * 1. The Tauri app to be built and running
 * 2. A Tauri-specific test adapter (like tauri-driver or WebDriver)
 * 3. The app to be accessible at a test URL
 *
 * For now, this demonstrates the test structure.
 * To run against a real Tauri app, you'll need to set up tauri-driver.
 *
 * See: https://tauri.app/v1/guides/testing/
 */

// Placeholder base URL - will need to be configured for Tauri
const APP_URL = 'http://localhost:14206'; // Tauri dev server default

test.describe('Dashboard', () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page, {
      commands: getStandardMocks(),
    });
  });

  test('should display dashboard on launch', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Wait for app to load
    await expect(page).toHaveTitle(/beefcake/i);

    // Check for main dashboard elements
    await expect(page.getByTestId('dashboard-view')).toBeVisible({
      timeout: 10000,
    });

    // Verify all navigation buttons are present
    await expect(page.getByTestId('dashboard-open-file-button')).toBeVisible();
    await expect(page.getByTestId('dashboard-powershell-button')).toBeVisible();
    await expect(page.getByTestId('dashboard-python-button')).toBeVisible();
    await expect(page.getByTestId('dashboard-sql-button')).toBeVisible();
  });

  test('should have functional navigation buttons', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Verify open file button exists and is clickable
    const openFileBtn = page.getByTestId('dashboard-open-file-button');
    await expect(openFileBtn).toBeVisible();
    await expect(openFileBtn).toBeEnabled();

    // Verify other navigation buttons are enabled
    await expect(page.getByTestId('dashboard-powershell-button')).toBeEnabled();
    await expect(page.getByTestId('dashboard-python-button')).toBeEnabled();
    await expect(page.getByTestId('dashboard-sql-button')).toBeEnabled();
  });

  test('should display version number', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Verify version number is displayed
    await expect(page.locator('h1 small')).toBeVisible();
    await expect(page.locator('h1 small')).toContainText('v');
  });

  test('should display dashboard stats cards', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Verify stats cards are present
    await expect(page.locator('.stat-card')).toHaveCount(3);
  });

  test('should display connection stats', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Verify connection stats card exists
    const statCards = page.locator('.stat-card');
    await expect(statCards.filter({ hasText: 'Connections' })).toBeVisible();
  });

  test('should display hero section', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Verify hero section with title
    await expect(page.locator('.hero')).toBeVisible();
    await expect(page.locator('.hero h1')).toContainText('beefcake');
  });
});

test.describe('Navigation', () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page, {
      commands: getStandardMocks(),
    });
  });

  test('should navigate to Python IDE', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Click Python button
    await page.getByTestId('dashboard-python-button').click();

    // Verify Python IDE view is visible
    await expect(page.getByTestId('python-ide-view')).toBeVisible({ timeout: 5000 });
    await expect(page.getByTestId('python-ide-toolbar')).toBeVisible();
    await expect(page.getByTestId('python-ide-run-button')).toBeVisible();
  });

  test('should navigate to SQL IDE', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Click SQL button
    await page.getByTestId('dashboard-sql-button').click();

    // Verify SQL IDE view is visible
    await expect(page.getByTestId('sql-ide-view')).toBeVisible({ timeout: 5000 });
    await expect(page.getByTestId('sql-ide-toolbar')).toBeVisible();
    await expect(page.getByTestId('sql-ide-run-button')).toBeVisible();
  });

  test('should navigate to PowerShell console', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Click PowerShell button
    await page.getByTestId('dashboard-powershell-button').click();

    // Verify PowerShell IDE view is visible
    await expect(page.getByTestId('powershell-ide-view')).toBeVisible({ timeout: 5000 });
    await expect(page.getByTestId('powershell-ide-toolbar')).toBeVisible();
    await expect(page.getByTestId('powershell-ide-run-button')).toBeVisible();
  });

  test('should navigate to Settings', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Click Settings nav button in sidebar
    await page.getByTestId('nav-settings').click();

    // Verify Settings view is visible
    await expect(page.locator('.settings-view')).toBeVisible({ timeout: 5000 });
  });

  test('should navigate back to Dashboard', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate away from dashboard
    await page.getByTestId('dashboard-python-button').click();
    await expect(page.getByTestId('python-ide-view')).toBeVisible();

    // Navigate back to dashboard
    await page.getByTestId('nav-dashboard').click();
    await expect(page.getByTestId('dashboard-view')).toBeVisible({ timeout: 5000 });
  });

  test('should navigate to Analyser view', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Click Analyser nav button
    await page.getByTestId('nav-analyser').click();

    // Verify Analyser view is visible
    await expect(page.getByTestId('analyser-view')).toBeVisible({ timeout: 5000 });
  });

  test('should navigate to Watcher view', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Click Watcher nav button
    await page.getByTestId('nav-watcher').click();

    // Verify Watcher view is visible
    await expect(page.getByTestId('watcher-view')).toBeVisible({ timeout: 5000 });
  });

  // NOTE: AI Assistant, Export, and Onboarding are not navigation views:
  // - AI Assistant is a collapsible sidebar (#ai-sidebar), not a main view
  // - Export is a modal triggered from IDE components, not a nav destination
  // - Onboarding is a first-run wizard modal, not a nav destination
  // Tests for these features should be in their respective component test files

  test('should navigate to Integrity view', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Click Integrity nav button
    await page.getByTestId('nav-integrity').click();

    // Verify Integrity view is visible
    await expect(page.getByTestId('integrity-view')).toBeVisible({ timeout: 5000 });
  });
});

test.describe('IDE Views', () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page, {
      commands: getStandardMocks(),
    });
  });

  test('should display PowerShell console with editor', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to PowerShell
    await page.getByTestId('dashboard-powershell-button').click();

    // Verify PowerShell view is visible
    await expect(page.getByTestId('powershell-ide-view')).toBeVisible({ timeout: 5000 });
    await expect(page.getByTestId('powershell-ide-editor')).toBeVisible();
    await expect(page.getByTestId('powershell-ide-toolbar')).toBeVisible();
    await expect(page.getByTestId('powershell-ide-run-button')).toBeVisible();
    await expect(page.getByTestId('powershell-ide-output-panel')).toBeVisible();
  });

  test('should have all IDE toolbar buttons present', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to Python IDE
    await page.getByTestId('dashboard-python-button').click();
    await expect(page.getByTestId('python-ide-view')).toBeVisible({ timeout: 5000 });

    // Verify toolbar buttons
    await expect(page.getByTestId('python-ide-save-button')).toBeVisible();
    await expect(page.getByTestId('python-ide-load-button')).toBeVisible();
    await expect(page.getByTestId('python-ide-clear-button')).toBeVisible();
    await expect(page.getByTestId('python-ide-export-button')).toBeVisible();
  });

  test('should have font size controls in Python IDE', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to Python IDE
    await page.getByTestId('dashboard-python-button').click();
    await expect(page.getByTestId('python-ide-view')).toBeVisible({ timeout: 5000 });

    // Verify font size controls
    await expect(page.getByTestId('python-ide-dec-font-button')).toBeVisible();
    await expect(page.getByTestId('python-ide-inc-font-button')).toBeVisible();
    await expect(page.getByTestId('python-ide-font-size')).toBeVisible();
  });

  test('should have font size controls in SQL IDE', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to SQL IDE
    await page.getByTestId('dashboard-sql-button').click();
    await expect(page.getByTestId('sql-ide-view')).toBeVisible({ timeout: 5000 });

    // Verify font size controls
    await expect(page.getByTestId('sql-ide-dec-font-button')).toBeVisible();
    await expect(page.getByTestId('sql-ide-inc-font-button')).toBeVisible();
    await expect(page.getByTestId('sql-ide-font-size')).toBeVisible();
  });

  test('should have PowerShell toolbar buttons', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to PowerShell
    await page.getByTestId('dashboard-powershell-button').click();
    await expect(page.getByTestId('powershell-ide-view')).toBeVisible({ timeout: 5000 });

    // Verify toolbar buttons
    await expect(page.getByTestId('powershell-ide-save-button')).toBeVisible();
    await expect(page.getByTestId('powershell-ide-load-button')).toBeVisible();
    await expect(page.getByTestId('powershell-ide-clear-button')).toBeVisible();
  });

  test('should have PowerShell font size controls', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to PowerShell
    await page.getByTestId('dashboard-powershell-button').click();
    await expect(page.getByTestId('powershell-ide-view')).toBeVisible({ timeout: 5000 });

    // Verify font size controls
    await expect(page.getByTestId('powershell-ide-dec-font-button')).toBeVisible();
    await expect(page.getByTestId('powershell-ide-inc-font-button')).toBeVisible();
    await expect(page.getByTestId('powershell-ide-font-size')).toBeVisible();
  });

  test('should have PowerShell output elements', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to PowerShell
    await page.getByTestId('dashboard-powershell-button').click();
    await expect(page.getByTestId('powershell-ide-view')).toBeVisible({ timeout: 5000 });

    // Verify output elements
    await expect(page.getByTestId('powershell-ide-output-panel')).toBeVisible();
    await expect(page.getByTestId('powershell-ide-output')).toBeVisible();
  });

  test('should have PowerShell layout with editor and output panels', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to PowerShell
    await page.getByTestId('dashboard-powershell-button').click();
    await expect(page.getByTestId('powershell-ide-view')).toBeVisible({ timeout: 5000 });

    // Verify layout components exist
    await expect(page.getByTestId('powershell-ide-editor')).toBeVisible();
    await expect(page.getByTestId('powershell-ide-output-panel')).toBeVisible();
    await expect(page.getByTestId('powershell-ide-toolbar')).toBeVisible();
  });

  test('should have run button in all IDEs', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Check Python IDE
    await page.getByTestId('dashboard-python-button').click();
    await expect(page.getByTestId('python-ide-run-button')).toBeVisible();

    // Check SQL IDE
    await page.getByTestId('nav-sql').click();
    await expect(page.getByTestId('sql-ide-run-button')).toBeVisible();

    // Check PowerShell IDE
    await page.getByTestId('nav-powershell').click();
    await expect(page.getByTestId('powershell-ide-run-button')).toBeVisible();
  });

  test('should have save and load buttons in all IDEs', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Check Python IDE
    await page.getByTestId('dashboard-python-button').click();
    await expect(page.getByTestId('python-ide-save-button')).toBeVisible();
    await expect(page.getByTestId('python-ide-load-button')).toBeVisible();

    // Check SQL IDE
    await page.getByTestId('nav-sql').click();
    await expect(page.getByTestId('sql-ide-save-button')).toBeVisible();
    await expect(page.getByTestId('sql-ide-load-button')).toBeVisible();

    // Check PowerShell IDE
    await page.getByTestId('nav-powershell').click();
    await expect(page.getByTestId('powershell-ide-save-button')).toBeVisible();
    await expect(page.getByTestId('powershell-ide-load-button')).toBeVisible();
  });

  test('should have clear button in all IDEs', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Check Python IDE
    await page.getByTestId('dashboard-python-button').click();
    await expect(page.getByTestId('python-ide-clear-button')).toBeVisible();

    // Check SQL IDE
    await page.getByTestId('nav-sql').click();
    await expect(page.getByTestId('sql-ide-clear-button')).toBeVisible();

    // Check PowerShell IDE
    await page.getByTestId('nav-powershell').click();
    await expect(page.getByTestId('powershell-ide-clear-button')).toBeVisible();
  });

  test('should display dashboard buttons with correct labels', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Verify dashboard buttons exist
    const pythonBtn = page.getByTestId('dashboard-python-button');
    await expect(pythonBtn).toBeVisible();
    await expect(pythonBtn).toContainText('Python');

    const sqlBtn = page.getByTestId('dashboard-sql-button');
    await expect(sqlBtn).toBeVisible();
    await expect(sqlBtn).toContainText('SQL');

    const powershellBtn = page.getByTestId('dashboard-powershell-button');
    await expect(powershellBtn).toBeVisible();
    await expect(powershellBtn).toContainText('PowerShell');
  });

  test('should have sidebar visible on all views', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Check sidebar on dashboard
    await expect(page.locator('.sidebar')).toBeVisible();

    // Check sidebar on Python IDE
    await page.getByTestId('dashboard-python-button').click();
    await expect(page.locator('.sidebar')).toBeVisible();

    // Check sidebar on Settings
    await page.getByTestId('nav-settings').click();
    await expect(page.locator('.sidebar')).toBeVisible();
  });
});

test.describe('Settings View', () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page, {
      commands: getStandardMocks(),
    });
  });

  test('should display settings view with sections', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to Settings
    await page.getByTestId('nav-settings').click();

    // Verify Settings view is visible
    await expect(page.locator('.settings-view')).toBeVisible({ timeout: 5000 });

    // Verify settings sections exist (multiple sections, check first one)
    await expect(page.locator('.settings-section').first()).toBeVisible();
  });

  test('should have add connection button', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to Settings
    await page.getByTestId('nav-settings').click();
    await expect(page.locator('.settings-view')).toBeVisible({ timeout: 5000 });

    // Verify add connection button exists
    await expect(page.getByTestId('settings-add-connection-button')).toBeVisible();
  });

  test('should have connection form fields', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to Settings
    await page.getByTestId('nav-settings').click();
    await expect(page.locator('.settings-view')).toBeVisible({ timeout: 5000 });

    // Click "Add Connection" button to show form
    await page.getByTestId('settings-add-connection-button').click();

    // Verify connection form fields exist
    await expect(page.getByTestId('settings-connection-name-input')).toBeVisible();
    await expect(page.getByTestId('settings-connection-host-input')).toBeVisible();
  });

  test('should have trusted paths section with add button', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to Settings
    await page.getByTestId('nav-settings').click();
    await expect(page.locator('.settings-view')).toBeVisible({ timeout: 5000 });

    // Verify trusted paths section exists
    await expect(page.getByTestId('settings-trusted-paths-section')).toBeVisible();
    await expect(page.getByTestId('settings-add-trusted-path-button')).toBeVisible();
  });

  test('should have AI config toggle', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to Settings
    await page.getByTestId('nav-settings').click();
    await expect(page.locator('.settings-view')).toBeVisible({ timeout: 5000 });

    // Verify AI config section exists
    await expect(page.getByTestId('settings-ai-enabled-toggle')).toBeVisible();
  });

  test('should have font size preferences', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to Settings
    await page.getByTestId('nav-settings').click();
    await expect(page.locator('.settings-view')).toBeVisible({ timeout: 5000 });

    // Verify font size preferences exist
    await expect(page.getByTestId('settings-font-size-section')).toBeVisible();
  });

  test('should have theme selector', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to Settings
    await page.getByTestId('nav-settings').click();
    await expect(page.locator('.settings-view')).toBeVisible({ timeout: 5000 });

    // Verify theme selector exists
    await expect(page.getByTestId('settings-theme-select')).toBeVisible();
  });
});

test.describe('Sidebar Navigation', () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page, {
      commands: getStandardMocks(),
    });
  });

  test('should have all sidebar navigation buttons', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Verify all navigation buttons are present
    await expect(page.getByTestId('nav-dashboard')).toBeVisible();
    await expect(page.getByTestId('nav-analyser')).toBeVisible();
    await expect(page.getByTestId('nav-lifecycle')).toBeVisible();
    await expect(page.getByTestId('nav-pipeline')).toBeVisible();
    await expect(page.getByTestId('nav-python')).toBeVisible();
    await expect(page.getByTestId('nav-sql')).toBeVisible();
    await expect(page.getByTestId('nav-settings')).toBeVisible();
  });

  test('should navigate through multiple views', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to Analyser
    await page.getByTestId('nav-analyser').click();
    // Note: Analyser appears after file load, so we just verify navigation works

    // Navigate to Lifecycle
    await page.getByTestId('nav-lifecycle').click();
    // Lifecycle view should appear

    // Navigate back to Dashboard
    await page.getByTestId('nav-dashboard').click();
    await expect(page.getByTestId('dashboard-view')).toBeVisible({ timeout: 5000 });
  });

  test('should navigate to Watcher view', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to Watcher
    await page.getByTestId('nav-watcher').click();
    // Watcher view should load
  });

  test('should navigate to Pipeline view', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to Pipeline
    await page.getByTestId('nav-pipeline').click();
    // Pipeline view should load
  });

  test('should navigate to Dictionary view', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to Dictionary
    await page.getByTestId('nav-dictionary').click();
    // Dictionary view should load
  });

  test('should navigate to Integrity view', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to Integrity
    await page.getByTestId('nav-integrity').click();
    // Integrity view should load
  });

  test('should navigate to Activity Log view', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to Activity Log
    await page.getByTestId('nav-activity-log').click();
    // Activity Log view should load
  });

  test('should navigate to Reference view', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to Reference
    await page.getByTestId('nav-reference').click();
    // Reference view should load
  });

  test('should navigate to CLI view', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to CLI
    await page.getByTestId('nav-cli').click();
    // CLI view should load
  });
});

test.describe('Pipeline Editor (P1 Workflows)', () => {
  test.skip('should have pipeline UI ready for testing', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Pipeline functionality will be tested once navigation and file loading implemented
    await expect(page).toHaveTitle(/beefcake/i);
  });

  test.skip('should support pipeline validation', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Pipeline validation test IDs are ready for implementation
    await expect(page).toHaveTitle(/beefcake/i);
  });

  test.skip('should support pipeline execution', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Pipeline execution test IDs are ready for implementation
    await expect(page).toHaveTitle(/beefcake/i);
  });
});

test.describe('Error Handling', () => {
  test.skip('should have loading state UI ready', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Loading state test IDs (loading-spinner, loading-message, btn-abort-op) are ready
    await expect(page).toHaveTitle(/beefcake/i);
  });

  test.skip('should have abort functionality ready', async ({ page }) => {
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Abort button test IDs are ready for implementation
    await expect(page).toHaveTitle(/beefcake/i);
  });
});
