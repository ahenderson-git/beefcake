# Beefcake GUI Functionality Test Matrix

## Overview

This document maps every user-facing GUI feature to specific test cases, including preconditions, steps, expected results, assertions, test type, automation status, and priority.

## Priority Levels

- **P0**: Critical user workflows - must work for app to be usable
- **P1**: Important features - significantly impact user experience
- **P2**: Polish features - nice to have, minor impact if broken

## Automation Status

- ✅ **Automated** - Test exists and runs in CI
- 🔨 **In Progress** - Test being implemented
- 📝 **Manual** - Not yet automated, tested manually
- ⏭️ **Planned** - Scheduled for automation

---

## 1. Dashboard Component

### 1.1 Open File via Button

| Field | Value |
|-------|-------|
| **Feature** | Open file via button click |
| **Priority** | P0 |
| **Preconditions** | App launched, Dashboard view active |
| **Steps** | 1. Click "Open File" button<br>2. Select valid CSV file from dialog<br>3. Click "Open" |
| **Expected Results** | File loaded, app switches to Analyser view, analysis begins |
| **UI Assertions** | `data-testid="load-file-button"` exists and clickable<br>Loading spinner appears<br>View transitions to Analyser |
| **Data Assertions** | Analysis response contains file name, row count, column summaries |
| **Test Type** | E2E |
| **Automation Status** | 📝 Manual |

### 1.2 Navigate to PowerShell Tool

| Field | Value |
|-------|-------|
| **Feature** | Navigate to PowerShell scripting tool |
| **Priority** | P1 |
| **Preconditions** | Dashboard view active |
| **Steps** | 1. Click "PowerShell" button |
| **Expected Results** | View switches to PowerShell editor |
| **UI Assertions** | `data-testid="btn-powershell"` clickable<br>PowerShell view renders with editor |
| **Data Assertions** | None |
| **Test Type** | E2E |
| **Automation Status** | 📝 Manual |

### 1.3 Navigate to Python Tool

| Field | Value |
|-------|-------|
| **Feature** | Navigate to Python scripting tool |
| **Priority** | P1 |
| **Steps** | Same as 1.2 but click Python button |
| **Test Type** | E2E |
| **Automation Status** | 📝 Manual |

### 1.4 Navigate to SQL Tool

| Field | Value |
|-------|-------|
| **Feature** | Navigate to SQL query tool |
| **Priority** | P1 |
| **Steps** | Same as 1.2 but click SQL button |
| **Test Type** | E2E |
| **Automation Status** | 📝 Manual |

---

## 2. Analyser Component

### 2.1 Load and Analyze File

| Field | Value |
|-------|-------|
| **Feature** | Load file and run analysis |
| **Priority** | P0 |
| **Preconditions** | Analyser view active (or triggered from Dashboard) |
| **Steps** | 1. Click "Open File" (`data-testid="btn-open-file"`)<br>2. Select `testdata/clean.csv`<br>3. Wait for analysis to complete |
| **Expected Results** | Analysis summary displayed with:<br>- File info (name, size, row/col count)<br>- Column list with types<br>- Health score<br>- Statistics panels |
| **UI Assertions** | `data-testid="analysis-summary-panel"` visible<br>`data-testid="dataset-preview-table"` shows columns<br>Health score badge present<br>No error toast |
| **Data Assertions** | `analysisResponse.row_count > 0`<br>`analysisResponse.summary.length > 0`<br>`analysisResponse.health.score >= 0` |
| **Test Type** | E2E + Integration (Rust) |
| **Automation Status** | 📝 Manual |

### 2.2 Expand Column Row for Details

| Field | Value |
|-------|-------|
| **Feature** | Expand row to see column statistics |
| **Priority** | P0 |
| **Preconditions** | File analyzed, column list visible |
| **Steps** | 1. Click on any row in analyser table (`data-testid="analyser-row-{colName}"`) |
| **Expected Results** | Row expands showing:<br>- Histogram/chart<br>- Detailed stats (min/max/mean/median)<br>- Interpretation/insights<br>- Sample values |
| **UI Assertions** | Expanded row container visible<br>Chart canvas rendered<br>Stats table present |
| **Data Assertions** | Chart data matches column stats |
| **Test Type** | E2E |
| **Automation Status** | 📝 Manual |

### 2.3 Toggle Column Selection Checkbox

| Field | Value |
|-------|-------|
| **Feature** | Select/deselect columns for processing |
| **Priority** | P0 |
| **Preconditions** | File analyzed |
| **Steps** | 1. Uncheck column checkbox (`data-testid="col-select-checkbox-{colName}"`) |
| **Expected Results** | Column deselected (tracked in state)<br>Checkbox unchecked |
| **UI Assertions** | Checkbox state matches selection |
| **Data Assertions** | `state.selectedColumns` updated |
| **Test Type** | Unit (TS state) + E2E |
| **Automation Status** | 📝 Manual |

### 2.4 Toggle Active Cleaning

| Field | Value |
|-------|-------|
| **Feature** | Enable/disable cleaning for a column |
| **Priority** | P0 |
| **Preconditions** | File analyzed, in Cleaned stage |
| **Steps** | 1. Toggle "Active" checkbox (`data-testid="clean-active-{colName}"`) |
| **Expected Results** | Cleaning config updated, column will be included/excluded from transforms |
| **UI Assertions** | Checkbox reflects state<br>Row styling changes (grayed out if inactive) |
| **Data Assertions** | `cleaningConfigs[colName].active === false` |
| **Test Type** | Unit (config) + E2E |
| **Automation Status** | 📝 Manual |

