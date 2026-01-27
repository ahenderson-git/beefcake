# Test ID Reference

**Last Updated**: 2026-01-27
**Purpose**: Complete reference of all `data-testid` attributes in the application

---

## Usage

All test IDs follow the pattern: `{component}-{element}-{type}` (e.g., `analyser-file-name`)

Use in Playwright tests:
```typescript
await page.getByTestId('analyser-view').click();
await expect(page.getByTestId('analyser-file-name')).toContainText('data.csv');
```

---

## Navigation / Sidebar

**File**: `src-frontend/renderers/layout.ts`

| Test ID | Element | Description |
|---------|---------|-------------|
| `nav-dashboard` | Button | Dashboard navigation |
| `nav-analyser` | Button | Analyser navigation |
| `nav-lifecycle` | Button | Lifecycle navigation |
| `nav-pipeline` | Button | Pipeline navigation |
| `nav-watcher` | Button | Watcher navigation |
| `nav-dictionary` | Button | Dictionary navigation |
| `nav-integrity` | Button | Integrity navigation |
| `nav-powershell` | Button | PowerShell IDE navigation |
| `nav-python` | Button | Python IDE navigation |
| `nav-sql` | Button | SQL IDE navigation |
| `nav-settings` | Button | Settings navigation |
| `nav-activity-log` | Button | Activity Log navigation |
| `nav-cli` | Button | CLI Help navigation |
| `nav-reference` | Button | Reference navigation |

---

## Dashboard

**File**: `src-frontend/renderers/dashboard.ts`

| Test ID | Element | Description |
|---------|---------|-------------|
| `dashboard-view` | Container | Main dashboard container |
| `dashboard-open-file-button` | Button | Open file button |
| `dashboard-powershell-button` | Button | PowerShell IDE shortcut |
| `dashboard-python-button` | Button | Python IDE shortcut |
| `dashboard-sql-button` | Button | SQL IDE shortcut |

---

## Analyser Component

**File**: `src-frontend/renderers/analyser/*.ts`

### Main Containers

| Test ID | Element | Description |
|---------|---------|-------------|
| `analyser-view` | Container | Main analyser container |
| `analyser-header` | Container | Header section |
| `analyser-metadata` | Container | Metadata container |
| `analyser-metrics-dashboard` | Container | Metrics overview dashboard |

### File Information

| Test ID | Element | Description |
|---------|---------|-------------|
| `analyser-file-name` | Text | File name display |
| `analyser-file-size` | Text | File size (e.g., "2.5 MB") |
| `analyser-row-count` | Text | Row count display |
| `analyser-column-count` | Text | Column count display |
| `analyser-analysis-duration` | Text | Analysis time (e.g., "0.5s") |

### Quality Metrics

| Test ID | Element | Description |
|---------|---------|-------------|
| `analyser-quality-score-card` | Container | Quality score card |
| `analyser-quality-score` | Text | Quality score percentage |
| `analyser-type-distribution-card` | Container | Type distribution card |

### Column Table

| Test ID | Element | Description |
|---------|---------|-------------|
| `analyser-column-row` | Container | Each column row (repeats for each column) |
| `analyser-column-checkbox` | Checkbox | Column selection checkbox |
| `analyser-column-expander` | Button | Expand/collapse button |
| `analyser-column-name` | Text | Column name display |
| `analyser-column-type` | Badge | Column type (Numeric, Text, etc.) |
| `analyser-column-quality` | ProgressBar | Quality bar indicator |
| `analyser-column-stats` | Container | Stats summary container |
| `analyser-column-null-pct` | Text | Null percentage pill |
| `analyser-column-unique-pct` | Text | Unique percentage pill |

### Actions

| Test ID | Element | Description |
|---------|---------|-------------|
| `analyser-open-file-button` | Button | Open/select file button |
| `analyser-export-button` | Button | Export button |

---

## Lifecycle Component

**File**: `src-frontend/renderers/lifecycle.ts`

