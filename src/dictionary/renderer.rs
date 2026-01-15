//! Markdown rendering for data dictionary snapshots.
//!
//! Generates human-readable documentation from dictionary snapshots.

use super::metadata::DataDictionary;
use anyhow::Result;

/// Render a data dictionary as Markdown documentation.
///
/// Generates a comprehensive Markdown document including:
/// - Dataset overview and metadata
/// - Column catalog with technical and business details
/// - Data quality summary
/// - Version lineage information
pub fn render_markdown(dict: &DataDictionary) -> Result<String> {
    let mut md = String::new();

    // Title and header
    md.push_str(&format!("# Data Dictionary: {}\n\n", dict.dataset_name));
    md.push_str(&format!("> **Snapshot ID:** `{}`  \n", dict.snapshot_id));
    md.push_str(&format!(
        "> **Created:** {}  \n",
        dict.export_timestamp.format("%Y-%m-%d %H:%M:%S UTC")
    ));
    md.push_str(&format!(
        "> **Documentation Completeness:** {:.1}%  \n\n",
        dict.documentation_completeness()
    ));

    // Table of contents
    md.push_str("## Table of Contents\n\n");
    md.push_str("1. [Dataset Overview](#dataset-overview)\n");
    md.push_str("2. [Column Catalog](#column-catalog)\n");
    md.push_str("3. [Data Quality Summary](#data-quality-summary)\n");
    md.push_str("4. [Technical Metadata](#technical-metadata)\n\n");

    md.push_str("---\n\n");

    // 1. Dataset Overview
    md.push_str("## Dataset Overview\n\n");
    render_dataset_business_metadata(&mut md, dict);
    md.push('\n');

    // 2. Column Catalog
    md.push_str("## Column Catalog\n\n");
    md.push_str(&format!("**Total Columns:** {}  \n", dict.columns.len()));
    md.push_str(&format!(
        "**Total Rows:** {}  \n\n",
        dict.dataset_metadata.technical.row_count
    ));

    for (i, col) in dict.columns.iter().enumerate() {
        md.push_str(&format!("### {} — `{}`\n\n", i + 1, col.current_name));
        render_column_metadata(&mut md, col);
        md.push_str("\n---\n\n");
    }

    // 3. Data Quality Summary
    md.push_str("## Data Quality Summary\n\n");
    render_quality_summary(&mut md, dict);
    md.push('\n');

    // 4. Technical Metadata
    md.push_str("## Technical Metadata\n\n");
    render_technical_metadata(&mut md, dict);

    // Version lineage
    if let Some(prev_id) = dict.previous_snapshot_id {
        md.push_str("\n---\n\n");
        md.push_str("## Version History\n\n");
        md.push_str(&format!("**Previous Snapshot:** `{prev_id}`  \n"));
    }

    Ok(md)
}

/// Render dataset-level business metadata section.
fn render_dataset_business_metadata(md: &mut String, dict: &DataDictionary) {
    let business = &dict.dataset_metadata.business;

    if let Some(desc) = &business.description {
        md.push_str(&format!("**Description:**  \n{desc}\n\n"));
    }

    if let Some(use_case) = &business.intended_use {
        md.push_str(&format!("**Intended Use:**  \n{use_case}\n\n"));
    }

    if let Some(owner) = &business.owner_or_steward {
        md.push_str(&format!("**Owner/Steward:** {owner}\n\n"));
    }

    if let Some(refresh) = &business.refresh_expectation {
        md.push_str(&format!("**Refresh Cadence:** {refresh}\n\n"));
    }

    if let Some(sensitivity) = &business.sensitivity_classification {
        md.push_str(&format!("**Sensitivity:** {sensitivity}\n\n"));
    }

    if let Some(limitations) = &business.known_limitations {
        md.push_str(&format!("**Known Limitations:**  \n{limitations}\n\n"));
    }

    if !business.tags.is_empty() {
        md.push_str(&format!("**Tags:** {}\n\n", business.tags.join(", ")));
    }

    // If no business metadata, show placeholder
    if business.description.is_none()
        && business.intended_use.is_none()
        && business.owner_or_steward.is_none()
    {
        md.push_str("*No dataset-level documentation provided.*\n\n");
    }
}

/// Render a single column's metadata.
fn render_column_metadata(md: &mut String, col: &super::metadata::ColumnMetadata) {
    // Business metadata section
    md.push_str("#### Business Definition\n\n");

    if let Some(def) = &col.business.business_definition {
        md.push_str(&format!("{def}\n\n"));
    } else {
        md.push_str("*No business definition provided.*\n\n");
    }

    if let Some(rules) = &col.business.business_rules {
        md.push_str(&format!("**Business Rules:** {rules}\n\n"));
    }

    if let Some(sensitivity) = &col.business.sensitivity_tag {
        md.push_str(&format!("**Sensitivity:** {sensitivity}\n\n"));
    }

    if !col.business.approved_examples.is_empty() {
        md.push_str(&format!(
            "**Approved Examples:** {}\n\n",
            col.business.approved_examples.join(", ")
        ));
    }

    if let Some(notes) = &col.business.notes {
        md.push_str(&format!("**Notes:** {notes}\n\n"));
    }

    // Technical metadata section (collapsible)
    md.push_str("<details>\n");
    md.push_str("<summary><strong>Technical Details</strong></summary>\n\n");

    md.push_str("| Property | Value |\n");
    md.push_str("|----------|-------|\n");
    md.push_str(&format!(
        "| **Data Type** | `{}` |\n",
        col.technical.data_type
    ));
    md.push_str(&format!("| **Nullable** | {} |\n", col.technical.nullable));
    md.push_str(&format!(
        "| **Null %** | {:.2}% |\n",
        col.technical.null_percentage
    ));
    md.push_str(&format!(
        "| **Distinct Values** | {} |\n",
        col.technical.distinct_count
    ));

    if let Some(min) = &col.technical.min_value {
        md.push_str(&format!("| **Min** | `{min}` |\n"));
    }

    if let Some(max) = &col.technical.max_value {
        md.push_str(&format!("| **Max** | `{max}` |\n"));
    }

    if let Some(original) = &col.original_name {
        md.push_str(&format!("| **Original Name** | `{original}` |\n"));
    }

    md.push('\n');

    // Sample values
    if !col.technical.sample_values.is_empty() {
        md.push_str("**Sample Values:**  \n");
        for sample in &col.technical.sample_values {
            md.push_str(&format!("- `{sample}`\n"));
        }
        md.push('\n');
    }

    // Warnings
    if !col.technical.warnings.is_empty() {
        md.push_str("**⚠️ Warnings:**  \n");
        for warning in &col.technical.warnings {
            md.push_str(&format!("- {warning}\n"));
        }
        md.push('\n');
    }

    md.push_str("</details>\n\n");
}

