# Adding data-testid Attributes for E2E Testing

## Why data-testid?

E2E tests need stable selectors to find UI elements. CSS classes and IDs can change during refactoring, but `data-testid` attributes are specifically for testing and won't be accidentally modified.

## Naming Convention

Format: `{component}-{element}-{type}`

Examples:
- `data-testid="analyser-run-button"`
- `data-testid="pipeline-editor-canvas"`
- `data-testid="lifecycle-stage-cleaned"`
- `data-testid="toast-success"`
- `data-testid="clean-trim-whitespace-age"`

### Guidelines

1. **Component prefix**: Start with component name (lowercase)
2. **Element name**: Describe what the element is
3. **Type suffix**: Button, input, panel, modal, etc.
4. **Dynamic elements**: Include identifier (e.g., column name, ID)

## Components That Need data-testid

### Priority Order (P0 first)

1. **DashboardComponent** - 4 test IDs
2. **AnalyserComponent** - 50+ test IDs (many dynamic per column)
3. **LifecycleComponent** - 15+ test IDs
4. **PipelineComponent** - 30+ test IDs
5. **WatcherComponent** - 12 test IDs
6. **ExportModal** - 10 test IDs
7. **PowerShellComponent** - 6 test IDs
8. **PythonComponent** - 8 test IDs
9. **SQLComponent** - 6 test IDs
10. **DictionaryComponent** - 12 test IDs
11. **SettingsComponent** - 15 test IDs
12. **Global UI** - 8 test IDs (toasts, loading, etc.)

---

## Implementation Examples

### Example 1: Dashboard Component

**Before:**
```typescript
container.innerHTML = `
  <div class="dashboard-view">
    <button id="btn-open-file" class="btn-primary">
      <i class="ph ph-folder-open"></i> Open File
    </button>
    <button id="btn-powershell" class="btn-secondary">
      <i class="ph ph-terminal"></i> PowerShell
    </button>
  </div>
`;
```

**After:**
```typescript
container.innerHTML = `
  <div class="dashboard-view">
    <button id="btn-open-file" data-testid="dashboard-open-file-button" class="btn-primary">
      <i class="ph ph-folder-open"></i> Open File
    </button>
    <button id="btn-powershell" data-testid="dashboard-powershell-button" class="btn-secondary">
      <i class="ph ph-terminal"></i> PowerShell
    </button>
  </div>
`;
```

### Example 2: Analyser Component (Dynamic IDs)

**Before:**
```typescript
html += `
  <div class="analyser-row" data-col="${colName}">
    <input type="checkbox" class="col-select-checkbox" />
    <span>${colName}</span>
    <input type="checkbox" class="row-action" data-prop="active" />
  </div>
`;
```

**After:**
```typescript
html += `
  <div class="analyser-row"
       data-col="${colName}"
       data-testid="analyser-row-${colName}">
    <input type="checkbox"
           class="col-select-checkbox"
           data-testid="col-select-checkbox-${colName}" />
    <span data-testid="col-name-${colName}">${colName}</span>
    <input type="checkbox"
           class="row-action"
           data-prop="active"
           data-testid="clean-active-${colName}" />
  </div>
`;
```

### Example 3: Pipeline Editor (Drag-and-Drop)

**Before:**
```typescript
function renderPipelineStep(step: any, index: number): string {
  return `
    <div class="pipeline-step" draggable="true">
      <h4>${step.op}</h4>
      <button class="delete-step">×</button>
    </div>
  `;
}
```

**After:**
```typescript
function renderPipelineStep(step: any, index: number): string {
  return `
    <div class="pipeline-step"
         draggable="true"
         data-testid="pipeline-step-${index}">
      <h4 data-testid="pipeline-step-name-${index}">${step.op}</h4>
      <button class="delete-step"
              data-testid="pipeline-step-delete-${index}">×</button>
    </div>
  `;
}
```

### Example 4: Toasts (Global UI)

**Before:**
```typescript
function showToast(message: string, type: 'success' | 'error' | 'info') {
  const toast = document.createElement('div');
  toast.className = `toast toast-${type}`;
  toast.textContent = message;
  document.body.appendChild(toast);
}
```

**After:**
```typescript
function showToast(message: string, type: 'success' | 'error' | 'info') {
  const toast = document.createElement('div');
  toast.className = `toast toast-${type}`;
  toast.setAttribute('data-testid', `toast-${type}`);
  toast.textContent = message;
  document.body.appendChild(toast);
}
```

### Example 5: Modal Components

**Before:**
```typescript
return `
  <div class="modal-overlay">
    <div class="modal-content">
      <div class="modal-header">
        <h3>Export Data</h3>
        <button class="modal-close">×</button>
      </div>
      <div class="modal-body">
        <button id="export-csv">Export CSV</button>
      </div>
    </div>
  </div>
`;
```

**After:**
```typescript
return `
  <div class="modal-overlay" data-testid="export-modal-overlay">
    <div class="modal-content" data-testid="export-modal">
      <div class="modal-header">
        <h3>Export Data</h3>
        <button class="modal-close" data-testid="export-modal-close">×</button>
      </div>
      <div class="modal-body" data-testid="export-modal-body">
        <button id="export-csv" data-testid="export-csv-button">Export CSV</button>
      </div>
    </div>
  </div>
`;
```

---

## Checklist for Each Component

When adding test IDs to a component:

- [ ] Identify all interactive elements (buttons, inputs, links)
- [ ] Identify all containers that E2E tests need to verify (panels, modals, tables)
- [ ] Add `data-testid` to each identified element
- [ ] Use consistent naming convention
- [ ] For dynamic lists, include identifier in test ID (e.g., row index, column name)
- [ ] Document test IDs in component comments
- [ ] Test that selectors work: `document.querySelector('[data-testid="your-id"]')`

