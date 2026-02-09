# Missing Test IDs Analysis

**Date**: 2026-01-27
**Purpose**: Track which `data-testid` attributes need to be added to unblock 24 P0 tests

---

## Summary

Based on analysis of E2E test files and renderer code:

| Component | Test IDs Present | Test IDs Missing | Status |
|-----------|------------------|------------------|--------|
| AnalyserComponent | 15+ | 0 | ✅ Complete |
| Export Modal | 0 | ~10 | ❌ Missing |
| LifecycleComponent | 0 | ~5 | ❌ Missing |
| Navigation | Partial | ~3 | ⚠️ Partial |

---

## Analyser Component ✅ COMPLETE

### Already Present Test IDs

From `src-frontend/renderers/analyser/*.ts`:

**Container/Views**:
- ✅ `data-testid="analyser-view"` - Main container
- ✅ `data-testid="analyser-header"` - Header section
- ✅ `data-testid="analyser-metrics-dashboard"` - Metrics overview

**File Metadata**:
- ✅ `data-testid="analyser-file-name"` - File name display
- ✅ `data-testid="analyser-file-size"` - File size display
- ✅ `data-testid="analyser-metadata"` - Metadata container
- ✅ `data-testid="analyser-row-count"` - Row count
- ✅ `data-testid="analyser-column-count"` - Column count
- ✅ `data-testid="analyser-analysis-duration"` - Analysis duration

**Quality Score**:
- ✅ `data-testid="analyser-quality-score-card"` - Quality card
- ✅ `data-testid="analyser-quality-score"` - Quality score value
- ✅ `data-testid="analyser-type-distribution-card"` - Type distribution card

**Column Rows**:
- ✅ `data-testid="analyser-column-row"` - Each column row
- ✅ `data-testid="analyser-column-checkbox"` - Selection checkbox
- ✅ `data-testid="analyser-column-expander"` - Expand button
- ✅ `data-testid="analyser-column-name"` - Column name
- ✅ `data-testid="analyser-column-type"` - Column type badge
- ✅ `data-testid="analyser-column-quality"` - Quality bar
- ✅ `data-testid="analyser-column-stats"` - Stats summary
- ✅ `data-testid="analyser-column-null-pct"` - Null percentage
- ✅ `data-testid="analyser-column-unique-pct"` - Unique percentage

**Buttons**:
- ✅ `data-testid="analyser-open-file-button"` - Open file button
- ✅ `data-testid="analyser-export-button"` - Export button

### Result
**No changes needed for AnalyserComponent** - All test IDs are present!

---

## Export Modal/Component ❌ NEEDS IMPLEMENTATION

### Current Status
- No test IDs found in export-related code
- Navigation to Export view causes page crash (noted in test skip comments)

### Required Test IDs

Based on `e2e/full-workflow.spec.ts` (lines 293-315):

**Modal/View**:
- ❌ `data-testid="export-view"` - Main export container/modal
- ❌ `data-testid="export-modal"` - Modal wrapper (if different from view)

**Destination Selection**:
- ❌ `data-testid="export-destination-select"` - Destination dropdown/radio
- ❌ `data-testid="export-destination-file"` - File export option
- ❌ `data-testid="export-destination-database"` - Database export option

**File Export**:
- ❌ `data-testid="export-file-format-select"` - Format selector (CSV, Parquet, JSON)
- ❌ `data-testid="export-file-path-input"` - File path input
- ❌ `data-testid="export-file-browse-button"` - Browse button

**Database Export**:
- ❌ `data-testid="export-db-connection-select"` - Connection dropdown
- ❌ `data-testid="export-db-table-input"` - Table name input

**Actions**:
- ❌ `data-testid="export-cancel-button"` - Cancel button
- ❌ `data-testid="export-confirm-button"` - Export/Confirm button

### Files to Modify
- `src-frontend/components/ExportModal.ts` (if exists)
- `src-frontend/renderers/export.ts`

---

## Lifecycle Component ❌ NEEDS IMPLEMENTATION

### Current Status
- Some navigation test IDs exist (from example.spec.ts)
- Missing stage-specific test IDs

### Required Test IDs

Based on `e2e/full-workflow.spec.ts` (lines 260-280) and `e2e/lifecycle-management.spec.ts`:

**Rail/Container**:
- ❌ `data-testid="lifecycle-rail"` - Main lifecycle rail container
- ❌ `data-testid="lifecycle-stage-indicator"` - Stage indicators container

**Stage Indicators** (dynamic):
- ❌ `data-testid="lifecycle-stage-Raw"` - Raw stage indicator
- ❌ `data-testid="lifecycle-stage-Profiled"` - Profiled stage indicator
- ❌ `data-testid="lifecycle-stage-Cleaned"` - Cleaned stage indicator
- ❌ `data-testid="lifecycle-stage-Advanced"` - Advanced stage indicator
- ❌ `data-testid="lifecycle-stage-Validated"` - Validated stage indicator
- ❌ `data-testid="lifecycle-stage-Published"` - Published stage indicator

