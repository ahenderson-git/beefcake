#![allow(
    clippy::let_underscore_must_use,
    clippy::let_underscore_untyped,
    clippy::print_stderr,
    clippy::exit,
    clippy::collapsible_if
)]
use crate::commands;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            // System
            commands::system::get_app_version,
            commands::system::read_text_file,
            commands::system::write_text_file,
            commands::system::get_config,
            commands::system::save_config,
            commands::system::get_standard_paths,
            commands::system::open_path,
            commands::system::list_trusted_paths,
            commands::system::add_trusted_path,
            commands::system::remove_trusted_path,
            commands::system::list_documentation_files,
            commands::system::read_documentation_file,
            commands::system::log_frontend_error,
            commands::system::log_frontend_event,
            commands::system::get_log_directory,
            commands::system::get_current_log_file,
            commands::system::get_current_error_log_file,

            // Analysis
            commands::analysis::analyze_file,
            commands::analysis::run_powershell,
            commands::analysis::run_python,
            commands::analysis::run_sql,
            commands::analysis::sanitize_headers,
            commands::analysis::push_to_db,
            commands::analysis::abort_processing,
            commands::analysis::reset_abort_signal,
            commands::analysis::test_connection,
            commands::analysis::delete_connection,
            commands::analysis::install_python_package,
            commands::analysis::check_python_environment,

            // Integrity
            commands::integrity::export_data,
            commands::integrity::verify_receipt,

            // Lifecycle
            commands::lifecycle::lifecycle_create_dataset,
            commands::lifecycle::lifecycle_apply_transforms,
            commands::lifecycle::lifecycle_set_active_version,
            commands::lifecycle::lifecycle_publish_version,
            commands::lifecycle::lifecycle_get_version_diff,
            commands::lifecycle::lifecycle_list_versions,
            commands::lifecycle::lifecycle_get_version_schema,

            // Pipeline
            commands::pipeline::save_pipeline_spec,
            commands::pipeline::load_pipeline_spec,
            commands::pipeline::validate_pipeline_spec,
            commands::pipeline::generate_powershell,
            commands::pipeline::pipeline_from_configs,
            commands::pipeline::execute_pipeline_spec,
            commands::pipeline::delete_pipeline_spec,
            commands::pipeline::list_pipeline_specs,
            commands::pipeline::list_pipeline_templates,
            commands::pipeline::load_pipeline_template,

            // Dictionary
            commands::dictionary::dictionary_load_snapshot,
            commands::dictionary::dictionary_list_snapshots,
            commands::dictionary::dictionary_update_business_metadata,
            commands::dictionary::dictionary_export_markdown,

            // Watcher
            commands::watcher::watcher_get_state,
            commands::watcher::watcher_start,
            commands::watcher::watcher_stop,
            commands::watcher::watcher_set_folder,
            commands::watcher::watcher_ingest_now,

            // AI
            commands::ai::ai_send_query,
            commands::ai::ai_set_api_key,
            commands::ai::ai_delete_api_key,
            commands::ai::ai_has_api_key,
            commands::ai::ai_test_connection,
            commands::ai::ai_get_config,
            commands::ai::ai_update_config,
        ])
        .setup(|app| {
            // Initialize watcher service
            if let Err(e) = beefcake::watcher::init(app.handle().clone()) {
                tracing::error!("Failed to initialize watcher service: {}", e);
            }
            if let Err(e) = beefcake::utils::ensure_standard_dirs() {
                tracing::error!("Failed to initialize standard directories: {}", e);
            }
            tracing::info!("Tauri setup complete");
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            if matches!(event, tauri::RunEvent::Exit) {
                // Flush any pending audit log entries before exit
                beefcake::config::flush_pending_audit_entries();
            }
        });
}
