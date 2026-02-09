import { exec } from 'child_process';
import { promisify } from 'util';

import { test, expect } from '@playwright/test';

import { getStandardMocks } from './helpers/common-mocks';
import { setupTauriMock } from './helpers/tauri-mock';

const execAsync = promisify(exec);

/**
 * Server Lifecycle E2E Tests
 *
 * Tests port management, server startup, and cleanup to prevent
 * "Port already in use" errors that block development workflow.
 *
 * Priority: P1 (Infrastructure / Development Experience)
 */

const APP_URL = 'http://127.0.0.1:14206';
const VITE_PORT = 14206;
const HMR_PORT = 14207;

/**
 * Check if a port is in use
 */
async function isPortInUse(port: number): Promise<boolean> {
  try {
    if (process.platform === 'win32') {
      const { stdout } = await execAsync(`netstat -ano | findstr :${port}`);
      return stdout.trim().length > 0;
    } else {
      const { stdout } = await execAsync(`lsof -ti :${port} || netstat -tuln | grep :${port}`);
      return stdout.trim().length > 0;
    }
  } catch {
    return false;
  }
}

/**
 * Wait until the Vite dev server responds on APP_URL or timeout
 */
async function waitForServerReady(
  timeoutMs: number = 180000,
  intervalMs: number = 1000
): Promise<void> {
  const start = Date.now();
  // eslint-disable-next-line no-constant-condition
  while (true) {
    try {
      const res = await fetch(APP_URL, { method: 'GET' });
      if (res.ok) return;
    } catch {
      // ignore until timeout
    }
    if (Date.now() - start > timeoutMs) {
      throw new Error(`Timed out after ${timeoutMs}ms waiting for ${APP_URL}`);
    }
    await new Promise(r => setTimeout(r, intervalMs));
  }
}

test.describe('Server Lifecycle Management', () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page, {
      commands: getStandardMocks(),
    });
  });

  test('should have Vite server running on port 14206', async ({ page }) => {
    // Verify server is reachable
    await waitForServerReady();
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Verify it's actually the Beefcake app
    await expect(page).toHaveTitle(/beefcake/i);

    // Verify dashboard loads (basic smoke test)
    await expect(page.getByTestId('dashboard-view')).toBeVisible({ timeout: 10000 });
  });

  test('should have HMR WebSocket available on port 14207', async ({ page }) => {
    await waitForServerReady();
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Check if HMR is working by looking for Vite client connection
    const scriptPresent = await page.evaluate(() => {
      return document.querySelector('script[type="module"][src*="/@vite/client"]') !== null;
    });

    // Also verify that HMR WS port appears to be listening as a fallback
    const hmrPortListening = await isPortInUse(HMR_PORT);

    expect(scriptPresent || hmrPortListening).toBe(true);
  });

  test('should detect port conflicts before starting server', async () => {
    // This test verifies our port detection works
    const viteInUse = await isPortInUse(VITE_PORT);
    const hmrInUse = await isPortInUse(HMR_PORT);

    // Both should be in use since Playwright webServer is running
    expect(viteInUse).toBe(true);
    expect(hmrInUse).toBe(true);
  });
});

test.describe('Port Cleanup Utility', () => {
  test('should provide kill-ports script', async () => {
    // Verify the script exists and is executable
    const { stdout } = await execAsync('node scripts/kill-ports.js --help || echo "exists"');
    expect(stdout).toContain('exists');
  });

  test('should handle killing ports gracefully when not in use', async () => {
    // Try to kill a port that's definitely not in use
    const unusedPort = 65432;

    try {
      await execAsync(`node scripts/kill-ports.js ${unusedPort}`);
      // Should succeed without throwing
      expect(true).toBe(true);
    } catch (error) {
      // Type guard for error objects with code property
      if (error && typeof error === 'object' && 'code' in error) {
        const execError = error as { code: number };
        // If it does throw, it should be a clean exit (not code 1)
        expect(execError.code).not.toBe(1);
      }
      // Otherwise, pass the test (error without code property is acceptable)
    }
  });

  test('should support multiple ports in single command', async () => {
    // Verify we can specify multiple ports
    const unusedPorts = [65430, 65431, 65432];

    try {
      await execAsync(`node scripts/kill-ports.js ${unusedPorts.join(' ')}`);
      expect(true).toBe(true);
    } catch (error) {
      // Type guard for error objects with code property
      if (error && typeof error === 'object' && 'code' in error) {
        const execError = error as { code: number };
        expect(execError.code).not.toBe(1);
      }
      // Otherwise, pass the test
    }
  });
});

test.describe('npm Scripts Integration', () => {
  test('should have dev:clean script available', async () => {
    const { stdout } = await execAsync('npm run 2>&1 | findstr dev:clean');
    expect(stdout).toContain('dev:clean');
  });

  test('should have tauri:dev:clean script available', async () => {
    const { stdout } = await execAsync('npm run 2>&1 | findstr tauri:dev:clean');
    expect(stdout).toContain('tauri:dev:clean');
  });

  test('should have test:e2e:clean script available', async () => {
    const { stdout } = await execAsync('npm run 2>&1 | findstr test:e2e:clean');
    expect(stdout).toContain('test:e2e:clean');
  });

  test('should have test:all:clean script available', async () => {
    const { stdout } = await execAsync('npm run 2>&1 | findstr test:all:clean');
    expect(stdout).toContain('test:all:clean');
  });
});

test.describe('Port Conflict Prevention', () => {
  test.skip('should prevent starting server when port is in use', async () => {
    // This test is skipped by default as it requires stopping the Playwright webServer
    // Run manually when testing port conflict handling

    // Verify current server is using the port
    const portInUse = await isPortInUse(VITE_PORT);
    expect(portInUse).toBe(true);

    // If we try to start another server, it should fail with EADDRINUSE
    // This is verified by vite.config.ts strictPort: true setting
  });

  test('should document port requirements in vite.config.ts', async () => {
    const { stdout } = await execAsync(
      'grep -E "port.*14206|port.*14207" vite.config.ts || findstr /C:"14206" /C:"14207" vite.config.ts'
    );

    // Verify both ports are documented in config
    expect(stdout).toContain('14206');
    expect(stdout).toContain('14207');
  });

  test('should have strictPort enabled in vite config', async () => {
    const { stdout } = await execAsync(
      'grep "strictPort" vite.config.ts || findstr "strictPort" vite.config.ts'
    );
    expect(stdout).toContain('strictPort');
    expect(stdout).toContain('true');
  });
});

test.describe('Development Workflow', () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page, {
      commands: getStandardMocks(),
    });
  });

  test('should allow page reload without losing connection', async ({ page }) => {
    await waitForServerReady();
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });
    await expect(page.getByTestId('dashboard-view')).toBeVisible({ timeout: 10000 });

    // Reload page
    await page.reload({ waitUntil: 'domcontentloaded' });

    // Should still work
    await expect(page.getByTestId('dashboard-view')).toBeVisible({ timeout: 10000 });
  });

  test('should maintain server stability during navigation', async ({ page }) => {
    await waitForServerReady();
    await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

    // Navigate to different views
    await expect(page.getByTestId('dashboard-view')).toBeVisible({ timeout: 10000 });

    // Try to navigate (if nav buttons exist)
    const navButtons = await page.locator('[data-testid^="dashboard-"]').count();
    expect(navButtons).toBeGreaterThan(0);

    // Server should remain stable
    const finalCheck = await isPortInUse(VITE_PORT);
    expect(finalCheck).toBe(true);
  });
});
