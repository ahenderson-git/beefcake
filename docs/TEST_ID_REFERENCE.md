# Test ID Reference

This document lists all `data-testid` attributes used in the Beefcake application for E2E testing with Playwright.

**Last Updated**: 2026-01-18

## Dashboard

| Test ID | Element | Location | Purpose |
|---------|---------|----------|---------|
| `dashboard-view` | Main dashboard container | `src-frontend/renderers/dashboard.ts:6` | Verify dashboard view is visible |
| `dashboard-open-file-button` | "Analyze New Dataset" button | `src-frontend/renderers/dashboard.ts:64` | Trigger file dialog for analysis |
| `dashboard-powershell-button` | "PowerShell Console" button | `src-frontend/renderers/dashboard.ts:67` | Navigate to PowerShell view |
| `dashboard-python-button` | "Python IDE" button | `src-frontend/renderers/dashboard.ts:70` | Navigate to Python view |
| `dashboard-sql-button` | "SQL Lab" button | `src-frontend/renderers/dashboard.ts:73` | Navigate to SQL view |

## Analyser

### Main Container
| Test ID | Element | Location | Purpose |
|---------|---------|----------|---------|
| `analyser-view` | Main analyser wrapper | `src-frontend/renderers/analyser.ts:523` | Verify analyser view is visible |
| `analyser-empty-state` | Empty state container | `src-frontend/renderers/analyser.ts:583` | Verify no data state |
| `empty-open-file-button` | Open file from empty state | `src-frontend/renderers/analyser.ts:587` | Trigger file dialog from empty state |

### Health & Summary
| Test ID | Element | Location | Purpose |
|---------|---------|----------|---------|
| `health-score-banner` | Health score container | `src-frontend/renderers/analyser.ts:528` | Verify health banner visibility |
| `health-score-badge` | Health score display wrapper | `src-frontend/renderers/analyser.ts:529` | Access health score element |
| `health-score-value` | Health percentage value | `src-frontend/renderers/analyser.ts:531` | Verify health score percentage |
| `analyser-row-count` | Dataset row count | `src-frontend/renderers/analyser.ts:94` | Verify row count display |
| `analyser-column-count` | Dataset column count | `src-frontend/renderers/analyser.ts:95` | Verify column count display |

### Actions
| Test ID | Element | Location | Purpose |
|---------|---------|----------|---------|
| `analyser-open-file-button` | "Select File" button | `src-frontend/renderers/analyser.ts:100` | Open file dialog from analyser |
| `analyser-reanalyze-button` | "Re-analyze" button | `src-frontend/renderers/analyser.ts:103` | Re-run analysis on current file |
| `btn-begin-cleaning` | "Begin Cleaning" button | `src-frontend/renderers/analyser.ts:107` | Transition to Cleaned stage |
| `btn-continue-advanced` | "Continue to Advanced" button | `src-frontend/renderers/analyser.ts:113` | Transition to Advanced stage |
| `btn-move-to-validated` | "Move to Validated" button | `src-frontend/renderers/analyser.ts:119` | Transition to Validated stage |
| `btn-export-analyser` | "Export / ETL" button | `src-frontend/renderers/analyser.ts:124` | Open export modal |

### Bulk Actions
| Test ID | Element | Location | Purpose |
|---------|---------|----------|---------|
| `header-active-all` | "Clean All" checkbox | `src-frontend/renderers/analyser.ts:139` | Toggle cleaning for all columns |
| `header-use-original-names` | "Original Names" checkbox | `src-frontend/renderers/analyser.ts:142` | Toggle between original and renamed columns |

### Column Rows (Dynamic)
| Test ID Pattern | Element | Location | Purpose |
|-----------------|---------|----------|---------|
| `analyser-row-{colName}` | Column row (clickable) | `src-frontend/renderers/analyser.ts:820` | Click to expand column details |
| `col-select-checkbox-{colName}` | Column selection checkbox | `src-frontend/renderers/analyser.ts:827` | Select/deselect column for operations |
| `clean-active-{colName}` | "Enable Cleaning" checkbox | `src-frontend/renderers/analyser.ts:903` | Enable/disable cleaning for specific column |

