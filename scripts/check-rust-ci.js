#!/usr/bin/env node

/**
 * Pre-push validation script for Rust code
 * Runs the same checks that GitHub CI runs to catch issues locally
 */

import { execSync } from 'child_process';
import { existsSync } from 'fs';

const CHECKS = [
  {
    name: 'Rust Format Check',
    command: 'cargo fmt --all -- --check',
    description: 'Checking Rust code formatting...'
  },
  {
    name: 'Clippy Linting',
    command: 'cargo clippy -- -D warnings',
    description: 'Running Clippy linter...'
  },
  {
    name: 'Cargo Check',
    command: 'cargo check --all-features',
    description: 'Checking Rust compilation and modules...'
  }
];

function run(command, description) {
  console.log(`\n📋 ${description}`);
  try {
    execSync(command, { stdio: 'inherit' });
    console.log('✅ Passed');
    return true;
  } catch (error) {
    console.error(`❌ Failed: ${command}`);
    return false;
  }
}

function checkUntrackedRustModules() {
  try {
    // Get untracked files
    const untrackedFiles = execSync('git ls-files --others --exclude-standard', {
      encoding: 'utf-8'
    });

    const untrackedRustFiles = untrackedFiles
      .split('\n')
      .filter(file => file.endsWith('.rs') && file.startsWith('src/'));

    if (untrackedRustFiles.length > 0) {
      console.log('\n⚠️  WARNING: Untracked Rust module files detected!');
      console.log('These files exist locally but are not tracked in git:');
      untrackedRustFiles.forEach(file => console.log(`  - ${file}`));
      console.log('\n❌ These files will be missing in CI, causing build failures!');
      console.log('💡 Fix: Run `git add <file>` to track these files\n');
      return false;
    }

    return true;
  } catch (error) {
    // If git command fails, continue anyway
    return true;
  }
}

function hasRustChanges() {
  try {
    // Check if there are any staged Rust files
    const stagedFiles = execSync('git diff --cached --name-only', { encoding: 'utf-8' });
    const hasRustFiles = stagedFiles.split('\n').some(file =>
      file.endsWith('.rs') || file === 'Cargo.toml' || file === 'Cargo.lock'
    );

    if (hasRustFiles) {
      return true;
    }

    // Also check for uncommitted Rust changes
    const unstagedFiles = execSync('git diff --name-only', { encoding: 'utf-8' });
    return unstagedFiles.split('\n').some(file =>
      file.endsWith('.rs') || file === 'Cargo.toml' || file === 'Cargo.lock'
    );
  } catch (error) {
    // If git command fails, run checks anyway to be safe
    return true;
  }
}

function main() {
  console.log('🦀 Rust CI Pre-Push Validation\n');
  console.log('Running the same checks that GitHub CI runs...');

  // Check if this is a Rust project
  if (!existsSync('Cargo.toml')) {
    console.log('⚠️  No Cargo.toml found, skipping Rust checks');
    process.exit(0);
  }

  // Check for untracked Rust module files first
  const noUntrackedModules = checkUntrackedRustModules();
  if (!noUntrackedModules) {
    process.exit(1);
  }

  // Skip if no Rust files were changed (unless --force flag is used)
  const forceRun = process.argv.includes('--force');
  if (!forceRun && !hasRustChanges()) {
    console.log('ℹ️  No Rust files changed, skipping checks (use --force to run anyway)');
    process.exit(0);
  }

  let allPassed = true;

  for (const check of CHECKS) {
    const passed = run(check.command, check.description);
    if (!passed) {
      allPassed = false;
    }
  }

  if (allPassed) {
    console.log('\n✨ All Rust CI checks passed! Safe to push.\n');
    process.exit(0);
  } else {
    console.log('\n❌ Some checks failed. Please fix the issues before pushing.\n');
    console.log('💡 Tips:');
    console.log('  - Run `cargo fmt --all` to auto-fix formatting');
    console.log('  - Run `cargo clippy --fix` to auto-fix some clippy issues');
    console.log('  - Run `npm run rust:ci` to run these checks anytime\n');
    process.exit(1);
  }
}

main();