/// Render data quality summary section.
fn render_quality_summary(md: &mut String, dict: &DataDictionary) {
    let quality = &dict.dataset_metadata.technical.quality_summary;

    md.push_str("| Metric | Value |\n");
    md.push_str("|--------|-------|\n");
    md.push_str(&format!(
        "| **Overall Quality Score** | {:.1}% |\n",
        quality.overall_score
    ));
    md.push_str(&format!(
        "| **Avg Null %** | {:.2}% |\n",
        quality.avg_null_percentage
    ));
    md.push_str(&format!(
        "| **Empty Columns** | {} |\n",
        quality.empty_column_count
    ));
    md.push_str(&format!(
        "| **Constant Columns** | {} |\n",
        quality.constant_column_count
    ));

    if let Some(dup_count) = quality.duplicate_row_count {
        md.push_str(&format!("| **Duplicate Rows** | {dup_count} |\n"));
    }

    md.push('\n');

    // Columns with warnings
    let warned_cols = dict.columns_with_warnings();
    if !warned_cols.is_empty() {
        md.push_str("### Columns with Warnings\n\n");
        for col in warned_cols {
            md.push_str(&format!("- **{}**: ", col.current_name));
            md.push_str(&col.technical.warnings.join("; "));
            md.push('\n');
        }
        md.push('\n');
    }
}

/// Render technical metadata section.
fn render_technical_metadata(md: &mut String, dict: &DataDictionary) {
    let tech = &dict.dataset_metadata.technical;

    md.push_str("| Property | Value |\n");
    md.push_str("|----------|-------|\n");
    md.push_str(&format!("| **Row Count** | {} |\n", tech.row_count));
    md.push_str(&format!("| **Column Count** | {} |\n", tech.column_count));
    md.push_str(&format!(
        "| **Export Format** | `{}` |\n",
        tech.export_format
    ));
    md.push_str(&format!(
        "| **Output Hash** | `{}` |\n",
        tech.output_dataset_hash
    ));

    if let Some(input_hash) = &tech.input_dataset_hash {
        md.push_str(&format!("| **Input Hash** | `{input_hash}` |\n"));
    }

    if let Some(pipeline_id) = tech.pipeline_id {
        md.push_str(&format!("| **Pipeline ID** | `{pipeline_id}` |\n"));
    }

    md.push('\n');

    // Input sources
    if !tech.input_sources.is_empty() {
        md.push_str("### Input Sources\n\n");
        for source in &tech.input_sources {
            md.push_str(&format!("- `{}`", source.path));
            if let Some(hash) = &source.hash {
                md.push_str(&format!(" (hash: `{}`)", &hash[..8]));
            }
            md.push('\n');
        }
        md.push('\n');
    }

    // Pipeline JSON (collapsible if present)
    if let Some(pipeline_json) = &tech.pipeline_json {
        md.push_str("<details>\n");
        md.push_str("<summary><strong>Pipeline Specification</strong></summary>\n\n");
        md.push_str("```json\n");
        md.push_str(pipeline_json);
        md.push_str("\n```\n\n");
        md.push_str("</details>\n\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::metadata::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_render_basic_markdown() {
        let dict = DataDictionary {
            snapshot_id: Uuid::new_v4(),
            dataset_name: "Test Dataset".to_owned(),
            export_timestamp: Utc::now(),
            dataset_metadata: DatasetMetadata {
                technical: TechnicalMetadata {
                    input_sources: vec![],
                    pipeline_id: None,
                    pipeline_json: None,
                    input_dataset_hash: None,
                    output_dataset_hash: "abc123".to_owned(),
                    row_count: 100,
                    column_count: 2,
                    export_format: "csv".to_owned(),
                    quality_summary: QualitySummary {
                        avg_null_percentage: 5.0,
                        empty_column_count: 0,
                        constant_column_count: 0,
                        duplicate_row_count: None,
                        overall_score: 95.0,
                    },
                },
                business: DatasetBusinessMetadata {
                    description: Some("A test dataset".to_owned()),
                    ..Default::default()
                },
            },
            columns: vec![],
            previous_snapshot_id: None,
        };

        let markdown = render_markdown(&dict).unwrap();

        assert!(markdown.contains("# Data Dictionary: Test Dataset"));
        assert!(markdown.contains("A test dataset"));
        assert!(markdown.contains("## Column Catalog"));
    }
}
