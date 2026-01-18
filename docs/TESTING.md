# Beefcake Testing Guide

> **Current Status**: 181 tests passing (91 TypeScript + 90 Rust) with 100% TypeScript coverage

## Quick Start

```bash
# Run all unit tests (fast)
npm test              # Frontend (vitest) - 91 tests
cargo test            # Rust (cargo test) - 90 tests

# Run all tests including integration
npm run test:all      # Runs Rust + TS + E2E

# Run specific test suites
npm run test:coverage          # Frontend with coverage (100%)
npm run test:rust:integration  # Rust integration tests
npm run test:e2e               # End-to-end GUI tests (skeleton only)

# Interactive modes
npm run test:ui       # Vitest UI for frontend tests
npm run test:watch    # Watch mode - re-runs on file changes
```

## Test Structure

```
beefcake/
├── src-frontend/              # TypeScript source
│   ├── *.test.ts              # Unit tests (colocated with source)
│   ├── utils.test.ts          # ✅ 22 tests - utility functions
│   ├── types.test.ts          # ✅ 4 tests - type guards
│   ├── api.test.ts            # ✅ 55 tests - Tauri API calls
│   └── components/
│       └── Component.test.ts  # ✅ 10 tests - base component
├── src/                       # Rust source
│   ├── analyser/
│   │   └── logic/
│   │       └── tests.rs       # ✅ 40+ tests
│   ├── pipeline/
│   │   └── executor.rs        # ✅ 10 tests (in #[cfg(test)])
│   ├── error.rs               # ✅ 3 tests
│   └── lib.rs
├── tests/                     # Rust integration tests
│   └── integration_analysis.rs # ✅ 15 tests
├── e2e/                       # End-to-end GUI tests
│   └── example.spec.ts        # ⚠️  Skeleton only (TODO)
├── testdata/                  # Test fixtures
│   ├── clean.csv
│   ├── missing_values.csv
│   ├── golden/                # Expected outputs
│   └── pipelines/             # Pipeline specs
└── docs/
    ├── TESTING.md             # This file
    ├── test-matrix.md         # Feature test matrix
    └── ADDING_TEST_IDS.md     # Guide for E2E selectors
```

## Test Layers

### 1. Unit Tests (< 10s total)

**TypeScript** (vitest):
```bash
npm test
```

Example test: `src-frontend/types.test.ts`

**Rust** (cargo test):
```bash
cargo test --lib
```

Example test: `src/analyser/logic/types_test.rs`

### 2. Integration Tests (< 30s total)

**Rust** - Full analysis pipeline with fixtures:
```bash
cargo test --test '*'
```

Example test: `tests/integration_analysis.rs`

### 3. E2E Tests (~ 5min)

**Playwright** - Full GUI automation:
```bash
npm run test:e2e
```

Example test: `e2e/example.spec.ts`

**Note**: E2E tests require the Tauri app to be built first:
```bash
npm run build
npm run tauri build
npm run test:e2e
```

## Writing Tests

### Frontend Unit Test Example

```typescript
// src-frontend/utils.test.ts
import { describe, test, expect } from 'vitest';
import { formatBytes } from './utils';

describe('formatBytes', () => {
  test('should format bytes correctly', () => {
    expect(formatBytes(0)).toBe('0 Bytes');
    expect(formatBytes(1024)).toBe('1 KB');
    expect(formatBytes(1048576)).toBe('1 MB');
  });
});
```

### Rust Unit Test Example

```rust
// src/analyser/logic/types.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_summary_null_pct() {
        let summary = ColumnSummary { /* ... */ };
        assert_eq!(summary.null_pct(), 20.0);
    }
}
```

### Rust Integration Test Example

```rust
// tests/integration_analysis.rs
use beefcake::analyser::logic::analysis::analyze_file;

#[test]
fn test_analyze_clean_csv() {
    let result = analyze_file("testdata/clean.csv");
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.row_count, 10);
    assert_eq!(response.column_count, 6);
}
```

### E2E Test Example

```typescript
// e2e/analyser.spec.ts
import { test, expect } from '@playwright/test';

test('should load and analyze file', async ({ page }) => {
  await page.goto('http://localhost:1420');

  // Click open file
  await page.getByTestId('dashboard-open-file-button').click();

  // Wait for analysis
  await expect(page.getByTestId('analysis-summary-panel')).toBeVisible();

  // Verify results
  const rowCount = await page.getByTestId('analyser-row-count').textContent();
  expect(rowCount).toBe('10');
});
```