### 2.5 Configure Trim Whitespace

| Field | Value |
|-------|-------|
| **Feature** | Enable trim whitespace cleaning |
| **Priority** | P0 |
| **Preconditions** | File analyzed, Cleaned stage |
| **Steps** | 1. Check "Trim" (`data-testid="clean-trim-whitespace-{colName}"`) |
| **Expected Results** | Config updated |
| **UI Assertions** | Checkbox checked |
| **Data Assertions** | `cleaningConfigs[colName].trim_whitespace === true` |
| **Test Type** | Unit + E2E |
| **Automation Status** | 📝 Manual |

### 2.6 Configure Imputation Mode

| Field | Value |
|-------|-------|
| **Feature** | Select imputation strategy for missing values |
| **Priority** | P0 |
| **Preconditions** | Advanced stage |
| **Steps** | 1. Select impute mode dropdown (`data-testid="clean-impute-{colName}"`) <br>2. Choose "Mean" |
| **Expected Results** | Config updated, missing values will be filled with mean |
| **UI Assertions** | Dropdown shows selected value |
| **Data Assertions** | `cleaningConfigs[colName].impute_mode === 'Mean'` |
| **Test Type** | Unit + Integration (Rust transform) |
| **Automation Status** | 📝 Manual |

### 2.7 Configure Rounding

| Field | Value |
|-------|-------|
| **Feature** | Set decimal places for numeric rounding |
| **Priority** | P1 |
| **Preconditions** | Advanced stage, numeric column |
| **Steps** | 1. Select rounding dropdown (`data-testid="clean-rounding-{colName}"`) <br>2. Choose "2 decimals" |
| **Expected Results** | Config updated |
| **Data Assertions** | `cleaningConfigs[colName].rounding === 2` |
| **Test Type** | Unit + Integration |
| **Automation Status** | 📝 Manual |

### 2.8 Bulk Toggle All Active

| Field | Value |
|-------|-------|
| **Feature** | Enable/disable all column cleaning at once |
| **Priority** | P1 |
| **Preconditions** | Cleaned stage |
| **Steps** | 1. Toggle "Active All" header checkbox (`data-testid="header-active-all"`) |
| **Expected Results** | All column configs updated |
| **UI Assertions** | All row checkboxes reflect bulk state |
| **Data Assertions** | All configs have matching `active` value |
| **Test Type** | Unit + E2E |
| **Automation Status** | 📝 Manual |

### 2.9 Bulk Set Imputation

| Field | Value |
|-------|-------|
| **Feature** | Apply imputation strategy to all columns |
| **Priority** | P1 |
| **Preconditions** | Advanced stage |
| **Steps** | 1. Select bulk impute dropdown (`data-testid="header-impute-all"`) |
| **Expected Results** | All configs updated |
| **Data Assertions** | All configs have same `impute_mode` |
| **Test Type** | Unit + E2E |
| **Automation Status** | 📝 Manual |

### 2.10 Begin Cleaning Transition

| Field | Value |
|-------|-------|
| **Feature** | Transition from Profiled to Cleaned stage |
| **Priority** | P0 |
| **Preconditions** | Profiled stage, dataset loaded |
| **Steps** | 1. Click "Begin Cleaning" (`data-testid="btn-begin-cleaning"`) <br>2. Wait for transition |
| **Expected Results** | New version created in Cleaned stage<br>Lifecycle rail updates<br>Cleaning controls unlocked<br>Success toast |
| **UI Assertions** | Button shows spinner during transition<br>Stage indicator updates<br>Toast appears |
| **Data Assertions** | `currentDataset.activeVersionId` points to new Cleaned version<br>Version has correct stage |
| **Test Type** | Integration + E2E |
| **Automation Status** | 📝 Manual |

### 2.11 Continue to Advanced Transition

| Field | Value |
|-------|-------|
| **Feature** | Transition from Cleaned to Advanced stage |
| **Priority** | P0 |
| **Preconditions** | Cleaned stage |
| **Steps** | 1. Click "Continue to Advanced" (`data-testid="btn-continue-advanced"`) |
| **Expected Results** | Pipeline built from cleaning configs<br>New Advanced version created<br>ML preprocessing unlocked |
| **UI Assertions** | Button spinner<br>Stage updates<br>Success toast |
| **Data Assertions** | New version contains transform pipeline JSON<br>ML options visible |
| **Test Type** | Integration + E2E |
| **Automation Status** | 📝 Manual |

### 2.12 Move to Validated Transition

| Field | Value |
|-------|-------|
| **Feature** | Transition from Advanced to Validated stage |
| **Priority** | P0 |
| **Preconditions** | Advanced stage |
| **Steps** | 1. Click "Move to Validated" (`data-testid="btn-move-to-validated"`) |
| **Expected Results** | Validated version created<br>Summary view shown<br>Publish options available |
| **UI Assertions** | Summary panel renders<br>Publish button visible |
| **Data Assertions** | Version stage is Validated |
| **Test Type** | Integration + E2E |
| **Automation Status** | 📝 Manual |

### 2.13 Export Data (Open Modal)

| Field | Value |
|-------|-------|
| **Feature** | Open export modal |
| **Priority** | P0 |
| **Preconditions** | File analyzed |
| **Steps** | 1. Click "Export" (`data-testid="btn-export-analyser"`) |
| **Expected Results** | Export modal opens |
| **UI Assertions** | `data-testid="export-modal"` visible<br>Source/destination options shown |
| **Data Assertions** | Modal state initialized with current file path |
| **Test Type** | E2E |
| **Automation Status** | 📝 Manual |

