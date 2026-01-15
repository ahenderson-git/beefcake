pub mod analysis;
pub mod cleaning;
pub mod flows;
pub mod health;
pub mod interpretation;
pub mod io;
pub mod ml;
pub mod naming;
pub mod profiling;
pub mod types;

pub use analysis::{
    analyse_df, analyse_df_lazy, calculate_correlation_matrix, run_full_analysis,
    run_full_analysis_streaming,
};
pub use flows::{analyze_file_flow, generate_auto_clean_configs, push_to_db_flow};
pub use cleaning::{auto_clean_df, clean_df, clean_df_lazy};
pub use health::calculate_file_health;
pub use io::{get_parquet_write_options, load_df, load_df_lazy, save_df};
pub use naming::{sanitize_column_name, sanitize_column_names};
pub use types::{
    AnalysisResponse, BooleanStats, ColumnCleanConfig, ColumnKind, ColumnStats, ColumnSummary,
    CorrelationMatrix, FileHealth, ImputeMode, MlModelKind, NormalisationMethod, NumericStats,
    TemporalStats, TextCase, TextStats,
};

#[cfg(test)]
mod tests;
