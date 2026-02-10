/**
 * IDE Sidebar Utilities
 *
 * Handles collapse/expand functionality for IDE column sidebars.
 * This module ensures sidebar toggle functionality works correctly
 * even when the sidebar DOM element is dynamically replaced.
 */

// Track if document-level listeners have been attached
let documentListenersAttached = false;

/**
 * Sets up collapse/expand event listeners for the IDE sidebar.
 *
 * This function:
 * - Loads and applies saved collapsed state from localStorage
 * - Attaches click event listeners to collapse button and collapsed tab
 * - Attaches double-click listener to sidebar header
 * - Handles toggling the 'collapsed' class and persisting state
 *
 * **Important**: Uses document-level event delegation so listeners
 * survive DOM replacements when sidebar is dynamically updated.
 *
 * @example
 * ```typescript
 * // Call after rendering Python IDE
 * setupIDESidebarToggle();
 *
 * // Call again after dynamically updating sidebar content
 * updateSidebarDisplay(state);
 * setupIDESidebarToggle(); // Applies saved state to new DOM
 * ```
 */
export function setupIDESidebarToggle(): void {
  const ideSidebar = document.getElementById('ide-sidebar');
  if (!ideSidebar) return;

  // Load saved collapsed state from localStorage
  const isCollapsed = localStorage.getItem('ide-sidebar-collapsed') === 'true';
  if (isCollapsed) {
    ideSidebar.classList.add('collapsed');
  } else {
    ideSidebar.classList.remove('collapsed');
  }

  // Only attach document-level listeners once
  // Event delegation means they work even after sidebar DOM updates
  if (documentListenersAttached) return;

  const toggleSidebar = (): void => {
    const sidebar = document.getElementById('ide-sidebar');
    if (!sidebar) return;

    sidebar.classList.toggle('collapsed');
    const collapsed = sidebar.classList.contains('collapsed');
    localStorage.setItem('ide-sidebar-collapsed', collapsed.toString());
  };

  // Use document-level event delegation so listeners survive DOM replacements
  document.addEventListener('click', (e: Event) => {
    const target = e.target as HTMLElement;
    const sidebar = document.getElementById('ide-sidebar');
    if (!sidebar) return;

    // Check if click is within the sidebar and targets collapse elements
    if (sidebar.contains(target)) {
      if (target.closest('#ide-collapse-btn') ?? target.closest('#ide-collapsed-tab')) {
        toggleSidebar();
      }
    }
  });

  // Handle double-click on header to toggle
  document.addEventListener('dblclick', (e: Event) => {
    const target = e.target as HTMLElement;
    const sidebar = document.getElementById('ide-sidebar');
    if (!sidebar) return;

    if (sidebar.contains(target) && target.closest('#ide-sidebar-header')) {
      toggleSidebar();
    }
  });

  documentListenersAttached = true;
}
