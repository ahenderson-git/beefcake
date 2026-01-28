#!/usr/bin/env node

/**
 * Simulates the CI build environment to test if build will succeed on GitHub
 * This script creates a clean temporary directory, checks out only committed files,
 * and runs the same build commands that CI runs.
 */

import { execSync } from 'child_process';
import { mkdtempSync, rmSync } from 'fs';
import { tmpdir } from 'os';
import { join } from 'path';

function run(command, description, options = {}) {
  console.log(`\n📋 ${description}`);
  try {
    const output = execSync(command, {
      encoding: 'utf-8',
      stdio: options.silent ? 'pipe' : 'inherit',
      ...options
    });
    console.log('✅ Passed');
    return output;
  } catch (error) {
    console.error(`❌ Failed: ${command}`);
    if (options.silent && error.stdout) {
      console.error(error.stdout);
    }
    throw error;
  }
}

function main() {
  console.log('🧪 CI Build Environment Simulation\n');
  console.log('This test simulates what GitHub CI will see when you push.\n');

  // Get current git info
  const currentBranch = execSync('git branch --show-current', { encoding: 'utf-8' }).trim();
  const currentCommit = execSync('git rev-parse HEAD', { encoding: 'utf-8' }).trim();

  console.log(`Current branch: ${currentBranch}`);
  console.log(`Current commit: ${currentCommit.substring(0, 8)}`);

  // Check for staged changes
  const stagedChanges = execSync('git diff --cached --name-only', { encoding: 'utf-8' }).trim();
  if (stagedChanges) {
    console.log('\n⚠️  Warning: You have staged changes that are not committed yet.');
    console.log('CI will NOT see these changes until you commit and push them.\n');
    console.log('Staged files:');
    stagedChanges.split('\n').forEach(file => console.log(`  - ${file}`));
    console.log('\n💡 This test will check if the build would pass WITHOUT these staged changes.');
    console.log('   To test WITH staged changes, commit them first (or use --amend).\n');
  }

  // Check for uncommitted changes
  const uncommittedChanges = execSync('git diff --name-only', { encoding: 'utf-8' }).trim();
  if (uncommittedChanges) {
    console.log('⚠️  You have uncommitted changes that CI will NOT see.\n');
  }

  // Create temporary directory
  const tempDir = mkdtempSync(join(tmpdir(), 'beefcake-ci-test-'));
  console.log(`\nCreated temporary test directory: ${tempDir}\n`);

  let success = false;
  try {
    // Clone the repository to temp directory (simulating CI checkout)
    console.log('📦 Simulating CI checkout (git clone)...');
    const repoPath = process.cwd();
    run(
      `git clone --branch ${currentBranch} --single-branch "${repoPath}" "${tempDir}"`,
      'Cloning repository',
      { silent: true }
    );

    // Change to temp directory
    process.chdir(tempDir);
    console.log(`Changed to test directory: ${tempDir}\n`);

    // Show git status
    console.log('Git status in test environment:');
    execSync('git log -1 --oneline', { stdio: 'inherit' });
    console.log();

    // Check if critical module files exist
    console.log('🔍 Checking for critical module files...');
    const criticalFiles = [
      'src/config.rs',
      'src/integrity.rs',
      'src/logging.rs',
      'src/commands/mod.rs'
    ];

    let missingFiles = [];
    for (const file of criticalFiles) {
      try {
        execSync(`test -f ${file}`, { stdio: 'pipe' });
        console.log(`  ✅ ${file}`);
      } catch {
        console.log(`  ❌ ${file} - MISSING!`);
        missingFiles.push(file);
      }
    }

    if (missingFiles.length > 0) {
      console.log('\n❌ CRITICAL: Missing module files detected!');
      console.log('These files are declared in src/lib.rs but not in git.');
      console.log('\n💡 Fix: These files need to be committed and pushed:');
      missingFiles.forEach(file => console.log(`  git add ${file}`));
      process.exit(1);
    }

    // Run the same checks that CI runs
    console.log('\n🦀 Running CI Build Checks...\n');

    run('cargo fmt --all -- --check', 'Checking Rust formatting');
    run('cargo clippy -- -D warnings', 'Running Clippy linter');
    run('cargo check --all-features', 'Checking compilation');

    console.log('\n✨ All CI build checks passed!');
    console.log('✅ Your code is ready to push to GitHub.\n');
    success = true;

  } catch (error) {
    console.log('\n❌ CI build simulation failed!');
    console.log('Fix these issues before pushing to avoid CI failures.\n');
    success = false;
  } finally {
    // Change back to original directory
    process.chdir(repoPath);

    // Clean up temp directory
    console.log(`\n🧹 Cleaning up temporary directory...`);
    try {
      rmSync(tempDir, { recursive: true, force: true });
      console.log('✅ Cleanup complete\n');
    } catch (error) {
      console.error(`⚠️  Failed to clean up ${tempDir}: ${error.message}`);
    }
  }

  process.exit(success ? 0 : 1);
}

main();