## Priority Test IDs to Add First (P0 Workflows)

### 1. File Loading
- `dashboard-open-file-button`
- `analyser-open-file-button`
- `file-dropzone` (if drag-and-drop supported)

### 2. Analysis
- `analyser-run-button`
- `analysis-summary-panel`
- `dataset-preview-table`
- `analyser-row-{colName}` (for each column)

### 3. Cleaning Configuration
- `clean-active-{colName}`
- `clean-trim-whitespace-{colName}`
- `clean-impute-{colName}`
- `header-active-all`
- `header-impute-all`

### 4. Lifecycle Transitions
- `btn-begin-cleaning`
- `btn-continue-advanced`
- `btn-move-to-validated`
- `lifecycle-stage-{stage}` (for rail navigation)

### 5. Export
- `btn-export`
- `btn-export-analyser`
- `export-modal`
- `export-dest-file`
- `export-dest-database`
- `export-confirm-button`

### 6. Error Handling
- `toast-success`
- `toast-error`
- `toast-info`
- `loading-spinner`
- `loading-message`
- `btn-abort-op`

---

## Testing Your Test IDs

After adding test IDs, verify they work:

### In Browser DevTools Console

```javascript
// Check if element exists
document.querySelector('[data-testid="dashboard-open-file-button"]');

// Check all elements with test IDs
document.querySelectorAll('[data-testid]');
```

### In E2E Tests

```typescript
import { test, expect } from '@playwright/test';

test('test IDs are present', async ({ page }) => {
  await page.goto('http://localhost:1420'); // Your Tauri app URL

  // Wait for button to exist
  await page.waitForSelector('[data-testid="dashboard-open-file-button"]');

  // Click it
  await page.getByTestId('dashboard-open-file-button').click();

  // Verify it works
  await expect(page.getByTestId('analyser-run-button')).toBeVisible();
});
```

---

## Files to Modify

### Renderer Files (HTML generation)
- `src-frontend/renderers/dashboard.ts`
- `src-frontend/renderers/analyser.ts`
- `src-frontend/renderers/lifecycle.ts`
- `src-frontend/renderers/watcher.ts`
- `src-frontend/renderers/export.ts`
- `src-frontend/renderers/settings.ts`
- `src-frontend/renderers/cli.ts`
- `src-frontend/renderers/activity.ts`
- `src-frontend/renderers/ide.ts`
- `src-frontend/renderers/reference.ts`
- `src-frontend/renderers/common.ts` (for shared components like toasts)

### Component Files (if they generate HTML inline)
- `src-frontend/components/DashboardComponent.ts`
- `src-frontend/components/AnalyserComponent.ts`
- `src-frontend/components/LifecycleComponent.ts`
- `src-frontend/components/PipelineComponent.ts`
- `src-frontend/components/PipelineEditor.ts`
- `src-frontend/components/PipelineLibrary.ts`
- `src-frontend/components/PipelineExecutor.ts`
- `src-frontend/components/StepPalette.ts`
- `src-frontend/components/StepConfigPanel.ts`
- `src-frontend/components/WatcherComponent.ts`
- `src-frontend/components/ExportModal.ts`
- `src-frontend/components/DictionaryComponent.ts`
- `src-frontend/components/SettingsComponent.ts`
- `src-frontend/components/PowerShellComponent.ts`
- `src-frontend/components/PythonComponent.ts`
- `src-frontend/components/SQLComponent.ts`

---

## Master Test ID Reference

Create a centralized reference file that lists all test IDs:

**File: `docs/TEST_ID_REFERENCE.md`**

```markdown
# Test ID Reference

## Dashboard
- `dashboard-open-file-button` - Open file dialog
- `dashboard-powershell-button` - Navigate to PowerShell
- `dashboard-python-button` - Navigate to Python
- `dashboard-sql-button` - Navigate to SQL

## Analyser
- `analyser-open-file-button` - Open file in analyser
- `analyser-row-{colName}` - Column row (clickable to expand)
- `col-select-checkbox-{colName}` - Column selection checkbox
- `clean-active-{colName}` - Enable/disable cleaning for column
- `clean-trim-whitespace-{colName}` - Trim whitespace toggle
- `clean-impute-{colName}` - Imputation mode dropdown
- `header-active-all` - Bulk toggle all cleaning
- `btn-begin-cleaning` - Transition to Cleaned stage
- `btn-continue-advanced` - Transition to Advanced stage
- `btn-move-to-validated` - Transition to Validated stage
...

(Continue for all components)
```

---

## Automation Script (Optional)

You could create a script to find elements that need test IDs:

```javascript
// find-missing-testids.js
const fs = require('fs');
const path = require('path');

const rendererFiles = fs.readdirSync('src-frontend/renderers');

rendererFiles.forEach(file => {
  const content = fs.readFileSync(path.join('src-frontend/renderers', file), 'utf8');

  // Find buttons without data-testid
  const buttons = content.match(/<button[^>]*>/g) || [];
  buttons.forEach(btn => {
    if (!btn.includes('data-testid')) {
      console.log(`Missing test ID in ${file}: ${btn}`);
    }
  });
});
```

---

## Next Steps

1. Start with P0 components (Dashboard, Analyser, Lifecycle)
2. Add test IDs to all interactive elements
3. Add test IDs to all assertion targets (panels, tables, etc.)
4. Write E2E tests using the new test IDs
5. Document all test IDs in TEST_ID_REFERENCE.md
6. Repeat for P1 and P2 components

**Estimated Time**: 8-16 hours for all components (can be parallelized across team)
