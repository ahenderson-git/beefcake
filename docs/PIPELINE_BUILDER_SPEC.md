# Pipeline Builder UI - Design Specification

## Overview

The Pipeline Builder UI will provide a graphical interface for creating, editing, and managing data transformation pipelines. Currently, all pipeline functionality exists in the backend (5 Tauri commands) but has no UI exposure.

## Current Backend Capabilities

From `src/tauri_app.rs`, these commands are available:

1. `execute_pipeline` - Execute a pipeline specification
2. `save_pipeline` - Save pipeline to JSON file
3. `load_pipeline` - Load pipeline from JSON file
4. `list_pipelines` - List all saved pipelines
5. `validate_pipeline` - Validate pipeline specification

## User Stories

### Primary Workflows

1. **Create New Pipeline**
   - User clicks "New Pipeline" button
   - User adds transformation steps from a palette
   - User configures each step with parameters
   - User previews results at each stage
   - User saves pipeline with descriptive name

2. **Execute Existing Pipeline**
   - User browses saved pipelines
   - User selects a pipeline to view
   - User sees pipeline steps and configuration
   - User executes pipeline on selected dataset
   - User views execution results and any errors

3. **Edit Pipeline**
   - User opens existing pipeline
   - User adds/removes/reorders steps
   - User modifies step parameters
   - User validates changes
   - User saves updated pipeline

4. **Pipeline Library Management**
   - User views all saved pipelines
   - User searches/filters pipelines
   - User duplicates pipeline as template
   - User exports/imports pipeline JSON
   - User deletes unused pipelines

## Component Architecture

### Main Component: `PipelineBuilder`

**Location**: `src-frontend/components/PipelineBuilder.ts`

**State Management**:
```typescript
interface PipelineBuilderState {
    // Current pipeline being edited
    currentPipeline: PipelineSpec | null;

    // List of saved pipelines
    savedPipelines: PipelineInfo[];

    // Available transformation types
    availableSteps: StepDefinition[];

    // Execution state
    isExecuting: boolean;
    executionProgress: number;
    executionError: string | null;

    // Preview data for current step
    previewData: PreviewResult | null;

    // UI state
    selectedStepIndex: number | null;
    viewMode: 'library' | 'editor' | 'executor';
}
```

### Sub-Components

1. **PipelineLibrary** (`components/PipelineLibrary.ts`)
   - List view of saved pipelines
   - Search and filter functionality
   - Preview/Edit/Delete/Execute actions

2. **PipelineEditor** (`components/PipelineEditor.ts`)
   - Step palette (drag-and-drop or click-to-add)
   - Pipeline canvas (visual step representation)
   - Step configuration panel
   - Validation feedback

3. **StepPalette** (`components/StepPalette.ts`)
   - Categorised list of available transformations
   - Search functionality
   - Descriptions and parameter info

4. **StepCard** (`components/StepCard.ts`)
   - Visual representation of a pipeline step
   - Edit/Delete/Reorder controls
   - Configuration summary
   - Validation status indicator

5. **StepConfigPanel** (`components/StepConfigPanel.ts`)
   - Dynamic form based on step type
   - Parameter inputs with validation
   - Help text and examples
   - Real-time validation feedback

6. **PipelineExecutor** (`components/PipelineExecutor.ts`)
   - Pipeline execution controls
   - Progress indicator
   - Step-by-step execution log
   - Result preview

## Data Flow

```
┌─────────────────┐
│  PipelineBuilder │
└────────┬─────────┘
         │
    ┌────┴────┐
    │         │
    ▼         ▼
┌─────────┐ ┌──────────┐
│ Library │ │  Editor  │
└────┬────┘ └────┬─────┘
     │           │
     ├───────────┴──────────────┐
     │                          │
     ▼                          ▼
┌────────────┐          ┌──────────────┐
│ list_      │          │ save_        │
│ pipelines  │          │ pipeline     │
└────────────┘          └──────────────┘
     │                          │
     └────────┬─────────────────┘
              ▼
     ┌─────────────────┐
     │  load_pipeline  │
     └────────┬─────────┘
              │
              ▼
     ┌─────────────────────┐
     │ validate_pipeline   │
     └────────┬────────────┘
              │
              ▼
     ┌─────────────────────┐
     │ execute_pipeline    │
     └─────────────────────┘
```

