pub mod analysis;
pub mod health;
pub mod interpretation;
pub mod ml;
pub mod types;

pub use analysis::{
    analyse_df, auto_clean_df, calculate_correlation_matrix, clean_df, clean_df_lazy,
    get_parquet_write_options, load_df, load_df_lazy, sanitize_column_name, sanitize_column_names,
    save_df,
};
pub use health::calculate_file_health;
pub use types::{
    AnalysisResponse, BooleanStats, ColumnCleanConfig, ColumnKind, ColumnStats, ColumnSummary,
    CorrelationMatrix, FileHealth, ImputeMode, MlModelKind, NormalizationMethod, NumericStats,
    TemporalStats, TextCase, TextStats,
};

#[cfg(test)]
mod tests;
