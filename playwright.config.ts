import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1, // Tauri apps should run one at a time
  reporter: 'html',

  use: {
    baseURL: 'http://localhost:14206',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
  },

  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:14206',
    reuseExistingServer: false, // Force a fresh Vite server to avoid stale/hung instances
    timeout: 60 * 1000,
    // Don't pipe stdout/stderr - this can cause hanging on Windows
    // Let Vite output go directly to console
  },

  projects: [
    {
      name: 'windows',
      use: { ...devices['Desktop Chrome'] },
    },
  ],

  timeout: 60000,
  expect: {
    timeout: 10000,
  },
});
