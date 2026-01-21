import DOMPurify from 'dompurify';
import { marked } from 'marked';

import * as api from '../api';
import * as renderers from '../renderers';
import { AppState, DocFileMetadata } from '../types';

import { Component, ComponentActions } from './Component';

/**
 * ReferenceComponent - Documentation viewer with markdown rendering
 *
 * Displays application documentation in a two-panel layout:
 * - Left sidebar: Hierarchical navigation of documentation files grouped by category
 * - Right panel: Rendered markdown content with syntax highlighting
 *
 * Features:
 * - Secure markdown rendering with XSS protection (DOMPurify)
 * - Search functionality across all documentation
 * - Category-based organisation
 * - Deep linking to specific documentation files
 */
export class ReferenceComponent extends Component {
  private docs: DocFileMetadata[] = [];
  private currentDoc: string | null = null;
  private searchQuery = '';

  constructor(containerId: string, actions: ComponentActions) {
    super(containerId, actions);
  }

  async render(state: AppState): Promise<void> {
    const container = this.getContainer();

    // Show loading state
    container.innerHTML = renderers.renderLoadingState('Loading documentation...');

    try {
      // Fetch documentation list
      this.docs = await api.listDocumentationFiles();

      // Render the documentation viewer
      container.innerHTML = renderers.renderDocumentationViewer(
        this.docs,
        this.currentDoc,
        this.searchQuery
      );

      // Load the default or selected document
      if (this.currentDoc) {
        await this.loadDocument(this.currentDoc);
      } else if (this.docs.length > 0 && this.docs[0]) {
        // Load the first doc (README.md) by default
        await this.loadDocument(this.docs[0].path);
      }

      this.bindEvents(state);
    } catch (err) {
      container.innerHTML = renderers.renderErrorState('Failed to load documentation', String(err));
    }
  }

  override bindEvents(_state: AppState): void {
    // Document navigation links
    const docLinks = document.querySelectorAll<HTMLElement>('.doc-nav-item');
    docLinks.forEach(link => {
      link.addEventListener('click', e => {
        e.preventDefault();
        const docPath = link.dataset.docPath;
        if (docPath) {
          void this.loadDocument(docPath);

          // Update active state
          docLinks.forEach(l => l.classList.remove('active'));
          link.classList.add('active');
        }
      });
    });

    // Search input
    const searchInput = document.getElementById('doc-search-input') as HTMLInputElement | null;
    if (searchInput) {
      searchInput.addEventListener('input', e => {
        this.searchQuery = (e.target as HTMLInputElement).value;
        this.filterDocs();
      });
    }

    // Category collapsing
    const categoryHeaders = document.querySelectorAll<HTMLElement>('.doc-category-header');
    categoryHeaders.forEach(header => {
      header.addEventListener('click', () => {
        const category = header.closest('.doc-category');
        category?.classList.toggle('collapsed');
      });
    });
  }

  /**
   * Load and render a documentation file
   */
  private async loadDocument(docPath: string): Promise<void> {
    this.currentDoc = docPath;

    const contentArea = document.getElementById('doc-content-area');
    if (!contentArea) return;

    // Show loading state
    contentArea.innerHTML = '<div class="doc-loading">Loading document...</div>';

    try {
      // Fetch markdown content
      const markdown = await api.readDocumentationFile(docPath);

      // Configure marked options
      marked.setOptions({
        breaks: true,
        gfm: true, // GitHub Flavored Markdown
      });

      // Convert markdown to HTML
      const rawHtml = await marked(markdown);

      // Sanitize HTML to prevent XSS
      const cleanHtml = DOMPurify.sanitize(rawHtml);

      // Update content
      contentArea.innerHTML = `<div class="markdown-content">${cleanHtml}</div>`;

      // Update breadcrumb
      const docMeta = this.docs.find(d => d.path === docPath);
      const breadcrumb = document.getElementById('doc-breadcrumb');
      if (breadcrumb && docMeta) {
        breadcrumb.innerHTML = `
          <span class="breadcrumb-item">${docMeta.category}</span>
          <i class="ph ph-caret-right"></i>
          <span class="breadcrumb-item">${docMeta.title}</span>
        `;
      }

      // Smooth scroll to top
      contentArea.scrollTo({ top: 0, behavior: 'smooth' });

      // Add click handlers for internal links
      this.handleInternalLinks(contentArea);
    } catch (err) {
      contentArea.innerHTML = `
        <div class="doc-error">
          <i class="ph ph-warning-circle"></i>
          <p>Failed to load documentation file: ${docPath}</p>
          <p class="error-detail">${String(err)}</p>
        </div>
      `;
    }
  }

  /**
   * Handle clicks on internal documentation links
   */
  private handleInternalLinks(contentArea: HTMLElement): void {
    const links = contentArea.querySelectorAll<HTMLAnchorElement>('a');
    links.forEach(link => {
      const href = link.getAttribute('href');
      if (href && href.endsWith('.md')) {
        link.addEventListener('click', e => {
          e.preventDefault();
          // Extract just the filename if it's a relative link
          const filename = href.split('/').pop() ?? href;
          const docMeta = this.docs.find(d => d.path === filename);
          if (docMeta) {
            void this.loadDocument(docMeta.path);
          }
        });
      }
    });
  }

  /**
   * Filter documentation list based on search query
   */
  private filterDocs(): void {
    const query = this.searchQuery.toLowerCase();

    // Get all doc items and categories
    const docItems = document.querySelectorAll<HTMLElement>('.doc-nav-item');
    const categories = document.querySelectorAll<HTMLElement>('.doc-category');

    docItems.forEach(item => {
      const title = item.textContent?.toLowerCase() ?? '';
      if (query === '' || title.includes(query)) {
        item.style.display = '';
      } else {
        item.style.display = 'none';
      }
    });

    // Hide empty categories
    categories.forEach(category => {
      const visibleItems = category.querySelectorAll<HTMLElement>(
        '.doc-nav-item:not([style*="display: none"])'
      );
      if (visibleItems.length === 0 && query !== '') {
        category.style.display = 'none';
      } else {
        category.style.display = '';
      }
    });
  }
}
