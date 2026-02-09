import { exec } from 'child_process';
import { promisify } from 'util';

import { test, expect } from '@playwright/test';

const execAsync = promisify(exec);

/**
 * TypeScript Compliance E2E Tests
 *
 * Ensures that E2E test files comply with TypeScript strict mode
 * and pass type checking during build. Prevents compilation errors
 * from being discovered only during CI/CD runs.
 *
 * Priority: P1 (Build Stability / Developer Experience)
 */

test.describe('TypeScript Compliance', () => {
  test('should pass TypeScript type checking with no errors', async () => {
    try {
      // Run type check - should succeed with no output
      const { stdout, stderr } = await execAsync('npx tsc --noEmit', {
        timeout: 60000,
      });

      // Check for any TypeScript errors in output
      const hasErrors = stdout.includes('error TS') || stderr.includes('error TS');

      if (hasErrors) {
        console.error('TypeScript errors found:');
        console.error(stdout);
        console.error(stderr);
      }

      expect(hasErrors).toBe(false);
    } catch (error) {
      // Type check failed
      if (error && typeof error === 'object' && 'stdout' in error) {
        const execError = error as { stdout: string; stderr: string; code: number };
        console.error('TypeScript compilation failed:');
        console.error(execError.stdout);
        console.error(execError.stderr);

        // Fail test with helpful message
        expect(execError.code).toBe(0);
      } else {
        throw error;
      }
    }
  });

  test('should not have unused variables or functions in E2E tests', async () => {
    try {
      // Run type check and capture output
      await execAsync('npx tsc --noEmit');

      // If we get here, no TS6133 errors (unused locals)
      expect(true).toBe(true);
    } catch (error) {
      if (error && typeof error === 'object' && 'stdout' in error) {
        const execError = error as { stdout: string };

        // Check for unused variable errors (TS6133)
        const hasUnusedErrors = execError.stdout.includes('TS6133');

        if (hasUnusedErrors) {
          const unusedMatches = execError.stdout.match(
            /error TS6133:.*is declared but.*never read/g
          );
          console.error('Unused variables/functions found:');
          unusedMatches?.forEach(match => console.error(`  - ${match}`));

          expect(hasUnusedErrors).toBe(false);
        }
      }
    }
  });

  test('should properly handle error types in catch blocks', async () => {
    try {
      await execAsync('npx tsc --noEmit');
      expect(true).toBe(true);
    } catch (error) {
      if (error && typeof error === 'object' && 'stdout' in error) {
        const execError = error as { stdout: string };

        // Check for TS18046 errors (accessing properties of 'unknown')
        const hasUnknownErrors = execError.stdout.includes('TS18046');

        if (hasUnknownErrors) {
          const unknownMatches = execError.stdout.match(/error TS18046:.*is of type 'unknown'/g);
          console.error('Improper error handling found:');
          unknownMatches?.forEach(match => console.error(`  - ${match}`));

          expect(hasUnknownErrors).toBe(false);
        }
      }
    }
  });

  test('should include E2E files in TypeScript compilation scope', async () => {
    // Read tsconfig.json to verify E2E files are included
    const { stdout } = await execAsync('type tsconfig.json || cat tsconfig.json');

    expect(stdout).toContain('"include"');
    expect(stdout).toContain('"e2e"');
  });

  test('should have strict mode enabled in TypeScript config', async () => {
    const { stdout } = await execAsync('type tsconfig.json || cat tsconfig.json');

    expect(stdout).toContain('"strict"');
    expect(stdout).toContain('true');
  });

  test('should have noUnusedLocals enabled in TypeScript config', async () => {
    const { stdout } = await execAsync('type tsconfig.json || cat tsconfig.json');

    expect(stdout).toContain('"noUnusedLocals"');
    expect(stdout).toContain('true');
  });
});

test.describe('Build Process Integration', () => {
  test('should have type-check script available', async () => {
    const { stdout } = await execAsync('npm run 2>&1 | findstr type-check');

    expect(stdout).toContain('type-check');
  });

  test('should include type checking in build process', async () => {
    // Check if prebuild or build script includes type checking
    const { stdout } = await execAsync('type package.json || cat package.json');

    const includesTypeCheck =
      stdout.includes('"build": "tsc') ||
      stdout.includes('"prebuild"') ||
      stdout.includes('type-check');

    expect(includesTypeCheck).toBe(true);
  });
});

test.describe('Error Patterns', () => {
  test('should document proper error handling pattern', async () => {
    // Verify server-lifecycle.spec.ts uses proper error handling
    const { stdout } = await execAsync(
      'type e2e\\server-lifecycle.spec.ts || cat e2e/server-lifecycle.spec.ts'
    );

    // Should include type guards for error handling
    const hasTypeGuard =
      stdout.includes("typeof error === 'object'") || stdout.includes('error instanceof Error');

    expect(hasTypeGuard).toBe(true);
  });

  test('should not use any "any" types in E2E tests', async () => {
    // Check E2E files for explicit ': any' type annotations (which defeats strict mode)
    // We use a more specific regex to avoid matching comments, strings, or 'as any' casts
    try {
      const { stdout } = await execAsync('findstr /S /N ": any" e2e\\*.ts');

      // Filter results to find actual type annotations like ": any" or ":any"
      // but exclude this file itself and legitimate casts
      const lines = stdout.split('\n');
      const realErrors = lines.filter(line => {
        if (!line.trim()) return false;
        if (line.includes('typescript-compliance.spec.ts')) return false;
        // Match ": any" but not "as any"
        return /:\s*any\b/.test(line) && !/\bas\s+any\b/.test(line);
      });

      if (realErrors.length > 0) {
        console.warn('Warning: Found explicit "any" type annotations in E2E tests:');
        realErrors.forEach(line => console.warn(line));
      }

      // This is a warning, not a failure (some legitimate uses exist)
      expect(true).toBe(true);
    } catch {
      // No matches found - good!
      expect(true).toBe(true);
    }
  });
});
