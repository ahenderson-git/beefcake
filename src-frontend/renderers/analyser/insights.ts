import { AnalysisResponse } from '../../types';
import { escapeHtml } from '../../utils';

interface DataInsight {
  type: 'warning' | 'info' | 'success';
  title: string;
  description: string;
  columns: string[];
}

export function generateInsights(response: AnalysisResponse): DataInsight[] {
  const insights: DataInsight[] = [];
  const columns = response.summary || [];

  // High null columns
  const highNullCols = columns.filter(col => {
    const nullPercent = col.count > 0 ? (col.nulls / col.count) * 100 : 0;
    return nullPercent > 20;
  });

  if (highNullCols.length > 0) {
    insights.push({
      type: 'warning',
      title: 'High Missing Values Detected',
      description: `${highNullCols.length} column(s) have more than 20% missing values. Consider imputation or removal.`,
      columns: highNullCols.map(c => c.name),
    });
  }

  // High cardinality categorical
  const highCardinalityCols = columns.filter(col => {
    if (col.kind !== 'Categorical' && col.kind !== 'Text') return false;
    let uniqueCount = 0;
    if (col.stats.Text?.distinct !== undefined) {
      uniqueCount = col.stats.Text.distinct;
    } else if (col.stats.Categorical) {
      uniqueCount = Object.keys(col.stats.Categorical).length;
    }
    return col.count > 10 && uniqueCount / col.count > 0.5;
  });

  if (highCardinalityCols.length > 0) {
    insights.push({
      type: 'info',
      title: 'High Cardinality Columns',
      description: `${highCardinalityCols.length} column(s) have high uniqueness (>50%). These may not benefit from categorical encoding.`,
      columns: highCardinalityCols.map(c => c.name),
    });
  }

  // Potential primary keys (very high uniqueness)
  const potentialKeysCols = columns.filter(col => {
    let uniqueCount = 0;
    if (col.stats.Numeric?.distinct_count !== undefined) {
      uniqueCount = col.stats.Numeric.distinct_count;
    } else if (col.stats.Text?.distinct !== undefined) {
      uniqueCount = col.stats.Text.distinct;
    }
    return col.count > 0 && uniqueCount / col.count > 0.95 && col.nulls === 0;
  });

  if (potentialKeysCols.length > 0) {
    insights.push({
      type: 'success',
      title: 'Potential Primary Keys',
      description: `${potentialKeysCols.length} column(s) have >95% unique values and no nulls. These may serve as identifiers.`,
      columns: potentialKeysCols.map(c => c.name),
    });
  }

  // Low variance numeric columns
  const lowVarianceCols = columns.filter(col => {
    if (col.kind !== 'Numeric') return false;
    const stats = col.stats.Numeric;
    if (!stats) return false;
    const { distinct_count } = stats;
    return col.count > 10 && distinct_count !== undefined && distinct_count < 5;
  });

  if (lowVarianceCols.length > 0) {
    insights.push({
      type: 'info',
      title: 'Low Variance Numeric Columns',
      description: `${lowVarianceCols.length} numeric column(s) have very few unique values (<5). Consider converting to categorical.`,
      columns: lowVarianceCols.map(c => c.name),
    });
  }

  // Temporal columns that may need parsing
  const textTemporalCols = columns.filter(col => {
    if (col.kind !== 'Text') return false;
    const name = col.name.toLowerCase();
    return (
      name.includes('date') ||
      name.includes('time') ||
      name.includes('created') ||
      name.includes('modified') ||
      name.includes('timestamp')
    );
  });

  if (textTemporalCols.length > 0) {
    insights.push({
      type: 'info',
      title: 'Potential Date Columns',
      description: `${textTemporalCols.length} text column(s) may contain dates. Consider parsing to Temporal type.`,
      columns: textTemporalCols.map(c => c.name),
    });
  }

  // All numeric columns with clean data
  const cleanNumericCols = columns.filter(col => {
    if (col.kind !== 'Numeric') return false;
    const nullPercent = col.count > 0 ? (col.nulls / col.count) * 100 : 0;
    return nullPercent < 5;
  });

  if (cleanNumericCols.length >= 3) {
    insights.push({
      type: 'success',
      title: 'Clean Numeric Data',
      description: `${cleanNumericCols.length} numeric column(s) are high quality (<5% nulls). Ready for ML preprocessing.`,
      columns: cleanNumericCols.slice(0, 5).map(c => c.name), // Show max 5
    });
  }

  return insights;
}

export function renderInsightsPanel(response: AnalysisResponse): string {
  const insights = generateInsights(response);

  if (insights.length === 0) {
    return '';
  }

  return `
    <div class="analyser-insights-panel" data-testid="analyser-insights-panel">
      <div class="insights-header">
        <i class="ph ph-lightbulb"></i>
        <h3>Smart Insights & Recommendations</h3>
      </div>
      <div class="insights-grid">
        ${insights
          .map(
            insight => `
          <div class="insight-card" data-insight-type="${insight.type}">
            <div class="insight-card-header">
              <i class="ph ${getInsightIcon(insight.type)} insight-icon ${insight.type}"></i>
              <h4>${escapeHtml(insight.title)}</h4>
            </div>
            <p>${escapeHtml(insight.description)}</p>
            <div class="insight-columns">
              ${insight.columns
                .slice(0, 8)
                .map(col => `<span class="insight-column-badge">${escapeHtml(col)}</span>`)
                .join('')}
              ${insight.columns.length > 8 ? `<span class="insight-column-badge">+${insight.columns.length - 8} more</span>` : ''}
            </div>
          </div>
        `
          )
          .join('')}
      </div>
    </div>
  `;
}

function getInsightIcon(type: string): string {
  switch (type) {
    case 'warning':
      return 'ph-warning';
    case 'info':
      return 'ph-info';
    case 'success':
      return 'ph-check-circle';
    default:
      return 'ph-info';
  }
}
