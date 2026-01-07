pub mod analysis;
pub mod health;
pub mod interpretation;
pub mod ml;
pub mod types;

pub use analysis::{
    auto_clean_df, analyse_df, calculate_correlation_matrix, clean_df, load_df, save_df,
};
pub use health::calculate_file_health;
pub use ml::train_model;
pub use types::{
    AnalysisResponse, BooleanStats, ColumnCleanConfig, ColumnStats, ColumnSummary,
    CorrelationMatrix, FileHealth, MlModelKind, MlResults, NumericStats, TemporalStats, TextStats,
};

#[cfg(test)]
mod tests;
