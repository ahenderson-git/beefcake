import { VerificationResult } from '../types';
import { escapeHtml, fmtBytes } from '../utils';

export function renderIntegrityView(
  result: VerificationResult | null,
  isVerifying: boolean
): string {
  return `
    <div class="integrity-view" data-testid="integrity-view">
      <div class="content-header">
        <h2><i class="ph ph-shield-check"></i> Integrity Verification</h2>
        <p class="subtitle">Verify exported file integrity using cryptographic receipts</p>
      </div>

      <div class="card">
        <h3>How It Works</h3>
        <ol class="help-list">
          <li>When you export data with "Create integrity receipt" enabled, Beefcake generates a <code>.receipt.json</code> file</li>
          <li>The receipt contains a SHA-256 cryptographic hash of your exported file</li>
          <li>Later, you can verify the file hasn't been modified by checking the receipt</li>
        </ol>
      </div>

      <div class="card">
        <h3>Verify Receipt</h3>
        <button id="btn-select-receipt" class="btn-primary" ${isVerifying ? 'disabled' : ''}>
          <i class="ph ph-folder-open"></i> ${isVerifying ? 'Verifying...' : 'Select Receipt File'}
        </button>
        ${isVerifying ? '<div class="spinner"></div>' : ''}
      </div>

      ${result ? renderVerificationResult(result) : ''}
    </div>
  `;
}

function renderVerificationResult(result: VerificationResult): string {
  const statusClass = result.passed ? 'verification-pass' : 'verification-fail';
  const icon = result.passed ? 'ph-check-circle' : 'ph-x-circle';

  return `
    <div class="card verification-result ${statusClass}">
      <div class="verification-header">
        <i class="ph ${icon}"></i>
        <h3>${result.passed ? 'VERIFICATION PASSED' : 'VERIFICATION FAILED'}</h3>
      </div>

      <div class="verification-details">
        <div class="detail-row">
          <span class="label">File</span>
          <span class="value">${escapeHtml(result.file_path)}</span>
        </div>
        <div class="detail-row">
          <span class="label">Expected Hash</span>
          <span class="value hash-value">${escapeHtml(result.expected_hash)}</span>
        </div>
        ${
          result.actual_hash
            ? `
          <div class="detail-row">
            <span class="label">Actual Hash</span>
            <span class="value hash-value">${escapeHtml(result.actual_hash)}</span>
          </div>
        `
            : ''
        }
        <div class="detail-row">
          <span class="label">Message</span>
          <span class="value">${escapeHtml(result.message)}</span>
        </div>
      </div>

      ${renderReceiptMetadata(result.receipt)}
    </div>
  `;
}

function renderReceiptMetadata(receipt: VerificationResult['receipt']): string {
  return `
    <details class="receipt-details">
      <summary>Receipt Metadata</summary>
      <div class="metadata-grid">
        <div class="meta-item">
          <span class="meta-label">Created</span>
          <span class="meta-value">${new Date(receipt.created_utc).toLocaleString()}</span>
        </div>
        <div class="meta-item">
          <span class="meta-label">Producer</span>
          <span class="meta-value">${receipt.producer.app_name} v${receipt.producer.app_version}</span>
        </div>
        <div class="meta-item">
          <span class="meta-label">File Size</span>
          <span class="meta-value">${fmtBytes(receipt.export.file_size_bytes)}</span>
        </div>
        <div class="meta-item">
          <span class="meta-label">Rows × Columns</span>
          <span class="meta-value">${receipt.export.row_count} × ${receipt.export.column_count}</span>
        </div>
        <div class="meta-item">
          <span class="meta-label">Hash Algorithm</span>
          <span class="meta-value">${receipt.integrity.hash_algorithm}</span>
        </div>
      </div>
    </details>
  `;
}
