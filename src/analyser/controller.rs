use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::path::PathBuf;
use eframe::egui;
use polars::prelude::DataFrame;
use anyhow::Result;
use crate::analyser::logic::{AnalysisReceiver, ColumnSummary, FileHealth, MlResults, MlModelKind};

pub struct AnalysisController {
    pub is_loading: bool,
    pub progress_counter: Arc<AtomicU64>,
    pub receiver: Option<AnalysisReceiver>,
    pub start_time: Option<std::time::Instant>,
    pub is_pushing: bool,
    pub push_start_time: Option<std::time::Instant>,
    pub push_receiver: Option<crossbeam_channel::Receiver<Result<()>>>,
    pub was_cleaning: bool,
    pub is_training: bool,
    pub training_receiver: Option<crossbeam_channel::Receiver<Result<MlResults>>>,
    pub is_testing: bool,
    pub test_receiver: Option<crossbeam_channel::Receiver<Result<()>>>,
}

impl Default for AnalysisController {
    fn default() -> Self {
        Self {
            is_loading: false,
            progress_counter: Arc::new(AtomicU64::new(0)),
            receiver: None,
            start_time: None,
            is_pushing: false,
            push_start_time: None,
            push_receiver: None,
            was_cleaning: false,
            is_training: false,
            training_receiver: None,
            is_testing: false,
            test_receiver: None,
        }
    }
}

impl AnalysisController {
    pub fn start_analysis(&mut self, ctx: egui::Context, path: PathBuf, trim_pct: f64) {
        self.is_loading = true;
        self.was_cleaning = false;
        self.progress_counter.store(0, Ordering::SeqCst);
        self.start_time = Some(std::time::Instant::now());

        let (tx, rx) = crossbeam_channel::unbounded();
        self.receiver = Some(rx);

        let progress = Arc::clone(&self.progress_counter);
        let path_str = path.to_string_lossy().to_string();

        std::thread::spawn(move || {
            let start = std::time::Instant::now();
            let result = (|| -> Result<(String, u64, Vec<ColumnSummary>, FileHealth, std::time::Duration, DataFrame)> {
                let df = super::logic::load_df(&path, &progress)?;
                let file_size = std::fs::metadata(&path)?.len();
                let summary = super::logic::analyse_df(&df, trim_pct)?;
                let health = super::logic::calculate_file_health(&summary);
                Ok((path_str, file_size, summary, health, start.elapsed(), df))
            })();

            if tx.send(result).is_err() {
                log::error!("Failed to send analysis result");
            }
            ctx.request_repaint();
        });
    }

    pub fn trigger_reanalysis(&mut self, ctx: egui::Context, df: DataFrame, file_path: String, file_size: u64, trim_pct: f64) {
        self.is_loading = true;
        self.was_cleaning = false;
        self.progress_counter.store(0, Ordering::SeqCst);
        self.start_time = Some(std::time::Instant::now());

        let (tx, rx) = crossbeam_channel::unbounded();
        self.receiver = Some(rx);

        std::thread::spawn(move || {
            let start = std::time::Instant::now();
            let result = (|| -> Result<(String, u64, Vec<ColumnSummary>, FileHealth, std::time::Duration, DataFrame)> {
                let summary = super::logic::analyse_df(&df, trim_pct)?;
                let health = super::logic::calculate_file_health(&summary);
                Ok((file_path, file_size, summary, health, start.elapsed(), df))
            })();

            if tx.send(result).is_err() {
                log::error!("Failed to send re-analysis result");
            }
            ctx.request_repaint();
        });
    }

    pub fn start_cleaning(
        &mut self,
        ctx: egui::Context,
        df: DataFrame,
        configs: std::collections::HashMap<String, super::logic::types::ColumnCleanConfig>,
        trim_pct: f64,
        file_path: Option<String>,
        file_size: u64,
    ) {
        let path_str = file_path.unwrap_or_else(|| "cleaned_file".to_owned());

        self.is_loading = true;
        self.was_cleaning = true;
        self.progress_counter.store(0, Ordering::SeqCst);
        self.start_time = Some(std::time::Instant::now());

        let (tx, rx) = crossbeam_channel::unbounded();
        self.receiver = Some(rx);

        std::thread::spawn(move || {
            let start = std::time::Instant::now();
            let result = (|| -> Result<(String, u64, Vec<ColumnSummary>, FileHealth, std::time::Duration, DataFrame)> {
                let cleaned_df = super::logic::analysis::clean_df(df, &configs)?;
                let summary = super::logic::analyse_df(&cleaned_df, trim_pct)?;
                let health = super::logic::calculate_file_health(&summary);
                Ok((path_str, file_size, summary, health, start.elapsed(), cleaned_df))
            })();

            if tx.send(result).is_err() {
                log::error!("Failed to send cleaning result");
            }
            ctx.request_repaint();
        });
    }

    pub fn start_push_to_db(
        &mut self,
        ctx: egui::Context,
        file_path: String,
        file_size: u64,
        health: FileHealth,
        summary: Vec<ColumnSummary>,
        df: DataFrame,
        pg_options: sqlx::postgres::PgConnectOptions,
        pg_schema: String,
        pg_table: String,
    ) {
        self.is_pushing = true;
        self.push_start_time = Some(std::time::Instant::now());

        let (tx, rx) = crossbeam_channel::unbounded();
        self.push_receiver = Some(rx);

        std::thread::spawn(move || {
            let result = (|| -> Result<()> {
                let rt = tokio::runtime::Runtime::new()?;
                rt.block_on(async {
                    let client = super::db::DbClient::connect(pg_options).await?;
                    client.init_schema().await?;
                    
                    let schema_opt = if pg_schema.is_empty() { None } else { Some(pg_schema.as_str()) };
                    let table_opt = if pg_table.is_empty() { None } else { Some(pg_table.as_str()) };

                    client.push_analysis(super::db::AnalysisPush {
                        file_path: &file_path,
                        file_size,
                        health: &health,
                        summaries: &summary,
                        df: &df,
                        schema_name: schema_opt,
                        table_name: table_opt,
                    }).await?;
                    Ok(())
                })
            })();

            if tx.send(result).is_err() {
                log::error!("Failed to send push result");
            }
            ctx.request_repaint();
        });
    }

    pub fn start_training(
        &mut self,
        ctx: egui::Context,
        df: DataFrame,
        target_col: String,
        model_kind: MlModelKind,
    ) {
        self.is_training = true;
        let (tx, rx) = crossbeam_channel::unbounded();
        self.training_receiver = Some(rx);

        std::thread::spawn(move || {
            let result = super::logic::train_model(&df, &target_col, model_kind);
            if tx.send(result).is_err() {
                log::error!("Failed to send training result");
            }
            ctx.request_repaint();
        });
    }

    pub fn start_test_connection(
        &mut self,
        ctx: egui::Context,
        pg_options: sqlx::postgres::PgConnectOptions,
    ) {
        self.is_testing = true;
        let (tx, rx) = crossbeam_channel::unbounded();
        self.test_receiver = Some(rx);

        std::thread::spawn(move || {
            let result = (|| -> Result<()> {
                let rt = tokio::runtime::Runtime::new()?;
                rt.block_on(async {
                    super::db::DbClient::connect(pg_options).await?;
                    Ok(())
                })
            })();

            if tx.send(result).is_err() {
                log::error!("Failed to send test connection result");
            }
            ctx.request_repaint();
        });
    }
}
