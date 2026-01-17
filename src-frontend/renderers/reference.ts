import { DocFileMetadata } from '../types';

/**
 * Render the documentation viewer with sidebar navigation and content area
 */
export function renderDocumentationViewer(
  docs: DocFileMetadata[],
  currentDoc: string | null,
  searchQuery: string
): string {
  // Group docs by category
  const categories = new Map<string, DocFileMetadata[]>();
  docs.forEach((doc) => {
    if (!categories.has(doc.category)) {
      categories.set(doc.category, []);
    }
    categories.get(doc.category)!.push(doc);
  });

  // Sort categories in a logical order
  const categoryOrder = [
    'Getting Started',
    'Guide',
    'Reference',
    'Architecture',
    'Learning',
    'Development',
    'Planning',
  ];

  const sortedCategories = Array.from(categories.keys()).sort((a, b) => {
    const aIndex = categoryOrder.indexOf(a);
    const bIndex = categoryOrder.indexOf(b);
    if (aIndex === -1 && bIndex === -1) return a.localeCompare(b);
    if (aIndex === -1) return 1;
    if (bIndex === -1) return -1;
    return aIndex - bIndex;
  });

  return `
    <div class="doc-viewer-container">
      <!-- Left Sidebar -->
      <div class="doc-sidebar">
        <div class="doc-sidebar-header">
          <h3><i class="ph ph-books"></i> Documentation</h3>
        </div>

        <!-- Search Box -->
        <div class="doc-search">
          <i class="ph ph-magnifying-glass"></i>
          <input
            type="text"
            id="doc-search-input"
            placeholder="Search documentation..."
            value="${searchQuery}"
          />
        </div>

        <!-- Navigation Tree -->
        <div class="doc-nav-tree">
          ${sortedCategories
            .map((category) => {
              const categoryDocs = categories.get(category) || [];
              return `
                <div class="doc-category">
                  <div class="doc-category-header">
                    <i class="ph ph-caret-right category-icon"></i>
                    <span>${category}</span>
                    <span class="doc-count">${categoryDocs.length}</span>
                  </div>
                  <div class="doc-category-items">
                    ${categoryDocs
                      .map(
                        (doc) => `
                      <div
                        class="doc-nav-item ${doc.path === currentDoc ? 'active' : ''}"
                        data-doc-path="${doc.path}"
                      >
                        <i class="ph ph-file-text"></i>
                        <span>${doc.title}</span>
                      </div>
                    `
                      )
                      .join('')}
                  </div>
                </div>
              `;
            })
            .join('')}
        </div>
      </div>

      <!-- Main Content Area -->
      <div class="doc-content">
        <!-- Breadcrumb -->
        <div class="doc-breadcrumb" id="doc-breadcrumb">
          <span class="breadcrumb-item">Documentation</span>
        </div>

        <!-- Content -->
        <div class="doc-content-area" id="doc-content-area">
          <div class="doc-placeholder">
            <i class="ph ph-book-open"></i>
            <p>Select a document from the sidebar to view it</p>
          </div>
        </div>
      </div>
    </div>
  `;
}

/**
 * Render a loading state for documentation
 */
export function renderLoadingState(message: string): string {
  return `
    <div class="doc-loading-container">
      <div class="loading-spinner"></div>
      <p>${message}</p>
    </div>
  `;
}

/**
 * Render an error state for documentation
 */
export function renderErrorState(title: string, detail: string): string {
  return `
    <div class="doc-error-container">
      <i class="ph ph-warning-circle"></i>
      <h3>${title}</h3>
      <p class="error-detail">${detail}</p>
    </div>
  `;
}
