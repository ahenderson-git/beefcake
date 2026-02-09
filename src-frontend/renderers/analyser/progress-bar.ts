import { CurrentDataset, LifecycleStage } from '../../types';
import { escapeHtml } from '../../utils';

interface StageInfo {
  stage: LifecycleStage;
  label: string;
  icon: string;
  description: string;
}

const STAGES: StageInfo[] = [
  {
    stage: 'Raw',
    label: 'Raw',
    icon: 'ph-file',
    description: 'Original data loaded',
  },
  {
    stage: 'Profiled',
    label: 'Profiled',
    icon: 'ph-chart-line',
    description: 'Data analyzed',
  },
  {
    stage: 'Cleaned',
    label: 'Cleaning',
    icon: 'ph-broom',
    description: 'Text & type transforms',
  },
  {
    stage: 'Advanced',
    label: 'Advanced',
    icon: 'ph-gear-six',
    description: 'ML preprocessing',
  },
  {
    stage: 'Validated',
    label: 'Validated',
    icon: 'ph-check-circle',
    description: 'Quality validated',
  },
  {
    stage: 'Published',
    label: 'Published',
    icon: 'ph-rocket-launch',
    description: 'Production ready',
  },
];

function getStageIndex(stage: LifecycleStage): number {
  return STAGES.findIndex(s => s.stage === stage);
}

function getCompletedStages(dataset: CurrentDataset | null): Set<LifecycleStage> {
  if (!dataset) return new Set();
  const completed = new Set<LifecycleStage>();
  dataset.versions.forEach(v => completed.add(v.stage));
  return completed;
}

export function renderStageProgressBar(
  currentStage: LifecycleStage | null,
  dataset: CurrentDataset | null
): string {
  const completedStages = getCompletedStages(dataset);
  const currentStageIndex = currentStage ? getStageIndex(currentStage) : -1;

  // For ad-hoc analysis (no dataset), treat Raw as completed when in Profiled stage
  if (!dataset && currentStage === 'Profiled') {
    completedStages.add('Raw');
  }

  const stagesHTML = STAGES.map((stageInfo, index) => {
    const isCompleted = completedStages.has(stageInfo.stage);
    const isCurrent = stageInfo.stage === currentStage;
    const isLocked = !isCompleted && !isCurrent;
    const isPast = index < currentStageIndex;

    const versionForStage = dataset?.versions.find(v => v.stage === stageInfo.stage);
    const versionLabel = versionForStage ? `v${dataset!.versions.indexOf(versionForStage)}` : '';

    const classes = [
      'progress-stage',
      isCurrent && 'stage-current',
      isCompleted && !isCurrent && 'stage-completed',
      isLocked && 'stage-locked',
      isPast && 'stage-past',
    ]
      .filter(Boolean)
      .join(' ');

    return `
      <div class="${classes}" data-stage="${escapeHtml(stageInfo.stage)}" title="${escapeHtml(stageInfo.description)}">
        <div class="progress-stage-icon">
          <i class="ph ${stageInfo.icon}"></i>
          ${isCompleted && !isCurrent ? '<i class="ph ph-check stage-check-icon"></i>' : ''}
          ${isLocked ? '<i class="ph ph-lock stage-lock-icon"></i>' : ''}
        </div>
        <div class="progress-stage-label">${escapeHtml(stageInfo.label)}</div>
        ${versionLabel ? `<div class="stage-version">${versionLabel}</div>` : ''}
        ${index < STAGES.length - 1 ? '<div class="progress-connector"></div>' : ''}
      </div>
    `;
  }).join('');

  return `
    <div class="analyser-progress-bar" data-testid="analyser-progress-bar">
      <div class="progress-bar-container">
        ${stagesHTML}
      </div>
    </div>
  `;
}
