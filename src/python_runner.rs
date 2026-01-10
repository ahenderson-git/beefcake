use anyhow::Result;
use beefcake::analyser::logic::ColumnCleanConfig;
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use uuid::Uuid;

pub fn python_preamble() -> String {
    r#"import os
import polars as pl
import sys

# Disable column truncation
pl.Config.set_tbl_cols(-1)
pl.Config.set_tbl_rows(100)
"#
    .to_string()
}

pub fn python_load_snippet(data_path: &str, lf_var: &str) -> String {
    format!(
        r#"
if {data_path}.endswith(".parquet"):
    {lf} = pl.scan_parquet({data_path})
elif {data_path}.endswith(".json"):
    {lf} = pl.read_json({data_path}).lazy()
else:
    {lf} = pl.scan_csv({data_path}, try_parse_dates=True)
"#,
        data_path = data_path,
        lf = lf_var
    )
}

pub fn python_adaptive_sink_snippet(lf_var: &str, output_path: &Path) -> String {
    format!(
        r#"
# Adaptive row group sizing
col_count = len({lf}.schema)
rgs = 65536
if col_count >= 200: rgs = 16384
elif col_count >= 100: rgs = 32768

env_rgs = os.environ.get('BEEFCAKE_PARQUET_ROW_GROUP_SIZE')
if env_rgs: 
    try: rgs = int(env_rgs)
    except: pass

# Use sink_parquet for memory efficiency
{lf}.sink_parquet(r"{}", row_group_size=rgs)
"#,
        output_path.to_string_lossy(),
        lf = lf_var
    )
}

pub async fn execute_python(
    script: &str,
    data_path: Option<String>,
    log_tag: &str,
) -> Result<String, String> {
    let mut cmd = if cfg!(target_os = "windows") {
        Command::new("python")
    } else {
        Command::new("python3")
    };

    cmd.env("PYTHONIOENCODING", "utf-8");
    cmd.env("POLARS_FMT_MAX_COLS", "-1");
    cmd.env("POLARS_FMT_MAX_ROWS", "100");
    cmd.env("POLARS_FMT_STR_LEN", "1000");

    if let Some(path) = &data_path {
        if !path.is_empty() {
            beefcake::utils::log_event(
                log_tag,
                &format!("Setting BEEFCAKE_DATA_PATH to: {}", path),
            );
            cmd.env("BEEFCAKE_DATA_PATH", path);
        }
    }

    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn python for {log_tag}: {e}"))?;

    let mut stdin = child.stdin.take().ok_or("Failed to open stdin")?;
    stdin
        .write_all(script.as_bytes())
        .map_err(|e| format!("Failed to write to stdin: {e}"))?;
    drop(stdin);

    let out = child
        .wait_with_output()
        .map_err(|e| format!("Failed to wait for python: {e}"))?;

    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();

    if out.status.success() {
        Ok(stdout)
    } else {
        Err(format!("Error: {stdout}\n{stderr}"))
    }
}

pub async fn prepare_data(
    data_path: Option<String>,
    configs: Option<HashMap<String, ColumnCleanConfig>>,
    log_tag: &str,
) -> Result<Option<String>, String> {
    if let (Some(path), Some(cfgs)) = (&data_path, &configs) {
        if !cfgs.is_empty() && !path.is_empty() {
            beefcake::utils::log_event(
                log_tag,
                "Applying cleaning configurations before execution (streaming)",
            );

            let lf = beefcake::analyser::logic::load_df_lazy(&PathBuf::from(path))
                .map_err(|e| format!("Failed to load data for cleaning: {e}"))?;

            let cleaned_lf = beefcake::analyser::logic::clean_df_lazy(lf, cfgs, false)
                .map_err(|e| format!("Failed to apply cleaning: {e}"))?;

            let temp_dir = std::env::temp_dir();
            let temp_path = temp_dir.join(format!(
                "beefcake_cleaned_data_{}_{}.parquet",
                log_tag.to_lowercase(),
                Uuid::new_v4()
            ));

            // Use adaptive sink_parquet for memory efficiency
            let options = beefcake::analyser::logic::get_parquet_write_options(&cleaned_lf)
                .map_err(|e| format!("Failed to determine Parquet options: {e}"))?;

            if let Some(rgs) = options.row_group_size {
                beefcake::utils::log_event(
                    log_tag,
                    &format!("Streaming to Parquet (adaptive). Row group size: {}", rgs),
                );
            }

            cleaned_lf
                .with_streaming(true)
                .sink_parquet(&temp_path, options, None)
                .map_err(|e| format!("Failed to save cleaned data to temp file: {e}"))?;

            return Ok(Some(temp_path.to_string_lossy().to_string()));
        }
    }
    Ok(data_path)
}
