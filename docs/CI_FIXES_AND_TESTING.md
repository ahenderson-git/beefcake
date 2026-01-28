# CI Fixes and Testing Guide

## Problem Summary

The GitHub CI build was failing with errors like:

```
error[E0583]: file not found for module `config`
error[E0583]: file not found for module `integrity`
error[E0583]: file not found for module `logging`
```

### Root Cause

Module files existed locally but were **not tracked in git**. When GitHub CI checked out the code, these files were missing, causing compilation failures.

## Fixes Implemented

### 1. Added Missing Module Files to Git

All critical module files have been staged and are ready to be committed:

- `src/config.rs` - Application configuration
- `src/integrity.rs` - Data integrity verification
- `src/logging.rs` - Logging initialization
- `src/integrity/` - Integrity submodules (hasher, receipt, verifier)
- `src/commands/` - All Tauri command handlers
- `src/analyser/logic/tests/` - Test modules

### 2. Enhanced Pre-Push Validation

**File**: `scripts/check-rust-ci.js`

Added automatic detection of untracked module files:

```javascript
function checkUntrackedRustModules() {
  // Scans for .rs files in src/ that aren't tracked in git
  // Blocks push if any untracked modules are found
  // Shows clear error message with file list and fix instructions
}
```

**What it checks**:
1. ✅ Untracked module files (NEW)
2. ✅ Rust formatting (`cargo fmt --all -- --check`)
3. ✅ Clippy linting (`cargo clippy -- -D warnings`)
4. ✅ Compilation & module resolution (`cargo check --all-features`)

### 3. CI Build Simulation Script

**File**: `scripts/test-ci-build.js`

Created a comprehensive test that simulates the exact CI build environment:

**What it does**:
- Creates a temporary directory
- Clones the repository (simulating CI checkout)
- Checks for critical module files
- Runs all CI build checks
- Warns about uncommitted/unstaged changes

**Why this matters**: This script sees **only what CI sees** - it won't use unstaged files or uncommitted changes, giving you accurate feedback before pushing.

## How to Use These Tools

### Before Every Push (Automatic)

The `.husky/pre-push` hook automatically runs validation:

```bash
git push
# Automatically runs check-rust-ci.js
# Blocks push if checks fail
```

### Manual Testing

#### Quick Check (uses local files)
```bash
npm run rust:ci
```

#### Full CI Simulation (simulates GitHub environment)
```bash
npm run ci:simulate
```

This is the **most accurate test** - it shows exactly what CI will see.

#### Individual Checks
```bash
# Check formatting
npm run rust:fmt:check
# or fix automatically
cargo fmt --all

# Check linting
npm run rust:clippy
# or fix automatically
cargo clippy --fix --allow-dirty

# Check compilation
cargo check --all-features
```

### Understanding the Output

#### ✅ Good - Ready to Push
```
✨ All Rust CI checks passed! Safe to push.
```

#### ❌ Untracked Modules Detected
```
⚠️  WARNING: Untracked Rust module files detected!
These files exist locally but are not tracked in git:
  - src/config.rs
  - src/integrity.rs

❌ These files will be missing in CI, causing build failures!
💡 Fix: Run `git add <file>` to track these files
```

**Fix**: `git add <files>` then commit

#### ❌ Formatting Issues
```
❌ Failed: cargo fmt --all -- --check
Diff in src/config.rs:25:
...
```

**Fix**: `cargo fmt --all`

#### ❌ Clippy Warnings
```
error: this `if` statement can be collapsed
  --> src/config.rs:196:5
```

**Fix**: `cargo clippy --fix --allow-dirty --allow-staged`

## Workflow Example

### Scenario 1: Pre-Push Check Failed

```bash
# Try to push
git push

# Pre-push hook fails with untracked modules
⚠️  WARNING: Untracked Rust module files detected!
  - src/new_feature.rs

# Fix: Add the file
git add src/new_feature.rs
git commit --amend --no-edit

# Push again
git push  # ✅ Success!
```

