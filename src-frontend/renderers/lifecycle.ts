import { DatasetVersion, LifecycleStage, DiffSummary, CurrentDataset } from '../types';
import { escapeHtml } from '../utils';

export interface StageConfig {
  stage: LifecycleStage;
  label: string;
  description: string;
  icon: string;
  isMutating: boolean;
  color: string;
}

const STAGE_CONFIGS: StageConfig[] = [
  {
    stage: 'Raw',
    label: 'Raw',
    description: 'Original data (immutable)',
    icon: 'ph-file',
    isMutating: false,
    color: 'grey',
  },
  {
    stage: 'Profiled',
    label: 'Profiled',
    description: 'Analysis complete • Read-only • No mutations',
    icon: 'ph-chart-line',
    isMutating: false,
    color: 'blue',
  },
  {
    stage: 'Cleaned',
    label: 'Cleaning',
    description: 'Text/type transforms • Reversible',
    icon: 'ph-broom',
    isMutating: true,
    color: 'blue',
  },
  {
    stage: 'Advanced',
    label: 'Advanced',
    description: 'ML preprocessing • Statistical operations',
    icon: 'ph-gear-six',
    isMutating: true,
    color: 'purple',
  },
  {
    stage: 'Validated',
    label: 'Validated',
    description: 'QA gates • Validation rules',
    icon: 'ph-check-circle',
    isMutating: false,
    color: 'amber',
  },
  {
    stage: 'Published',
    label: 'Published',
    description: 'Production ready • Finalized',
    icon: 'ph-rocket-launch',
    isMutating: false,
    color: 'green',
  },
];

function getStageColor(stage: LifecycleStage): string {
  const config = STAGE_CONFIGS.find(s => s.stage === stage);
  if (config) {
    return `var(--stage-${config.color})`;
  }
  // Fallback for unknown stages or EXTERNAL_FRAGMENT
  return 'var(--stage-grey)';
}

function getStageConfig(stage: LifecycleStage): StageConfig {
  const config = STAGE_CONFIGS.find(s => s.stage === stage);
  if (!config) {
    // Fallback for unknown stages
    return {
      stage: stage,
      label: stage,
      description: 'Unknown stage',
      icon: 'ph-question',
      isMutating: false,
      color: 'grey',
    };
  }
  return config;
}

function getStageIndex(stage: LifecycleStage): number {
  return STAGE_CONFIGS.findIndex(s => s.stage === stage);
}

function getCompletedStages(versions: DatasetVersion[]): Set<LifecycleStage> {
  const completed = new Set<LifecycleStage>();
  versions.forEach(v => completed.add(v.stage));
  return completed;
}

function getActiveStage(dataset: CurrentDataset): LifecycleStage {
  const activeVersion = dataset.versions.find(v => v.id === dataset.activeVersionId);
  return activeVersion?.stage ?? 'Raw';
}

function canProgressTo(
  currentStage: LifecycleStage,
  targetStage: LifecycleStage,
  completedStages: Set<LifecycleStage>
): boolean {
  const currentIdx = getStageIndex(currentStage);
  const targetIdx = getStageIndex(targetStage);

  // Can always go backward
  if (targetIdx <= currentIdx) return true;

  // Can skip forward if all required prerequisites are met
  // For simplicity, we allow skipping if you have at least Raw
  return completedStages.has('Raw');
}