**Actions**:
- ❌ `data-testid="btn-view-diff"` - View differences button
- ❌ `data-testid="diff-modal"` - Diff modal container

### Files to Modify
- `src-frontend/components/LifecycleComponent.ts`
- `src-frontend/renderers/lifecycle.ts`

---

## Navigation (Sidebar) ⚠️ PARTIALLY COMPLETE

### Already Present (from example.spec.ts tests passing)
- ✅ `data-testid="nav-dashboard"` - Dashboard nav button
- ✅ `data-testid="nav-analyser"` - Analyser nav button
- ✅ `data-testid="nav-lifecycle"` - Lifecycle nav button
- ✅ `data-testid="nav-pipeline"` - Pipeline nav button
- ✅ `data-testid="nav-python"` - Python nav button
- ✅ `data-testid="nav-sql"` - SQL nav button
- ✅ `data-testid="nav-powershell"` - PowerShell nav button
- ✅ `data-testid="nav-settings"` - Settings nav button
- ✅ `data-testid="nav-watcher"` - Watcher nav button
- ✅ `data-testid="nav-dictionary"` - Dictionary nav button
- ✅ `data-testid="nav-activity-log"` - Activity log nav button
- ✅ `data-testid="nav-reference"` - Reference nav button
- ✅ `data-testid="nav-cli"` - CLI nav button

### Missing (from skipped tests)
- ❌ `data-testid="nav-integrity"` - Integrity nav button
- ❌ `data-testid="nav-onboarding"` - Onboarding nav button
- ❌ `data-testid="nav-ai-assistant"` - AI Assistant nav button (causes timeout)
- ❌ `data-testid="nav-export"` - Export nav button (causes crash)

### Files to Modify
- `src-frontend/renderers/layout.ts` (sidebar renderer)

---

## Settings Component (P1 Priority)

### Required Test IDs

Based on `e2e/example.spec.ts` (lines 442-515):

**Connections**:
- ❌ `data-testid="settings-add-connection-button"`
- ❌ `data-testid="settings-connection-name-input"`
- ❌ `data-testid="settings-connection-type-select"`
- ❌ `data-testid="settings-connection-host-input"`

**Security**:
- ❌ `data-testid="settings-trusted-paths-section"`
- ❌ `data-testid="settings-add-trusted-path-button"`

**AI Configuration**:
- ❌ `data-testid="settings-ai-enabled-toggle"`

**Preferences**:
- ❌ `data-testid="settings-font-size-section"`
- ❌ `data-testid="settings-theme-select"`

### Files to Modify
- `src-frontend/components/SettingsComponent.ts`
- `src-frontend/renderers/settings.ts`

---

## Implementation Priority

### P0 (Blocks 24 tests) - DO FIRST
1. **Export Modal** - 10 test IDs - Blocks 3 critical workflow tests
2. **Lifecycle Component** - 5 test IDs - Blocks 3 workflow + 20 lifecycle tests
3. **Navigation (Export/Integrity)** - 2 test IDs - Blocks 2 navigation tests

**Total**: ~17 test IDs to unblock 24 P0 tests

### P1 (Blocks 9 tests) - DO SECOND
1. **Settings Component** - 9 test IDs - Blocks 6 settings tests
2. **Navigation (Onboarding/AI)** - 2 test IDs - Blocks 2 navigation tests + 1 Settings test

**Total**: ~11 test IDs to unblock 9 P1 tests

---

## Action Plan

### Step 1: Export Modal (2-3 hours)
1. Check if `ExportModal.ts` exists or if export is in another component
2. Add test IDs to renderer (`src-frontend/renderers/export.ts`)
3. Verify with: `npm run test:e2e -- e2e/full-workflow.spec.ts`

### Step 2: Lifecycle Component (2-3 hours)
1. Add test IDs to `src-frontend/renderers/lifecycle.ts`
2. Add test IDs to stage indicators (dynamic based on stage name)
3. Verify with: `npm run test:e2e -- e2e/lifecycle-management.spec.ts`

### Step 3: Navigation (1 hour)
1. Add missing nav button test IDs in `src-frontend/renderers/layout.ts`
2. Investigate Export navigation crash
3. Investigate AI Assistant timeout
4. Verify with: `npm run test:e2e -- e2e/example.spec.ts`

### Step 4: Settings Component (P1 - 2-3 hours)
1. Add test IDs to `src-frontend/renderers/settings.ts`
2. Verify with: `npm run test:e2e -- e2e/example.spec.ts`

---

## Verification Checklist

After adding test IDs, verify:

- [ ] Tests can find elements (no "element not found" errors)
- [ ] Tests execute without timeouts
- [ ] Test IDs don't interfere with styling
- [ ] Test IDs are documented in TEST_ID_REFERENCE.md
- [ ] All 34 skipped tests reviewed for readiness

---

**Document Owner**: Development Team
**Last Updated**: 2026-01-27
