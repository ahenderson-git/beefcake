use super::system::run_on_worker_thread;
use crate::export;

#[tauri::command]
pub async fn export_data(options: export::ExportOptions) -> Result<(), String> {
    use super::system::LARGE_FILE_WARNING_THRESHOLD;
    use beefcake::analyser::logic::types::ImputeMode;

    beefcake::utils::reset_abort_signal();

    // Memory safeguard logic
    let mut high_mem_ops = 0;
    #[expect(clippy::iter_over_hash_type)]
    for config in options.configs.values() {
        if config.active && config.ml_preprocessing {
            if config.impute_mode == ImputeMode::Median || config.impute_mode == ImputeMode::Mode {
                high_mem_ops += 1;
            }
            if config.clip_outliers {
                high_mem_ops += 1;
            }
        }
    }

    if high_mem_ops > 0
        && matches!(
            options.source.source_type,
            export::ExportSourceType::Analyser
        )
        && let Some(path) = &options.source.path
        && let Ok(meta) = std::fs::metadata(path)
        && meta.len() > LARGE_FILE_WARNING_THRESHOLD
    {
        beefcake::config::log_event(
            "Export",
            &format!(
                "Warning: {} memory-intensive operations selected for a large file ({}). This may cause OOM.",
                high_mem_ops,
                beefcake::utils::fmt_bytes(meta.len())
            ),
        );
    }

    run_on_worker_thread("export-worker", move || async move {
        let mut temp_files = beefcake::utils::TempFileCollection::new();
        let res = export::export_data_execution(options, &mut temp_files).await;

        if let Err(e) = &res {
            beefcake::config::log_event("Export", &format!("Export failed: {e}"));
        }

        // temp_files will be automatically cleaned up when it goes out of scope
        res.map_err(String::from)
    })
    .await
}

#[tauri::command]
pub async fn verify_receipt(
    receipt_path: String,
) -> Result<beefcake::integrity::VerificationResult, String> {
    use std::path::Path;

    beefcake::config::log_event("Integrity", &format!("Verifying receipt: {receipt_path}"));

    let path = Path::new(&receipt_path);
    if !path.exists() {
        return Err("Receipt file not found".to_owned());
    }

    beefcake::integrity::verify_receipt(path).map_err(|e| e.to_string())
}
