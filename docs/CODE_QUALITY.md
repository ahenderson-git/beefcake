# Code Quality Guide

This document explains the code quality tools and standards used in the Beefcake project.

## Overview

Beefcake uses a comprehensive quality toolchain to ensure consistent, maintainable code:

| Tool | Purpose | Language | Auto-fix |
|------|---------|----------|----------|
| **ESLint** | Linting & bug detection | TypeScript | ✅ Yes |
| **Prettier** | Code formatting | TypeScript/JSON/Markdown | ✅ Yes |
| **TypeScript** | Type checking | TypeScript | ❌ No |
| **type-coverage** | Type coverage analysis | TypeScript | ❌ No |
| **Clippy** | Linting | Rust | ⚠️ Partial |
| **cargo fmt** | Code formatting | Rust | ✅ Yes |

## Quick Start

```bash
# Run all quality checks (TypeScript + Rust)
make quality

# Format all code
make fmt

# Lint TypeScript code
npm run lint

# Fix linting issues automatically
npm run lint:fix

# Check formatting without modifying files
npm run format:check

# Format TypeScript code
npm run format
```

## ESLint Configuration

### Rules Overview

Our ESLint configuration enforces:

**Type Safety:**
- ❌ No `any` types (`@typescript-eslint/no-explicit-any`)
- ⚠️ Explicit function return types (`@typescript-eslint/explicit-function-return-type`)
- ❌ No floating promises (`@typescript-eslint/no-floating-promises`)
- ❌ No misused promises (`@typescript-eslint/no-misused-promises`)

**Code Quality:**
- ❌ No unused variables (prefix with `_` to ignore)
- ❌ Use `===` instead of `==` (`eqeqeq`)
- ❌ No `var`, use `const` or `let` (`no-var`, `prefer-const`)
- ⚠️ Prefer optional chaining (`?.`) over manual checks

**Import Organization:**
- Alphabetically sorted imports
- Grouped by: builtin → external → internal → parent → sibling → index
- Newlines between groups

### Running ESLint

```bash
# Check for issues
npm run lint

# Fix issues automatically
npm run lint:fix

# Check specific files
npx eslint src-frontend/main.ts

# Check with detailed output
npx eslint src-frontend --ext .ts --format=verbose
```

### Common ESLint Fixes

**Problem: Implicit `any` type**
```typescript
// ❌ Bad
function handleEvent(event) {
  console.log(event.payload);
}

// ✅ Good
function handleEvent(event: CustomEvent<string>) {
  console.log(event.payload);
}
```

**Problem: Floating promise**
```typescript
// ❌ Bad
async function loadData() {
  api.fetchData(); // Promise not awaited
}

// ✅ Good
async function loadData() {
  await api.fetchData();
}
```

**Problem: Missing return type**
```typescript
// ⚠️ Warning
function calculateTotal(items: Item[]) {
  return items.reduce((sum, item) => sum + item.price, 0);
}

// ✅ Good
function calculateTotal(items: Item[]): number {
  return items.reduce((sum, item) => sum + item.price, 0);
}
```

## Prettier Configuration

Prettier handles all formatting automatically. Our settings:

```json
{
  "semi": true,              // Use semicolons
  "singleQuote": true,       // Single quotes for strings
  "trailingComma": "es5",    // Trailing commas where valid
  "printWidth": 100,         // Line length limit
  "tabWidth": 2,             // 2 spaces for indentation
  "arrowParens": "avoid"     // (x) => x, not (x) => x
}
```

### Running Prettier

```bash
# Format all TypeScript files
npm run format

# Check formatting without changing files
npm run format:check

# Format specific files
npx prettier --write src-frontend/main.ts
```

## Type Coverage

Type coverage measures how much of your code is explicitly typed (vs. `any` or implicit types).

### Checking Type Coverage

```bash
# Check type coverage (target: 95%+)
npm run type-coverage

# See detailed breakdown
npm run type-coverage:detail
```

**Example output:**
```
src-frontend/main.ts:
  172:9: Promise.race has implicit any type
  433:34: Parameter 'event' implicitly has any

Type coverage: 92.3% (2450/2656 symbols)
Target: 95%
```

### Improving Type Coverage

1. **Add explicit types to function parameters**
```typescript
// Before: 90% coverage
function process(data) {
  return data.map(x => x * 2);
}

// After: 100% coverage
function process(data: number[]): number[] {
  return data.map(x => x * 2);
}
```

2. **Type event handlers**
```typescript
// Before
element.addEventListener('click', (e) => {
  console.log(e.target);
});

// After
element.addEventListener('click', (e: MouseEvent) => {
  console.log((e.target as HTMLElement));
});
```

3. **Avoid type assertions when possible**
```typescript
// Less ideal
const result = apiCall() as ApiResponse;

// Better
const result: ApiResponse = await apiCall();
```

## Pre-commit Hooks

All quality checks run automatically before each commit via Husky.