## Test Data Fixtures

All test fixtures are in `testdata/`:

- **clean.csv** - Perfect dataset (10 rows, 6 columns, no nulls)
- **missing_values.csv** - 20% missing values
- **mixed_types.csv** - Ambiguous type columns
- **special_chars.csv** - Unicode, emojis, whitespace edge cases
- **wide.csv** - 20+ columns for UI stress testing
- **invalid_format.txt** - For error handling tests

### Using Fixtures in Tests

```rust
// Rust
fn example() {
    let result = analyze_file("testdata/clean.csv");
}
```

```typescript
// TypeScript
const response = await api.analyseFile('./testdata/clean.csv');
```

## CI/CD Integration

Tests run automatically on every PR via GitHub Actions (`.github/workflows/test.yml`):

- ✅ Rust unit tests
- ✅ Rust integration tests
- ✅ Frontend unit tests
- ✅ Clippy lints
- ✅ TypeScript type checking
- 🔄 E2E tests (main branch only, or with `[e2e]` in commit message)

### Running CI Locally

```bash
# Simulate CI environment
npm run test:all
cargo clippy --all-targets -- -D warnings
npx tsc --noEmit
```

## Test Coverage

### Current Coverage Status

**TypeScript**: ✅ **100% coverage** on tested files
- `api.ts`: 100% (55 tests)
- `utils.ts`: 100% (22 tests)
- `types.ts`: 100% (4 tests)
- `Component.ts`: 100% (10 tests)

**Rust**: ~70% estimated (90 tests across core modules)

### Generate Coverage Reports

```bash
# Frontend coverage (vitest)
npm run test:coverage
# Opens HTML report in coverage/index.html

# Rust coverage (requires cargo-tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
# Opens HTML report in tarpaulin-report.html
```

### Coverage Goals
- ✅ Core API layer: **100%** (achieved)
- ✅ Utility functions: **100%** (achieved)
- 🎯 Core logic (analysis, transformations): >80% (in progress)
- 🎯 Rust backend: >70% (current ~70%)
- ⚠️ UI rendering code: Not enforced (complex to test)
- 🎯 Integration tests: Cover all P0 workflows

## Debugging Tests

### TypeScript Tests

**Interactive UI Mode**:
```bash
npm run test:ui
# Opens Vitest UI at http://localhost:51204
# Features: test filtering, coverage view, file tree, re-run on change
```

**Watch Mode**:
```bash
npm run test:watch
# Re-runs tests automatically when files change
# Press 'a' to run all tests
# Press 'f' to run only failed tests
# Press 'u' to update snapshots
# Press 'q' to quit
```

**VSCode Debugging**:
```json
{
  "type": "node",
  "request": "launch",
  "name": "Debug Vitest Tests",
  "runtimeExecutable": "npm",
  "runtimeArgs": ["run", "test"],
  "console": "integratedTerminal",
  "internalConsoleOptions": "neverOpen"
}
```

### Rust Tests

**Run specific test**:
```bash
cargo test test_column_summary_null_pct
```

**Show stdout output**:
```bash
cargo test -- --nocapture
```

**VSCode Debugging**:
```json
{
  "type": "lldb",
  "request": "launch",
  "name": "Debug Rust Test",
  "cargo": {
    "args": ["test", "--no-run", "--lib"],
    "filter": {
      "name": "beefcake",
      "kind": "lib"
    }
  },
  "args": ["test_name", "--nocapture"],
  "cwd": "${workspaceFolder}"
}
```

## Quality Checks

### Full CI Simulation

Run everything CI runs locally:
```bash
# TypeScript checks
npm test                         # Unit tests
npm run test:coverage            # Coverage report
npx tsc --noEmit                 # Type checking
npm run lint                     # ESLint

# Rust checks
cargo test                       # All unit tests
cargo test --test '*'            # Integration tests
cargo clippy --all-targets -- -D warnings  # Lints
cargo fmt -- --check             # Formatting

# Optional: E2E tests (requires built app)
npm run tauri build
npm run test:e2e
```

### Pre-commit Checks

Recommended pre-commit script (`.git/hooks/pre-commit`):
```bash
#!/bin/sh
npm test && cargo test --lib && cargo clippy -- -D warnings
```

### Continuous Quality Monitoring

```bash
# Watch for test failures and coverage changes
npm run test:watch

# Watch for type errors
npx tsc --noEmit --watch

# Watch for linting issues
npm run lint -- --watch
```