### Lifecycle Rail

| Test ID | Element | Description |
|---------|---------|-------------|
| `lifecycle-rail` | Container | Main lifecycle rail container |
| `lifecycle-stages` | Container | Stages container |
| `lifecycle-stage-raw` | Container | Raw stage indicator |
| `lifecycle-stage-profiled` | Container | Profiled stage indicator |
| `lifecycle-stage-cleaned` | Container | Cleaned stage indicator |
| `lifecycle-stage-advanced` | Container | Advanced stage indicator |
| `lifecycle-stage-validated` | Container | Validated stage indicator |
| `lifecycle-stage-published` | Container | Published stage indicator |

**Note**: Stage indicators have classes `stage-completed`, `stage-active`, `stage-locked` to indicate status.

---

## Export Modal

**File**: `src-frontend/renderers/export.ts`

### Modal Structure

| Test ID | Element | Description |
|---------|---------|-------------|
| `export-modal-overlay` | Container | Modal overlay |
| `export-modal` | Container | Modal container |
| `export-modal-close` | Button | Close button (X) |
| `export-modal-body` | Container | Modal body |

### Destination Selection

| Test ID | Element | Description |
|---------|---------|-------------|
| `export-dest-file` | Button | File destination toggle |
| `export-dest-database` | Button | Database destination toggle |

### File Export

| Test ID | Element | Description |
|---------|---------|-------------|
| `export-file-path-input` | Input | File path input field ✅ NEW |
| `export-file-browse-button` | Button | Browse for file button ✅ NEW |

### Database Export

| Test ID | Element | Description |
|---------|---------|-------------|
| `export-db-connection-select` | Select | Database connection dropdown ✅ NEW |

### Actions

| Test ID | Element | Description |
|---------|---------|-------------|
| `export-cancel-button` | Button | Cancel button |
| `export-confirm-button` | Button | Confirm/Start Export button |
| `export-abort-button` | Button | Abort export button (appears during export) |

---

## Python IDE

**File**: `src-frontend/renderers/ide.ts`

| Test ID | Element | Description |
|---------|---------|-------------|
| `python-ide-view` | Container | Main Python IDE container |
| `python-ide-run-button` | Button | Run script button |
| `python-ide-output` | Container | Output panel |
| `python-ide-clear-button` | Button | Clear output button |
| `python-ide-save-button` | Button | Save script button |
| `python-ide-load-button` | Button | Load script button |
| `python-ide-copy-output-button` | Button | Copy output button |
| `python-ide-font-increase` | Button | Increase font size |
| `python-ide-font-decrease` | Button | Decrease font size |

---

## SQL IDE

**File**: `src-frontend/renderers/ide.ts`

| Test ID | Element | Description |
|---------|---------|-------------|
| `sql-ide-view` | Container | Main SQL IDE container |
| `sql-ide-run-button` | Button | Run query button |
| `sql-ide-output` | Container | Output panel |
| `sql-ide-clear-button` | Button | Clear output button |
| `sql-ide-save-button` | Button | Save query button |
| `sql-ide-load-button` | Button | Load query button |
| `sql-ide-copy-output-button` | Button | Copy output button |
| `sql-ide-font-increase` | Button | Increase font size |
| `sql-ide-font-decrease` | Button | Decrease font size |
| `sql-ide-skip-cleaning-checkbox` | Checkbox | Skip cleaning checkbox |
| `sql-ide-install-polars-button` | Button | Install Polars button |

---

## PowerShell Console

**File**: `src-frontend/renderers/ide.ts`

| Test ID | Element | Description |
|---------|---------|-------------|
| `powershell-console-view` | Container | Main PowerShell console container |
| `powershell-run-button` | Button | Run script button |
| `powershell-output` | Container | Output panel |
| `powershell-clear-button` | Button | Clear output button |
| `powershell-save-button` | Button | Save script button |
| `powershell-load-button` | Button | Load script button |
| `powershell-copy-output-button` | Button | Copy output button |
| `powershell-font-increase` | Button | Increase font size |
| `powershell-font-decrease` | Button | Decrease font size |