## UI Layout

### Library View

```
┌──────────────────────────────────────────────────────┐
│  Pipeline Builder                      [+ New] [⚙]   │
├──────────────────────────────────────────────────────┤
│                                                      │
│  🔍 Search pipelines...                              │
│                                                      │
│  ┌──────────────────────────────────────────────┐   │
│  │ 📄 Data Cleaning Pipeline           Edit  ⚙  │   │
│  │ Created: 2026-01-10                           │   │
│  │ Steps: 5 | Last run: 2026-01-12              │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
│  ┌──────────────────────────────────────────────┐   │
│  │ 📄 Feature Engineering                Edit  ⚙ │   │
│  │ Created: 2026-01-08                           │   │
│  │ Steps: 8 | Last run: Never                   │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
│  ┌──────────────────────────────────────────────┐   │
│  │ 📄 Export Preparation              Edit  ⚙   │   │
│  │ Created: 2026-01-05                           │   │
│  │ Steps: 3 | Last run: 2026-01-11              │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
└──────────────────────────────────────────────────────┘
```

### Editor View

```
┌──────────────────────────────────────────────────────────────┐
│  ← Back to Library    Data Cleaning Pipeline    [Save] [⚙]   │
├──────────────┬───────────────────────────────────────────────┤
│              │                                               │
│ Step Palette │  Pipeline Canvas                              │
│              │                                               │
│ 🔍 Search... │  ┌──────────────────────┐                     │
│              │  │ 1. Remove Nulls      │ ⚙ 🗑                │
│ ▼ Data Clean │  │ Columns: all         │                     │
│  • Remove    │  └──────────────────────┘                     │
│    Nulls     │           ▼                                   │
│  • Fill      │  ┌──────────────────────┐                     │
│    Missing   │  │ 2. Normalize Values  │ ⚙ 🗑                │
│  • Dedupe    │  │ Columns: amount      │                     │
│              │  └──────────────────────┘                     │
│ ▼ Transform  │           ▼                                   │
│  • Filter    │  ┌──────────────────────┐                     │
│  • Select    │  │ 3. Filter Rows       │ ⚙ 🗑                │
│  • Sort      │  │ Condition: amt > 0   │                     │
│  • Group     │  └──────────────────────┘                     │
│              │           ▼                                   │
│ ▼ Create     │  [+ Add Step]                                 │
│  • Column    │                                               │
│  • Formula   │                                               │
│              │                                               │
├──────────────┴───────────────────────────────────────────────┤
│ Step Configuration: Remove Nulls                             │
│                                                              │
│ Strategy: ● Remove rows  ○ Fill with value  ○ Forward fill  │
│ Columns:  ☑ All columns                                      │
│                                                              │
│ [Preview] [Validate]                                         │
└──────────────────────────────────────────────────────────────┘
```

### Executor View

```
┌──────────────────────────────────────────────────────┐
│  ← Back    Execute: Data Cleaning Pipeline    [▶]    │
├──────────────────────────────────────────────────────┤
│                                                      │
│  Input Dataset: sales_data.csv (125,432 rows)       │
│  Output: [create_new_version]                        │
│                                                      │
│  ┌──────────────────────────────────────────────┐   │
│  │ Execution Progress          [Abort]          │   │
│  │                                               │   │
│  │ ████████████████░░░░░░░░░░░░░ 60%           │   │
│  │                                               │   │
│  │ ✓ Step 1: Remove Nulls      (2.3s)          │   │
│  │   Removed 1,234 rows with nulls              │   │
│  │                                               │   │
│  │ ✓ Step 2: Normalize Values  (5.1s)          │   │
│  │   Normalized 3 columns                       │   │
│  │                                               │   │
│  │ ⚙ Step 3: Filter Rows       (running...)    │   │
│  │                                               │   │
│  │ ⏸ Step 4: Group By          (pending)        │   │
│  │                                               │   │
│  │ ⏸ Step 5: Sort              (pending)        │   │
│  │                                               │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
│  Result Preview (after Step 2):                     │
│  ┌──────────────────────────────────────────────┐   │
│  │ id │ amount   │ date       │ status          │   │
│  ├────┼──────────┼────────────┼─────────────────┤   │
│  │ 1  │ 0.85     │ 2026-01-01 │ completed       │   │
│  │ 2  │ 0.92     │ 2026-01-02 │ completed       │   │
│  │ 3  │ 0.73     │ 2026-01-03 │ pending         │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
└──────────────────────────────────────────────────────┘
```

