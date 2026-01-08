use anyhow::{Context as _, Result, anyhow};
use linfa::prelude::*;
use linfa_linear::LinearRegression;
use linfa_logistic::LogisticRegression;
use linfa_trees::DecisionTree;
use ndarray::Array1;
use polars::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use super::types::{MlModelKind, MlResults};

#[expect(clippy::too_many_lines)]
pub fn train_model(
    df: &DataFrame,
    target_col: &str,
    model_kind: MlModelKind,
    progress: &Arc<AtomicU64>,
) -> Result<MlResults> {
    let start = Instant::now();
    progress.store(10, Ordering::SeqCst);

    // 0. Filter out rows where the target is null, as we cannot train on them
    let df = df.filter(
        &df.column(target_col)
            .context("Target column not found")?
            .is_not_null(),
    )?;
    progress.store(20, Ordering::SeqCst);

    if df.height() == 0 {
        return Err(anyhow!(
            "Training failed: All rows in target column '{target_col}' are empty (null)."
        ));
    }

    // 1. Prepare Features
    let feature_cols: Vec<String> = df
        .get_column_names()
        .iter()
        .map(|s| s.to_string())
        .filter(|s| s != target_col)
        .filter(|s| {
            let col = df.column(s).expect("Column exists");
            col.dtype().is_numeric() || col.dtype().is_bool()
        })
        .collect();
    progress.store(40, Ordering::SeqCst);

    if feature_cols.is_empty() {
        return Err(anyhow!(
            "No numeric feature columns found for training. Make sure to clean/preprocess your data first."
        ));
    }

    let x = df
        .select(&feature_cols)?
        .iter()
        .map(|s| s.cast(&DataType::Float64).map(Column::from))
        .collect::<PolarsResult<Vec<_>>>()
        .map(DataFrame::new)?
        .map_err(|e| anyhow!("Failed to cast features: {e}"))?
        .to_ndarray::<Float64Type>(IndexOrder::C)
        .context("Failed to create feature matrix")?;
    progress.store(60, Ordering::SeqCst);

    // 2. Prepare Target
    let target_series = df
        .column(target_col)
        .context("Target column not found")?
        .as_materialized_series();

    // Validate target for classification
    if matches!(
        model_kind,
        MlModelKind::LogisticRegression | MlModelKind::DecisionTree
    ) {
        let n_unique = target_series.n_unique()?;
        if n_unique < 2 {
            return Err(anyhow!(
                "{} failed: The target column '{}' must have at least two distinct classes for classification. Found {}.",
                model_kind.as_str(),
                target_col,
                n_unique
            ));
        }
    }

    let mut results = MlResults {
        model_kind,
        target_column: target_col.to_owned(),
        feature_columns: feature_cols.clone(),
        r2_score: None,
        accuracy: None,
        mse: None,
        duration: start.elapsed(),
        coefficients: None,
        intercept: None,
        interpretation: Vec::new(),
    };

    match model_kind {
        MlModelKind::LinearRegression => {
            let y: Array1<f64> = target_series
                .cast(&DataType::Float64)?
                .f64()?
                .into_no_null_iter()
                .collect();
            let dataset = Dataset::new(x, y);
            let model = LinearRegression::default()
                .fit(&dataset)
                .map_err(|e| anyhow!("Linear Regression training failed: {e}"))?;

            let prediction = model.predict(&dataset);
            results.r2_score = Some(prediction.r2(&dataset)?);
            results.mse = Some(prediction.mean_squared_error(&dataset)?);

            let mut coeffs = HashMap::new();
            for (i, name) in feature_cols.iter().enumerate() {
                coeffs.insert(name.clone(), model.params()[i]);
            }
            results.coefficients = Some(coeffs);
            results.intercept = Some(model.intercept());
        }
        MlModelKind::DecisionTree => {
            let y: Array1<usize> = target_series
                .cast(&DataType::UInt32)?
                .u32()?
                .into_no_null_iter()
                .map(|v| v as usize)
                .collect();
            let dataset = Dataset::new(x, y);
            let model = DecisionTree::params()
                .fit(&dataset)
                .map_err(|e| anyhow!("Decision Tree training failed: {e}"))?;

            let prediction = model.predict(&dataset);
            let cm = prediction.confusion_matrix(&dataset)?;
            results.accuracy = Some(cm.accuracy() as f64);
        }
        MlModelKind::LogisticRegression => {
            let y: Array1<usize> = target_series
                .cast(&DataType::UInt32)?
                .u32()?
                .into_no_null_iter()
                .map(|v| v as usize)
                .collect();
            let dataset = Dataset::new(x, y);
            let model = LogisticRegression::default()
                .fit(&dataset)
                .map_err(|e| anyhow!("Logistic Regression training failed: {e}"))?;

            let prediction = model.predict(&dataset);
            let cm = prediction.confusion_matrix(&dataset)?;
            results.accuracy = Some(cm.accuracy() as f64);
        }
    }

    results.duration = start.elapsed();
    generate_interpretation(&mut results);
    progress.store(100, Ordering::SeqCst);
    Ok(results)
}

fn generate_interpretation(res: &mut MlResults) {
    let target = &res.target_column;
    match res.model_kind {
        MlModelKind::LinearRegression => {
            if let Some(r2) = res.r2_score {
                let pct = (r2 * 100.0).max(0.0);
                if r2 > 0.7 {
                    res.interpretation.push(format!(
                        "Strong predictive model: explains {pct:.1}% of the variation in {target}."
                    ));
                } else if r2 > 0.3 {
                    res.interpretation.push(format!("Moderate predictive model: explains {pct:.1}% of the variation in {target}."));
                } else {
                    res.interpretation.push(format!("Weak predictive model: only explains {pct:.1}% of the variation in {target}. Other factors are likely at play."));
                }
            }

            if let Some(coeffs) = &res.coefficients {
                let mut sorted_coeffs: Vec<_> = coeffs.iter().collect();
                sorted_coeffs.sort_by(|a, b| {
                    b.1.abs()
                        .partial_cmp(&a.1.abs())
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                for (name, val) in sorted_coeffs.iter().take(3) {
                    let direction = if **val > 0.0 { "increase" } else { "decrease" };
                    res.interpretation.push(format!("Primary Driver: A higher '{name}' usually leads to an {direction} in {target}."));
                }
            }
        }
        MlModelKind::DecisionTree | MlModelKind::LogisticRegression => {
            if let Some(acc) = res.accuracy {
                let pct = acc * 100.0;
                res.interpretation.push(format!(
                    "The model correctly identifies the '{target}' category {pct:.1}% of the time."
                ));
                if acc > 0.8 {
                    res.interpretation
                        .push("This is considered a very reliable classification.".to_owned());
                } else if acc < 0.6 {
                    res.interpretation.push("The model is not much better than a coin flip; consider adding more relevant features.".to_owned());
                }
            }
        }
    }
}
