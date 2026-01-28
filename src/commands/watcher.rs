#[tauri::command]
pub async fn watcher_get_state() -> Result<beefcake::watcher::WatcherStatusPayload, String> {
    beefcake::watcher::get_state().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn watcher_start(
    folder: String,
) -> Result<beefcake::watcher::WatcherStatusPayload, String> {
    beefcake::config::log_event("Watcher", &format!("Started watching: {folder}"));
    beefcake::watcher::start(std::path::PathBuf::from(folder)).map_err(|e| e.to_string())?;
    beefcake::watcher::get_state().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn watcher_stop() -> Result<beefcake::watcher::WatcherStatusPayload, String> {
    beefcake::config::log_event("Watcher", "Stopped watching");
    beefcake::watcher::stop().map_err(|e| e.to_string())?;
    beefcake::watcher::get_state().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn watcher_set_folder(
    folder: String,
) -> Result<beefcake::watcher::WatcherStatusPayload, String> {
    beefcake::config::log_event("Watcher", &format!("Changed folder to: {folder}"));
    beefcake::watcher::set_folder(std::path::PathBuf::from(folder)).map_err(|e| e.to_string())?;
    beefcake::watcher::get_state().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn watcher_ingest_now(path: String) -> Result<(), String> {
    beefcake::watcher::ingest_now(std::path::PathBuf::from(path)).map_err(|e| e.to_string())
}
