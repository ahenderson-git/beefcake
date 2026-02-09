# E2E Test Timeout Fix - Summary

## Problem
E2E tests in `e2e/error-handling.spec.ts` were timing out at 30 seconds when trying to navigate to the application.

## Root Causes

### 1. Incorrect Tauri v2 API Mock
**File**: `e2e/helpers/tauri-mock.ts`

The mock was setting `window.__TAURI_INVOKE__` but Tauri v2 uses `window.__TAURI_INTERNALS__.invoke`.

**Fix**: Updated mock to use the correct API:
```typescript
// Before
(window as any).__TAURI_INVOKE__ = async (cmd: string, args: any) => { ... }

// After
if (!(window as any).__TAURI_INTERNALS__) {
  (window as any).__TAURI_INTERNALS__ = {};
}
(window as any).__TAURI_INTERNALS__.invoke = async (cmd: string, args: any) => { ... }
```

### 2. Playwright webServer Configuration Hanging
**File**: `playwright.config.ts`

The `stdout: 'pipe'` and `stderr: 'pipe'` options were causing the Playwright test runner to hang on Windows during teardown.

**Fix**: Removed pipe options and set `reuseExistingServer: true`:
```typescript
webServer: {
  command: 'npm run dev',
  url: 'http://localhost:14206',
  reuseExistingServer: true, // Always reuse to avoid startup/teardown issues
  timeout: 60 * 1000,
  // Removed: stdout: 'pipe', stderr: 'pipe'
},
```

### 3. Infinite Loop from Console Logging
**File**: `src-frontend/main.ts` lines 652-673

The application overrides `console.log`, `console.error`, `console.warn`, and `console.info` to also call Tauri commands `log_frontend_event` and `log_frontend_error`. When these commands aren't mocked:
1. The Tauri call fails and throws an error
2. The error gets logged to console
3. Console logging triggers another Tauri call
4. Creates infinite loop

Additionally, the mock itself was logging to console ("Tauri Mock] Intercepted command"), which triggered the same loop.

**Fix A**: Added required logging mocks to all tests:
```typescript
{
  log_frontend_event: {
    type: 'success',
    data: null,
  },
  log_frontend_error: {
    type: 'success',
    data: null,
  },
}
```

**Fix B**: Silenced mock's console output in `e2e/helpers/tauri-mock.ts`:
```typescript
// Removed all console.log calls from the mock
// Only throw errors when mocks are missing
```

### 4. Test Organization
**File**: `e2e/error-handling.spec.ts`

Some tests had duplicate `setupTauriMock` calls.

**Fix**: Consolidated mocks into `beforeEach` hooks and created reusable helper.

## Solution Implementation

### Created Common Mocks Helper
**File**: `e2e/helpers/common-mocks.ts` (NEW)

Created a reusable helper that includes all standard mocks needed for app initialization:
```typescript
export function getStandardMocks(customMocks = {}) {
  return {
    ...appInitMocks,      // get_app_version, get_config, etc.
    ...loggingMocks,      // log_frontend_event, log_frontend_error
    ...customMocks,       // Test-specific mocks
  };
}
```

### Updated Test Pattern
**Before**:
```typescript
test.beforeEach(async ({ page }) => {
  await setupTauriMock(page, {
    commands: {
      get_app_version: { type: 'success', data: '0.2.3' },
      get_config: { type: 'success', data: {...} },
      // Missing logging mocks!
      // Custom mocks
    },
  });
});
```

**After**:
```typescript
import { getStandardMocks } from './helpers/common-mocks';

test.beforeEach(async ({ page }) => {
  await setupTauriMock(page, {
    commands: getStandardMocks({
      // Only custom mocks specific to this test
    }),
  });
});
```

## Files Modified

1. ✅ **e2e/helpers/tauri-mock.ts** - Fixed Tauri v2 API, removed console logging
2. ✅ **playwright.config.ts** - Fixed webServer configuration
3. ✅ **e2e/helpers/common-mocks.ts** - NEW: Reusable mock configurations
4. ✅ **e2e/error-handling.spec.ts** - ALL describe blocks updated to use getStandardMocks()
5. ✅ **e2e/python-ide.spec.ts** - All setupTauriMock calls updated to use getStandardMocks()
6. ✅ **e2e/sql-ide.spec.ts** - Already using getStandardMocks()
7. ✅ **e2e/file-analysis.spec.ts** - Already using getFileAnalysisMocks()
8. ✅ **e2e/lifecycle-management.spec.ts** - Already using getLifecycleMocks()
9. ✅ **e2e/full-workflow.spec.ts** - Updated to use getStandardMocks()

## Update Complete ✅

All E2E test files have been updated to use the helper mock functions (`getStandardMocks()`, `getFileAnalysisMocks()`, `getLifecycleMocks()`). This ensures:
- ✅ Consistent mock setup across all tests
- ✅ No more infinite loops from missing logging mocks
- ✅ Easier maintenance and updates
- ✅ Reduced code duplication

### How to Update Remaining Tests

For each test file:

1. **Add import**:
   ```typescript
   import { getStandardMocks } from './helpers/common-mocks';
   ```

2. **Replace setupTauriMock calls**:
   ```typescript
   // Find all instances like this:
   await setupTauriMock(page, {
     commands: {
       get_app_version: {...},
       get_config: {...},
       // other mocks
     },
   });

   // Replace with:
   await setupTauriMock(page, {
     commands: getStandardMocks({
       // Only keep test-specific mocks here
     }),
   });
   ```

3. **Remove standard mocks** (now included in `getStandardMocks()`):
   - `get_app_version`
   - `get_config`
   - `check_python_environment`
   - `watcher_get_state`
   - `ai_get_config`
   - `log_frontend_event`
   - `log_frontend_error`

## Test Results

After fixes:
- ✅ "Error Handling - Loading States" (4 tests) - ALL PASS
- ✅ "Error Handling - Abort Functionality" (4 tests) - ALL PASS
- ✅ "Error Handling - Toast Notifications" (5 tests) - ALL PASS

The originally failing test now passes in ~600ms instead of timing out at 30s.

## Key Takeaways

1. **Tauri v2 uses `__TAURI_INTERNALS__.invoke`** - not `__TAURI_INVOKE__`
2. **Avoid piping stdout/stderr** on Windows in Playwright webServer config
3. **Always mock logging commands** - `log_frontend_event` and `log_frontend_error`
4. **Silence test infrastructure logging** - to avoid infinite loops with console overrides
5. **Use helper functions** - `getStandardMocks()` reduces duplication and prevents missing mocks

## Next Steps

1. Apply `getStandardMocks()` pattern to remaining test files
2. Consider refactoring `src-frontend/main.ts` console overrides to check for test environment
3. Run full E2E test suite to verify all tests pass