## E2E Testing Setup

E2E tests require stable `data-testid` attributes on all UI elements.

### Adding Test IDs

See [`ADDING_TEST_IDS.md`](ADDING_TEST_IDS.md) for detailed guide.

Example:
```typescript
// Before
container.innerHTML = `<button id="btn-open-file">Open File</button>`;

// After
container.innerHTML = `
  <button id="btn-open-file" data-testid="dashboard-open-file-button">
    Open File
  </button>
`;
```

### Finding Elements in E2E Tests

```typescript
// Recommended: data-testid
await page.getByTestId('dashboard-open-file-button').click();

// Avoid: CSS selectors (brittle)
await page.locator('.btn-primary').click(); // Bad!
```

## Test Matrix

All GUI features are mapped in [`test-matrix.md`](test-matrix.md):

- 69 total features mapped
- 28 P0 (critical) features
- 38 P1 (important) features
- 3 P2 (polish) features

Each feature includes:
- Preconditions
- Test steps
- Expected results
- UI/data assertions
- Test type (unit/integration/E2E)
- Automation status

## Troubleshooting

### Tests Fail Locally But Pass in CI

1. Check for hardcoded paths (use `testdata/` relative paths)
2. Ensure fixtures are committed to Git
3. Verify Rust/Node versions match CI (see `.github/workflows/test.yml`)

### E2E Tests Timeout

1. Increase timeout in `playwright.config.ts`:
   ```typescript
   const config = {
     timeout: 60000, // 60 seconds
   };
   ```
2. Check if app is building correctly: `npm run tauri build`
3. Verify test fixtures exist

### Flaky Tests

1. Add explicit waits:
   ```typescript
   await page.waitForSelector('[data-testid="loading-spinner"]', {
     state: 'hidden',
   });
   ```
2. Avoid time-based assertions
3. Use stable selectors (`data-testid`)

### Rust Tests Fail on Windows

1. Use `std::path::Path` for cross-platform paths
2. Check file path separators (use `/` or `PathBuf`)
3. Verify temp directory permissions

## Best Practices

### General

✅ **DO**:
- Write tests before or alongside code (TDD)
- Use descriptive test names
- Test edge cases and error paths
- Keep tests fast and focused
- Use fixtures for test data

❌ **DON'T**:
- Share state between tests
- Use real external services (mock them)
- Hardcode file paths
- Test implementation details (test behavior)

### TypeScript/Vitest

✅ **DO**:
```typescript
test('should format large numbers with commas', () => {
  expect(formatNumber(1000000)).toBe('1,000,000');
});
```

❌ **DON'T**:
```typescript
test('test1', () => {
  expect(something).toBeTruthy(); // Vague
});
```

### Rust

✅ **DO**:
```rust
#[test]
fn test_invalid_file_returns_descriptive_error() {
    let result = analyze_file("nonexistent.csv");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}
```

❌ **DON'T**:
```rust
#[test]
fn test() {
    let result = do_something();
    assert!(result.is_ok()); // Not checking specific behavior
}
```

### E2E

✅ **DO**:
```typescript
test('should display error toast for invalid file', async ({ page }) => {
  await page.getByTestId('btn-open-file').click();
  // ... load invalid file ...
  await expect(page.getByTestId('toast-error')).toContainText(
    'Invalid file format'
  );
});
```

❌ **DON'T**:
```typescript
test('error', async ({ page }) => {
  await page.locator('.btn').click(); // Brittle selector
  await page.waitForTimeout(5000); // Time-based wait
});
```

## Next Steps

1. **Add data-testid attributes** to all UI components (see [`ADDING_TEST_IDS.md`](ADDING_TEST_IDS.md))
2. **Write unit tests** for new features as you build them
3. **Add integration tests** for cross-module workflows
4. **Implement E2E tests** for P0 features first, then P1
5. **Monitor coverage** and aim for >80% on core logic

## Resources

- [Test Strategy](TESTING.md) - Overall test plan
- [Test Matrix](test-matrix.md) - Feature-by-feature test mapping
- [Adding Test IDs](ADDING_TEST_IDS.md) - E2E selector guide
- [Vitest Docs](https://vitest.dev/)
- [Playwright Docs](https://playwright.dev/)
- [Rust Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Tauri Testing Guide](https://tauri.app/v1/guides/testing/)

---

**Questions?** See docs or open an issue.
**Contributing?** All tests must pass before PR merge.
