import { AppConfig } from '../types';
import { escapeHtml } from '../utils';

export function renderActivityLogView(config: AppConfig): string {
  // Safely handle audit_log which might be undefined, null, or not an array
  const auditLogArray = Array.isArray(config.audit_log) ? config.audit_log : [];
  const auditLogs = [...auditLogArray].reverse();

  return `
    <div class="activity-view">
      <div class="activity-header">
        <h3><i class="ph ph-list-bullets"></i> Activity Log</h3>
        <button id="btn-clear-log" class="btn-secondary btn-small">Clear History</button>
      </div>
      <div class="activity-list">
        ${auditLogs.length === 0 ? '<p class="empty-msg">No recent activity.</p>' : ''}
        ${auditLogs
          .map(
            entry => `
          <div class="activity-entry">
            <div class="entry-icon">
              <i class="ph ${entry.action === 'Database' ? 'ph-database' : entry.action === 'Export' ? 'ph-export' : 'ph-info'}"></i>
            </div>
            <div class="entry-content">
              <div class="entry-meta">
                <span class="entry-action">${escapeHtml(entry.action)}</span>
                <span class="entry-time">${escapeHtml(entry.timestamp)}</span>
              </div>
              <div class="entry-details">${escapeHtml(entry.details)}</div>
            </div>
          </div>
        `
          )
          .join('')}
      </div>
    </div>
  `;
}
