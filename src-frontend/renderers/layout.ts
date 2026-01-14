export function renderLayout(): string {
  return `
    <div class="layout">
      <aside class="sidebar">
        <div class="sidebar-logo">
          <i class="ph ph-cake"></i>
          beefcake
        </div>
        <nav>
          <button class="nav-item active" data-view="Dashboard">
            <i class="ph ph-layout"></i> Dashboard
          </button>
          <button class="nav-item" data-view="Analyser">
            <i class="ph ph-chart-bar"></i> Analyser
          </button>
          <button class="nav-item" data-view="Lifecycle">
            <i class="ph ph-git-branch"></i> Lifecycle
          </button>
          <button class="nav-item" data-view="Pipeline">
            <i class="ph ph-flow-arrow"></i> Pipeline
          </button>
          <button class="nav-item" data-view="Watcher">
            <i class="ph ph-eye"></i> Watcher
          </button>
          <button class="nav-item" data-view="PowerShell">
            <i class="ph ph-terminal"></i> PowerShell
          </button>
          <button class="nav-item" data-view="Python">
            <i class="ph ph-code"></i> Python IDE
          </button>
          <button class="nav-item" data-view="SQL">
            <i class="ph ph-database"></i> SQL IDE
          </button>
          <button class="nav-item" data-view="Settings">
            <i class="ph ph-gear"></i> Settings
          </button>
          <button class="nav-item" data-view="ActivityLog">
            <i class="ph ph-clock-counter-clockwise"></i> Activity Log
          </button>
          <button class="nav-item" data-view="CLI">
            <i class="ph ph-command"></i> CLI Help
          </button>
          <button class="nav-item" data-view="Reference">
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
    </div>
    <div id="toast-container"></div>
  `;
}
