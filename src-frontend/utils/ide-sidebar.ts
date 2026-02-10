/**
 * IDE Sidebar Utilities
 *
 * Handles collapse/expand functionality for IDE column sidebars.
 * This module ensures sidebar toggle functionality works correctly
 * even when the sidebar DOM element is dynamically replaced.
 */

/**
 * Sets up collapse/expand event listeners for the IDE sidebar.
 *
 * This function:
 * - Loads and applies saved collapsed state from localStorage
 * - Attaches click event listeners to collapse button and collapsed tab
 * - Attaches double-click listener to sidebar header
 * - Handles toggling the 'collapsed' class and persisting state
 *
 * **Important**: This function removes and re-adds event listeners to prevent
 * duplicate listeners when called multiple times (e.g., after sidebar DOM updates).
 *
 * @example
 * ```typescript
 * // Call after rendering Python IDE
 * setupIDESidebarToggle();
 *
 * // Call again after dynamically updating sidebar content
 * updateSidebarDisplay(state);
 * setupIDESidebarToggle(); // Re-bind events to new DOM
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

  // Remove any existing event listeners by cloning and replacing the element
  // This prevents duplicate listeners when setupIDESidebarToggle is called multiple times
  const newSidebar = ideSidebar.cloneNode(true) as HTMLElement;
  ideSidebar.replaceWith(newSidebar);

  const toggleSidebar = (): void => {
    newSidebar.classList.toggle('collapsed');
    const collapsed = newSidebar.classList.contains('collapsed');
    localStorage.setItem('ide-sidebar-collapsed', collapsed.toString());
  };

  // Handle collapse button click (uses event delegation for dynamically created buttons)
  newSidebar.addEventListener('click', (e: Event) => {
    const target = e.target as HTMLElement;
    if (target.closest('#ide-collapse-btn') ?? target.closest('#ide-collapsed-tab')) {
      toggleSidebar();
    }
  });

  // Handle double-click on header to toggle
  newSidebar.addEventListener('dblclick', (e: Event) => {
    const target = e.target as HTMLElement;
    if (target.closest('#ide-sidebar-header')) {
      toggleSidebar();
    }
  });
}
