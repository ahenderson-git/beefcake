use beefcake::dictionary::storage::SnapshotMetadata;
use beefcake::dictionary::{DataDictionary, list_snapshots, load_snapshot, save_snapshot};
use std::path::PathBuf;

fn get_dictionary_dir() -> PathBuf {
    beefcake::utils::standard_paths()
        .base_dir
        .join("dictionaries")
}

#[tauri::command]
pub async fn dictionary_load_snapshot(snapshot_id: String) -> Result<DataDictionary, String> {
    let snapshot_id = uuid::Uuid::parse_str(&snapshot_id).map_err(|e| e.to_string())?;
    load_snapshot(&snapshot_id, &get_dictionary_dir()).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn dictionary_list_snapshots(
    dataset_hash: Option<String>,
) -> Result<Vec<SnapshotMetadata>, String> {
    list_snapshots(&get_dictionary_dir(), dataset_hash.as_deref()).map_err(|e| e.to_string())
}

#[derive(serde::Deserialize)]
pub struct UpdateBusinessMetadataRequest {
    pub snapshot_id: String,
    pub column_name: String,
    pub description: Option<String>,
    pub data_owner: Option<String>,
}

#[tauri::command]
pub async fn dictionary_update_business_metadata(
    request: UpdateBusinessMetadataRequest,
) -> Result<String, String> {
    let snapshot_id = uuid::Uuid::parse_str(&request.snapshot_id).map_err(|e| e.to_string())?;
    let dictionary_dir = get_dictionary_dir();

    let mut dictionary = load_snapshot(&snapshot_id, &dictionary_dir).map_err(|e| e.to_string())?;

    if let Some(col) = dictionary
        .columns
        .iter_mut()
        .find(|c| c.current_name == request.column_name)
    {
        if let Some(desc) = request.description {
            col.business.business_definition = Some(desc);
        }
        if let Some(owner) = request.data_owner {
            col.business.notes = Some(format!("Data Owner: {owner}"));
        }
    } else {
        return Err(format!(
            "Column '{}' not found in dictionary",
            request.column_name
        ));
    }

    save_snapshot(&dictionary, &dictionary_dir).map_err(|e| e.to_string())?;

    Ok("Metadata updated successfully".to_owned())
}

#[tauri::command]
pub async fn dictionary_export_markdown(
    snapshot_id: String,
    output_path: String,
) -> Result<(), String> {
    let snapshot_id = uuid::Uuid::parse_str(&snapshot_id).map_err(|e| e.to_string())?;
    let dictionary =
        load_snapshot(&snapshot_id, &get_dictionary_dir()).map_err(|e| e.to_string())?;

    let markdown = beefcake::dictionary::render_markdown(&dictionary).map_err(|e| e.to_string())?;
    std::fs::write(output_path, markdown).map_err(|e| e.to_string())
}
