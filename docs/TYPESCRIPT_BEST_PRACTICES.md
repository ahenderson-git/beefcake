# TypeScript Best Practices

## Overview

This document outlines TypeScript best practices for the Beefcake project, covering patterns, common pitfalls, and guidelines for maintaining type-safe code.

## Table of Contents

1. [Strict Mode Configuration](#strict-mode-configuration)
2. [Error Handling](#error-handling)
3. [Type Safety Patterns](#type-safety-patterns)
4. [Common Pitfalls](#common-pitfalls)
5. [Testing Guidelines](#testing-guidelines)
6. [IDE Integration](#ide-integration)
7. [CI/CD Integration](#cicd-integration)

---

## Strict Mode Configuration

### Current Settings (`tsconfig.json`)

```json
{
  "compilerOptions": {
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noImplicitReturns": true,
    "noFallthroughCasesInSwitch": true,
    "noUncheckedIndexedAccess": true,
    "exactOptionalPropertyTypes": true,
    "noImplicitOverride": true
  }
}
```

### What This Means

✅ **Benefits:**
- Catches errors at compile time
- Forces explicit type annotations
- Prevents common runtime errors
- Improves code documentation

⚠️ **Trade-offs:**
- More verbose code
- Requires type guards for unknown types
- Stricter function signatures

---

## Error Handling

### ❌ Incorrect Pattern

```typescript
try {
  await someAsyncOperation();
} catch (error) {
  // ERROR: 'error' is of type 'unknown' (TS18046)
  console.error(error.message);
}
```

### ✅ Correct Patterns

#### Pattern 1: Type Guard with `instanceof`

```typescript
try {
  await someAsyncOperation();
} catch (error) {
  if (error instanceof Error) {
    console.error(error.message);
    console.error(error.stack);
  } else {
    console.error('Unknown error:', error);
  }
}
```

#### Pattern 2: Type Guard with `typeof`

```typescript
try {
  await execAsync('some command');
} catch (error) {
  // For exec errors with code/stdout/stderr
  if (error && typeof error === 'object' && 'code' in error) {
    const execError = error as { code: number; stdout: string; stderr: string };
    console.error(`Command failed with code ${execError.code}`);
    console.error(execError.stderr);
  } else {
    console.error('Unexpected error:', error);
  }
}
```

#### Pattern 3: Custom Type Guard Function

```typescript
function isExecError(error: unknown): error is { code: number; stdout: string; stderr: string } {
  return (
    error !== null &&
    typeof error === 'object' &&
    'code' in error &&
    typeof (error as any).code === 'number'
  );
}

try {
  await execAsync('some command');
} catch (error) {
  if (isExecError(error)) {
    console.error(`Exit code: ${error.code}`);
  }
}
```

---

## Type Safety Patterns

### 1. Avoid `any` - Use `unknown` Instead

#### ❌ Incorrect

```typescript
function parseJSON(input: string): any {
  return JSON.parse(input);
}
```

#### ✅ Correct

```typescript
function parseJSON(input: string): unknown {
  return JSON.parse(input);
}

// Force type checking at usage site
const data = parseJSON('{"name": "Alice"}');
if (typeof data === 'object' && data !== null && 'name' in data) {
  console.log((data as { name: string }).name);
}
```

### 2. Use Type Assertions Sparingly

#### ❌ Incorrect (Unsafe)

```typescript
const data = await fetchData() as MyType;
// No runtime validation!
```

#### ✅ Correct (With Runtime Validation)

```typescript
import { z } from 'zod';

const MyTypeSchema = z.object({
  name: z.string(),
  age: z.number(),
});

type MyType = z.infer<typeof MyTypeSchema>;

const data = await fetchData();
const validated = MyTypeSchema.parse(data); // Throws if invalid
```

### 3. Prefer Interfaces for Objects

#### ❌ Acceptable (Type Alias)

```typescript
type User = {
  id: string;
  name: string;
};
```

#### ✅ Preferred (Interface)

```typescript
interface User {
  id: string;
  name: string;
}

// Interfaces can be extended
interface AdminUser extends User {
  permissions: string[];
}
```

### 4. Use Union Types for Discriminated Unions

```typescript
type Result<T> =
  | { success: true; data: T }
  | { success: false; error: string };

function handleResult<T>(result: Result<T>) {
  if (result.success) {
    // TypeScript knows result.data exists
    console.log(result.data);
  } else {
    // TypeScript knows result.error exists
    console.error(result.error);
  }
}
```

---

## Common Pitfalls

### 1. Unused Variables/Functions (TS6133)

**Problem:** Declared but never used

**Detection:**
- Build fails with `error TS6133`
- Pre-commit hook catches it
- ESLint shows warning

**Fix:**
```typescript
// Remove unused code
// OR prefix with underscore if intentionally unused
function _helperForFuture() { }
```

### 2. Unknown Type Errors (TS18046)

**Problem:** Accessing properties on `unknown` type

**Detection:**
- Build fails with `error TS18046`
- E2E compliance tests catch it

**Fix:** Use type guards (see [Error Handling](#error-handling))

### 3. Nullable Array Access (noUncheckedIndexedAccess)

**Problem:** Array access might be undefined

```typescript
// ❌ Incorrect
const items = ['a', 'b', 'c'];
const first = items[0]; // Type: string | undefined
console.log(first.toUpperCase()); // Error!
```

```typescript
// ✅ Correct
const items = ['a', 'b', 'c'];
const first = items[0];
if (first !== undefined) {
  console.log(first.toUpperCase());
}

// OR use optional chaining
console.log(items[0]?.toUpperCase());
```

### 4. Missing Return Statements

**Problem:** Function doesn't return in all code paths

```typescript
// ❌ Incorrect
function getValue(flag: boolean): string {
  if (flag) {
    return 'yes';
  }
  // Missing return!
}
```

```typescript
// ✅ Correct
function getValue(flag: boolean): string {
  if (flag) {
    return 'yes';
  }
  return 'no';
}
```

---

## Testing Guidelines

### E2E Tests (`e2e/` directory)

**ESLint Override:** E2E tests allow `console.log` but enforce type safety

```json
{
  "files": ["e2e/**/*.ts"],
  "rules": {
    "@typescript-eslint/no-unused-vars": "error",
    "@typescript-eslint/no-explicit-any": "error",
    "no-console": "off"
  }
}
```

**Best Practices:**
- Always use type guards in catch blocks
- Import types from `@playwright/test`
- Use `execAsync` for shell commands (returns typed errors)
- Test TypeScript compliance (see `e2e/typescript-compliance.spec.ts`)

### Unit Tests (`src-frontend/**/*.test.ts`)

**Best Practices:**
- Mock with proper TypeScript types
- Use `vi.mocked()` from Vitest for type-safe mocks
- Test error scenarios with type guards
- Avoid `as any` - use proper type assertions

---

## IDE Integration

### VS Code Settings (`.vscode/settings.json`)

```json
{
  "typescript.tsdk": "node_modules/typescript/lib",
  "typescript.suggest.autoImports": true,
  "typescript.updateImportsOnFileMove.enabled": "always",
  "typescript.inlayHints.parameterNames.enabled": "all",
  "typescript.inlayHints.functionLikeReturnTypes.enabled": true,
  "typescript.inlayHints.variableTypes.enabled": true
}
```

**Features:**
- ✅ Inline type hints
- ✅ Auto-import suggestions
- ✅ Update imports on file move
- ✅ Parameter name hints

### Recommended Extensions

- **ESLint** (`dbaeumer.vscode-eslint`)
- **Prettier** (`esbenp.prettier-vscode`)
- **Error Lens** (`usernamehw.errorlens`) - Shows errors inline

---

## CI/CD Integration

### Pre-Commit Hook (`.husky/pre-commit`)

```bash
#!/usr/bin/env sh
. "$(dirname -- "$0")/_/husky.sh"

npx lint-staged
npm run type-check
```

**What It Does:**
- Runs ESLint on staged files
- Runs TypeScript type checking
- Blocks commit if errors found

### Build Pipeline (`package.json`)

```json
{
  "prebuild": "npm run type-check && node scripts/clean.js",
  "build": "tsc && vite build --emptyOutDir false"
}
```

**Steps:**
1. `prebuild`: Type check catches errors early
2. `tsc`: Compiles TypeScript (generates no output with `--noEmit`)
3. `vite build`: Bundles application

### Type Checking Utility

```bash
# Quick check
npm run type-check

# Detailed error report
node scripts/check-types.js
```

**Features:**
- Groups errors by file
- Categorizes errors (unused, type mismatches, etc.)
- Color-coded output
- Exit codes for automation

---

## Quick Reference

### Common Commands

```bash
# Type checking
npm run type-check              # Fast check
node scripts/check-types.js     # Detailed report

# Linting
npm run lint                    # Check for issues
npm run lint:fix                # Auto-fix issues

# Testing
npm test                        # Unit tests
npm run test:e2e                # E2E tests (includes TS compliance)

# Build
npm run build                   # Full build (includes type check)
```

### Error Code Reference

| Code   | Description                | Fix                          |
|--------|----------------------------|------------------------------|
| TS6133 | Unused variable/function   | Remove or prefix with `_`    |
| TS18046| Unknown type access        | Add type guard               |
| TS2322 | Type mismatch              | Check types, add assertion   |
| TS2345 | Argument type mismatch     | Verify function signature    |
| TS2551 | Property doesn't exist     | Check object structure       |

---

## Resources

### Internal Documentation

- [Port Management Guide](./PORT_MANAGEMENT.md)
- [Test ID Reference](./TEST_ID_REFERENCE.md)
- [TypeScript Patterns](../docs/TYPESCRIPT_PATTERNS.md)

### External Resources

- [TypeScript Handbook](https://www.typescriptlang.org/docs/handbook/intro.html)
- [TypeScript Do's and Don'ts](https://www.typescriptlang.org/docs/handbook/declaration-files/do-s-and-don-ts.html)
- [@typescript-eslint Rules](https://typescript-eslint.io/rules/)

---

## Contributing

When adding new TypeScript code:

1. ✅ Run `npm run type-check` before committing
2. ✅ Use proper error handling patterns
3. ✅ Avoid `any` - use `unknown` and type guards
4. ✅ Add tests for new patterns
5. ✅ Update this document for new best practices

---

## Troubleshooting

### "Build works locally but fails in CI"

**Cause:** Different TypeScript versions

**Fix:**
```bash
# Lock TypeScript version in package.json
npm install --save-dev typescript@5.3.3 --save-exact
```

### "ESLint and TypeScript report different errors"

**Cause:** ESLint uses a different parser

**Fix:**
```bash
# Ensure ESLint uses same tsconfig
# Already configured in .eslintrc.json:
"parserOptions": {
  "project": "./tsconfig.json"
}
```

### "Type errors in node_modules"

**Cause:** Third-party type definitions

**Fix:** Already configured in `tsconfig.json`:
```json
{
  "skipLibCheck": true
}
```

---

**Last Updated:** 2026-01-28
**Maintained By:** Development Team
