import { escapeHtml } from '../utils';

export const IMPUTE_OPTIONS = [
  { value: 'None', label: 'None' },
  { value: 'Mean', label: 'Mean' },
  { value: 'Median', label: 'Median' },
  { value: 'Zero', label: 'Zero' },
  { value: 'Mode', label: 'Mode' },
];

export const NORM_OPTIONS = [
  { value: 'None', label: 'None' },
  { value: 'ZScore', label: 'Z-Score' },
  { value: 'MinMax', label: 'Min-Max' },
];

export const CASE_OPTIONS = [
  { value: 'None', label: 'None' },
  { value: 'Lowercase', label: 'Lower' },
  { value: 'Uppercase', label: 'Upper' },
  { value: 'TitleCase', label: 'Title' },
];

export const ROUND_OPTIONS = [
  { value: 'none', label: 'None' },
  { value: '0', label: '0' },
  { value: '1', label: '1' },
  { value: '2', label: '2' },
  { value: '3', label: '3' },
  { value: '4', label: '4' },
];

export function renderSelect(
  options: { value: string; label: string }[],
  selectedValue: string,
  className: string,
  dataAttrs: Record<string, string>,
  placeholder?: string,
  disabled?: boolean
): string {
  const attrs = Object.entries(dataAttrs)
    .map(([k, v]) => `data-${k}="${escapeHtml(v)}"`)
    .join(' ');
  const placeholderHtml = placeholder ? `<option value="">${escapeHtml(placeholder)}</option>` : '';
  return `
    <select class="${className}" ${attrs} ${disabled ? 'disabled' : ''}>
      ${placeholderHtml}
      ${options
        .map(
          opt => `
        <option value="${escapeHtml(opt.value)}" ${opt.value === selectedValue ? 'selected' : ''}>${escapeHtml(opt.label)}</option>
      `
        )
        .join('')}
    </select>
  `;
}

export function renderLoading(message: string, isAborting: boolean): string {
  return `
    <div class="loading-overlay">
      <div class="loading-spinner"></div>
      <p>${escapeHtml(message)}</p>
      <div class="loading-actions">
        ${isAborting ? '<p class="aborting-text">Aborting...</p>' : '<button id="btn-abort-op" class="btn-danger btn-small">Abort</button>'}
      </div>
    </div>
  `;
}

export function renderToast(message: string, type: 'success' | 'error' | 'info' = 'info'): string {
  const icon =
    type === 'success' ? 'ph-check-circle' : type === 'error' ? 'ph-x-circle' : 'ph-info';
  return `
    <div class="toast toast-${type}">
      <i class="ph ${icon}"></i>
      <span>${escapeHtml(message)}</span>
    </div>
  `;
}
