import type { Page } from '@playwright/test';

const APP_URL = 'http://localhost:14206';
const NAVIGATION_TIMEOUT_MS = 15000;
const RETRY_DELAY_MS = 500;
const MAX_ATTEMPTS = 3;

export async function gotoApp(page: Page): Promise<void> {
  let lastError: unknown;
  for (let attempt = 1; attempt <= MAX_ATTEMPTS; attempt += 1) {
    try {
      await page.goto(APP_URL, { waitUntil: 'commit', timeout: NAVIGATION_TIMEOUT_MS });
      return;
    } catch (err) {
      lastError = err;
      await page.waitForTimeout(RETRY_DELAY_MS * attempt);
    }
  }

  throw lastError;
}