export function renderLifecycleRail(dataset: CurrentDataset | null): string {
  if (!dataset) {
    return '<div class="lifecycle-rail lifecycle-rail-empty">No dataset loaded</div>';
  }

  const completedStages = getCompletedStages(dataset.versions);
  const activeStage = getActiveStage(dataset);

  const stagesHTML = STAGE_CONFIGS.map((config, index) => {
    const isCompleted = completedStages.has(config.stage);
    const isActive = config.stage === activeStage;
    const isLocked = !canProgressTo(activeStage, config.stage, completedStages);

    const versionForStage = dataset.versions.find(v => v.stage === config.stage);
    const versionLabel = versionForStage ? `v${dataset.versions.indexOf(versionForStage)}` : '';

    const classes = [
      'lifecycle-stage',
      `stage-${config.color}`,
      isCompleted && 'stage-completed',
      isActive && 'stage-active',
      isLocked && 'stage-locked',
    ]
      .filter(Boolean)
      .join(' ');

    return `
      <div class="${classes}" data-stage="${escapeHtml(config.stage)}" data-testid="lifecycle-stage-${config.stage.toLowerCase()}" title="${escapeHtml(config.description)}">
        <div class="stage-icon">
          <i class="ph ${config.icon}"></i>
          ${isCompleted ? '<i class="ph ph-check stage-check"></i>' : ''}
          ${isLocked ? '<i class="ph ph-lock stage-lock"></i>' : ''}
        </div>
        <div class="stage-label">${escapeHtml(config.label)}</div>
        ${versionLabel ? `<div class="stage-version">${versionLabel}</div>` : ''}
        ${isActive ? '<div class="stage-active-indicator"></div>' : ''}
      </div>
      ${index < STAGE_CONFIGS.length - 1 ? '<div class="stage-connector"></div>' : ''}
    `;
  }).join('');

  return `
    <div class="lifecycle-rail" data-dataset-id="${escapeHtml(dataset.id)}" data-testid="lifecycle-rail">
      <div class="lifecycle-rail-header">
        <span class="lifecycle-dataset-name">${escapeHtml(dataset.name)}</span>
        <span class="lifecycle-stage-count">${completedStages.size}/${STAGE_CONFIGS.length} stages</span>
      </div>
      <div class="lifecycle-stages" data-testid="lifecycle-stages">
        ${stagesHTML}
      </div>
    </div>
  `;
}

export function renderVersionChip(version: DatasetVersion, isActive: boolean): string {
  const stageConfig = getStageConfig(version.stage);
  const stageColor = getStageColor(version.stage);
  const date = new Date(version.created_at).toLocaleDateString();

  return `
    <div class="version-chip ${isActive ? 'version-chip-active' : ''}" data-version-id="${escapeHtml(version.id)}">
      <div class="version-chip-badge" style="background-color: ${stageColor}">
        <i class="ph ${stageConfig.icon}"></i>
      </div>
      <div class="version-chip-content">
        <div class="version-chip-title">${escapeHtml(stageConfig.label)}</div>
        <div class="version-chip-meta">${date}</div>
      </div>
      ${isActive ? '<i class="ph ph-check-circle version-chip-active-icon"></i>' : ''}
    </div>
  `;
}

export function renderDiffBadge(diff: DiffSummary | null): string {
  if (!diff) return '';

  const badges = [];

  // Schema changes
  if (diff.schema_changes.columns_added.length > 0) {
    badges.push(
      `<span class="diff-badge diff-badge-add">+${diff.schema_changes.columns_added.length} cols</span>`
    );
  }
  if (diff.schema_changes.columns_removed.length > 0) {
    badges.push(
      `<span class="diff-badge diff-badge-remove">-${diff.schema_changes.columns_removed.length} cols</span>`
    );
  }

  // Row changes
  const rowChange = diff.row_changes.rows_v2 - diff.row_changes.rows_v1;
  if (rowChange !== 0) {
    const pct = ((rowChange / diff.row_changes.rows_v1) * 100).toFixed(1);
    badges.push(
      `<span class="diff-badge diff-badge-rows">${rowChange > 0 ? '+' : ''}${pct}% rows</span>`
    );
  }

  // Statistical changes (null reduction is common)
  const nullChanges = diff.statistical_changes.filter(c => c.metric.toLowerCase().includes('null'));
  if (nullChanges.length > 0) {
    const avgNullChange =
      nullChanges.reduce((sum, c) => sum + (c.change_percent ?? 0), 0) / nullChanges.length;
    if (Math.abs(avgNullChange) > 1) {
      badges.push(
        `<span class="diff-badge diff-badge-nulls">${avgNullChange > 0 ? '+' : ''}${avgNullChange.toFixed(1)}% nulls</span>`
      );
    }
  }

  return badges.join('');
}