### 2.14 Re-analyze Current File

| Field | Value |
|-------|-------|
| **Feature** | Re-run analysis on current file |
| **Priority** | P1 |
| **Preconditions** | File already analyzed |
| **Steps** | 1. Click "Re-analyze" (`data-testid="btn-reanalyze"`) |
| **Expected Results** | Analysis runs again, fresh results |
| **UI Assertions** | Loading state shown<br>Results updated |
| **Data Assertions** | New analysis response |
| **Test Type** | E2E |
| **Automation Status** | 📝 Manual |

### 2.15 Standardize All Headers

| Field | Value |
|-------|-------|
| **Feature** | Bulk standardize column names |
| **Priority** | P1 |
| **Preconditions** | File analyzed |
| **Steps** | 1. Click standardize icon (`data-testid="header-standardize-all"`) |
| **Expected Results** | All column configs use standardized names<br>Success toast |
| **UI Assertions** | Toast shows "Headers standardized" |
| **Data Assertions** | All `new_name` fields match `standardized_name` |
| **Test Type** | Unit + E2E |
| **Automation Status** | 📝 Manual |

### 2.16 Handle Invalid File Error

| Field | Value |
|-------|-------|
| **Feature** | Gracefully handle invalid file selection |
| **Priority** | P0 |
| **Preconditions** | Analyser view |
| **Steps** | 1. Attempt to load `testdata/invalid_format.txt` |
| **Expected Results** | Error toast appears<br>App remains stable<br>Previous state preserved |
| **UI Assertions** | `data-testid="toast-error"` appears with message |
| **Data Assertions** | No crash, no corrupt state |
| **Test Type** | Integration + E2E |
| **Automation Status** | 📝 Manual |

---

## 3. Lifecycle Component

### 3.1 View Version Tree

| Field | Value |
|-------|-------|
| **Feature** | Display all dataset versions in tree |
| **Priority** | P0 |
| **Preconditions** | Dataset with multiple versions |
| **Steps** | 1. Navigate to Lifecycle view |
| **Expected Results** | Version tree shows all stages<br>Active version highlighted |
| **UI Assertions** | `data-testid="lifecycle-version-tree"` visible<br>Each version has node (`data-testid="lifecycle-version-{id}"`) |
| **Data Assertions** | Version count matches dataset |
| **Test Type** | E2E |
| **Automation Status** | 📝 Manual |

### 3.2 Set Active Version

| Field | Value |
|-------|-------|
| **Feature** | Switch to different version |
| **Priority** | P0 |
| **Preconditions** | Multiple versions exist |
| **Steps** | 1. Click "Set Active" on version node (`data-testid="lifecycle-set-active-{versionId}"`) |
| **Expected Results** | Active version changes<br>UI updates to show that version's data<br>Success toast |
| **UI Assertions** | Active indicator moves<br>Toast appears |
| **Data Assertions** | `currentDataset.activeVersionId` updated |
| **Test Type** | Integration + E2E |
| **Automation Status** | 📝 Manual |

### 3.3 View Version Diff

| Field | Value |
|-------|-------|
| **Feature** | Compare two versions |
| **Priority** | P1 |
| **Preconditions** | Version with parent exists |
| **Steps** | 1. Click "View Diff" (`data-testid="lifecycle-view-diff-{versionId}"`) |
| **Expected Results** | Diff modal opens showing:<br>- Row changes<br>- Schema changes (added/removed cols)<br>- Statistical changes |
| **UI Assertions** | `data-testid="diff-modal"` visible<br>Diff tables rendered |
| **Data Assertions** | Diff contains accurate deltas |
| **Test Type** | Integration (Rust diff engine) + E2E |
| **Automation Status** | 📝 Manual |

### 3.4 Publish Version (View Mode)

| Field | Value |
|-------|-------|
| **Feature** | Publish as SQL view |
| **Priority** | P0 |
| **Preconditions** | Validated stage |
| **Steps** | 1. Click "Publish" (`data-testid="btn-publish-version"`)<br>2. Select "View" mode<br>3. Confirm |
| **Expected Results** | Version published<br>New Published version created<br>Success toast |
| **UI Assertions** | Modal appears (`data-testid="publish-modal"`)<br>View/Snapshot buttons<br>Success toast |
| **Data Assertions** | Published version exists with correct mode |
| **Test Type** | Integration + E2E |
| **Automation Status** | 📝 Manual |

### 3.5 Publish Version (Snapshot Mode)

| Field | Value |
|-------|-------|
| **Feature** | Publish as snapshot file |
| **Priority** | P0 |
| **Steps** | Same as 3.4 but select Snapshot |
| **Expected Results** | Snapshot file created on disk |
| **Data Assertions** | File exists at expected path |
| **Test Type** | Integration + E2E |
| **Automation Status** | 📝 Manual |

### 3.6 Navigate Stages via Rail

| Field | Value |
|-------|-------|
| **Feature** | Click lifecycle rail stages to switch versions |
| **Priority** | P0 |
| **Preconditions** | Multiple stages unlocked |
| **Steps** | 1. Click stage indicator (`data-testid="lifecycle-stage-{stage}"`) |
| **Expected Results** | Active version switches to that stage<br>UI updates |
| **UI Assertions** | Stage highlighted<br>View content updates |
| **Data Assertions** | Active version matches stage |
| **Test Type** | E2E |
| **Automation Status** | 📝 Manual |

### 3.7 Locked Stage Behavior