### Scenario 2: Testing Before Committing

```bash
# Made changes to Rust code
# Want to test before committing

# Run quick local check
npm run rust:ci

# If you want to be extra sure, simulate CI
npm run ci:simulate
# This creates a clean clone and tests it

# If all passes, commit
git add .
git commit -m "Add new feature"
git push
```

## NPM Scripts Reference

| Script | Description | When to Use |
|--------|-------------|-------------|
| `npm run rust:fmt:check` | Check formatting only | Quick format check |
| `npm run rust:clippy` | Run clippy linting | Check for code issues |
| `npm run rust:ci` | Full local CI check | Before committing |
| `npm run ci:simulate` | Simulate GitHub CI | Most accurate pre-push test |
| `npm run ci:local` | TypeScript + Rust checks | Full local validation |

## Technical Details

### Why Files Were Missing

Git doesn't automatically track new files. They must be explicitly added with `git add`.

**Common scenarios**:
- Created new module file but forgot to `git add` it
- Generated files from templates
- Moved/renamed files and git didn't track the new location

### How the Pre-Push Hook Works

1. User runs `git push`
2. Git triggers `.husky/pre-push` hook
3. Hook runs `scripts/check-rust-ci.js`
4. Script checks for untracked modules
5. If found, script exits with error code 1
6. Git aborts the push
7. User sees error message with fix instructions

### How CI Simulation Works

The `ci:simulate` script:

1. Creates temporary directory in system temp folder
2. Runs `git clone` to copy the repository
3. Changes to that directory (isolated from your working directory)
4. Checks for module files (simulating CI checkout)
5. Runs all CI build commands
6. Reports results
7. Cleans up temporary directory

**Key difference**: Uses `git clone`, so it only sees committed files, not:
- Unstaged changes
- Staged but uncommitted changes
- Untracked files

This is exactly how GitHub CI works.

## Prevention Strategy

### For Developers

1. **Always run `npm run rust:ci` before pushing**
2. **Use `npm run ci:simulate` when in doubt**
3. **Pay attention to pre-push hook failures**
4. **When creating new modules, immediately `git add` them**

### For Code Reviews

Check that PRs include all necessary module files:
```bash
# After checkout PR branch
npm run ci:simulate
```

## Troubleshooting

### "Module not found" in CI but works locally

**Cause**: File exists locally but isn't in git

**Fix**:
```bash
git status  # Check for untracked files
git add <missing-file>
git commit --amend --no-edit  # Add to last commit
git push --force-with-lease   # Update PR
```

### Pre-push hook blocks my push

**Don't skip the hook!** It's catching real issues.

**Fix the issues**:
```bash
# See what failed
npm run rust:ci

# Fix formatting
cargo fmt --all

# Fix clippy issues
cargo clippy --fix --allow-dirty

# Try again
git push
```

### CI simulation fails but local tests pass

**This is the most important scenario!** CI simulation sees what GitHub sees.

**Common causes**:
- Uncommitted changes
- Untracked files
- Platform-specific dependencies

**Fix**: Compare the output and commit any missing files.

## Files Modified

- `.husky/pre-push` - Pre-push git hook
- `scripts/check-rust-ci.js` - Enhanced with untracked file detection
- `scripts/test-ci-build.js` - New CI simulation script
- `package.json` - Added `ci:simulate` script
- All previously untracked module files - Now staged for commit

## Next Steps

1. **Commit all staged files**:
   ```bash
   git commit -m "Fix: Add missing module files and enhance CI validation"
   ```

2. **Push to GitHub**:
   ```bash
   git push
   ```

3. **Verify CI passes**: Check GitHub Actions

4. **Use these tools going forward**: Run `npm run rust:ci` or `npm run ci:simulate` before every push

## Success Metrics

After these changes:
- ✅ Pre-push validation catches issues before they reach GitHub
- ✅ CI failures reduced significantly
- ✅ Faster feedback loop (catch issues locally)
- ✅ Clear error messages guide developers to fixes
- ✅ Automated checks prevent human error