export function renderPublishModal(): string {
  return `
    <div class="modal-overlay" data-testid="publish-modal-overlay">
      <div class="modal-content modal-publish" data-testid="publish-modal">
        <div class="modal-header">
          <h3>Publish Dataset Version</h3>
          <button class="modal-close" id="modal-close" data-testid="publish-modal-close">
            <i class="ph ph-x"></i>
          </button>
        </div>

        <div class="modal-body">
          <p class="modal-description">
            Choose how to publish this dataset version. This action creates a new published version.
          </p>

          <div class="publish-mode-options">
            <div class="publish-mode-card" data-mode="view">
              <div class="publish-mode-icon">
                <i class="ph ph-eye"></i>
              </div>
              <h4>View (Lazy)</h4>
              <p>Logical view computed on access. Always fresh, no storage cost.</p>
              <ul>
                <li>✓ No data duplication</li>
                <li>✓ Always up-to-date with source</li>
                <li>✓ Instant creation</li>
                <li>⚠ Computed on each access</li>
              </ul>
              <button class="btn btn-primary" id="btn-publish-view" data-testid="btn-publish-view">Publish as View</button>
            </div>

            <div class="publish-mode-card" data-mode="snapshot">
              <div class="publish-mode-icon">
                <i class="ph ph-floppy-disk"></i>
              </div>
              <h4>Snapshot (Materialized)</h4>
              <p>Physical copy frozen in time. Fast access, uses storage.</p>
              <ul>
                <li>✓ Instant query performance</li>
                <li>✓ Immutable snapshot</li>
                <li>✓ Production-ready</li>
                <li>⚠ Requires storage space</li>
              </ul>
              <button class="btn btn-primary" id="btn-publish-snapshot" data-testid="btn-publish-snapshot">Publish as Snapshot</button>
            </div>
          </div>

          <div class="publish-warning">
            <i class="ph ph-warning"></i>
            <span>Publishing creates a new immutable version. This action cannot be undone.</span>
          </div>
        </div>
      </div>
    </div>
  `;
}

export function renderVersionTree(dataset: CurrentDataset): string {
  // Sort versions by stage order first, then by creation time
  const stageOrder: Record<string, number> = {
    Raw: 0,
    Profiled: 1,
    Cleaned: 2,
    Advanced: 3,
    Validated: 4,
    Published: 5,
  };

  const sortedVersions = [...dataset.versions].sort((a, b) => {
    const stageA = stageOrder[a.stage] ?? 999;
    const stageB = stageOrder[b.stage] ?? 999;
    if (stageA !== stageB) {
      return stageA - stageB;
    }
    // Same stage - sort by creation time
    return new Date(a.created_at).getTime() - new Date(b.created_at).getTime();
  });

  const versionsHTML = sortedVersions
    .map((version, index) => {
      const isActive = version.id === dataset.activeVersionId;
      const stageConfig = getStageConfig(version.stage);
      const stageColor = getStageColor(version.stage);

      return `
      <div class="version-tree-node ${isActive ? 'version-tree-node-active' : ''}" data-version-id="${escapeHtml(version.id)}">
        <div class="version-tree-connector"></div>
        <div class="version-tree-content">
          <div class="version-tree-badge" style="background-color: ${stageColor}">
            <i class="ph ${stageConfig.icon}"></i>
          </div>
          <div class="version-tree-details">
            <div class="version-tree-title">
              <strong>v${index} ${escapeHtml(stageConfig.label)}</strong>
              ${isActive ? '<span class="badge badge-active">Active</span>' : ''}
            </div>
            <div class="version-tree-meta">
              ${new Date(version.created_at).toLocaleString()}
              ${version.metadata.row_count !== null ? ` • ${version.metadata.row_count.toLocaleString()} rows` : ''}
              ${version.metadata.column_count !== null ? ` • ${version.metadata.column_count} columns` : ''}
            </div>
            <div class="version-tree-description">${escapeHtml(version.metadata.description)}</div>
          </div>
          <div class="version-tree-actions">
            ${!isActive ? `<button class="btn btn-small" data-action="set-active" data-version-id="${escapeHtml(version.id)}" data-testid="lifecycle-set-active-${escapeHtml(version.id)}">Set Active</button>` : ''}
            ${index > 0 ? `<button class="btn btn-small" data-action="view-diff" data-version-id="${escapeHtml(version.id)}" data-testid="lifecycle-view-diff-${escapeHtml(version.id)}">View Diff</button>` : `<button class="btn btn-small" disabled title="Cannot compute diff - this is the first version with no parent">View Diff</button>`}
          </div>
        </div>
      </div>
    `;
    })
    .join('');

  return `
    <div class="version-tree">
      ${versionsHTML}
    </div>
  `;
}