| Field | Value |
|-------|-------|
| **Feature** | Cannot navigate to locked stages |
| **Priority** | P1 |
| **Preconditions** | Only Raw and Profiled exist |
| **Steps** | 1. Click locked Cleaned stage |
| **Expected Results** | Error toast<br>No navigation |
| **UI Assertions** | Toast: "Cannot switch to Cleaned stage yet"<br>No view change |
| **Data Assertions** | Active version unchanged |
| **Test Type** | E2E |
| **Automation Status** | 📝 Manual |

---

## 4. Pipeline Component

### 4.1 View Pipeline Library

| Field | Value |
|-------|-------|
| **Feature** | Display saved pipelines and templates |
| **Priority** | P1 |
| **Preconditions** | Pipeline view active |
| **Steps** | 1. Navigate to Pipeline view |
| **Expected Results** | Library shows 8 built-in templates<br>User-saved pipelines (if any) |
| **UI Assertions** | `data-testid="pipeline-library"` visible<br>Template cards rendered |
| **Data Assertions** | At least 8 templates present |
| **Test Type** | E2E |
| **Automation Status** | 📝 Manual |

### 4.2 Use Template

| Field | Value |
|-------|-------|
| **Feature** | Load pre-configured pipeline template |
| **Priority** | P1 |
| **Preconditions** | Library view |
| **Steps** | 1. Click "Use Template" on any template card (`data-testid="pipeline-use-template-{templateName}"`) |
| **Expected Results** | Editor opens with template steps pre-loaded |
| **UI Assertions** | Editor view (`data-testid="pipeline-editor"`) visible<br>Steps populated in canvas |
| **Data Assertions** | Pipeline spec matches template |
| **Test Type** | E2E |
| **Automation Status** | 📝 Manual |

### 4.3 Open Pipeline Editor

| Field | Value |
|-------|-------|
| **Feature** | Create new pipeline from scratch |
| **Priority** | P1 |
| **Preconditions** | Library view |
| **Steps** | 1. Click "New Pipeline" (`data-testid="pipeline-new-button"`) |
| **Expected Results** | Editor opens with empty canvas |
| **UI Assertions** | Editor visible<br>Step palette visible<br>Canvas empty |
| **Data Assertions** | Pipeline spec has empty steps array |
| **Test Type** | E2E |
| **Automation Status** | 📝 Manual |

### 4.4 Drag Step from Palette

| Field | Value |
|-------|-------|
| **Feature** | Add transformation step via drag-and-drop |
| **Priority** | P0 |
| **Preconditions** | Editor open, empty canvas |
| **Steps** | 1. Drag "Trim Whitespace" from palette (`data-testid="palette-step-trim"`) to canvas |
| **Expected Results** | Step card appears in canvas<br>Config panel shows step options |
| **UI Assertions** | Step card visible in canvas (`data-testid="pipeline-step-0"`)<br>Config panel updates |
| **Data Assertions** | Pipeline spec has one step with type "trim_whitespace" |
| **Test Type** | E2E |
| **Automation Status** | 📝 Manual |

### 4.5 Configure Step Parameters

| Field | Value |
|-------|-------|
| **Feature** | Set parameters for pipeline step |
| **Priority** | P0 |
| **Preconditions** | Step added to canvas |
| **Steps** | 1. Click step card<br>2. Set columns in config panel (`data-testid="step-config-columns"`) |
| **Expected Results** | Step config updated |
| **UI Assertions** | Config panel reflects changes |
| **Data Assertions** | Step parameters match input |
| **Test Type** | Unit + E2E |
| **Automation Status** | 📝 Manual |

### 4.6 Reorder Steps via Drag

| Field | Value |
|-------|-------|
| **Feature** | Change step execution order |
| **Priority** | P1 |
| **Preconditions** | Multiple steps in canvas |
| **Steps** | 1. Drag step 2 above step 1 |
| **Expected Results** | Steps reorder in canvas and spec |
| **UI Assertions** | Visual order changes |
| **Data Assertions** | Spec steps array reordered |
| **Test Type** | E2E |
| **Automation Status** | 📝 Manual |

### 4.7 Delete Step

| Field | Value |
|-------|-------|
| **Feature** | Remove step from pipeline |
| **Priority** | P1 |
| **Preconditions** | Step in canvas |
| **Steps** | 1. Click delete icon on step card (`data-testid="pipeline-step-delete-0"`) |
| **Expected Results** | Step removed |
| **UI Assertions** | Card disappears |
| **Data Assertions** | Step removed from spec |
| **Test Type** | E2E |
| **Automation Status** | 📝 Manual |

### 4.8 Save Pipeline

| Field | Value |
|-------|-------|
| **Feature** | Save pipeline spec to file |
| **Priority** | P0 |
| **Preconditions** | Pipeline configured |
| **Steps** | 1. Click "Save" (`data-testid="pipeline-save-button"`)<br>2. Choose file location<br>3. Save |
| **Expected Results** | JSON file written to disk |
| **UI Assertions** | Success toast |
| **Data Assertions** | File exists, valid JSON, matches spec |
| **Test Type** | Integration + E2E |
| **Automation Status** | 📝 Manual |

### 4.9 Load Pipeline

| Field | Value |
|-------|-------|
| **Feature** | Load pipeline from JSON file |
| **Priority** | P0 |
| **Preconditions** | Saved pipeline file exists |
| **Steps** | 1. Click "Load" (`data-testid="pipeline-load-button"`)<br>2. Select file<br>3. Open |
| **Expected Results** | Pipeline loaded into editor<br>Steps rendered |
| **UI Assertions** | Canvas shows all steps |
| **Data Assertions** | Loaded spec matches file |
| **Test Type** | Integration + E2E |
| **Automation Status** | 📝 Manual |

