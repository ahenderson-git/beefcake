use beefcake::ai::client::AIAssistant;
use beefcake::config::{AIConfig, load_app_config, save_app_config};

#[tauri::command]
pub async fn ai_send_query(query: String, context: Option<String>) -> Result<String, String> {
    // Get API key from keyring
    let api_key = beefcake::utils::get_ai_api_key().ok_or("AI API key not configured")?;

    // Get AI config from app settings
    let config = load_app_config();
    let ai_config = config.settings().ai_config.clone();

    // Create AI assistant
    let assistant = AIAssistant::new(api_key, ai_config)
        .map_err(|e| format!("Failed to initialize AI assistant: {e}"))?;

    // Send query
    assistant
        .send_query(&query, context.as_deref())
        .await
        .map_err(|e| format!("AI Query failed: {e}"))
}

#[tauri::command]
pub async fn ai_set_api_key(api_key: String) -> Result<(), String> {
    beefcake::utils::set_ai_api_key(&api_key).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ai_delete_api_key() -> Result<(), String> {
    beefcake::utils::delete_ai_api_key().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn ai_has_api_key() -> bool {
    beefcake::utils::get_ai_api_key().is_some()
}

#[tauri::command]
pub async fn ai_test_connection() -> Result<(), String> {
    // Get API key from keyring
    let api_key = beefcake::utils::get_ai_api_key().ok_or("AI API key not configured")?;

    // Get AI config from app settings
    let config = load_app_config();

    let ai_config = config.settings().ai_config.clone();

    // Create AI assistant and test
    let assistant = AIAssistant::new(api_key, ai_config)
        .map_err(|e| format!("Failed to initialize AI assistant: {e}"))?;

    assistant
        .test_connection()
        .await
        .map_err(|e| format!("Connection test failed: {e}"))
}

#[tauri::command]
pub fn ai_get_config() -> AIConfig {
    let config = load_app_config();

    config.settings().ai_config.clone()
}

#[tauri::command]
pub async fn ai_update_config(ai_config: AIConfig) -> Result<(), String> {
    let mut config = load_app_config();

    config.settings_mut().ai_config = ai_config;

    save_app_config(&config).map_err(|e| format!("Failed to save config: {e}"))
}