### What Runs on Pre-commit

1. **lint-staged**: Only checks files you modified
2. **ESLint --fix**: Auto-fixes linting issues
3. **Prettier**: Auto-formats code

### Hook Workflow

```
git add src-frontend/main.ts
git commit -m "Update main"
  ↓
[Running pre-commit hook]
  ✓ ESLint (0.5s)
  ✓ Prettier (0.2s)
  ✓ Type check (1.2s)
[Commit successful]
```

### Bypassing Hooks

**⚠️ Not recommended** - CI will still fail if checks don't pass:

```bash
git commit --no-verify -m "Skip hooks"
```

## CI/CD Integration

All quality checks run in parallel on every push/PR:

```yaml
✓ ESLint (TypeScript Linting)
✓ Prettier (Code Formatting)
✓ TypeScript Type Check
✓ TypeScript Type Coverage (non-blocking)
✓ Clippy (Rust Linting)
✓ Frontend Unit Tests
✓ Rust Unit Tests
✓ Rust Integration Tests
```

CI fails if any check fails (except type-coverage, which is advisory).

## VS Code Integration

### Recommended Extensions

Install these for the best experience:

- **ESLint** (`dbaeumer.vscode-eslint`) - Shows linting errors inline
- **Prettier** (`esbenp.prettier-vscode`) - Formats on save
- **Vitest** (`ZixuanChen.vitest-explorer`) - Test runner UI

### Auto-fix on Save

The `.vscode/settings.json` is configured to:

1. **Format on save** with Prettier
2. **Fix ESLint errors** on save
3. **Organize imports** on save
4. **Show rulers** at 100 characters

No manual formatting needed!

### Keyboard Shortcuts

- **Format Document**: `Shift + Alt + F`
- **Organize Imports**: `Shift + Alt + O`
- **Show ESLint output**: `Ctrl + Shift + U` → ESLint

## Troubleshooting

### ESLint shows too many errors

Start by auto-fixing:

```bash
npm run lint:fix
```

This fixes ~80% of issues automatically.

### Prettier conflicts with ESLint

Our config includes `eslint-config-prettier` which disables conflicting rules. If you see conflicts:

1. Update dependencies: `npm update`
2. Clear ESLint cache: `rm .eslintcache`

### Type coverage is below 95%

This is advisory only (CI won't fail). To improve:

1. Run `npm run type-coverage:detail` to see specific issues
2. Add explicit types to flagged locations
3. Avoid `any` types

### Pre-commit hook is slow

lint-staged only checks modified files, so it should be fast (<5s). If slow:

1. Check file count: `git diff --cached --name-only`
2. Ensure `.eslintcache` exists (speeds up subsequent runs)
3. Consider splitting large commits

### CI passes locally but fails in GitHub Actions

1. Ensure all files are committed
2. Run exactly what CI runs:
   ```bash
   npm ci  # Clean install
   npm run lint
   npm run format:check
   npm run type-check
   ```
3. Check Node.js version matches CI (18.x)

## Best Practices

### DO ✅

- **Run quality checks before pushing**
  ```bash
  make quality
  ```
- **Fix issues immediately** - Don't accumulate technical debt
- **Use auto-fix** - Let tools format your code
- **Add types** - Explicit is better than implicit
- **Check diffs** - Review what pre-commit hooks changed

### DON'T ❌

- **Disable rules without discussion** - Rules exist for good reasons
- **Bypass pre-commit hooks** - You'll fail in CI anyway
- **Use `any` type** - Defeats TypeScript's purpose
- **Ignore warnings** - Warnings become errors in production
- **Manually format code** - Let Prettier handle it

## Makefile Targets

All quality tools are available via Make:

```bash
make fmt          # Format all code (Rust + TypeScript)
make lint         # Run ESLint
make quality      # Run all quality checks
make clippy       # Run Rust Clippy
make test-ts      # Run TypeScript tests with coverage
```

## Configuration Files

```
beefcake/
├── .eslintrc.json          # ESLint rules
├── .prettierrc.json        # Prettier formatting
├── .prettierignore         # Files to skip formatting
├── tsconfig.json           # TypeScript compiler options
├── .husky/
│   └── pre-commit          # Pre-commit hook script
├── .vscode/
│   ├── settings.json       # VS Code workspace settings
│   └── extensions.json     # Recommended extensions
└── package.json
    ├── scripts             # npm commands
    └── lint-staged         # Pre-commit file patterns
```

## Resources

- [ESLint Rules](https://eslint.org/docs/latest/rules/)
- [TypeScript ESLint](https://typescript-eslint.io/)
- [Prettier Options](https://prettier.io/docs/en/options.html)
- [Husky Documentation](https://typicode.github.io/husky/)
- [type-coverage](https://github.com/plantain-00/type-coverage)

---

**Questions?** See [CONTRIBUTING.md](../CONTRIBUTING.md) or open an issue.