### 4.10 Validate Pipeline

| Field | Value |
|-------|-------|
| **Feature** | Check pipeline against input schema |
| **Priority** | P0 |
| **Preconditions** | Pipeline configured, input file selected |
| **Steps** | 1. Click "Validate" (`data-testid="pipeline-validate-button"`) |
| **Expected Results** | Validation runs<br>Errors displayed if invalid<br>Success toast if valid |
| **UI Assertions** | Validation results panel (`data-testid="pipeline-validation-results"`)<br>Toast indicates status |
| **Data Assertions** | Validation errors list correct (e.g., "Column 'age' not found") |
| **Test Type** | Integration (Rust validation) + E2E |
| **Automation Status** | 📝 Manual |

### 4.11 Execute Pipeline

| Field | Value |
|-------|-------|
| **Feature** | Run pipeline on input data |
| **Priority** | P0 |
| **Preconditions** | Valid pipeline, input file selected |
| **Steps** | 1. Click "Execute" (`data-testid="pipeline-execute-button"`)<br>2. Wait for completion |
| **Expected Results** | Pipeline runs<br>Output file created<br>Execution log shown<br>Success toast |
| **UI Assertions** | Progress modal (`data-testid="pipeline-executor-modal"`)<br>Log output<br>Completion toast |
| **Data Assertions** | Output file exists<br>Transforms applied correctly (compare to golden) |
| **Test Type** | Integration + E2E |
| **Automation Status** | 📝 Manual |

### 4.12 Export to PowerShell

| Field | Value |
|-------|-------|
| **Feature** | Generate PowerShell script from pipeline |
| **Priority** | P1 |
| **Preconditions** | Valid pipeline |
| **Steps** | 1. Click "Export PowerShell" (`data-testid="pipeline-export-ps"`) |
| **Expected Results** | .ps1 file generated<br>Script executable standalone |
| **UI Assertions** | Success toast |
| **Data Assertions** | Script file exists, syntactically valid PowerShell |
| **Test Type** | Integration + E2E |
| **Automation Status** | 📝 Manual |

### 4.13 Handle Validation Errors

| Field | Value |
|-------|-------|
| **Feature** | Display validation errors clearly |
| **Priority** | P0 |
| **Preconditions** | Invalid pipeline (e.g., references non-existent column) |
| **Steps** | 1. Validate pipeline |
| **Expected Results** | Error panel shows specific issues |
| **UI Assertions** | Error list visible<br>Each error actionable (shows which step) |
| **Data Assertions** | Error messages accurate |
| **Test Type** | Integration + E2E |
| **Automation Status** | 📝 Manual |

---

## 5. Watcher Component

### 5.1 Start Watcher

| Field | Value |
|-------|-------|
| **Feature** | Enable filesystem watcher |
| **Priority** | P1 |
| **Preconditions** | Watcher view, folder configured |
| **Steps** | 1. Click "Start Watcher" (`data-testid="watcher-start-button"`) |
| **Expected Results** | Watcher starts monitoring<br>Status changes to "Watching"<br>Success toast |
| **UI Assertions** | Status indicator (`data-testid="watcher-status"`) shows "Watching"<br>Start button disabled, Stop button enabled |
| **Data Assertions** | `watcherState.enabled === true`<br>`watcherState.state === 'watching'` |
| **Test Type** | Integration + E2E |
| **Automation Status** | 📝 Manual |

### 5.2 Stop Watcher

| Field | Value |
|-------|-------|
| **Feature** | Disable filesystem watcher |
| **Priority** | P1 |
| **Preconditions** | Watcher running |
| **Steps** | 1. Click "Stop Watcher" (`data-testid="watcher-stop-button"`) |
| **Expected Results** | Watcher stops<br>Status changes to "Idle" |
| **UI Assertions** | Status shows "Idle"<br>Buttons toggle |
| **Data Assertions** | `watcherState.enabled === false` |
| **Test Type** | Integration + E2E |
| **Automation Status** | 📝 Manual |

### 5.3 Set Watch Folder

| Field | Value |
|-------|-------|
| **Feature** | Configure folder to monitor |
| **Priority** | P1 |
| **Preconditions** | Watcher view |
| **Steps** | 1. Click "Choose Folder" (`data-testid="watcher-choose-folder"`)<br>2. Select folder<br>3. Confirm |
| **Expected Results** | Folder path saved<br>Displayed in UI |
| **UI Assertions** | Folder path shown (`data-testid="watcher-folder-path"`) |
| **Data Assertions** | `watcherState.folder === selectedPath` |
| **Test Type** | Integration + E2E |
| **Automation Status** | 📝 Manual |

### 5.4 Auto-Ingest File

| Field | Value |
|-------|-------|
| **Feature** | Detect new file and ingest automatically |
| **Priority** | P0 |
| **Preconditions** | Watcher running, folder configured |
| **Steps** | 1. Copy `testdata/clean.csv` into watched folder<br>2. Wait for detection |
| **Expected Results** | File detected<br>Ingestion starts<br>Dataset created (Raw stage)<br>Activity feed updates |
| **UI Assertions** | Activity item appears (`data-testid="watcher-activity-{id}"`) with "Success" status<br>Dataset info shown |
| **Data Assertions** | New dataset exists in lifecycle registry |
| **Test Type** | Integration + E2E |
| **Automation Status** | 📝 Manual |

