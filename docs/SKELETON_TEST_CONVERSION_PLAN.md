# Skeleton Test Conversion Plan

**Date**: 2026-01-27
**Goal**: Convert ~60 skeleton tests to real implementation tests
**Priority**: P1 (High ROI - Test IDs already present)

---

## Summary

Skeleton tests are placeholder tests that:
- Have `TODO: Once {feature} is connected:` comments
- Only check `await expect(page).toHaveTitle(/beefcake/i)`
- Don't actually test the feature functionality

**Total Skeleton Tests**: ~60
- Python IDE: 20 tests
- SQL IDE: 23 tests (estimated)
- Full Workflow: 17 tests

---

## Python IDE Skeleton Tests (20 tests)

### Category 1: Editor Features (2 tests)

| Line | Test | Current State | Conversion Plan |
|------|------|---------------|-----------------|
| 136 | Syntax highlighting | TODO + title check | Test for Monaco editor theme, colored keywords |
| 152 | Code completion | TODO + title check | Test autocomplete menu appears, Polars methods suggested |

**Implementation Status**: ⚠️ **Depends on Monaco integration**
- Monaco editor may not be fully interactive in E2E tests
- Consider marking as `.skip()` with note about Monaco limitations

---

### Category 2: Script Execution (4 tests)

| Line | Test | Current State | Conversion Plan |
|------|------|---------------|-----------------|
| 256 | Execute simple script | TODO + title check | Mock `execute_python`, verify output contains result |
| 275 | Execute with data path | TODO + title check | Mock execution with dataset, verify output |
| 291 | Clear output | TODO + title check | Click clear button, verify output is empty |
| 309 | Show execution time | TODO + title check | Verify execution time displays after run |

**Implementation Status**: ✅ **Ready to implement**
- Tauri command: `execute_python`
- Mock response with output
- Verify output panel updates

---

### Category 3: Security Warning (3 tests)

| Line | Test | Current State | Conversion Plan |
|------|------|---------------|-----------------|
| 380 | Show warning on first run | TODO + title check | Mock first_run: false, verify warning modal |
| 398 | Allow execution after accept | TODO + title check | Click accept, verify script runs |
| 413 | Cancel execution | TODO + title check | Click cancel, verify script doesn't run |

**Implementation Status**: ⚠️ **Partially ready**
- Requires security modal test IDs
- Mock `security_warning_acknowledged: false`
- Test modal appearance and buttons

---

### Category 4: Install Polars (2 tests)

| Line | Test | Current State | Conversion Plan |
|------|------|---------------|-----------------|
| 431 | Show install button | TODO + title check | Already passing (button exists) - **DONE** |
| 459 | Install Polars flow | TODO + title check | Mock `install_polars`, verify success message |

**Implementation Status**: ✅ **Ready to implement**
- Mock `install_polars` command
- Verify loading state during install
- Verify success toast

---

### Category 5: Error Handling (2 tests)

| Line | Test | Current State | Conversion Plan |
|------|------|---------------|-----------------|
| 484 | Display import errors | TODO + title check | Mock error response, verify error in output |
| 512 | Display syntax errors | TODO + title check | Mock Python syntax error, verify formatted error |

**Implementation Status**: ✅ **Ready to implement**
- Mock Tauri error response
- Verify error formatting in output panel
- Check for error styling (red text, etc.)

---

### Category 6: Script Management (4 tests)

| Line | Test | Current State | Conversion Plan |
|------|------|---------------|-----------------|
| 552 | Save script to file | TODO + title check | Mock `save_file_dialog`, verify save command |
| 568 | Load script from file | TODO + title check | Mock `open_file_dialog`, verify content loads |
| 584 | Auto-save on switch | TODO + title check | Switch views, verify script persists |
| 602 | Copy output | TODO + title check | Mock clipboard, click copy, verify copied |

**Implementation Status**: ⚠️ **Partially ready**
- File dialogs can be mocked ✅
- Clipboard API may need special handling
- Auto-save requires state management