## API Integration

### TypeScript API Functions

**New file**: `src-frontend/api-pipeline.ts`

```typescript
/**
 * Pipeline-specific API functions.
 *
 * Wraps Tauri commands for pipeline management and execution.
 */

import { invoke } from '@tauri-apps/api/core';

/**
 * Information about a saved pipeline.
 */
export interface PipelineInfo {
    name: string;
    path: string;
    created: string;
    modified: string;
    stepCount: number;
}

/**
 * Pipeline specification with transformation steps.
 */
export interface PipelineSpec {
    name: string;
    description?: string;
    steps: PipelineStep[];
}

/**
 * A single transformation step in a pipeline.
 */
export interface PipelineStep {
    type: string;
    params: Record<string, unknown>;
}

/**
 * Result of pipeline execution.
 */
export interface ExecutionResult {
    success: boolean;
    outputPath?: string;
    rowCount?: number;
    columnCount?: number;
    error?: string;
    stepResults?: StepResult[];
}

export interface StepResult {
    stepIndex: number;
    success: boolean;
    duration: number;
    message?: string;
}

/**
 * Lists all saved pipeline specifications.
 *
 * @returns Promise resolving to array of pipeline info
 */
export async function listPipelines(): Promise<PipelineInfo[]> {
    return invoke<PipelineInfo[]>('list_pipelines');
}

/**
 * Loads a pipeline specification from file.
 *
 * @param path - Path to pipeline JSON file
 * @returns Promise resolving to pipeline specification
 */
export async function loadPipeline(path: string): Promise<PipelineSpec> {
    return invoke<PipelineSpec>('load_pipeline', { path });
}

/**
 * Saves a pipeline specification to file.
 *
 * @param path - Path where to save pipeline
 * @param spec - Pipeline specification to save
 * @returns Promise resolving to saved path
 */
export async function savePipeline(
    path: string,
    spec: PipelineSpec
): Promise<string> {
    return invoke<string>('save_pipeline', { path, spec });
}

/**
 * Validates a pipeline specification.
 *
 * @param spec - Pipeline specification to validate
 * @returns Promise resolving to validation result
 */
export async function validatePipeline(
    spec: PipelineSpec
): Promise<{ valid: boolean; errors?: string[] }> {
    return invoke('validate_pipeline', { spec });
}

/**
 * Executes a pipeline on a dataset.
 *
 * @param spec - Pipeline specification to execute
 * @param inputPath - Path to input dataset
 * @param outputPath - Optional path for output dataset
 * @returns Promise resolving to execution result
 */
export async function executePipeline(
    spec: PipelineSpec,
    inputPath: string,
    outputPath?: string
): Promise<ExecutionResult> {
    return invoke<ExecutionResult>('execute_pipeline', {
        spec,
        inputPath,
        outputPath,
    });
}
```

## Step Type Definitions

Common transformation steps that should be available:

### Data Cleaning

- **Remove Nulls**: Remove or fill null values
  - Params: `columns`, `strategy` (remove_rows | fill_value | forward_fill)
  - Example: `{ "columns": ["amount"], "strategy": "remove_rows" }`

- **Deduplicate**: Remove duplicate rows
  - Params: `columns` (subset to check), `keep` (first | last)
  - Example: `{ "columns": ["id"], "keep": "first" }`

- **Fill Missing**: Fill missing values with strategy
  - Params: `columns`, `value` | `strategy` (mean | median | mode)
  - Example: `{ "columns": ["age"], "strategy": "median" }`