### 5.5 Activity Feed Updates

| Field | Value |
|-------|-------|
| **Feature** | Real-time activity log |
| **Priority** | P1 |
| **Preconditions** | Watcher active, ingestion occurred |
| **Steps** | 1. View activity feed (`data-testid="watcher-activity-feed"`) |
| **Expected Results** | Feed shows chronological list of ingestions<br>Timestamps, filenames, statuses |
| **UI Assertions** | Activity items visible<br>Status badges color-coded |
| **Data Assertions** | Activity count matches ingestions |
| **Test Type** | E2E |
| **Automation Status** | 📝 Manual |

### 5.6 Manual Ingest Now

| Field | Value |
|-------|-------|
| **Feature** | Manually trigger ingestion of specific file |
| **Priority** | P2 |
| **Preconditions** | Watcher view |
| **Steps** | 1. Click "Ingest Now" on file<br>2. Wait for completion |
| **Expected Results** | File ingested immediately |
| **UI Assertions** | Activity feed updates |
| **Data Assertions** | Dataset created |
| **Test Type** | Integration + E2E |
| **Automation Status** | 📝 Manual |

---

## 6. Export Modal

### 6.1 Export to CSV File

| Field | Value |
|-------|-------|
| **Feature** | Export processed data as CSV |
| **Priority** | P0 |
| **Preconditions** | Export modal open, data available |
| **Steps** | 1. Select "File" destination (`data-testid="export-dest-file"`)<br>2. Select "CSV" format<br>3. Choose output path<br>4. Click "Export" (`data-testid="export-confirm-button"`) |
| **Expected Results** | CSV file created<br>Contains processed data<br>Success toast |
| **UI Assertions** | Toast: "Export successful" |
| **Data Assertions** | File exists<br>Row/column counts match<br>Transforms applied |
| **Test Type** | Integration + E2E |
| **Automation Status** | 📝 Manual |

### 6.2 Export to JSON File

| Field | Value |
|-------|-------|
| **Feature** | Export as JSON |
| **Priority** | P1 |
| **Steps** | Same as 6.1 but select JSON format |
| **Expected Results** | Valid JSON file created |
| **Data Assertions** | JSON parseable, data correct |
| **Test Type** | Integration + E2E |
| **Automation Status** | 📝 Manual |

### 6.3 Export to Parquet File

| Field | Value |
|-------|-------|
| **Feature** | Export as Parquet |
| **Priority** | P1 |
| **Steps** | Same as 6.1 but select Parquet format |
| **Expected Results** | Parquet file created, loadable |
| **Data Assertions** | File valid, data correct |
| **Test Type** | Integration + E2E |
| **Automation Status** | 📝 Manual |

### 6.4 Push to Database

| Field | Value |
|-------|-------|
| **Feature** | Export to PostgreSQL database |
| **Priority** | P1 |
| **Preconditions** | Database connection configured |
| **Steps** | 1. Select "Database" destination (`data-testid="export-dest-database"`)<br>2. Select connection<br>3. Confirm |
| **Expected Results** | Data uploaded to database<br>Table created/updated |
| **UI Assertions** | Success toast |
| **Data Assertions** | Database table exists with correct data |
| **Test Type** | Integration (requires test DB) + E2E |
| **Automation Status** | 📝 Manual |

### 6.5 Create Data Dictionary on Export

| Field | Value |
|-------|-------|
| **Feature** | Generate dictionary snapshot during export |
| **Priority** | P1 |
| **Preconditions** | Export modal open |
| **Steps** | 1. Check "Create Dictionary" (`data-testid="export-create-dictionary"`)<br>2. Export |
| **Expected Results** | Dictionary snapshot created<br>Available in Dictionary view |
| **UI Assertions** | Checkbox checked<br>Success toast mentions dictionary |
| **Data Assertions** | Snapshot exists in storage |
| **Test Type** | Integration + E2E |
| **Automation Status** | 📝 Manual |

---

## 7. PowerShell Component

### 7.1 Execute PowerShell Script

| Field | Value |
|-------|-------|
| **Feature** | Run PowerShell code with data context |
| **Priority** | P1 |
| **Preconditions** | PowerShell view, data loaded |
| **Steps** | 1. Type script in editor<br>2. Click "Run" (`data-testid="powershell-run-button"`) |
| **Expected Results** | Script executes<br>Output displayed with ANSI colors<br>Errors shown if any |
| **UI Assertions** | Output panel (`data-testid="powershell-output"`) updates<br>Loading state during execution |
| **Data Assertions** | Output matches expected |
| **Test Type** | Integration (Windows only) + E2E |
| **Automation Status** | 📝 Manual |

### 7.2 Handle PowerShell Errors

| Field | Value |
|-------|-------|
| **Feature** | Display script errors clearly |
| **Priority** | P1 |
| **Preconditions** | PowerShell view |
| **Steps** | 1. Type invalid script<br>2. Run |
| **Expected Results** | Error output shown<br>No crash |
| **UI Assertions** | Error displayed in output panel |
| **Data Assertions** | Error message accurate |
| **Test Type** | Integration + E2E |
| **Automation Status** | 📝 Manual |

---

## 8. Python Component

### 8.1 Execute Python Script