---

### Category 7: Lifecycle Integration (3 tests)

| Line | Test | Current State | Conversion Plan |
|------|------|---------------|-----------------|
| 723 | Show stage selector | TODO + title check | Load dataset with versions, verify selector |
| 739 | Update path on switch | TODO + title check | Switch stage, verify data_path updates |
| 757 | Show refactor button | TODO + title check | Switch Raw→Cleaned, verify button appears |

**Implementation Status**: ✅ **Ready to implement**
- Mock dataset with multiple versions
- Test stage selector visibility
- Test data path updates

---

## SQL IDE Skeleton Tests (23 tests - estimated)

Similar categories to Python IDE:
1. Editor Features (2 tests)
2. Query Execution (4 tests)
3. Security Warning (3 tests)
4. Install Polars (2 tests)
5. Error Handling (2 tests)
6. Script Management (4 tests)
7. Lifecycle Integration (3 tests)
8. Query-specific features (3 tests)

**Status**: Not yet analyzed in detail

---

## Full Workflow Skeleton Tests (17 tests)

| Line | Test | Current State | Why Skeleton |
|------|------|---------------|--------------|
| 145 | Open file dialog | .skip() | Needs file loading implementation |
| 187 | Empty analyser state | .skip() | Needs navigation to analyser |
| 198 | Display health score | .skip() | Needs file loaded |
| 209 | Show column statistics | .skip() | Needs file loaded |
| 224 | Expand column row | .skip() | Needs file loaded + interaction |
| 238 | Enable cleaning | .skip() | Needs file loaded + config |
| 249 | Bulk cleaning | .skip() | Needs file loaded + config |
| 260-366 | Various workflows | .skip() | End-to-end flows |

**Status**: Most are intentionally skipped waiting for integration

---

## Conversion Priority Matrix

### Tier 1: Quick Wins (Estimated: 2-3 hours)

**Python IDE** (6 tests):
- ✅ Execute simple script (line 256)
- ✅ Execute with data path (line 275)
- ✅ Clear output (line 291)
- ✅ Install Polars flow (line 459)
- ✅ Display import errors (line 484)
- ✅ Display syntax errors (line 512)

**SQL IDE** (6 tests - similar):
- Execute simple query
- Execute with WHERE clause
- Clear output
- Display SQL errors
- Display column errors
- Show helpful error messages

**Estimated Impact**: +12 real tests

---

### Tier 2: Medium Effort (Estimated: 3-4 hours)

**Python IDE** (7 tests):
- ✅ Show stage selector (line 723)
- ✅ Update path on switch (line 739)
- ✅ Show refactor button (line 757)
- ⚠️ Show warning on first run (line 380)
- ⚠️ Allow execution after accept (line 398)
- ⚠️ Save script to file (line 552)
- ⚠️ Load script from file (line 568)

**SQL IDE** (7 tests - similar):
- Stage selector tests (3)
- Security warning tests (2)
- Save/load tests (2)

**Estimated Impact**: +14 real tests

---

### Tier 3: Challenging (Estimated: 4-6 hours)

**Python IDE** (5 tests):
- ⚠️ Syntax highlighting (line 136) - Monaco limitations
- ⚠️ Code completion (line 152) - Monaco limitations
- ⚠️ Auto-save on switch (line 584) - State management
- ⚠️ Copy output (line 602) - Clipboard API
- ⚠️ Cancel execution (line 413) - Modal interaction

**SQL IDE** (5 tests - similar)

**Full Workflow** (17 tests):
- Requires full integration testing
- File loading workflows
- Lifecycle transitions
- Export workflows

**Estimated Impact**: +27 real tests (challenging)

---

## Implementation Strategy

### Phase 1: Quick Wins (Week 1)
**Goal**: Convert Tier 1 tests (12 tests)