**Example Usage**:
```typescript
// For a column named "age"
await page.getByTestId('analyser-row-age').click();
await page.getByTestId('clean-active-age').check();
```

## Lifecycle

| Test ID | Element | Location | Purpose |
|---------|---------|----------|---------|
| `lifecycle-rail` | Lifecycle rail container | `src-frontend/renderers/lifecycle.ts:161` | Verify lifecycle rail is visible |
| `lifecycle-stages` | Stages container | `src-frontend/renderers/lifecycle.ts:166` | Access all lifecycle stages |
| `lifecycle-stage-raw` | Raw stage indicator | `src-frontend/renderers/lifecycle.ts:146` | Click or verify Raw stage |
| `lifecycle-stage-profiled` | Profiled stage indicator | `src-frontend/renderers/lifecycle.ts:146` | Click or verify Profiled stage |
| `lifecycle-stage-cleaned` | Cleaned stage indicator | `src-frontend/renderers/lifecycle.ts:146` | Click or verify Cleaned stage |
| `lifecycle-stage-advanced` | Advanced stage indicator | `src-frontend/renderers/lifecycle.ts:146` | Click or verify Advanced stage |
| `lifecycle-stage-validated` | Validated stage indicator | `src-frontend/renderers/lifecycle.ts:146` | Click or verify Validated stage |
| `lifecycle-stage-published` | Published stage indicator | `src-frontend/renderers/lifecycle.ts:146` | Click or verify Published stage |

### Publish Modal
| Test ID | Element | Location | Purpose |
|---------|---------|----------|---------|
| `publish-modal-overlay` | Modal overlay | `src-frontend/renderers/lifecycle.ts:235` | Verify modal is open |
| `publish-modal` | Publish modal content | `src-frontend/renderers/lifecycle.ts:236` | Access modal content |
| `publish-modal-close` | Close button | `src-frontend/renderers/lifecycle.ts:239` | Close modal |
| `btn-publish-view` | "Publish as View" button | `src-frontend/renderers/lifecycle.ts:262` | Publish as logical view |
| `btn-publish-snapshot` | "Publish as Snapshot" button | `src-frontend/renderers/lifecycle.ts:277` | Publish as materialized snapshot |

## Export Modal

| Test ID | Element | Location | Purpose |
|---------|---------|----------|---------|
| `export-modal-overlay` | Modal overlay | `src-frontend/renderers/export.ts:55` | Verify export modal is open |
| `export-modal` | Export modal content | `src-frontend/renderers/export.ts:56` | Access modal content |
| `export-modal-close` | Close button | `src-frontend/renderers/export.ts:59` | Close modal |
| `export-modal-body` | Modal body container | `src-frontend/renderers/export.ts:61` | Access modal body |
| `export-dest-file` | "Local File" toggle button | `src-frontend/renderers/export.ts:65` | Select file export destination |
| `export-dest-database` | "Database" toggle button | `src-frontend/renderers/export.ts:68` | Select database export destination |
| `export-cancel-button` | "Cancel" button | `src-frontend/renderers/export.ts:87` | Cancel export operation |
| `export-confirm-button` | "Start Export" button | `src-frontend/renderers/export.ts:94` | Begin export operation |
| `export-abort-button` | "Abort" button (during export) | `src-frontend/renderers/export.ts:92` | Abort running export |

## Global UI

### Loading States
| Test ID | Element | Location | Purpose |
|---------|---------|----------|---------|
| `loading-overlay` | Loading overlay container | `src-frontend/renderers/common.ts:61` | Verify loading state is visible |
| `loading-spinner` | Loading spinner animation | `src-frontend/renderers/common.ts:62` | Verify spinner is animating |
| `loading-message` | Loading status message | `src-frontend/renderers/common.ts:63` | Read loading message text |
| `btn-abort-op` | "Abort" button | `src-frontend/renderers/common.ts:65` | Abort long-running operation |

### Toast Notifications
| Test ID | Element | Location | Purpose |
|---------|---------|----------|---------|
| `toast-success` | Success toast | `src-frontend/renderers/common.ts:75` | Verify success notification |
| `toast-error` | Error toast | `src-frontend/renderers/common.ts:75` | Verify error notification |
| `toast-info` | Info toast | `src-frontend/renderers/common.ts:75` | Verify info notification |

