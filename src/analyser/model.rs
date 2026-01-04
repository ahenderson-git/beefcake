use super::logic::{
    ColumnSummary, FileHealth,
    types::{ColumnCleanConfig, MlModelKind, MlResults},
};
use polars::prelude::DataFrame;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MyJoinType {
    #[default]
    Inner,
    Left,
    Outer,
}

#[derive(Default, Deserialize, Serialize)]
pub struct AnalysisModel {
    pub file_path: Option<String>,
    pub file_size: u64,
    pub summary: Vec<ColumnSummary>,
    pub correlation_matrix: Option<super::logic::CorrelationMatrix>,
    pub health: Option<FileHealth>,
    pub trim_pct: f64,
    pub show_full_range: bool,
    pub categorical_as_pie: bool,
    pub last_duration: Option<std::time::Duration>,

    // Join Config
    pub join_key_primary: String,
    pub join_key_secondary: String,
    pub join_type: MyJoinType,

    // Database Connection Info (Split)
    pub pg_type: String,
    pub pg_host: String,
    pub pg_port: String,
    pub pg_user: String,
    #[serde(skip)]
    pub pg_password: SecretString,
    pub pg_database: String,

    pub pg_schema: String,
    pub pg_table: String,
    pub save_password: bool,
    pub push_last_duration: Option<std::time::Duration>,
    pub cleaning_configs: HashMap<String, ColumnCleanConfig>,
    pub ml_target: Option<String>,
    pub ml_model_kind: MlModelKind,
    pub ml_results: Option<MlResults>,
    #[serde(skip)]
    pub df: Option<DataFrame>,
    #[serde(skip)]
    pub secondary_df: Option<DataFrame>,
    pub secondary_summary: Vec<ColumnSummary>,
}
