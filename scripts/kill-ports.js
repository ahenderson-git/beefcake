#!/usr/bin/env node

/**
 * Port Cleanup Utility
 *
 * Kills processes using ports 14206 (Vite) and 14207 (HMR WebSocket)
 * to prevent "Port already in use" errors during development.
 *
 * Usage:
 *   npm run kill-ports
 *   node scripts/kill-ports.js
 *   node scripts/kill-ports.js 14206 14207 8080
 */

import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

// Default ports to clean up
const DEFAULT_PORTS = [14206, 14207];

// ANSI color codes
const colors = {
  reset: '\x1b[0m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  red: '\x1b[31m',
  cyan: '\x1b[36m',
};

function log(message, color = 'reset') {
  console.log(`${colors[color]}${message}${colors.reset}`);
}

/**
 * Find and kill process using a specific port on Windows
 */
async function killPortWindows(port) {
  try {
    // Find process ID using the port
    const { stdout } = await execAsync(`netstat -ano | findstr :${port}`);

    if (!stdout.trim()) {
      log(`  Port ${port}: Not in use`, 'green');
      return { port, killed: false, reason: 'not_in_use' };
    }

    // Extract PID from netstat output
    // Format: TCP    0.0.0.0:14206    0.0.0.0:0    LISTENING    12345
    const lines = stdout.trim().split('\n');
    const pids = new Set();

    for (const line of lines) {
      const parts = line.trim().split(/\s+/);
      const pid = parts[parts.length - 1];
      if (pid && /^\d+$/.test(pid)) {
        pids.add(pid);
      }
    }

    if (pids.size === 0) {
      log(`  Port ${port}: No PID found`, 'yellow');
      return { port, killed: false, reason: 'no_pid' };
    }

    // Kill all processes using this port
    for (const pid of pids) {
      try {
        await execAsync(`taskkill /PID ${pid} /F`);
        log(`  Port ${port}: Killed process ${pid}`, 'green');
      } catch (killError) {
        // Process may have already exited
        log(`  Port ${port}: Could not kill process ${pid} (may already be gone)`, 'yellow');
      }
    }

    return { port, killed: true, pids: Array.from(pids) };
  } catch (error) {
    // netstat failed - port is likely not in use
    log(`  Port ${port}: Not in use`, 'green');
    return { port, killed: false, reason: 'not_in_use' };
  }
}

/**
 * Find and kill process using a specific port on Unix-like systems
 */
async function killPortUnix(port) {
  try {
    // Try lsof first (more reliable)
    try {
      const { stdout } = await execAsync(`lsof -ti :${port}`);
      const pids = stdout.trim().split('\n').filter(Boolean);

      if (pids.length === 0) {
        log(`  Port ${port}: Not in use`, 'green');
        return { port, killed: false, reason: 'not_in_use' };
      }

      for (const pid of pids) {
        await execAsync(`kill -9 ${pid}`);
        log(`  Port ${port}: Killed process ${pid}`, 'green');
      }

      return { port, killed: true, pids };
    } catch {
      // lsof not available, try netstat
      const { stdout } = await execAsync(`netstat -tuln | grep :${port}`);

      if (!stdout.trim()) {
        log(`  Port ${port}: Not in use`, 'green');
        return { port, killed: false, reason: 'not_in_use' };
      }

      log(`  Port ${port}: In use but cannot determine PID (run with sudo?)`, 'yellow');
      return { port, killed: false, reason: 'no_permission' };
    }
  } catch (error) {
    log(`  Port ${port}: Not in use`, 'green');
    return { port, killed: false, reason: 'not_in_use' };
  }
}

/**
 * Main function to kill ports
 */
async function killPorts(ports) {
  log('\n🔧 Port Cleanup Utility', 'cyan');
  log('━'.repeat(50), 'cyan');

  const isWindows = process.platform === 'win32';
  log(`Platform: ${isWindows ? 'Windows' : 'Unix-like'}`, 'cyan');
  log(`Ports to check: ${ports.join(', ')}`, 'cyan');
  log('');

  const results = [];

  for (const port of ports) {
    try {
      const result = isWindows
        ? await killPortWindows(port)
        : await killPortUnix(port);
      results.push(result);
    } catch (error) {
      log(`  Port ${port}: Error - ${error.message}`, 'red');
      results.push({ port, killed: false, reason: 'error', error: error.message });
    }
  }

  // Summary
  log('');
  log('━'.repeat(50), 'cyan');
  const killedCount = results.filter(r => r.killed).length;
  const notInUseCount = results.filter(r => r.reason === 'not_in_use').length;

  if (killedCount > 0) {
    log(`✓ Killed ${killedCount} process(es)`, 'green');
  }
  if (notInUseCount > 0) {
    log(`✓ ${notInUseCount} port(s) already free`, 'green');
  }

  const errors = results.filter(r => r.reason === 'error' || r.reason === 'no_permission');
  if (errors.length > 0) {
    log(`⚠ ${errors.length} port(s) could not be cleaned`, 'yellow');
  }

  log('');
  return results;
}

// Parse command line arguments
const args = process.argv.slice(2);
const ports = args.length > 0
  ? args.map(arg => parseInt(arg, 10)).filter(port => !isNaN(port) && port > 0 && port < 65536)
  : DEFAULT_PORTS;

if (ports.length === 0) {
  log('❌ No valid ports specified', 'red');
  log('Usage: node scripts/kill-ports.js [port1] [port2] ...', 'yellow');
  process.exit(1);
}

// Run cleanup
killPorts(ports).catch(error => {
  log(`\n❌ Fatal error: ${error.message}`, 'red');
  process.exit(1);
});
