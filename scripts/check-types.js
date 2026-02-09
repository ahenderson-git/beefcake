#!/usr/bin/env node

/**
 * TypeScript Type Checking Utility
 *
 * Runs TypeScript compiler in check-only mode and provides detailed
 * error reporting for CI/CD pipelines and local development.
 *
 * Usage:
 *   npm run type-check
 *   node scripts/check-types.js
 *   node scripts/check-types.js --watch
 *
 * Exit codes:
 *   0 - No type errors
 *   1 - Type errors found
 *   2 - Fatal error (TypeScript not found, etc.)
 */

import { exec } from 'child_process';
import { promisify } from 'util';
import { existsSync } from 'fs';
import { resolve } from 'path';

const execAsync = promisify(exec);

// ANSI color codes
const colors = {
  reset: '\x1b[0m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  red: '\x1b[31m',
  cyan: '\x1b[36m',
  gray: '\x1b[90m',
  bold: '\x1b[1m',
};

function log(message, color = 'reset') {
  console.log(`${colors[color]}${message}${colors.reset}`);
}

/**
 * Parse TypeScript error output
 */
function parseTypeScriptErrors(output) {
  const lines = output.split('\n');
  const errors = [];
  let currentError = null;

  for (const line of lines) {
    // Match error lines: file.ts(line,col): error TSXXXX: message
    const errorMatch = line.match(/^(.+)\((\d+),(\d+)\): error (TS\d+): (.+)$/);

    if (errorMatch) {
      if (currentError) {
        errors.push(currentError);
      }

      currentError = {
        file: errorMatch[1],
        line: parseInt(errorMatch[2], 10),
        column: parseInt(errorMatch[3], 10),
        code: errorMatch[4],
        message: errorMatch[5],
        context: [],
      };
    } else if (currentError && line.trim()) {
      // Additional context lines
      currentError.context.push(line);
    }
  }

  if (currentError) {
    errors.push(currentError);
  }

  return errors;
}

/**
 * Group errors by file
 */
function groupErrorsByFile(errors) {
  const grouped = {};

  for (const error of errors) {
    if (!grouped[error.file]) {
      grouped[error.file] = [];
    }
    grouped[error.file].push(error);
  }

  return grouped;
}

/**
 * Categorize errors by type
 */
function categorizeErrors(errors) {
  const categories = {
    unused: [],
    unknown: [],
    missing: [],
    type: [],
    other: [],
  };

  for (const error of errors) {
    if (error.code === 'TS6133') {
      categories.unused.push(error);
    } else if (error.code === 'TS18046') {
      categories.unknown.push(error);
    } else if (error.code.startsWith('TS2')) {
      categories.type.push(error);
    } else if (error.code.startsWith('TS23') || error.code.startsWith('TS24')) {
      categories.missing.push(error);
    } else {
      categories.other.push(error);
    }
  }

  return categories;
}

/**
 * Format error for display
 */
function formatError(error, index, total) {
  const prefix = `${index + 1}/${total}`;
  const location = `${error.file}:${error.line}:${error.column}`;

  return [
    `${colors.gray}[${prefix}]${colors.reset} ${colors.red}${error.code}${colors.reset} ${location}`,
    `  ${error.message}`,
  ].join('\n');
}

/**
 * Main type checking function
 */
async function checkTypes() {
  log('\n🔍 TypeScript Type Checking', 'cyan');
  log('━'.repeat(60), 'cyan');

  // Check if tsconfig.json exists
  const tsconfigPath = resolve(process.cwd(), 'tsconfig.json');
  if (!existsSync(tsconfigPath)) {
    log('❌ tsconfig.json not found', 'red');
    log(`   Expected at: ${tsconfigPath}`, 'gray');
    return 2;
  }

  log('Config: tsconfig.json', 'gray');
  log('Running: tsc --noEmit\n', 'gray');

  try {
    // Run TypeScript compiler in check-only mode
    const { stdout, stderr } = await execAsync('npx tsc --noEmit', {
      timeout: 120000, // 2 minute timeout
      maxBuffer: 10 * 1024 * 1024, // 10MB buffer for large projects
    });

    // If we get here, no errors
    log('✓ No type errors found', 'green');
    log('━'.repeat(60), 'cyan');
    log('');
    return 0;
  } catch (error) {
    // Type errors found
    if (error && typeof error === 'object' && 'stdout' in error) {
      const execError = error as { stdout: string; stderr: string; code: number };
      const output = execError.stdout + execError.stderr;

      // Parse errors
      const errors = parseTypeScriptErrors(output);

      if (errors.length === 0) {
        // Non-TypeScript error (e.g., syntax error)
        log('❌ TypeScript compilation failed', 'red');
        log('', 'reset');
        console.error(output);
        log('━'.repeat(60), 'cyan');
        log('');
        return 1;
      }

      // Group and categorize errors
      const grouped = groupErrorsByFile(errors);
      const categories = categorizeErrors(errors);

      // Summary
      log(`${colors.red}✗ Found ${errors.length} type error(s)${colors.reset}\n`, 'reset');

      // Category breakdown
      if (categories.unused.length > 0) {
        log(`  • ${categories.unused.length} unused variable(s)/function(s) (TS6133)`, 'yellow');
      }
      if (categories.unknown.length > 0) {
        log(`  • ${categories.unknown.length} unknown type issue(s) (TS18046)`, 'yellow');
      }
      if (categories.type.length > 0) {
        log(`  • ${categories.type.length} type mismatch(es) (TS2xxx)`, 'yellow');
      }
      if (categories.missing.length > 0) {
        log(`  • ${categories.missing.length} missing property/method(s) (TS23xx/TS24xx)`, 'yellow');
      }
      if (categories.other.length > 0) {
        log(`  • ${categories.other.length} other error(s)`, 'yellow');
      }

      log('');

      // File-by-file breakdown
      log('Errors by file:', 'cyan');
      for (const [file, fileErrors] of Object.entries(grouped)) {
        const relativeFile = file.startsWith(process.cwd()) ? file.substring(process.cwd().length + 1) : file;
        log(`\n${colors.bold}${relativeFile}${colors.reset} (${fileErrors.length} error${fileErrors.length > 1 ? 's' : ''}):`, 'reset');

        for (let i = 0; i < fileErrors.length; i++) {
          console.log(formatError(fileErrors[i], i, fileErrors.length));
        }
      }

      log('');
      log('━'.repeat(60), 'cyan');
      log('\nFix these errors by running:', 'yellow');
      log('  npm run type-check', 'cyan');
      log('');

      return 1;
    }

    // Unknown error
    log('❌ Fatal error during type checking', 'red');
    console.error(error);
    return 2;
  }
}

// Parse command line arguments
const args = process.argv.slice(2);

if (args.includes('--help') || args.includes('-h')) {
  log('\nTypeScript Type Checking Utility\n', 'cyan');
  log('Usage:', 'cyan');
  log('  npm run type-check');
  log('  node scripts/check-types.js\n');
  log('Options:', 'cyan');
  log('  --help, -h    Show this help message\n');
  log('Exit codes:', 'cyan');
  log('  0 - No type errors');
  log('  1 - Type errors found');
  log('  2 - Fatal error\n');
  process.exit(0);
}

// Run type checking
checkTypes().then(exitCode => {
  process.exit(exitCode);
}).catch(error => {
  log(`\n❌ Fatal error: ${error.message}`, 'red');
  process.exit(2);
});
