# TypeScript Patterns in Beefcake

This guide explains TypeScript and frontend patterns used in the Beefcake UI.

## Table of Contents
- [TypeScript Basics](#typescript-basics)
- [Async/Await & Promises](#asyncawait--promises)
- [Tauri Bridge Pattern](#tauri-bridge-pattern)
- [Component Architecture](#component-architecture)
- [State Management](#state-management)
- [Event Handling](#event-handling)
- [Type Safety](#type-safety)
- [Common Patterns](#common-patterns)
  - [Event Delegation for Dynamic Elements](#event-delegation-for-dynamic-elements)
  - [Double-Click Events](#double-click-events)
  - [LocalStorage for State Persistence](#localstorage-for-state-persistence)
  - [Markdown Rendering with Security](#markdown-rendering-with-security)

---

## TypeScript Basics

### What is TypeScript?

TypeScript = JavaScript + Type System

```typescript
// JavaScript (dynamic typing)
let appName = "Beefcake";
appName = 123;  // OK, but dangerous

// TypeScript (static typing)
let typedAppName: string = "Beefcake";
typedAppName = 123;  // ERROR: Type 'number' not assignable to 'string'
```

### Core Types

```typescript
// Primitives
let countVal: number = 42;
let nameStr: string = "Beefcake";
let isEnabled: boolean = true;
```

### Interfaces (src-frontend/types.ts)

```typescript
// Define object shape
export interface ColumnSummary {
  name: string;
  kind: string;
  count: number;
  nulls: number;
  stats: ColumnStats;
}

// Use it
function displayColumn(col: ColumnSummary) {
  console.log(`${col.name}: ${col.kind}`);
}
```

### Type Aliases

```typescript
// Union of string literals
export type View = 'Dashboard' | 'Analyser' | 'Settings' | 'SQL';

// Function signature
type AnalysisFn = (path: string) => Promise<AnalysisResponse>;

// Complex type
type Result<T, E> = { ok: true; value: T } | { ok: false; error: E };
```

### Enums (use string literals instead in modern TS)

```typescript
// Old way - enums
enum TextCase {
  None,
  Lowercase,
  Uppercase
}

// Modern way - string literal union
type TextCase = "None" | "Lowercase" | "Uppercase" | "TitleCase";

// With type safety
function formatText(text: string, case_type: TextCase): string {
  switch (case_type) {
    case "Lowercase": return text.toLowerCase();
    case "Uppercase": return text.toUpperCase();
    case "TitleCase": return toTitleCase(text);
    case "None": return text;
    // TypeScript knows we've covered all cases!
  }
}
```

---

## Async/Await & Promises

### Core Concept

JavaScript is single-threaded but can handle multiple operations asynchronously.

### Promises

```typescript
// Promise represents a future value
const promise: Promise<string> = loadData();

// Handle with .then() and .catch()
promise
  .then(data => console.log("Success:", data))
  .catch(error => console.error("Failed:", error));
```

### Async/Await (Modern Approach)

```typescript
// Async function returns a Promise
async function loadAnalysis(path: string): Promise<AnalysisResponse> {
  try {
    // await pauses until Promise resolves
    const data = await api.analyseFile(path);
    return data;
  } catch (error) {
    console.error("Analysis failed:", error);
    throw error;  // Re-throw to caller
  }
}
```

### Examples from Beefcake (src-frontend/main.ts)

#### Handling Multiple Async Operations
```typescript
async function init() {
  try {
    // Run in parallel
    const [config, version] = await Promise.all([
      api.loadAppConfig(),
      api.getAppVersion()
    ]);
    // ...
  } catch (err) {
    console.error('Initialization failed:', err);
  }
}
```

#### Sequential Operations (main.ts:198)
```typescript
async function handleAnalysis(path: string) {
  try {
    // ...
    const response = await api.analyseFile(path);
    // ...
  } catch (err) {
    console.error(`Analysis failed: ${err}`);
  }
}
```

#### Fire-and-Forget Pattern
```typescript
// Don't await - let it run in background
async function createLifecycleDatasetAsync(fileName: string, path: string) {
  try {
    const datasetId = await api.createDataset(fileName, path);
    // ...
  } catch (err) {
    console.error('Failed to create lifecycle dataset:', err);
  }
}
```

### Common Async Patterns

```typescript
// Wait for first to complete
const raceResult = await Promise.race([
  loadFromCache(),
  loadFromNetwork()
]);

// Run all, collect results (even if some fail)
const results = await Promise.allSettled([
  operation1(),
  operation2(),
  operation3()
]);

// Timeout pattern
const timeout = new Promise((_, reject) =>
  setTimeout(() => reject(new Error('Timeout')), 5000)
);
const fastResult = await Promise.race([dataPromise, timeout]);
```

---

## Tauri Bridge Pattern

### Core Concept

Tauri connects TypeScript frontend to Rust backend via IPC (Inter-Process Communication).

### How It Works

```
┌─────────────────┐        IPC         ┌─────────────────┐
│   TypeScript    │ <─────────────────> │      Rust       │
│   (Frontend)    │                     │    (Backend)    │
│                 │                     │                 │
│  invoke("cmd")  │ ──────────────────> │ #[tauri::cmd]  │
│                 │                     │                 │
│  <─ Promise ─── │ <────────────────── │  return Result  │
└─────────────────┘                     └─────────────────┘
```

### Frontend: invoke() (src-frontend/api.ts)

```typescript
import { invoke } from "@tauri-apps/api/core";

export async function analyseFile(path: string): Promise<AnalysisResponse> {
  // invoke calls a Rust function by name
  // Arguments passed as object
  return await invoke("analyze_file", { path });
  //                   ↑ command name   ↑ arguments
}

export async function runPython(
  script: string,
  dataPath?: string,
  configs?: Record<string, ColumnCleanConfig>
): Promise<string> {
  return await invoke("run_python", { script, dataPath, configs });
  // TypeScript property shorthand: { script: script } → { script }
}
```

### Backend: `#[tauri::command]` (src/tauri_app.rs)

```rust
#[tauri::command]
async fn analyze_file(path: String) -> Result<AnalysisResponse, String> {
    // Function name must match invoke() call
    let response = beefcake::analyser::logic::analyze_file(&path)
        .map_err(|e| e.to_string())?;  // Convert Error to String for JSON
    Ok(response)
}

// Register commands in Tauri builder (Rust)
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            analyze_file,
            run_python,
            get_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Type Safety Across the Bridge

```typescript
// Define types matching Rust structs (types.ts)
export interface AnalysisResponse {
  file_name: string;
  row_count: number;
  column_count: number;
  summary: ColumnSummary[];
  correlation_matrix: CorrelationMatrix | null;
  health: FileHealth;
}
```

```rust
// Rust struct (automatically serialized to JSON)
#[derive(Serialize)]
pub struct AnalysisResponse {
    pub file_name: String,
    pub row_count: usize,
    pub column_count: usize,
    pub summary: Vec<ColumnSummary>,
    pub correlation_matrix: Option<CorrelationMatrix>,
    pub health: FileHealth,
}
```

### Error Handling Across Bridge

```typescript
try {
  const result = await api.analyseFile(path);
  console.log("Success:", result);
} catch (error) {
  // error is a string (Rust error converted to String)
  console.error("Backend error:", error);
  showToast(`Analysis failed: ${error}`, 'error');
}
```

---

## Component Architecture

### Pattern: Component Classes

Beefcake uses a class-based component system (not React/Vue).

### Base Component (src-frontend/components/Component.ts)

```typescript
export interface ComponentActions {
  onStateChange: () => void;
  showToast: (msg: string, type?: 'info' | 'error' | 'success') => void;
  runAnalysis: (path: string) => Promise<void>;
  switchView: (view: View) => void;
}

export abstract class Component {
  protected containerId: string;
  protected actions: ComponentActions;

  constructor(containerId: string, actions: ComponentActions) {
    this.containerId = containerId;
    this.actions = actions;
  }

  // Each component implements this
  abstract render(state: AppState): void;

  // Helper to get container element
  protected getContainer(): HTMLElement | null {
    return document.getElementById(this.containerId);
  }
}
```

### Example Component (src-frontend/components/AnalyserComponent.ts)

```typescript
export class AnalyserComponent extends Component {
  render(state: AppState): void {
    const container = this.getContainer();
    if (!container) return;

    // Generate HTML
    container.innerHTML = renderers.renderAnalyser(state);

    // Attach event listeners
    this.attachEventHandlers(state);
  }

  private attachEventHandlers(state: AppState): void {
    // Find button in DOM
    const analyzeBtn = document.getElementById('analyze-btn');
    if (analyzeBtn) {
      analyzeBtn.addEventListener('click', async () => {
        const path = document.getElementById('file-path-input')?.value;
        if (path) {
          // Call action from main app
          await this.actions.runAnalysis(path);
        }
      });
    }
  }
}
```

### Component Lifecycle

```typescript
class BeefcakeApp {
  // 1. Create components once
  private initComponents() {
    this.components = {
      'Dashboard': new DashboardComponent('view-container', actions),
      'Analyser': new AnalyserComponent('view-container', actions),
      // ...
    };
  }

  // 2. Render when state changes
  private render() {
    const component = this.components[this.state.currentView];
    if (component) {
      component.render(this.state);  // Re-renders with new state
    }
  }

  // 3. User interaction triggers state change
  private switchView(view: View) {
    this.state.currentView = view;
    this.render();  // Trigger re-render
  }
}
```

---

## State Management

### Centralized State Pattern (main.ts:29)

```typescript
class BeefcakeApp {
  // Single source of truth
  private state: AppState = {
    version: '0.0.0',
    currentView: 'Dashboard',
    analysisResponse: null,
    expandedRows: new Set(),
    cleaningConfigs: {},
    isLoading: false,
    currentDataset: null,
    selectedColumns: new Set()
  };

  // State changes trigger re-render
  private updateState(changes: Partial<AppState>) {
    this.state = { ...this.state, ...changes };
    this.render();
  }
}
```

### State Flow

```
User Action (click button)
    ↓
Event Handler
    ↓
Update State
    ↓
Call render()
    ↓
Component reads state
    ↓
Update DOM
```

### Immutable State Updates

```typescript
// Bad: mutating state directly
this.state.expandedRows.add(rowId);  // Components won't re-render!

// Good: create new state
this.state = {
  ...this.state,
  expandedRows: new Set([...this.state.expandedRows, rowId])
};
this.render();
```

### Collections in State

```typescript
// Set for unique values
const expandedRows: Set<string> = new Set();
expandedRows.add('row-123');
expandedRows.has('row-123');  // true
expandedRows.delete('row-123');

// Map for key-value pairs
const cleaningConfigs: Map<string, ColumnCleanConfig> = new Map();
cleaningConfigs.set('age', config);
cleaningConfigs.get('age');

// Plain object also works (more JSON-friendly)
const cleaningConfigsObj: Record<string, ColumnCleanConfig> = {};
cleaningConfigsObj['age'] = config;
```

---

## Event Handling

### DOM Events

```typescript
// Add listener
const button = document.getElementById('submit-btn');
button?.addEventListener('click', (event: MouseEvent) => {
  event.preventDefault();  // Stop default behaviour
  handleSubmit();
});

// Remove listener (important to prevent memory leaks)
const handler = () => console.log('clicked');
button?.addEventListener('click', handler);
button?.removeEventListener('click', handler);
```

### Event Delegation Pattern

```typescript
// Instead of attaching to each row
function renderTable(rows: string[]) {
  const html = rows.map(row =>
    `<tr data-id="${row}"><td>${row}</td></tr>`
  ).join('');

  const table = document.getElementById('data-table');
  table.innerHTML = html;

  // Attach ONE listener to table
  table?.addEventListener('click', (e: MouseEvent) => {
    const target = e.target as HTMLElement;
    const row = target.closest('tr');
    if (row) {
      const id = row.dataset.id;
      handleRowClick(id);
    }
  });
}
```

### Custom Event Pattern

```typescript
// Emit custom event
const event = new CustomEvent('dataLoaded', {
  detail: { datasetId: 'abc-123' }
});
document.dispatchEvent(event);

// Listen for custom event
document.addEventListener('dataLoaded', (e: Event) => {
  const customEvent = e as CustomEvent;
  console.log('Dataset loaded:', customEvent.detail.datasetId);
});
```

---

## Type Safety

### Optional Chaining

```typescript
// Old way (verbose)
const oldName = user && user.profile && user.profile.name;

// New way (optional chaining)
const newName = user?.profile?.name;
```

### Nullish Coalescing

```typescript
// Old way (falsy values are problematic)
const oldCount = value || 0;  // Problem: value=0 → 0, value="" → 0

// New way (only null/undefined trigger default)
const newCount = value ?? 0;  // value=0 → 0, value=null → 0
```

### Type Guards

```typescript
function processData(data: string | number) {
  // TypeScript doesn't know which type here
  if (typeof data === 'string') {
    // Now TypeScript knows it's a string
    return data.toUpperCase();
  } else {
    // TypeScript knows it's a number
    return data.toFixed(2);
  }
}
```

### Type Assertions

```typescript
// Tell TypeScript "trust me, I know the type"
const inputElement = document.getElementById('name') as HTMLInputElement;
inputElement.value = 'text';  // OK, TS knows it's an input

// Alternative syntax (different element to avoid redeclaration)
const emailInput = <HTMLInputElement>document.getElementById('email');

// Be careful! Can cause runtime errors if wrong
const wrongValue = "hello" as unknown as number;  // Double cast often needed for incompatible types
```

### Generics

```typescript
// Generic function
function first<T>(array: T[]): T | undefined {
  return array[0];
}

const num = first([1, 2, 3]);     // num: number | undefined
const str = first(['a', 'b']);     // str: string | undefined

// Generic interface
interface Response<T> {
  data: T;
  status: number;
}

// Example: Response<User> type
interface User {
  name: string;
}

// When you use Response<User>, TypeScript knows:
// - response.data is of type User
// - response.status is of type number
type UserResponse = Response<User>;
// This is equivalent to: { data: User; status: number }
```

---

## Common Patterns

### Debouncing (delay execution)

```typescript
function debounce<T extends any[]>(
  fn: (...args: T) => any,
  delay: number
): (...args: T) => void {
  let timeoutId: number;
  return (...args: T) => {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => fn(...args), delay);
  };
}

// Usage: only call after 500ms of no typing
const searchInput = document.getElementById('search');
const debouncedSearch = debounce((query: string) => {
  api.search(query);
}, 500);

searchInput?.addEventListener('input', (e) => {
  debouncedSearch((e.target as HTMLInputElement).value);
});
```

### Loading State Pattern

```typescript
async function loadData() {
  try {
    this.state.isLoading = true;
    this.state.loadingMessage = 'Loading data...';
    this.render();  // Show loading spinner

    const data = await api.fetchData();

    this.state.data = data;
    this.state.isLoading = false;
    this.render();  // Show data
  } catch (error) {
    this.state.isLoading = false;
    this.state.error = error.message;
    this.render();  // Show error
  }
}
```

### Builder Pattern

```typescript
class QueryBuilder {
  private query: string[] = [];

  select(...columns: string[]): this {
    this.query.push(`SELECT ${columns.join(', ')}`);
    return this;
  }

  from(table: string): this {
    this.query.push(`FROM ${table}`);
    return this;
  }

  where(condition: string): this {
    this.query.push(`WHERE ${condition}`);
    return this;
  }

  build(): string {
    return this.query.join(' ');
  }
}

// Usage (method chaining)
const sql = new QueryBuilder()
  .select('name', 'age')
  .from('users')
  .where('age > 18')
  .build();
```

### Event Delegation for Dynamic Elements

**Problem**: When elements are created dynamically after page load, directly attached event listeners won't work.

**Solution**: Use event delegation by attaching the listener to a parent element that exists at load time.

**Example from AI Assistant** (`main.ts:setupAISidebarToggle`):
```typescript
// ❌ This won't work - button doesn't exist yet
const collapseBtn = document.getElementById('ai-collapse-btn');
collapseBtn?.addEventListener('click', toggleSidebar);  // null!

// ✅ This works - use event delegation
const aiSidebar = document.getElementById('ai-sidebar');  // Exists at load
aiSidebar?.addEventListener('click', (e: MouseEvent) => {
  const target = e.target as HTMLElement;
  // Check if click was on the dynamically created button
  if (target.closest('#ai-collapse-btn')) {
    toggleSidebar();
  }
});
```

**Why this works**:
- `ai-sidebar` exists when page loads
- Clicks on child elements "bubble up" to parent
- We check if the click originated from our dynamic element
- `closest()` traverses up the DOM to find matching element

**Use cases in Beefcake**:
- AI Assistant collapse button (created by `AIAssistantComponent.render()`)
- Pipeline step cards (created dynamically when steps added)
- Modal dialog buttons (created when modal opens)

### Double-Click Events

**Use case**: Quick toggle gestures for power users.

**Example from AI Assistant**:
```typescript
// Collapse AI sidebar by double-clicking the header
aiSidebar.addEventListener('dblclick', (e: MouseEvent) => {
  const target = e.target as HTMLElement;
  if (target.closest('#ai-sidebar-header')) {
    toggleSidebar();
  }
});
```

**Best practices**:
- Use for optional shortcuts, not primary interactions
- Combine with visible button for discoverability
- Add visual feedback (cursor change) to hint at functionality

```html
<div class="ai-sidebar-header" id="ai-sidebar-header">AI Assistant Header</div>
<button id="ai-collapse-btn">Collapse</button>
<div id="ai-collapsed-tab">🤖</div>

<style>
.ai-sidebar-header {
  cursor: pointer;  /* Hints that header is clickable */
  user-select: none;  /* Prevents text selection on double-click */
}
</style>
```

### LocalStorage for State Persistence

**Use case**: Remember UI preferences across sessions.

**Example from AI Assistant**:
```typescript
// Save collapsed state
function toggleSidebar() {
  aiSidebar.classList.toggle('collapsed');
  const collapsed = aiSidebar.classList.contains('collapsed');
  localStorage.setItem('ai-sidebar-collapsed', collapsed.toString());
}

// Restore on load
function setupAISidebarToggle() {
  const isCollapsed = localStorage.getItem('ai-sidebar-collapsed') === 'true';
  if (!isCollapsed) {
    aiSidebar.classList.remove('collapsed');
  }
}
```

**Type-safe localStorage helpers**:
```typescript
// Generic typed localStorage wrapper
function getStoredValue<T>(key: string, defaultValue: T): T {
  const stored = localStorage.getItem(key);
  if (stored === null) return defaultValue;
  try {
    return JSON.parse(stored) as T;
  } catch {
    return defaultValue;
  }
}

function setStoredValue<T>(key: string, value: T): void {
  localStorage.setItem(key, JSON.stringify(value));
}

// Usage
const collapsed = getStoredValue('ai-sidebar-collapsed', false);
setStoredValue('ai-sidebar-collapsed', true);
```

### Markdown Rendering with Security

**Use case**: Display user-generated or AI-generated content safely.

**Example from AI Assistant** (`AIAssistantComponent.ts:formatContent`):
```typescript
function formatContent(content: string): string {
  // Basic markdown support with XSS protection
  const formatted = content
    // Code blocks first (to prevent interference)
    .replace(/```(\w+)?\n([\s\S]*?)```/g, '<pre><code>$2</code></pre>')
    // Inline code
    .replace(/`([^`]+)`/g, '<code>$1</code>')
    // Links - IMPORTANT: always open in new tab with security attributes
    .replace(/\[([^\]]+)\]\(([^)]+)\)/g,
      '<a href="$2" target="_blank" rel="noopener noreferrer">$1</a>')
    // Bold
    .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
    // Italic
    .replace(/\*([^*]+)\*/g, '<em>$1</em>')
    // Line breaks
    .replace(/\n/g, '<br>');

  return formatted;
}
```

**Security notes**:
- `target="_blank"`: Opens links in new tab (prevents navigation away)
- `rel="noopener noreferrer"`: Prevents access to `window.opener` (security)
- Order matters: Process code blocks first to avoid double-processing
- For production: Consider using a library like `marked` + `DOMPurify`

**Testing markdown rendering**:
```typescript
// Test cases
const testCases = [
  ['**bold text**', '<strong>bold text</strong>'],
  ['[link](https://example.com)', '<a href="https://example.com" target="_blank" rel="noopener noreferrer">link</a>'],
  ['`code`', '<code>code</code>'],
  ['```\ncode block\n```', '<pre><code>code block</code></pre>'],
];

testCases.forEach(([input, expected]) => {
  const result = formatContent(input);
  console.assert(result.includes(expected), `Failed: ${input}`);
});
```

---

## Resources

### Official
- [TypeScript Handbook](https://www.typescriptlang.org/docs/handbook/intro.html)
- [TypeScript Playground](https://www.typescriptlang.org/play)
- [Tauri Docs](https://tauri.app/v1/guides/)

### Tools
- **VS Code**: Best TypeScript IDE
- **TypeScript ESLint**: Linting rules
- **Prettier**: Code formatting

### Beefcake Specific
- Read `src-frontend/types.ts` for all type definitions
- Study `src-frontend/main.ts` for architecture
- Look at component files for patterns

---

Happy coding! 🚀