---

## Settings

**File**: `src-frontend/renderers/settings.ts`

**Note**: Settings test IDs are not yet fully implemented. Expected IDs based on tests:

| Test ID | Element | Description | Status |
|---------|---------|-------------|--------|
| `settings-add-connection-button` | Button | Add connection button | ⚠️ To be added (P1) |
| `settings-connection-name-input` | Input | Connection name field | ⚠️ To be added (P1) |
| `settings-connection-type-select` | Select | Connection type dropdown | ⚠️ To be added (P1) |
| `settings-trusted-paths-section` | Container | Trusted paths section | ⚠️ To be added (P1) |
| `settings-ai-enabled-toggle` | Checkbox | AI enabled toggle | ⚠️ To be added (P1) |
| `settings-font-size-section` | Container | Font size settings | ⚠️ To be added (P1) |
| `settings-theme-select` | Select | Theme selector | ⚠️ To be added (P1) |

---

## Loading States

**File**: `src-frontend/renderers/common.ts`

| Test ID | Element | Description |
|---------|---------|-------------|
| `loading-overlay` | Container | Loading overlay |
| `loading-message` | Text | Loading message text |
| `btn-abort-op` | Button | Abort operation button (ID, not test ID) |

---

## Toast Notifications

**File**: `src-frontend/renderers/common.ts`

| Test ID | Element | Description |
|---------|---------|-------------|
| `toast-container` | Container | Toast container (ID, not test ID) |
| `toast-{type}` | Container | Individual toast (success, error, info, warning) |

**Note**: Toast test IDs may need to be added for better E2E testing.

---

## Test ID Naming Conventions

### Pattern

```
{component}-{element}-{type}
```

### Examples

- `analyser-file-name` - Component: analyser, Element: file name
- `export-modal-close` - Component: export modal, Element: close button
- `lifecycle-stage-raw` - Component: lifecycle, Element: raw stage
- `python-ide-run-button` - Component: python ide, Element: run button

### Types

- No suffix: Text/Display elements
- `-button`: Clickable buttons
- `-input`: Text input fields
- `-select`: Dropdown/select fields
- `-checkbox`: Checkboxes
- `-view`: Main container/view
- `-modal`: Modal containers
- `-card`: Card containers

---

## Finding Test IDs in Code

### Search by Component

```bash
# Find all test IDs in analyser
grep -r "data-testid.*analyser" src-frontend/renderers/

# Find all button test IDs
grep -r "data-testid.*button" src-frontend/
```

### Add New Test IDs

1. Add to renderer file (`src-frontend/renderers/*.ts`)
2. Use kebab-case naming
3. Follow naming convention: `{component}-{element}-{type}`
4. Update this document
5. Test with E2E tests

---

## Related Documentation

- **E2E Test Guide**: `e2e/README.md`
- **Testing Guide**: `docs/TESTING.md`
- **Test Implementation Status**: `docs/TESTING_IMPLEMENTATION_STATUS.md`
- **Missing Test IDs Analysis**: `docs/MISSING_TEST_IDS.md`
- **Test Rollout Status**: `docs/TEST_ROLLOUT_STATUS.md`
- **Test ID Implementation Summary**: `docs/TEST_ID_IMPLEMENTATION_SUMMARY.md`

---

## Changelog

### 2026-01-27
- ✅ Added 3 export modal test IDs
  - `export-file-path-input`
  - `export-file-browse-button`
  - `export-db-connection-select`
- ✅ Documented all existing test IDs across components
- ✅ Verified Analyser and Lifecycle test IDs are complete

### Previous
- Initial test ID implementation across all components
- Comprehensive E2E test infrastructure setup

---

**Document Owner**: Development Team
**Maintenance**: Update when adding new test IDs
