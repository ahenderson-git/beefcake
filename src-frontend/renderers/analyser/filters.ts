import { AnalysisResponse } from '../../types';

export interface FilterState {
  searchTerm: string;
  typeFilters: Set<string>; // 'Numeric', 'Text', 'Categorical', 'Temporal', 'Boolean'
  qualityFilter: string | null; // 'all', 'excellent', 'good', 'fair', 'poor'
  activeFilter: string | null; // 'all', 'active', 'inactive'
}

export function createDefaultFilterState(): FilterState {
  return {
    searchTerm: '',
    typeFilters: new Set(),
    qualityFilter: null,
    activeFilter: null,
  };
}

export function renderFilterToolbar(response: AnalysisResponse, filterState: FilterState): string {
  const stats = {
    Numeric: (response.summary || []).filter(c => c.kind === 'Numeric').length,
    Text: (response.summary || []).filter(c => c.kind === 'Text').length,
    Categorical: (response.summary || []).filter(c => c.kind === 'Categorical').length,
    Temporal: (response.summary || []).filter(c => c.kind === 'Temporal').length,
    Boolean: (response.summary || []).filter(c => c.kind === 'Boolean').length,
  };

  const hasActiveFilters =
    filterState.searchTerm !== '' ||
    filterState.typeFilters.size > 0 ||
    filterState.qualityFilter !== null ||
    filterState.activeFilter !== null;

  return `
    <div class="analyser-filter-toolbar" data-testid="analyser-filter-toolbar">
      <div class="filter-search-box">
        <i class="ph ph-magnifying-glass filter-search-icon"></i>
        <input
          type="text"
          id="column-search"
          placeholder="Search columns..."
          value="${filterState.searchTerm}"
          data-testid="column-search-input"
        />
      </div>

      <div class="filter-type-pills">
        ${
          stats.Numeric > 0
            ? `<button class="filter-pill ${filterState.typeFilters.has('Numeric') ? 'active' : ''}" data-filter-type="Numeric" data-testid="filter-numeric">
          <i class="ph ph-hash"></i>
          Numeric (${stats.Numeric})
        </button>`
            : ''
        }

        ${
          stats.Text > 0
            ? `<button class="filter-pill ${filterState.typeFilters.has('Text') ? 'active' : ''}" data-filter-type="Text" data-testid="filter-text">
          <i class="ph ph-text-t"></i>
          Text (${stats.Text})
        </button>`
            : ''
        }

        ${
          stats.Categorical > 0
            ? `<button class="filter-pill ${filterState.typeFilters.has('Categorical') ? 'active' : ''}" data-filter-type="Categorical" data-testid="filter-categorical">
          <i class="ph ph-tag"></i>
          Category (${stats.Categorical})
        </button>`
            : ''
        }

        ${
          stats.Temporal > 0
            ? `<button class="filter-pill ${filterState.typeFilters.has('Temporal') ? 'active' : ''}" data-filter-type="Temporal" data-testid="filter-temporal">
          <i class="ph ph-calendar"></i>
          Date (${stats.Temporal})
        </button>`
            : ''
        }

        ${
          stats.Boolean > 0
            ? `<button class="filter-pill ${filterState.typeFilters.has('Boolean') ? 'active' : ''}" data-filter-type="Boolean" data-testid="filter-boolean">
          <i class="ph ph-check-square"></i>
          Boolean (${stats.Boolean})
        </button>`
            : ''
        }
      </div>

      ${hasActiveFilters ? '<button class="filter-clear-btn" id="clear-filters" data-testid="clear-filters"><i class="ph ph-x"></i> Clear Filters</button>' : ''}
    </div>
  `;
}

export function shouldShowColumn(
  columnName: string,
  columnKind: string,
  columnQuality: number,
  isActive: boolean,
  filterState: FilterState
): boolean {
  // Search term filter
  if (filterState.searchTerm !== '') {
    const searchLower = filterState.searchTerm.toLowerCase();
    if (!columnName.toLowerCase().includes(searchLower)) {
      return false;
    }
  }

  // Type filter
  if (filterState.typeFilters.size > 0) {
    if (!filterState.typeFilters.has(columnKind)) {
      return false;
    }
  }

  // Quality filter
  if (filterState.qualityFilter !== null && filterState.qualityFilter !== 'all') {
    const qualityLevel = getQualityLevel(columnQuality);
    if (qualityLevel !== filterState.qualityFilter) {
      return false;
    }
  }

  // Active/Inactive filter
  if (filterState.activeFilter !== null && filterState.activeFilter !== 'all') {
    if (filterState.activeFilter === 'active' && !isActive) {
      return false;
    }
    if (filterState.activeFilter === 'inactive' && isActive) {
      return false;
    }
  }

  return true;
}

function getQualityLevel(qualityPercent: number): string {
  if (qualityPercent >= 95) return 'excellent';
  if (qualityPercent >= 80) return 'good';
  if (qualityPercent >= 60) return 'fair';
  return 'poor';
}
