use beefcake::analyser::lifecycle::transforms::{TransformPipeline, TransformSpec};
use beefcake::analyser::lifecycle::{DatasetRegistry, LifecycleStage};
use std::path::PathBuf;
use std::sync::Arc;

use super::system::run_on_worker_thread;

static REGISTRY: std::sync::OnceLock<Arc<DatasetRegistry>> = std::sync::OnceLock::new();

pub fn get_or_create_registry() -> Result<Arc<DatasetRegistry>, String> {
    if let Some(registry) = REGISTRY.get() {
        Ok(registry.clone())
    } else {
        let paths = beefcake::utils::standard_paths();
        let registry_path = paths.base_dir.join("datasets");

        if !registry_path.exists() {
            std::fs::create_dir_all(&registry_path).map_err(|e| e.to_string())?;
        }

        let registry = DatasetRegistry::new(registry_path).map_err(|e| e.to_string())?;
        let registry = Arc::new(registry);
        REGISTRY
            .set(registry.clone())
            .map_err(|_err| "Failed to set registry".to_owned())?;
        Ok(registry)
    }
}

#[derive(serde::Deserialize)]
pub struct CreateDatasetRequest {
    pub name: String,
    pub source_path: String,
}

#[tauri::command]
pub async fn lifecycle_create_dataset(request: CreateDatasetRequest) -> Result<String, String> {
    let registry = get_or_create_registry()?;
    let dataset_id = registry
        .create_dataset(request.name, PathBuf::from(request.source_path))
        .map_err(|e| e.to_string())?;
    Ok(dataset_id.to_string())
}

#[derive(serde::Deserialize)]
pub struct ApplyTransformsRequest {
    pub dataset_id: String,
    pub transforms: Vec<TransformSpec>,
    pub next_stage: LifecycleStage,
}

#[tauri::command]
pub async fn lifecycle_apply_transforms(request: ApplyTransformsRequest) -> Result<String, String> {
    let registry = get_or_create_registry()?;
    let dataset_id = uuid::Uuid::parse_str(&request.dataset_id).map_err(|e| e.to_string())?;

    run_on_worker_thread("lifecycle-worker", move || async move {
        let pipeline = TransformPipeline::new(request.transforms);
        let new_version_id = registry
            .apply_transforms(&dataset_id, pipeline, request.next_stage)
            .map_err(|e: anyhow::Error| e.to_string())?;
        Ok(new_version_id.to_string())
    })
    .await
}

#[derive(serde::Deserialize)]
pub struct SetActiveVersionRequest {
    pub dataset_id: String,
    pub version_id: String,
}

#[tauri::command]
pub async fn lifecycle_set_active_version(request: SetActiveVersionRequest) -> Result<(), String> {
    let registry = get_or_create_registry()?;
    let dataset_id = uuid::Uuid::parse_str(&request.dataset_id).map_err(|e| e.to_string())?;
    let version_id = uuid::Uuid::parse_str(&request.version_id).map_err(|e| e.to_string())?;

    registry
        .set_active_version(&dataset_id, &version_id)
        .map_err(|e| e.to_string())
}

#[derive(serde::Deserialize)]
pub struct PublishVersionRequest {
    pub dataset_id: String,
    pub version_id: String,
}

#[tauri::command]
pub async fn lifecycle_publish_version(request: PublishVersionRequest) -> Result<String, String> {
    let registry = get_or_create_registry()?;
    let dataset_id = uuid::Uuid::parse_str(&request.dataset_id).map_err(|e| e.to_string())?;
    let version_id = uuid::Uuid::parse_str(&request.version_id).map_err(|e| e.to_string())?;

    run_on_worker_thread("publish-worker", move || async move {
        let published_version_id = registry
            .publish_version(
                &dataset_id,
                &version_id,
                beefcake::analyser::lifecycle::PublishMode::Snapshot,
            )
            .map_err(|e| e.to_string())?;
        Ok(published_version_id.to_string())
    })
    .await
}

#[derive(serde::Deserialize)]
pub struct GetVersionDiffRequest {
    pub dataset_id: String,
    pub from_version_id: String,
    pub to_version_id: String,
}

#[tauri::command]
pub async fn lifecycle_get_version_diff(
    request: GetVersionDiffRequest,
) -> Result<beefcake::analyser::lifecycle::DiffSummary, String> {
    let registry = get_or_create_registry()?;
    let dataset_id = uuid::Uuid::parse_str(&request.dataset_id).map_err(|e| e.to_string())?;
    let from_id = uuid::Uuid::parse_str(&request.from_version_id).map_err(|e| e.to_string())?;
    let to_id = uuid::Uuid::parse_str(&request.to_version_id).map_err(|e| e.to_string())?;

    registry
        .compute_diff(&dataset_id, &from_id, &to_id)
        .map_err(|e| e.to_string())
}

#[derive(serde::Deserialize)]
pub struct ListVersionsRequest {
    pub dataset_id: String,
}

#[tauri::command]
pub async fn lifecycle_list_versions(request: ListVersionsRequest) -> Result<String, String> {
    let registry = get_or_create_registry()?;
    let dataset_id = uuid::Uuid::parse_str(&request.dataset_id).map_err(|e| e.to_string())?;

    let dataset = registry
        .get_dataset(&dataset_id)
        .map_err(|e| e.to_string())?;

    // Return array of versions, not the entire dataset
    let versions = dataset.versions.list_all();
    serde_json::to_string(&versions).map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
pub struct ColumnInfo {
    pub name: String,
    pub dtype: String,
}

#[derive(serde::Deserialize)]
pub struct GetVersionSchemaRequest {
    pub dataset_id: String,
    pub version_id: String,
}

#[tauri::command]
pub async fn lifecycle_get_version_schema(
    request: GetVersionSchemaRequest,
) -> Result<Vec<ColumnInfo>, String> {
    let registry = get_or_create_registry()?;
    let dataset_id = uuid::Uuid::parse_str(&request.dataset_id).map_err(|e| e.to_string())?;
    let version_id = uuid::Uuid::parse_str(&request.version_id).map_err(|e| e.to_string())?;

    let version = registry
        .get_version(&dataset_id, &version_id)
        .map_err(|e| e.to_string())?;

    // Load actual schema from the version's data path
    let mut lf =
        polars::prelude::LazyFrame::scan_parquet(version.data_location.path(), Default::default())
            .map_err(|e| e.to_string())?;

    let schema = lf.collect_schema().map_err(|e| e.to_string())?;

    let columns = schema
        .iter()
        .map(|(name, dtype)| ColumnInfo {
            name: name.to_string(),
            dtype: format!("{dtype:?}"),
        })
        .collect();

    Ok(columns)
}
