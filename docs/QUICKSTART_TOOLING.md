# TypeScript Tooling - Quick Start Guide

## Installation

### Step 1: Install Dependencies

```bash
npm install
```

This automatically installs:
- ESLint (code linting)
- Prettier (code formatting)
- Husky (pre-commit hooks)
- type-coverage (type analysis)
- All supporting packages

**Note:** Husky pre-commit hooks are installed automatically via the `prepare` script.

### Step 2: Verify Installation

```bash
# Check ESLint works
npm run lint

# Check Prettier works
npm run format:check

# Check type checking works
npm run type-check

# Run all quality checks
npm run quality
```

If all commands succeed, you're ready to go!

### Step 3: Install VS Code Extensions (Optional but Recommended)

Open VS Code and install these extensions:

1. **ESLint** (`dbaeumer.vscode-eslint`)
2. **Prettier** (`esbenp.prettier-vscode`)
3. **Vitest** (`ZixuanChen.vitest-explorer`)

VS Code should prompt you automatically based on `.vscode/extensions.json`.

## Daily Workflow

### Writing Code

1. **Edit files as normal** - VS Code auto-formats on save
2. **Commit your changes** - Pre-commit hooks auto-run
3. **Push to GitHub** - CI runs all quality checks

That's it! The tooling is transparent and automatic.

### Running Quality Checks Manually

```bash
# Fix all auto-fixable issues
npm run lint:fix

# Format all TypeScript files
npm run format

# Check everything
npm run quality
# OR (on Windows with PowerShell)
.\make.ps1 quality
# OR
Import-Module ./Beefcake.psm1
make quality
```

### Common Commands

| Command | Purpose | When to Use |
|---------|---------|-------------|
| `npm run lint` | Check for linting errors | Before committing |
| `npm run lint:fix` | Auto-fix linting errors | When you have many errors |
| `npm run format` | Format all TS files | Rarely (auto on save) |
| `npm run format:check` | Check if formatted | In CI or before push |
| `npm run type-check` | Check TypeScript types | Before committing large changes |
| `npm run quality` | Run all checks | Before creating a PR |
| `.\make.ps1 quality` | Same as above (via PowerShell shim) | Quick command (Windows) |
| `make quality` | Same as above (via PowerShell module) | Native-feeling command (Windows) |

## What Happens on Commit

When you run `git commit`, this happens automatically:

```
1. Pre-commit hook triggers
2. lint-staged runs on modified files:
   ├─ ESLint --fix (auto-fixes issues)
   └─ Prettier (formats code)
3. If all pass → Commit succeeds ✅
4. If any fail → Commit blocked ❌ (fix errors and retry)
```

**Example:**
```bash
$ git commit -m "Add new feature"
✔ Preparing lint-staged...
✔ Running tasks for staged files...
  ✔ package.json — 2 files
    ✔ *.ts — 2 files
      ✔ eslint --fix
      ✔ prettier --write
✔ Applying modifications from tasks...
✔ Cleaning up temporary files...
[main abc1234] Add new feature
 2 files changed, 50 insertions(+)
```

## What Happens in CI (GitHub Actions)

On every push/PR, CI runs these jobs in parallel:

**✅ Must Pass:**
- ESLint (TypeScript linting)
- Prettier (formatting check)
- TypeScript type check
- Rust unit tests
- Rust integration tests
- Frontend unit tests
- Clippy (Rust linting)

**⚠️ Advisory (continues on error):**
- Type coverage (target: 95%)

If any required job fails, the PR cannot be merged.

## Troubleshooting

### "Commit blocked by pre-commit hook"

**Solution 1: Fix the issues automatically**
```bash
npm run lint:fix
git add .
git commit -m "Your message"
```

**Solution 2: See what failed**
```bash
npm run lint  # Check linting
npm run format:check  # Check formatting
```

**Solution 3: Bypass hook (not recommended)**
```bash
git commit --no-verify -m "Your message"
```
⚠️ Warning: CI will still fail if code doesn't pass checks.

### "ESLint shows too many errors"

Run auto-fix first:
```bash
npm run lint:fix
```

This fixes ~80% of issues. Remaining issues need manual fixes.

### "VS Code not auto-formatting"

1. Check default formatter:
   - `Ctrl+Shift+P` → "Format Document With..." → Select Prettier
2. Verify extension installed:
   - Extensions → Search "Prettier"
3. Check settings:
   - `.vscode/settings.json` should exist with `"editor.formatOnSave": true`

### "CI passes locally but fails in GitHub"

Run what CI runs:
```bash
# Use clean install (like CI does)
npm ci

# Run checks
npm run lint
npm run format:check
npm run type-check
```

## Next Steps

### Add Tests (Phase 4)

Current test coverage: ~1% (2 test files)
Target: 70%+ coverage

See [TESTING.md](TESTING.md) for testing guidelines.

### Improve Type Coverage (Phase 6)

Check current coverage:
```bash
npm run type-coverage:detail
```

Fix implicit `any` types to reach 95% coverage.

### Learn More

- **Detailed quality guide:** [docs/CODE_QUALITY.md](docs/CODE_QUALITY.md)
- **Contributing guidelines:** [CONTRIBUTING.md](CONTRIBUTING.md)
- **Implementation summary:** [TYPESCRIPT_TOOLING_SETUP.md](TYPESCRIPT_TOOLING_SETUP.md)
- **Testing guide:** [TESTING.md](TESTING.md)

## Quick Reference

### NPM Scripts

```bash
# Development
npm run dev                  # Start dev server
npm run build                # Build for production

# Testing
npm test                     # Run unit tests
npm run test:coverage        # With coverage
npm run test:e2e             # E2E tests

# Quality
npm run lint                 # Check linting
npm run lint:fix             # Fix linting
npm run format               # Format code
npm run format:check         # Check formatting
npm run type-check           # Check types
npm run type-coverage        # Check type coverage
npm run quality              # Run all checks

# Documentation
npm run docs                 # Generate all docs
npm run docs:ts              # TypeScript docs only
```

### Makefile Targets

```bash
# Development
make dev                     # Start dev server
make build                   # Build for production

# Quality
make lint                    # Run ESLint
make fmt                     # Format Rust + TypeScript
make quality                 # Run all quality checks
make clippy                  # Run Rust Clippy

# Testing
make test                    # Run Rust tests
make test-ts                 # Run TypeScript tests with coverage

# Documentation
make docs                    # Generate all docs
make docs-open               # Generate and open docs
```

---

**Ready to contribute?** See [CONTRIBUTING.md](CONTRIBUTING.md)

**Need help?** Open an issue or check [docs/CODE_QUALITY.md](docs/CODE_QUALITY.md)
