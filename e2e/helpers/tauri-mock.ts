/**
 * Tauri IPC Mocking Helper for Playwright E2E Tests
 *
 * This helper provides utilities to mock Tauri IPC commands during E2E testing.
 * It intercepts calls to the Tauri invoke API and allows tests to provide mock responses.
 *
 * NOTE: This file uses `any` types intentionally for window object manipulation
 * and Tauri API mocking. ESLint rules are disabled for this test utility file.
 */

/* eslint-disable @typescript-eslint/no-explicit-any */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-return */
/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/require-await */
/* eslint-disable @typescript-eslint/prefer-nullish-coalescing */
/* eslint-disable no-console */

import { Page } from '@playwright/test';

export interface MockResponse {
  type: 'success' | 'error';
  data?: unknown;
  error?: string;
}

export interface TauriMockOptions {
  // Mock responses for specific commands
  commands?: Record<string, MockResponse | ((args: unknown) => MockResponse)>;
  // Mock file dialog responses
  fileDialog?: {
    openFile?: string | null;
    saveFile?: string | null;
  };
}

/**
 * Set up Tauri IPC mocking in the page context
 * This must be called before navigating to the app
 */
export async function setupTauriMock(page: Page, options: TauriMockOptions = {}): Promise<void> {
  await page.addInitScript((opts: TauriMockOptions) => {
    // Create mock storage
    const mockResponses = opts.commands || {};
    const fileDialogMocks = opts.fileDialog || {};

    // Mock the Tauri invoke function
    (window as any).__TAURI_INVOKE__ = async (cmd: string, args: any) => {
      console.log('[Tauri Mock] Intercepted command:', cmd, args);

      // Check if we have a mock for this command
      const mockResponse = mockResponses[cmd];

      if (mockResponse) {
        const response = typeof mockResponse === 'function' ? mockResponse(args) : mockResponse;

        if (response.type === 'error') {
          throw new Error(response.error || 'Mock error');
        }

        return response.data;
      }

      // If no mock, throw an error to make tests fail explicitly
      console.warn('[Tauri Mock] No mock defined for command:', cmd);
      throw new Error(`No mock defined for Tauri command: ${cmd}`);
    };

    // Mock the dialog plugin
    if (!(window as any).__TAURI__) {
      (window as any).__TAURI__ = {};
    }

    (window as any).__TAURI__.dialog = {
      open: async (options: any) => {
        console.log('[Tauri Mock] File dialog open requested:', options);

        if (fileDialogMocks.openFile !== undefined) {
          return fileDialogMocks.openFile;
        }

        console.warn('[Tauri Mock] No mock defined for file dialog');
        return null;
      },
      save: async (options: any) => {
        console.log('[Tauri Mock] File dialog save requested:', options);

        if (fileDialogMocks.saveFile !== undefined) {
          return fileDialogMocks.saveFile;
        }

        console.warn('[Tauri Mock] No mock defined for save dialog');
        return null;
      },
    };

    console.log('[Tauri Mock] Setup complete');
  }, options);
}

/**
 * Mock a specific Tauri command response
 */
export async function mockCommand(
  page: Page,
  command: string,
  response: MockResponse
): Promise<void> {
  await page.evaluate(
    ({ cmd, resp }) => {
      if (!(window as any).__TAURI_MOCK_RESPONSES__) {
        (window as any).__TAURI_MOCK_RESPONSES__ = {};
      }
      (window as any).__TAURI_MOCK_RESPONSES__[cmd] = resp;
    },
    { cmd: command, resp: response }
  );
}

/**
 * Mock file dialog to return a specific path
 */
export async function mockFileDialog(page: Page, filePath: string | null): Promise<void> {
  await page.evaluate(path => {
    if (!(window as any).__TAURI__) {
      (window as any).__TAURI__ = {};
    }

    (window as any).__TAURI__.dialog = {
      open: async () => {
        console.log('[Tauri Mock] Returning mocked file path:', path);
        return path;
      },
      save: async () => path,
    };
  }, filePath);
}

/**
 * Wait for Tauri command to be invoked (useful for verifying interactions)
 */
export async function waitForCommand(page: Page, command: string, timeout = 5000): Promise<void> {
  await page.waitForFunction(
    cmd => {
      const calls = (window as any).__TAURI_COMMAND_CALLS__ || [];
      return calls.some((call: any) => call.command === cmd);
    },
    command,
    { timeout }
  );
}

/**
 * Get all Tauri command calls (for test assertions)
 */
export async function getCommandCalls(page: Page): Promise<Array<{ command: string; args: any }>> {
  return await page.evaluate(() => {
    return (window as any).__TAURI_COMMAND_CALLS__ || [];
  });
}

/**
 * Clear command call history
 */
export async function clearCommandCalls(page: Page): Promise<void> {
  await page.evaluate(() => {
    (window as any).__TAURI_COMMAND_CALLS__ = [];
  });
}