export function renderLifecycleView(dataset: CurrentDataset | null): string {
  if (!dataset) {
    return `
      <div class="lifecycle-view-empty">
        <i class="ph ph-git-branch"></i>
        <h3>No Dataset Loaded</h3>
        <p>Analyse a file to create a dataset and track its lifecycle.</p>
      </div>
    `;
  }

  return `
    <div class="lifecycle-view">
      <div class="lifecycle-view-header">
        <h3>${escapeHtml(dataset.name)}</h3>
        <div class="lifecycle-view-actions">
          <button class="btn" id="btn-publish-version" data-testid="btn-publish-version">
            <i class="ph ph-rocket-launch"></i> Publish Version
          </button>
        </div>
      </div>

      <div class="lifecycle-view-rail">
        ${renderLifecycleRail(dataset)}
      </div>

      <div class="lifecycle-view-body">
        <div class="lifecycle-view-section">
          <h4><i class="ph ph-git-branch"></i> Version History</h4>
          <p class="section-description">Timeline of dataset transformations - click to set active or view changes</p>
          ${renderVersionTree(dataset)}
        </div>

        <div class="lifecycle-view-section">
          <h4><i class="ph ph-info-circle"></i> Active Version Details</h4>
          <p class="section-description">Information about the currently active dataset version</p>
          ${renderActiveVersionDetails(dataset)}
        </div>
      </div>
    </div>
  `;
}

function renderActiveVersionDetails(dataset: CurrentDataset): string {
  const activeVersion = dataset.versions.find(v => v.id === dataset.activeVersionId);
  if (!activeVersion) return '<p>No active version</p>';

  const stageConfig = getStageConfig(activeVersion.stage);
  const stageColor = getStageColor(activeVersion.stage);

  return `
    <div class="version-details">
      <div class="version-details-header">
        <div class="version-details-badge" style="background-color: ${stageColor}">
          <i class="ph ${stageConfig.icon}"></i>
        </div>
        <div>
          <h5>${escapeHtml(stageConfig.label)}</h5>
          <p>${escapeHtml(stageConfig.description)}</p>
        </div>
      </div>

      <div class="version-details-stats">
        <div class="stat-card">
          <div class="stat-label">Rows</div>
          <div class="stat-value">${activeVersion.metadata.row_count?.toLocaleString() ?? 'N/A'}</div>
        </div>
        <div class="stat-card">
          <div class="stat-label">Columns</div>
          <div class="stat-value">${activeVersion.metadata.column_count ?? 'N/A'}</div>
        </div>
        <div class="stat-card">
          <div class="stat-label">Created</div>
          <div class="stat-value">${new Date(activeVersion.created_at).toLocaleDateString()}</div>
        </div>
      </div>

      ${
        activeVersion.pipeline.transforms.length > 0
          ? `
        <div class="version-details-transforms">
          <h6>Applied Transforms</h6>
          <ul>
            ${activeVersion.pipeline.transforms
              .map(
                t => `
              <li><code>${escapeHtml(t.transform_type)}</code></li>
            `
              )
              .join('')}
          </ul>
        </div>
      `
          : ''
      }
    </div>
  `;
}
