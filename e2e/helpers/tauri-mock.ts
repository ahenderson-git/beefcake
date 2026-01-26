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
  commands?: Record<
    string,
    MockResponse | ((args: unknown) => MockResponse | Promise<MockResponse>)
  >;
  // Mock file dialog responses
  fileDialog?: {
    openFile?: string | string[] | null; // Support single file or multiple files
    saveFile?: string | null;
    directory?: string | null; // Support directory picker
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

    // Mock the Tauri v2 internals
    if (!(window as any).__TAURI_INTERNALS__) {
      (window as any).__TAURI_INTERNALS__ = {};
    }

    // Mock the invoke function (used by @tauri-apps/api/core)
    (window as any).__TAURI_INTERNALS__.invoke = async (cmd: string, args: any) => {
      // Don't log successful commands to avoid infinite loops with console logging overrides
      // Only log warnings for missing mocks

      // Handle dialog plugin commands
      if (cmd === 'plugin:dialog|open') {
        // Handle directory picker
        if (args?.directory && fileDialogMocks.directory !== undefined) {
          return fileDialogMocks.directory;
        }

        // Handle file picker
        if (fileDialogMocks.openFile !== undefined) {
          return fileDialogMocks.openFile;
        }

        return null;
      }

      if (cmd === 'plugin:dialog|save') {
        if (fileDialogMocks.saveFile !== undefined) {
          return fileDialogMocks.saveFile;
        }

        return null;
      }

      // Check if we have a mock for this command
      const mockResponse = mockResponses[cmd];

      if (mockResponse) {
        const response =
          typeof mockResponse === 'function' ? await mockResponse(args) : mockResponse;

        if (response.type === 'error') {
          throw new Error(response.error || 'Mock error');
        }

        return response.data;
      }

      // If no mock, throw an error to make tests fail explicitly
      // Use a direct console method to avoid triggering overridden console
      if ((window as any)._originalConsoleWarn) {
        (window as any)._originalConsoleWarn('[Tauri Mock] No mock defined for command:', cmd);
      }
      throw new Error(`No mock defined for Tauri command: ${cmd}`);
    };

    // Mock the dialog plugin
    if (!(window as any).__TAURI__) {
      (window as any).__TAURI__ = {};
    }

    (window as any).__TAURI__.dialog = {
      open: async (options: any) => {
        // Silent - no logging to avoid infinite loops

        // Handle directory picker
        if (options?.directory && fileDialogMocks.directory !== undefined) {
          return fileDialogMocks.directory;
        }

        // Handle file picker
        if (fileDialogMocks.openFile !== undefined) {
          return fileDialogMocks.openFile;
        }

        return null;
      },
      save: async (_options: any) => {
        // Silent - no logging to avoid infinite loops

        if (fileDialogMocks.saveFile !== undefined) {
          return fileDialogMocks.saveFile;
        }

        return null;
      },
    };

    // Setup complete - no console.log to avoid infinite loop with console overrides
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
 * Can be used dynamically during a test (after page load)
 */
export async function mockFileDialog(
  page: Page,
  filePath: string | string[] | null,
  type: 'open' | 'save' | 'directory' = 'open'
): Promise<void> {
  await page.evaluate(
    ({ path, dialogType }) => {
      if (!(window as any).__TAURI__) {
        (window as any).__TAURI__ = {};
      }

      if (!(window as any).__TAURI__.dialog) {
        (window as any).__TAURI__.dialog = {};
      }

      if (dialogType === 'directory') {
        (window as any).__TAURI__.dialog.open = async (options: any) => {
          if (options?.directory) {
            return path;
          }
          return null;
        };
      } else if (dialogType === 'save') {
        (window as any).__TAURI__.dialog.save = async () => path;
      } else {
        (window as any).__TAURI__.dialog.open = async () => path;
      }
    },
    { path: filePath, dialogType: type }
  );
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