| Field | Value |
|-------|-------|
| **Feature** | Run Python code with pandas DataFrame context |
| **Priority** | P1 |
| **Preconditions** | Python view, data loaded (available as `df` variable) |
| **Steps** | 1. Type `print(df.head())`<br>2. Click "Run" (`data-testid="python-run-button"`) |
| **Expected Results** | Script executes<br>DataFrame head displayed<br>ANSI colors preserved |
| **UI Assertions** | Output panel updates (`data-testid="python-output"`) |
| **Data Assertions** | Output shows first 5 rows |
| **Test Type** | Integration + E2E |
| **Automation Status** | 📝 Manual |

### 8.2 Install Python Package

| Field | Value |
|-------|-------|
| **Feature** | Install pip package from GUI |
| **Priority** | P2 |
| **Preconditions** | Python view |
| **Steps** | 1. Click "Install Package" (`data-testid="python-install-package"`)<br>2. Enter package name<br>3. Confirm |
| **Expected Results** | Package installs<br>Progress shown<br>Success/failure toast |
| **UI Assertions** | Install log shown |
| **Data Assertions** | Package importable after install |
| **Test Type** | Integration + E2E |
| **Automation Status** | 📝 Manual |

---

## 9. SQL Component

### 9.1 Execute SQL Query

| Field | Value |
|-------|-------|
| **Feature** | Run SQL query against loaded data |
| **Priority** | P1 |
| **Preconditions** | SQL view, data loaded |
| **Steps** | 1. Type `SELECT * FROM df LIMIT 10`<br>2. Click "Run" (`data-testid="sql-run-button"`) |
| **Expected Results** | Query executes<br>Results displayed as table |
| **UI Assertions** | Output panel (`data-testid="sql-output"`) shows table |
| **Data Assertions** | 10 rows returned |
| **Test Type** | Integration + E2E |
| **Automation Status** | 📝 Manual |

### 9.2 Handle SQL Syntax Errors

| Field | Value |
|-------|-------|
| **Feature** | Display SQL errors |
| **Priority** | P1 |
| **Preconditions** | SQL view |
| **Steps** | 1. Type invalid SQL<br>2. Run |
| **Expected Results** | Error message shown |
| **UI Assertions** | Error in output panel |
| **Data Assertions** | Error message descriptive |
| **Test Type** | Integration + E2E |
| **Automation Status** | 📝 Manual |

---

## 10. Dictionary Component

### 10.1 List Snapshots

| Field | Value |
|-------|-------|
| **Feature** | View all data dictionary snapshots |
| **Priority** | P1 |
| **Preconditions** | Dictionary view, snapshots exist |
| **Steps** | 1. Navigate to Dictionary view |
| **Expected Results** | List of snapshots shown with metadata |
| **UI Assertions** | Snapshot cards visible (`data-testid="dictionary-snapshot-{id}"`) |
| **Data Assertions** | Snapshot count correct |
| **Test Type** | E2E |
| **Automation Status** | 📝 Manual |

### 10.2 Load Snapshot

| Field | Value |
|-------|-------|
| **Feature** | Load snapshot details |
| **Priority** | P1 |
| **Preconditions** | Snapshots exist |
| **Steps** | 1. Click snapshot card |
| **Expected Results** | Snapshot details displayed<br>Column metadata shown |
| **UI Assertions** | Detail view renders (`data-testid="dictionary-detail"`) |
| **Data Assertions** | Metadata loaded correctly |
| **Test Type** | Integration + E2E |
| **Automation Status** | 📝 Manual |

### 10.3 Edit Business Metadata

| Field | Value |
|-------|-------|
| **Feature** | Update dataset/column business metadata |
| **Priority** | P1 |
| **Preconditions** | Snapshot loaded |
| **Steps** | 1. Click edit icon<br>2. Update description field (`data-testid="dictionary-edit-description"`)<br>3. Save |
| **Expected Results** | Metadata saved<br>Success toast |
| **UI Assertions** | Toast confirms save |
| **Data Assertions** | Snapshot updated with new metadata |
| **Test Type** | Integration + E2E |
| **Automation Status** | 📝 Manual |

### 10.4 Export Dictionary to Markdown

| Field | Value |
|-------|-------|
| **Feature** | Generate markdown documentation |
| **Priority** | P2 |
| **Preconditions** | Snapshot loaded |
| **Steps** | 1. Click "Export Markdown" (`data-testid="dictionary-export-md"`)<br>2. Choose path<br>3. Save |
| **Expected Results** | .md file created with formatted documentation |
| **UI Assertions** | Success toast |
| **Data Assertions** | Markdown file exists, well-formatted |
| **Test Type** | Integration + E2E |
| **Automation Status** | 📝 Manual |

---

## 11. Settings Component

### 11.1 Add Database Connection

| Field | Value |
|-------|-------|
| **Feature** | Configure new database connection |
| **Priority** | P1 |
| **Preconditions** | Settings view |
| **Steps** | 1. Click "Add Connection" (`data-testid="settings-add-connection"`)<br>2. Fill form (host, port, user, password, database)<br>3. Click "Test Connection"<br>4. Save |
| **Expected Results** | Connection tested<br>Success toast if valid<br>Connection saved to config |
| **UI Assertions** | Form visible<br>Test button works<br>Connection appears in list |
| **Data Assertions** | `appConfig.connections` includes new entry<br>Password stored in system keyring |
| **Test Type** | Integration + E2E |
| **Automation Status** | 📝 Manual |

### 11.2 Test Database Connection

| Field | Value |
|-------|-------|
| **Feature** | Verify connection settings |
| **Priority** | P1 |
| **Preconditions** | Connection form filled |
| **Steps** | 1. Click "Test Connection" (`data-testid="settings-test-connection"`) |
| **Expected Results** | Backend attempts connection<br>Success/failure toast |
| **UI Assertions** | Loading state during test<br>Toast shows result |
| **Data Assertions** | Error message helpful if failure |
| **Test Type** | Integration (requires test DB) + E2E |
| **Automation Status** | 📝 Manual |

