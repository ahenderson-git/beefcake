export function renderLayout(): string {
  return `
    <div class="layout">
      <aside class="sidebar">
        <div class="sidebar-logo">
          <i class="ph ph-cake"></i>
          beefcake
        </div>
        <nav>
          <button class="nav-item active" data-view="Dashboard" data-testid="nav-dashboard">
            <i class="ph ph-layout"></i> Dashboard
          </button>
          <button class="nav-item" data-view="Analyser" data-testid="nav-analyser">
            <i class="ph ph-chart-bar"></i> Analyser
          </button>
          <button class="nav-item" data-view="Lifecycle" data-testid="nav-lifecycle">
            <i class="ph ph-git-branch"></i> Lifecycle
          </button>
          <button class="nav-item" data-view="Pipeline" data-testid="nav-pipeline">
            <i class="ph ph-flow-arrow"></i> Pipeline
          </button>
          <button class="nav-item" data-view="Watcher" data-testid="nav-watcher">
            <i class="ph ph-eye"></i> Watcher
          </button>
          <button class="nav-item" data-view="Dictionary" data-testid="nav-dictionary">
            <i class="ph ph-book-open"></i> Dictionary
          </button>
          <button class="nav-item" data-view="Integrity" data-testid="nav-integrity">
            <i class="ph ph-shield-check"></i> Integrity
          </button>
          <button class="nav-item" data-view="PowerShell" data-testid="nav-powershell">
            <i class="ph ph-terminal"></i> PowerShell
          </button>
          <button class="nav-item" data-view="Python" data-testid="nav-python">
            <i class="ph ph-code"></i> Python IDE
          </button>
          <button class="nav-item" data-view="SQL" data-testid="nav-sql">
            <i class="ph ph-database"></i> SQL IDE
          </button>
          <button class="nav-item" data-view="Settings" data-testid="nav-settings">
            <i class="ph ph-gear"></i> Settings
          </button>
          <button class="nav-item" data-view="ActivityLog" data-testid="nav-activity-log">
            <i class="ph ph-clock-counter-clockwise"></i> Activity Log
          </button>
          <button class="nav-item" data-view="CLI" data-testid="nav-cli">
            <i class="ph ph-command"></i> CLI Help
          </button>
          <button class="nav-item" data-view="Reference" data-testid="nav-reference">
            <i class="ph ph-book"></i> Reference
          </button>
        </nav>
        <div class="sidebar-footer">
          <div class="status-indicator">
            <div class="status-dot"></div>
            <span>System Ready</span>
          </div>
        </div>
      </aside>
      <div class="main-content">
        <header>
          <h2 id="view-title">Dashboard</h2>
        </header>
        <main id="view-container"></main>
      </div>
      <aside class="ai-sidebar collapsed" id="ai-sidebar">
        <div class="ai-sidebar-content" id="ai-sidebar-container"></div>
        <div class="ai-sidebar-collapsed-tab" id="ai-collapsed-tab">
          <i class="ph ph-robot"></i>
        </div>
      </aside>
    </div>
    <div id="toast-container"></div>
    <div id="modal-container"></div>
  `;
}