### Transformation

- **Filter Rows**: Keep only rows matching condition
  - Params: `column`, `operator`, `value`
  - Example: `{ "column": "amount", "operator": ">", "value": 0 }`

- **Select Columns**: Keep only specified columns
  - Params: `columns`
  - Example: `{ "columns": ["id", "name", "amount"] }`

- **Rename Columns**: Rename one or more columns
  - Params: `mapping` (old -> new)
  - Example: `{ "mapping": { "amt": "amount", "dt": "date" } }`

- **Sort**: Sort by columns
  - Params: `columns`, `descending`
  - Example: `{ "columns": ["date", "amount"], "descending": [false, true] }`

- **Group By**: Aggregate data by groups
  - Params: `by`, `aggregations`
  - Example: `{ "by": ["category"], "aggregations": { "amount": "sum" } }`

### Column Creation

- **Add Column**: Create new column with expression
  - Params: `name`, `expression`
  - Example: `{ "name": "total", "expression": "amount * quantity" }`

- **Cast Type**: Change column data type
  - Params: `column`, `dtype`
  - Example: `{ "column": "date", "dtype": "datetime" }`

## Implementation Phases

### Phase 1: Foundation (Week 1)
- [ ] Create `api-pipeline.ts` with all API wrappers
- [ ] Create basic `PipelineBuilder` component shell
- [ ] Create `PipelineLibrary` component
- [ ] Implement list/load/display functionality
- [ ] Add navigation to main app

### Phase 2: Pipeline Editor (Week 2)
- [ ] Create `PipelineEditor` component
- [ ] Create `StepPalette` with basic transformations
- [ ] Create `StepCard` for visual representation
- [ ] Implement add/remove/reorder steps
- [ ] Add save functionality

### Phase 3: Configuration (Week 3)
- [ ] Create `StepConfigPanel` with dynamic forms
- [ ] Implement validation logic
- [ ] Add help text and examples
- [ ] Create preview functionality

### Phase 4: Execution (Week 4)
- [ ] Create `PipelineExecutor` component
- [ ] Implement progress tracking
- [ ] Add step-by-step logging
- [ ] Create result preview
- [ ] Add abort functionality

### Phase 5: Polish (Week 5)
- [ ] Add search/filter to library
- [ ] Implement drag-and-drop for steps
- [ ] Add export/import pipeline JSON
- [ ] Add pipeline templates
- [ ] Write user documentation

## Backend Requirements

The backend commands are already implemented, but may need enhancements:

1. **Progress Callbacks**: Modify `execute_pipeline` to emit progress events
2. **Preview Mode**: Add ability to execute pipeline up to specific step
3. **Step Catalog**: Consider adding command to list available step types
4. **Validation Details**: Enhance `validate_pipeline` to return detailed errors

## Testing Strategy

### Unit Tests
- Test each component in isolation
- Mock Tauri API calls
- Test state management logic
- Validate form inputs

### Integration Tests
- Test full pipeline creation workflow
- Test pipeline execution flow
- Test error handling
- Test validation logic

### E2E Tests
- Create pipeline from scratch
- Edit existing pipeline
- Execute pipeline on real data
- Handle execution errors

## Success Metrics

1. **User can create a pipeline in < 5 minutes**
2. **Pipeline execution progress is clearly visible**
3. **Validation errors are actionable**
4. **Library can handle 50+ pipelines without lag**
5. **Step configuration is intuitive (< 3 clicks)**

## Documentation Needs

1. **User Guide**: How to create and execute pipelines
2. **Step Reference**: Documentation for each transformation type
3. **Pipeline JSON Format**: Schema documentation
4. **Examples**: Sample pipelines for common tasks

## Open Questions

1. Should pipelines support branching/conditional steps?
2. Should we add visual preview of data changes at each step?
3. Should pipelines be shareable/exportable as templates?
4. Should we support scheduling pipeline execution?
5. Should we integrate with dataset lifecycle stages?

---

**Next Steps**: Review this specification and decide on implementation timeline. Consider starting with Phase 1 to establish foundation.