## AI Assistant (Bonus)

| Test ID | Element | Location | Purpose |
|---------|---------|----------|---------|
| `ai-assistant` | AI sidebar container | `src-frontend/components/AIAssistantComponent.ts:47` | Verify AI assistant is visible |
| `ai-status` | AI status indicator | `src-frontend/components/AIAssistantComponent.ts:54` | Verify AI connection status |
| `ai-messages` | Messages container | `src-frontend/components/AIAssistantComponent.ts:63` | Access AI message history |
| `ai-input-area` | Input area container | `src-frontend/components/AIAssistantComponent.ts:76` | Access input area |
| `ai-input` | Message input textarea | `src-frontend/components/AIAssistantComponent.ts:79` | Type AI messages |
| `ai-send-button` | Send message button | `src-frontend/components/AIAssistantComponent.ts:87` | Send message to AI |
| `ai-clear-button` | Clear conversation button | `src-frontend/components/AIAssistantComponent.ts:95` | Clear AI conversation |
| `ai-message-user` | User message | `src-frontend/components/AIAssistantComponent.ts:230` | Access user messages |
| `ai-message-assistant` | Assistant message | `src-frontend/components/AIAssistantComponent.ts:230` | Access assistant messages |

## Usage in E2E Tests

### Basic Element Selection
```typescript
// By test ID
await page.getByTestId('dashboard-open-file-button').click();

// Wait for visibility
await expect(page.getByTestId('analyser-view')).toBeVisible();

// Check if enabled
await expect(page.getByTestId('btn-begin-cleaning')).toBeEnabled();
```

### Dynamic Test IDs
```typescript
// For columns with dynamic names
const columnName = 'age';
await page.getByTestId(`analyser-row-${columnName}`).click();
await page.getByTestId(`clean-active-${columnName}`).check();
```

### Toast Notifications
```typescript
// Wait for success toast
await expect(page.getByTestId('toast-success')).toBeVisible();
await expect(page.getByTestId('toast-success')).toContainText('Export successful');

// Wait for error toast
await expect(page.getByTestId('toast-error')).toBeVisible();
```

### Modal Interactions
```typescript
// Open and interact with export modal
await page.getByTestId('btn-export-analyser').click();
await expect(page.getByTestId('export-modal')).toBeVisible();
await page.getByTestId('export-dest-file').click();
await page.getByTestId('export-confirm-button').click();
```

### Loading States
```typescript
// Verify loading indicator appears
await expect(page.getByTestId('loading-spinner')).toBeVisible();
await expect(page.getByTestId('loading-message')).toContainText('Analyzing');

// Abort operation
await page.getByTestId('btn-abort-op').click();

// Wait for loading to complete
await expect(page.getByTestId('loading-spinner')).toBeHidden();
```

## Naming Conventions

Test IDs follow this pattern: `{component}-{element}-{type}`

- **Component prefix**: Lowercase component name (e.g., `dashboard`, `analyser`, `export`)
- **Element name**: Descriptive element identifier (e.g., `open-file`, `row-count`, `modal`)
- **Type suffix**: Element type when applicable (e.g., `button`, `checkbox`, `modal`, `overlay`)

### Special Cases

- **Dynamic IDs**: Use template literals with variable names (e.g., `analyser-row-{colName}`)
- **State-based IDs**: Include state in name (e.g., `toast-success`, `toast-error`, `toast-info`)
- **Stage indicators**: Include stage name (e.g., `lifecycle-stage-cleaned`)

## Test ID Coverage

**Summary**: 50+ unique test IDs defined across 6 renderer files

- ✅ Dashboard: 5 test IDs
- ✅ Analyser: 20+ test IDs (+ dynamic column IDs)
- ✅ Lifecycle: 10 test IDs
- ✅ Export Modal: 9 test IDs
- ✅ Global UI: 7 test IDs
- ✅ AI Assistant: 9 test IDs

**Status**: Ready for comprehensive E2E testing implementation