1. **Python IDE - Script Execution** (Day 1-2)
   - Add mock for `execute_python` command
   - Test simple execution
   - Test with dataset
   - Test clear output

2. **Python IDE - Error Handling** (Day 2)
   - Mock error responses
   - Verify error display
   - Test formatting

3. **SQL IDE - Query Execution** (Day 3-4)
   - Mirror Python IDE approach
   - Add SQL-specific mocks
   - Test query execution

4. **Verify & Document** (Day 5)
   - Run all converted tests
   - Update documentation
   - Celebrate wins! 🎉

---

### Phase 2: Medium Effort (Week 2)
**Goal**: Convert Tier 2 tests (14 tests)

1. **Lifecycle Integration** (Day 1-2)
   - Both Python and SQL IDE
   - Mock datasets with versions
   - Test stage switching

2. **Security Warnings** (Day 3)
   - Add modal test IDs if needed
   - Test warning flow
   - Test accept/cancel

3. **File Management** (Day 4)
   - Mock file dialogs
   - Test save/load
   - Verify content persistence

4. **Verify & Document** (Day 5)

---

### Phase 3: Challenging (Week 3-4)
**Goal**: Convert Tier 3 tests (27 tests)

1. **Monaco Editor Features** (Day 1-2)
   - Research Monaco E2E testing
   - Implement or document limitations
   - May need to mark as `.skip()` with notes

2. **Clipboard & State** (Day 3-4)
   - Handle clipboard API in tests
   - Test auto-save behavior
   - Test modal interactions

3. **Full Workflow Tests** (Day 5-10)
   - Integration testing
   - File loading → Analysis → Export
   - End-to-end user journeys

---

## Success Metrics

| Metric | Current | After Phase 1 | After Phase 2 | After Phase 3 |
|--------|---------|---------------|---------------|---------------|
| Skeleton Tests | 60 | 48 | 34 | 7 |
| Real Implementation Tests | ~108 | ~120 | ~134 | ~161 |
| Test Quality Score | 50% | 60% | 67% | 88% |
| Estimated Effort | - | 16-24 hrs | 24-32 hrs | 40-60 hrs |

---

## Test Template

### Skeleton Test (Before)
```typescript
test('should execute Python script', async ({ page }) => {
  await gotoApp(page);
  await page.getByTestId('nav-python').click();

  // TODO: Once execution is connected:
  // 1. Type script in editor
  // 2. Click run button
  // 3. Verify output appears

  await expect(page).toHaveTitle(/beefcake/i);
});
```

### Real Implementation Test (After)
```typescript
test('should execute Python script', async ({ page }) => {
  await setupTauriMock(page, {
    commands: getStandardMocks({
      execute_python: {
        type: 'success',
        data: {
          stdout: 'Hello World\n',
          stderr: '',
          exit_code: 0,
          execution_time: 0.15,
        },
      },
    }),
  });

  await gotoApp(page);
  await page.getByTestId('nav-python').click();

  // Type script (simulated - Monaco may not be editable)
  // Click run button
  await page.getByTestId('python-ide-run-button').click();

  // Verify output appears
  await expect(page.getByTestId('python-ide-output')).toContainText('Hello World');
  await expect(page.getByTestId('python-execution-time')).toContainText('0.15s');
});
```

---

## Next Steps

1. **Start with Tier 1** - Highest ROI
2. **Pick one test file** - python-ide.spec.ts
3. **Convert 2-3 tests** - Validate approach
4. **Run tests** - Ensure they pass
5. **Iterate** - Continue with remaining Tier 1 tests

---

## Related Documentation

- **Test Rollout Status**: `docs/TEST_ROLLOUT_STATUS.md`
- **Test ID Reference**: `docs/TEST_ID_REFERENCE.md`
- **E2E Test Guide**: `e2e/README.md`
- **Common Mocks**: `e2e/helpers/common-mocks.ts`

---

**Document Owner**: Development Team
**Last Updated**: 2026-01-27
