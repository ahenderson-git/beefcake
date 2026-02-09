#!/usr/bin/env node

/**
 * Port Health Check Utility
 *
 * Checks if required ports are available before starting dev servers.
 * Useful for CI/CD pipelines and automated testing.
 *
 * Usage:
 *   npm run check-ports
 *   node scripts/check-ports.js
 *   node scripts/check-ports.js 14206 14207 8080
 *
 * Exit codes:
 *   0 - All ports available
 *   1 - One or more ports in use
 *   2 - Invalid arguments
 */

import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

// Default ports to check
const DEFAULT_PORTS = [14206, 14207];

// ANSI color codes
const colors = {
  reset: '\x1b[0m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  red: '\x1b[31m',
  cyan: '\x1b[36m',
  gray: '\x1b[90m',
};

function log(message, color = 'reset') {
  console.log(`${colors[color]}${message}${colors.reset}`);
}

/**
 * Check if a port is in use on Windows
 */
async function isPortInUseWindows(port) {
  try {
    const { stdout } = await execAsync(`netstat -ano | findstr :${port}`);
    const lines = stdout.trim().split('\n');

    // Filter for LISTENING state
    const listening = lines.filter(line =>
      line.includes('LISTENING') || line.includes(`0.0.0.0:${port}`) || line.includes(`127.0.0.1:${port}`)
    );

    if (listening.length === 0) {
      return { inUse: false, port };
    }

    // Extract PIDs
    const pids = new Set();
    for (const line of listening) {
      const parts = line.trim().split(/\s+/);
      const pid = parts[parts.length - 1];
      if (pid && /^\d+$/.test(pid)) {
        pids.add(pid);
      }
    }

    return { inUse: true, port, pids: Array.from(pids) };
  } catch {
    // netstat failed - port is not in use
    return { inUse: false, port };
  }
}

/**
 * Check if a port is in use on Unix-like systems
 */
async function isPortInUseUnix(port) {
  try {
    // Try lsof first
    try {
      const { stdout } = await execAsync(`lsof -ti :${port}`);
      const pids = stdout.trim().split('\n').filter(Boolean);

      if (pids.length === 0) {
        return { inUse: false, port };
      }

      return { inUse: true, port, pids };
    } catch {
      // lsof not available, try netstat
      try {
        const { stdout } = await execAsync(`netstat -tuln | grep :${port}`);

        if (!stdout.trim()) {
          return { inUse: false, port };
        }

        return { inUse: true, port, pids: [] };
      } catch {
        return { inUse: false, port };
      }
    }
  } catch {
    return { inUse: false, port };
  }
}

/**
 * Get process name by PID (Windows only)
 */
async function getProcessName(pid) {
  try {
    const { stdout } = await execAsync(`tasklist /FI "PID eq ${pid}" /FO CSV /NH`);
    const match = stdout.match(/"([^"]+)"/);
    return match ? match[1] : 'unknown';
  } catch {
    return 'unknown';
  }
}

/**
 * Main function to check ports
 */
async function checkPorts(ports) {
  log('\n🔍 Port Health Check', 'cyan');
  log('━'.repeat(60), 'cyan');

  const isWindows = process.platform === 'win32';
  log(`Platform: ${isWindows ? 'Windows' : 'Unix-like'}`, 'gray');
  log(`Checking ports: ${ports.join(', ')}`, 'gray');
  log('');

  const results = [];
  let allClear = true;

  for (const port of ports) {
    try {
      const result = isWindows
        ? await isPortInUseWindows(port)
        : await isPortInUseUnix(port);

      results.push(result);

      if (result.inUse) {
        allClear = false;
        const pidInfo = result.pids?.length > 0 ? ` (PID: ${result.pids.join(', ')})` : '';

        log(`  ❌ Port ${port}: IN USE${pidInfo}`, 'red');

        // Try to get process names on Windows
        if (isWindows && result.pids && result.pids.length > 0) {
          for (const pid of result.pids) {
            const processName = await getProcessName(pid);
            log(`     └─ ${processName} (${pid})`, 'gray');
          }
        }
      } else {
        log(`  ✓ Port ${port}: Available`, 'green');
      }
    } catch (error) {
      log(`  ⚠ Port ${port}: Error checking - ${error.message}`, 'yellow');
      results.push({ inUse: false, port, error: error.message });
    }
  }

  // Summary
  log('');
  log('━'.repeat(60), 'cyan');

  if (allClear) {
    log('✓ All ports available', 'green');
    log('');
    return 0; // Success
  } else {
    const inUseCount = results.filter(r => r.inUse).length;
    log(`❌ ${inUseCount} port(s) in use`, 'red');
    log('');
    log('Suggestions:', 'yellow');
    log('  • Run: npm run kill-ports', 'yellow');
    log('  • Or run: node scripts/kill-ports.js', 'yellow');
    log('  • Or manually kill the processes listed above', 'yellow');
    log('');
    return 1; // Failure
  }
}

// Parse command line arguments
const args = process.argv.slice(2);

// Support --help flag
if (args.includes('--help') || args.includes('-h')) {
  log('\nPort Health Check Utility\n', 'cyan');
  log('Usage:', 'cyan');
  log('  npm run check-ports');
  log('  node scripts/check-ports.js');
  log('  node scripts/check-ports.js 14206 14207 8080\n');
  log('Options:', 'cyan');
  log('  --help, -h    Show this help message\n');
  log('Exit codes:', 'cyan');
  log('  0 - All ports available');
  log('  1 - One or more ports in use');
  log('  2 - Invalid arguments\n');
  process.exit(0);
}

const ports = args.length > 0
  ? args.map(arg => parseInt(arg, 10)).filter(port => !isNaN(port) && port > 0 && port < 65536)
  : DEFAULT_PORTS;

if (ports.length === 0) {
  log('❌ No valid ports specified', 'red');
  log('Usage: node scripts/check-ports.js [port1] [port2] ...', 'yellow');
  log('Run with --help for more information', 'gray');
  process.exit(2);
}

// Run health check
checkPorts(ports).then(exitCode => {
  process.exit(exitCode);
}).catch(error => {
  log(`\n❌ Fatal error: ${error.message}`, 'red');
  process.exit(2);
});