### 11.3 Delete Database Connection

| Field | Value |
|-------|-------|
| **Feature** | Remove saved connection |
| **Priority** | P1 |
| **Preconditions** | Connection exists |
| **Steps** | 1. Click delete icon (`data-testid="settings-delete-connection-{id}"`)<br>2. Confirm |
| **Expected Results** | Connection removed<br>Keyring entry deleted<br>Success toast |
| **UI Assertions** | Connection disappears from list |
| **Data Assertions** | `appConfig.connections` no longer includes entry |
| **Test Type** | Integration + E2E |
| **Automation Status** | 📝 Manual |

### 11.4 Persist Settings

| Field | Value |
|-------|-------|
| **Feature** | Save settings to disk |
| **Priority** | P1 |
| **Preconditions** | Settings changed |
| **Steps** | 1. Modify any setting<br>2. Restart app<br>3. Verify setting persisted |
| **Expected Results** | Settings loaded on restart |
| **UI Assertions** | Settings match previous state |
| **Data Assertions** | Config file updated |
| **Test Type** | Integration + E2E |
| **Automation Status** | 📝 Manual |

---

## 12. Global UI Features

### 12.1 Toast Notifications (Success)

| Field | Value |
|-------|-------|
| **Feature** | Show success toast |
| **Priority** | P0 |
| **Preconditions** | Any successful action |
| **Steps** | Trigger successful action (e.g., save file) |
| **Expected Results** | Green toast appears<br>Auto-dismisses after 3s |
| **UI Assertions** | `data-testid="toast-success"` visible<br>Correct message |
| **Data Assertions** | N/A |
| **Test Type** | E2E |
| **Automation Status** | 📝 Manual |

### 12.2 Toast Notifications (Error)

| Field | Value |
|-------|-------|
| **Feature** | Show error toast |
| **Priority** | P0 |
| **Preconditions** | Any failed action |
| **Steps** | Trigger error (e.g., load invalid file) |
| **Expected Results** | Red toast appears<br>Shows error message |
| **UI Assertions** | `data-testid="toast-error"` visible<br>Descriptive message |
| **Data Assertions** | N/A |
| **Test Type** | E2E |
| **Automation Status** | 📝 Manual |

### 12.3 Loading State

| Field | Value |
|-------|-------|
| **Feature** | Display loading spinner during async operations |
| **Priority** | P0 |
| **Preconditions** | Long-running operation (e.g., analysis) |
| **Steps** | Trigger operation |
| **Expected Results** | Spinner shown<br>UI disabled<br>Loading message displayed |
| **UI Assertions** | `data-testid="loading-spinner"` visible<br>`data-testid="loading-message"` shows message |
| **Data Assertions** | N/A |
| **Test Type** | E2E |
| **Automation Status** | 📝 Manual |

### 12.4 Abort Long Operation

| Field | Value |
|-------|-------|
| **Feature** | Cancel in-progress operation |
| **Priority** | P1 |
| **Preconditions** | Long operation running |
| **Steps** | 1. Click "Abort" button (`data-testid="btn-abort-op"`) |
| **Expected Results** | Operation cancelled<br>UI returns to previous state<br>Toast confirms abort |
| **UI Assertions** | Abort button appears during operation<br>Toast: "Operation cancelled" |
| **Data Assertions** | Abort signal propagated to backend |
| **Test Type** | Integration + E2E |
| **Automation Status** | 📝 Manual |

---

## Summary Statistics

| Category | P0 Features | P1 Features | P2 Features | Total |
|----------|-------------|-------------|-------------|-------|
| Dashboard | 1 | 3 | 0 | 4 |
| Analyser | 11 | 5 | 0 | 16 |
| Lifecycle | 4 | 3 | 0 | 7 |
| Pipeline | 6 | 7 | 0 | 13 |
| Watcher | 1 | 4 | 1 | 6 |
| Export | 2 | 3 | 0 | 5 |
| PowerShell | 0 | 2 | 0 | 2 |
| Python | 0 | 1 | 1 | 2 |
| SQL | 0 | 2 | 0 | 2 |
| Dictionary | 0 | 3 | 1 | 4 |
| Settings | 0 | 4 | 0 | 4 |
| Global UI | 3 | 1 | 0 | 4 |
| **TOTAL** | **28** | **38** | **3** | **69** |

## Automation Roadmap

### Phase 1: Foundation (Weeks 1-2)
- Set up test frameworks
- Create fixtures and golden outputs
- Add data-testid attributes to all components
- Write 50 unit tests (TS + Rust)

### Phase 2: Integration (Weeks 3-4)
- Write 30 integration tests (Rust)
- Test analysis, cleaning, lifecycle, pipeline execution
- Test Tauri command boundary

### Phase 3: E2E Critical Path (Weeks 5-6)
- Automate all 28 P0 features with E2E tests
- Focus on happy path workflows
- Set up CI for P0 tests

### Phase 4: E2E Extended (Weeks 7-10)
- Automate 38 P1 features
- Add error scenario tests
- Full CI integration

### Phase 5: Polish (Weeks 11-12)
- Automate P2 features
- Refactor flaky tests
- Performance optimization
- Documentation finalization

---

**Status**: 📝 All manual (ready for automation kickoff)
**Last Updated**: 2026-01-16
**Owner**: QA/Testing Team
