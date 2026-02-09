/**
 * Playwright Global Teardown
 *
 * Runs after all E2E tests complete. Cleans up ports 14206 and 14207
 * to prevent "Port already in use" errors on subsequent test runs.
 *
 * This is automatically executed by Playwright after the test suite finishes.
 */

import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

async function globalTeardown() {
  try {
    console.log('\n🧹 Global Teardown: Cleaning up ports 14206 and 14207...');
    await execAsync('node scripts/kill-ports.js 14206 14207');
    console.log('✓ Port cleanup complete\n');
  } catch (error) {
    // Port cleanup failed - this is usually fine (ports might not be in use)
    console.warn(
      'Port cleanup warning (this is usually fine):',
      error instanceof Error ? error.message : String(error)
    );
  }
}

export default globalTeardown;
