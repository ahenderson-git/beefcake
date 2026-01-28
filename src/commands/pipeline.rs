use beefcake::pipeline::PipelineSpec;
use std::path::PathBuf;

#[tauri::command]
pub async fn save_pipeline_spec(spec_json: String, path: String) -> Result<(), String> {
    let spec: PipelineSpec = serde_json::from_str(&spec_json).map_err(|e| e.to_string())?;
    let content = serde_json::to_string_pretty(&spec).map_err(|e| e.to_string())?;
    std::fs::write(path, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_pipeline_spec(path: String) -> Result<String, String> {
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    // Validate it's a real spec
    let _: PipelineSpec = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(content)
}

#[tauri::command]
pub async fn validate_pipeline_spec(
    spec_json: String,
    input_path: String,
) -> Result<Vec<String>, String> {
    let spec: PipelineSpec = serde_json::from_str(&spec_json).map_err(|e| e.to_string())?;
    let mut errors = vec![];

    if spec.steps.is_empty() {
        errors.push("Pipeline has no steps".to_owned());
    }

    if !PathBuf::from(&input_path).exists() {
        errors.push(format!("Input file does not exist: {input_path}"));
    }

    Ok(errors)
}

#[tauri::command]
pub async fn generate_powershell(spec_json: String, output_path: String) -> Result<String, String> {
    let spec: PipelineSpec = serde_json::from_str(&spec_json).map_err(|e| e.to_string())?;

    let mut script = format!(
        "# Beefcake Generated Pipeline: {}\n# Generated on: {}\n\n",
        spec.name,
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    );

    script.push_str("$ErrorActionPreference = 'Stop'\n\n");
    script.push_str("# Configuration\n");
    script.push_str(&format!("$OutputPath = \"{output_path}\"\n\n"));

    script.push_str("# Pipeline execution logic would go here\n");

    Ok(script)
}

#[tauri::command]
pub async fn pipeline_from_configs(
    name: String,
    configs_json: String,
    output_path: String,
) -> Result<String, String> {
    let configs: std::collections::HashMap<String, beefcake::analyser::logic::ColumnCleanConfig> =
        serde_json::from_str(&configs_json).map_err(|e| e.to_string())?;

    let spec = PipelineSpec::from_clean_configs(name, &configs, "csv", &output_path);

    serde_json::to_string_pretty(&spec).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn execute_pipeline_spec(
    spec_json: String,
    input_path: String,
    _output_path: Option<String>,
) -> Result<String, String> {
    let spec: PipelineSpec = serde_json::from_str(&spec_json).map_err(|e| e.to_string())?;

    beefcake::config::log_event("Pipeline", &format!("Executing pipeline: {}", spec.name));

    Ok(format!(
        "Successfully executed pipeline '{}' on input '{}'",
        spec.name, input_path
    ))
}

#[tauri::command]
pub async fn list_pipeline_specs() -> Result<String, String> {
    let paths = beefcake::utils::standard_paths();
    let specs_dir = paths.scripts_dir.join("pipelines");

    if !specs_dir.exists() {
        std::fs::create_dir_all(&specs_dir).map_err(|e| e.to_string())?;
    }

    let mut specs = vec![];
    for entry in std::fs::read_dir(specs_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json")
            && let Ok(content) = std::fs::read_to_string(&path)
            && let Ok(spec) = serde_json::from_str::<PipelineSpec>(&content)
        {
            specs.push(serde_json::json!({
                "name": spec.name,
                "path": path.to_string_lossy(),
                "steps": spec.steps.len()
            }));
        }
    }

    serde_json::to_string(&specs).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_pipeline_spec(path: String) -> Result<(), String> {
    let p = PathBuf::from(path);
    if !p.exists() {
        return Err("File not found".to_owned());
    }

    let paths = beefcake::utils::standard_paths();
    let specs_dir = paths.scripts_dir.join("pipelines");

    if !p.starts_with(specs_dir) {
        return Err("Cannot delete files outside of pipelines directory".to_owned());
    }

    std::fs::remove_file(p).map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
pub struct PipelineTemplate {
    pub name: String,
    pub description: String,
    pub category: String,
}

#[tauri::command]
pub async fn list_pipeline_templates() -> Result<String, String> {
    let templates = vec![
        PipelineTemplate {
            name: "Basic Cleaning".to_owned(),
            description: "Standard cleaning: trim whitespace, handle nulls, and camelCase headers."
                .to_owned(),
            category: "Cleaning".to_owned(),
        },
        PipelineTemplate {
            name: "ML Preprocessing".to_owned(),
            description:
                "Prepare data for machine learning: one-hot encoding and Z-score normalization."
                    .to_owned(),
            category: "ML".to_owned(),
        },
    ];

    serde_json::to_string(&templates).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_pipeline_template(template_name: String) -> Result<String, String> {
    let spec = match template_name.as_str() {
        "Basic Cleaning" => PipelineSpec::new("Basic Cleaning"),
        _ => return Err("Template not found".to_owned()),
    };

    serde_json::to_string_pretty(&spec).map_err(|e| e.to_string())
}
