use crate::analyser::logic::{
    AnalysisReceiver, AnalysisResponse, ColumnSummary, FileHealth, MlModelKind, MlResults,
};
use anyhow::Result;
use eframe::egui;
use polars::prelude::DataFrame;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct AnalysisController {
    pub is_loading: bool,
    pub progress_counter: Arc<AtomicU64>,
    pub receiver: Option<AnalysisReceiver>,
    pub start_time: Option<std::time::Instant>,
    pub secondary_receiver: Option<AnalysisReceiver>,
    pub is_pushing: bool,
    pub push_start_time: Option<std::time::Instant>,
    pub push_receiver: Option<crossbeam_channel::Receiver<Result<()>>>,
    pub was_cleaning: bool,
    pub is_training: bool,
    pub training_receiver: Option<crossbeam_channel::Receiver<Result<MlResults>>>,
    pub is_testing: bool,
    pub test_receiver: Option<crossbeam_channel::Receiver<Result<()>>>,
    pub is_exporting: bool,
    pub export_receiver: Option<crossbeam_channel::Receiver<Result<()>>>,
    pub training_start_time: Option<std::time::Instant>,
    pub training_progress: Arc<AtomicU64>,
}

impl Default for AnalysisController {
    fn default() -> Self {
        Self {
            is_loading: false,
            progress_counter: Arc::new(AtomicU64::new(0)),
            receiver: None,
            start_time: None,
            secondary_receiver: None,
            is_pushing: false,
            push_start_time: None,
            push_receiver: None,
            was_cleaning: false,
            is_training: false,
            training_receiver: None,
            is_testing: false,
            test_receiver: None,
            is_exporting: false,
            export_receiver: None,
            training_start_time: None,
            training_progress: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl AnalysisController {
    fn prepare_analysis(
        &mut self,
        was_cleaning: bool,
    ) -> crossbeam_channel::Sender<Result<AnalysisResponse>> {
        self.is_loading = true;
        self.was_cleaning = was_cleaning;
        self.progress_counter.store(0, Ordering::SeqCst);
        self.start_time = Some(std::time::Instant::now());

        let (tx, rx) = crossbeam_channel::unbounded();
        self.receiver = Some(rx);
        tx
    }

    fn spawn_task<T, F>(ctx: egui::Context, tx: crossbeam_channel::Sender<Result<T>>, f: F)
    where
        T: Send + 'static,
        F: FnOnce() -> Result<T> + Send + 'static,
    {
        std::thread::spawn(move || {
            let result = f();
            if tx.send(result).is_err() {
                log::error!("Failed to send result");
            }
            ctx.request_repaint();
        });
    }

    pub fn start_analysis(&mut self, ctx: egui::Context, path: PathBuf, trim_pct: f64) {
        let tx = self.prepare_analysis(false);
        let progress = Arc::clone(&self.progress_counter);
        let path_str = path.to_string_lossy().to_string();

        Self::spawn_task(ctx, tx, move || {
            let start = std::time::Instant::now();
            let df = super::logic::load_df(&path, &progress)?;
            let file_size = std::fs::metadata(&path)?.len();
            Self::run_full_analysis(df, path_str, file_size, trim_pct, start)
        });
    }

    pub fn trigger_reanalysis(
        &mut self,
        ctx: egui::Context,
        df: DataFrame,
        file_path: String,
        file_size: u64,
        trim_pct: f64,
    ) {
        let tx = self.prepare_analysis(false);
        Self::spawn_task(ctx, tx, move || {
            let start = std::time::Instant::now();
            Self::run_full_analysis(df, file_path, file_size, trim_pct, start)
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
        let tx = self.prepare_analysis(true);
        let path_str = file_path.unwrap_or_else(|| "cleaned_file".to_owned());

        Self::spawn_task(ctx, tx, move || {
            let start = std::time::Instant::now();
            let cleaned_df = super::logic::analysis::clean_df(df, &configs)?;
            Self::run_full_analysis(cleaned_df, path_str, file_size, trim_pct, start)
        });
    }

    #[expect(clippy::too_many_arguments)]
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

        Self::spawn_task(ctx, tx, move || {
            crate::utils::TOKIO_RUNTIME.block_on(async {
                let client = super::db::DbClient::connect(pg_options).await?;
                client.init_schema().await?;

                let schema_opt = if pg_schema.is_empty() {
                    None
                } else {
                    Some(pg_schema.as_str())
                };
                let table_opt = if pg_table.is_empty() {
                    None
                } else {
                    Some(pg_table.as_str())
                };

                client
                    .push_analysis(super::db::AnalysisPush {
                        file_path: &file_path,
                        file_size,
                        health: &health,
                        summaries: &summary,
                        df: &df,
                        schema_name: schema_opt,
                        table_name: table_opt,
                    })
                    .await?;
                Ok(())
            })
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
        self.training_start_time = Some(std::time::Instant::now());
        self.training_progress.store(0, Ordering::SeqCst);

        let (tx, rx) = crossbeam_channel::unbounded();
        self.training_receiver = Some(rx);

        let progress = Arc::clone(&self.training_progress);
        Self::spawn_task(ctx, tx, move || {
            super::logic::train_model(&df, &target_col, model_kind, &progress)
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

        Self::spawn_task(ctx, tx, move || {
            crate::utils::TOKIO_RUNTIME.block_on(async {
                super::db::DbClient::connect(pg_options).await?;
                Ok(())
            })
        });
    }

    pub fn start_export(&mut self, ctx: egui::Context, mut df: DataFrame, path: PathBuf) {
        self.is_exporting = true;
        let (tx, rx) = crossbeam_channel::unbounded();
        self.export_receiver = Some(rx);

        Self::spawn_task(ctx, tx, move || {
            super::logic::analysis::save_df(&mut df, &path)
        });
    }

    pub fn start_secondary_analysis(&mut self, ctx: egui::Context, path: PathBuf, trim_pct: f64) {
        let (tx, rx) = crossbeam_channel::unbounded();
        self.secondary_receiver = Some(rx);

        let progress = Arc::clone(&self.progress_counter);
        let path_str = path.to_string_lossy().to_string();

        Self::spawn_task(ctx, tx, move || {
            let start = std::time::Instant::now();
            let df = super::logic::load_df(&path, &progress)?;
            let file_size = std::fs::metadata(&path)?.len();
            Self::run_full_analysis(df, path_str, file_size, trim_pct, start)
        });
    }

    fn run_full_analysis(
        df: DataFrame,
        file_path: String,
        file_size: u64,
        trim_pct: f64,
        start_time: std::time::Instant,
    ) -> Result<AnalysisResponse> {
        let summary = super::logic::analyse_df(&df, trim_pct)?;
        let health = super::logic::calculate_file_health(&summary);
        let correlation_matrix = super::logic::calculate_correlation_matrix(&df)?;
        Ok(AnalysisResponse {
            file_path,
            file_size,
            summary,
            health,
            duration: start_time.elapsed(),
            df,
            correlation_matrix,
        })
    }
}
