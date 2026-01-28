import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1, // Tauri apps should run one at a time
  reporter: 'html',

  use: {
    baseURL: 'http://127.0.0.1:14206',
    // Ensure Playwright waits long enough for initial navigation on slow cold starts
    navigationTimeout: 180000,
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
  },

  webServer: {
    // Ensure any stale servers are terminated before starting a fresh one
    command: 'npm run dev:clean',
    url: 'http://127.0.0.1:14206',
    // If something is already running locally (dev session), reuse it to avoid failures
    reuseExistingServer: true,
    // Vite + large deps (e.g., Monaco) on Windows/OneDrive can take >60s to boot
    timeout: 180 * 1000,
    // Don't pipe stdout/stderr - this can cause hanging on Windows
    // Let Vite output go directly to console
  },

  // Global teardown hook for port cleanup
  globalTeardown: './e2e/helpers/global-teardown.ts',

  projects: [
    {
      name: 'windows',
      use: { ...devices['Desktop Chrome'] },
    },
  ],

  // Allow more time for initial navigation on slower Windows environments
  timeout: 180000,
  expect: {
    timeout: 10000,
  },
});
