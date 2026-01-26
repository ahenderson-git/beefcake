use beefcake::analyser::logic::ColumnCleanConfig;
use beefcake::error::{BeefcakeError, Result, ResultExt as _};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncWriteExt as _;
use tokio::process::Command;
use tokio::time::timeout;
use uuid::Uuid;

const DEFAULT_PYTHON_TIMEOUT_SECS: u64 = 300;

fn python_timeout() -> Duration {
    let timeout = std::env::var("BEEFCAKE_PYTHON_TIMEOUT_SECS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_PYTHON_TIMEOUT_SECS);
    Duration::from_secs(timeout)
}

pub fn python_preamble() -> String {
    r#"import os
import polars as pl
import sys

# Configure Polars display settings
pl.Config.set_tbl_cols(-1)  # Show all columns
pl.Config.set_tbl_rows(100)  # Show up to 100 rows
pl.Config.set_tbl_width_chars(10000)  # Allow wide tables
"#
    .to_owned()
}

pub fn python_load_snippet(data_path: &str, lf_var: &str) -> String {
    format!(
        r#"
if {data_path}.endswith(".parquet"):
    {lf_var} = pl.scan_parquet({data_path})
elif {data_path}.endswith(".json"):
    {lf_var} = pl.read_json({data_path}).lazy()
else:
    {lf_var} = pl.scan_csv({data_path}, try_parse_dates=True)
"#
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
) -> Result<String> {
    execute_python_with_env(script, data_path, None, log_tag).await
}

pub async fn execute_python_with_env(
    script: &str,
    data_path: Option<String>,
    sql_query: Option<String>,
    log_tag: &str,
) -> Result<String> {
    let mut cmd = if cfg!(target_os = "windows") {
        Command::new("python")
    } else {
        Command::new("python3")
    };

    cmd.env("PYTHONIOENCODING", "utf-8");
    cmd.env("POLARS_FMT_MAX_COLS", "-1");
    cmd.env("POLARS_FMT_MAX_ROWS", "100");
    cmd.env("POLARS_FMT_STR_LEN", "1000");

    // Force Rich to output ANSI codes even when not in a real terminal
    cmd.env("FORCE_COLOR", "1");
    cmd.env("TERM", "xterm-256color");
    // Disable Windows legacy console mode in Rich
    cmd.env("COLORTERM", "truecolor");
    cmd.env("NO_COLOR", ""); // Set but empty to allow FORCE_COLOR to work

    if let Some(path) = &data_path
        && !path.is_empty()
    {
        beefcake::config::log_event(log_tag, &format!("Setting BEEFCAKE_DATA_PATH to: {path}"));
        cmd.env("BEEFCAKE_DATA_PATH", path);
    }

    if let Some(query) = &sql_query
        && !query.is_empty()
    {
        beefcake::config::log_event(log_tag, "Setting BEEFCAKE_SQL_QUERY environment variable");
        cmd.env("BEEFCAKE_SQL_QUERY", query);
    }

    beefcake::config::log_event(log_tag, "Spawning Python process...");

    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .with_context(|| format!("Failed to spawn python for {log_tag}"))?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| BeefcakeError::Python("Failed to open stdin".to_owned()))?;
    stdin
        .write_all(script.as_bytes())
        .await
        .context("Failed to write script to stdin")?;
    drop(stdin);

    beefcake::config::log_event(log_tag, "Waiting for Python to complete...");

    let timeout_duration = python_timeout();
    let out = match timeout(timeout_duration, child.wait_with_output()).await {
        Ok(result) => result.context("Failed to wait for python process")?,
        Err(_) => {
            return Err(BeefcakeError::Python(format!(
                "Python execution timed out after {} seconds",
                timeout_duration.as_secs()
            )));
        }
    };

    beefcake::config::log_event(
        log_tag,
        &format!(
            "Python process completed with exit code: {:?}",
            out.status.code()
        ),
    );

    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();

    if out.status.success() {
        Ok(stdout)
    } else {
        let mut error_msg = format!("Error: {stdout}\n{stderr}");
        if error_msg.contains("the name 'literal' passed to `LazyFrame.with_columns` is duplicate")
        {
            error_msg.push_str("\n\nTip: When selecting multiple constant values in SQL, you must give them unique names using 'AS'.\nExample: SELECT 1 AS col1, 2 AS col2 FROM data");

            if let Some(query) = &sql_query
                && let Some(fixed) = suggest_sql_fix(query)
            {
                error_msg.push_str(&format!(
                    "\n\nFixed query suggestion:\n```sql\n{fixed}\n```"
                ));
            }
        } else if error_msg.contains("duplicate column names")
            || error_msg.contains("duplicate output name")
        {
            error_msg.push_str("\n\nTip: Ensure all selected columns in your SQL query have unique names using 'AS' where necessary.");
        }
        Err(BeefcakeError::Python(error_msg))
    }
}

/// Prepares data for Python execution, optionally applying cleaning configurations.
/// Returns a tuple of (`path_to_use`, `optional_temp_guard`).
/// If cleaning was applied, a temp file guard is returned to ensure cleanup.
pub async fn prepare_data(
    data_path: Option<String>,
    configs: Option<HashMap<String, ColumnCleanConfig>>,
    log_tag: &str,
) -> Result<(Option<String>, Option<beefcake::utils::TempFileGuard>)> {
    // Early return if no data path or no configs
    let Some(path) = &data_path else {
        return Ok((data_path, None));
    };

    let Some(cfgs) = &configs else {
        return Ok((data_path, None));
    };

    if path.is_empty() || cfgs.is_empty() {
        return Ok((data_path, None));
    }

    // Optimisation: Check if any configs are actually active before proceeding
    let has_active_configs = cfgs.values().any(|config| config.active);
    if !has_active_configs {
        beefcake::config::log_event(
            log_tag,
            "No active cleaning configs, skipping data preparation",
        );
        return Ok((data_path, None));
    }

    beefcake::config::log_event(
        log_tag,
        "Applying cleaning configurations before execution (streaming)",
    );

    let lf = beefcake::analyser::logic::load_df_lazy(&PathBuf::from(path))
        .context("Failed to load data for cleaning")?;

    let cleaned_lf = beefcake::analyser::logic::clean_df_lazy(lf, cfgs, false)
        .context("Failed to apply cleaning")?;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!(
        "beefcake_cleaned_data_{}_{}.parquet",
        log_tag.to_lowercase(),
        Uuid::new_v4()
    ));

    // Use adaptive sink_parquet for memory efficiency
    let options = beefcake::analyser::logic::get_parquet_write_options(&cleaned_lf)
        .context("Failed to determine Parquet options")?;

    if let Some(rgs) = options.row_group_size {
        beefcake::config::log_event(
            log_tag,
            &format!("Streaming to Parquet (adaptive). Row group size: {rgs}"),
        );
    }

    cleaned_lf
        .with_streaming(true)
        .sink_parquet(&temp_path, options, None)
        .context("Failed to save cleaned data to temp file")?;

    let path_str = temp_path.to_string_lossy().to_string();
    let guard = beefcake::utils::TempFileGuard::new(temp_path);
    Ok((Some(path_str), Some(guard)))
}

fn suggest_sql_fix(query: &str) -> Option<String> {
    use regex::Regex;

    // Find the SELECT list.
    // We look for SELECT followed by anything until FROM (non-greedy).
    // Use (?s) to allow . to match newlines
    let select_re = Regex::new(r"(?is)SELECT\s+(?P<list>.*?)\s+FROM").ok()?;
    let caps = select_re.captures(query)?;
    let list_str = caps.name("list")?.as_str();

    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    let mut in_quote = false;

    for c in list_str.chars() {
        match c {
            '\'' => in_quote = !in_quote,
            '(' if !in_quote => depth += 1,
            ')' if !in_quote => depth -= 1,
            ',' if !in_quote && depth == 0 => {
                parts.push(current.trim().to_owned());
                current = String::new();
                continue;
            }
            _ => {}
        }
        current.push(c);
    }
    parts.push(current.trim().to_owned());

    let mut fixed_parts = Vec::new();
    let mut changed = false;
    let mut literal_idx = 0;
    let mut used_aliases = std::collections::HashSet::new();

    let as_re = Regex::new(r"(?i)\s+AS\s+").ok()?;

    for part in parts {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            fixed_parts.push(part);
            continue;
        }

        // Check if it's a literal (string or number)
        let is_string_literal = trimmed.starts_with('\'') && trimmed.ends_with('\'');
        let is_numeric_literal = !trimmed.is_empty()
            && trimmed
                .chars()
                .all(|c| c.is_ascii_digit() || c == '.' || c == '-');

        let is_literal = is_string_literal || is_numeric_literal;

        // Check if it already has an alias
        let has_alias = as_re.is_match(trimmed) || (!is_literal && trimmed.contains(' '));

        if is_literal && !has_alias {
            changed = true;
            literal_idx += 1;

            let mut alias = if is_string_literal {
                trimmed
                    .trim_matches('\'')
                    .chars()
                    .filter(|c| c.is_alphanumeric() || *c == '_')
                    .collect::<String>()
            } else {
                String::new()
            };

            if alias.is_empty() || alias.len() > 30 {
                alias = format!("col_{literal_idx}");
            }

            // Ensure uniqueness
            let mut final_alias = alias.clone();
            let mut counter = 1;
            while used_aliases.contains(&final_alias) {
                final_alias = format!("{alias}_{counter}");
                counter += 1;
            }
            used_aliases.insert(final_alias.clone());

            fixed_parts.push(format!("{trimmed} AS \"{final_alias}\""));
        } else {
            // Track existing aliases to avoid conflicts with new ones
            if has_alias && let Some(caps) = as_re.captures(trimmed) {
                let alias_part = &trimmed[caps.get(0).map_or(trimmed.len(), |m| m.end())..].trim();
                used_aliases.insert(alias_part.trim_matches('"').to_owned());
            }
            fixed_parts.push(part.clone());
        }
    }

    if changed {
        let new_list = fixed_parts.join(", ");
        // Replace only the select list part
        let start = query.find(list_str)?;
        let end = start + list_str.len();
        let mut fixed_query = query.to_owned();
        fixed_query.replace_range(start..end, &new_list);
        Some(fixed_query)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggest_sql_fix() {
        let q1 = "SELECT 'a', 'b' FROM data";
        let f1 = suggest_sql_fix(q1).unwrap();
        assert!(f1.contains("'a' AS \"a\""));
        assert!(f1.contains("'b' AS \"b\""));

        let q2 = "SELECT 1, 2 FROM data";
        let f2 = suggest_sql_fix(q2).unwrap();
        assert!(f2.contains("1 AS \"col_1\""));
        assert!(f2.contains("2 AS \"col_2\""));

        let q3 = "SELECT 'Order ID $', 'CUSTOMER Id' FROM data LIMIT 10";
        let f3 = suggest_sql_fix(q3).unwrap();
        assert!(f3.contains("'Order ID $' AS \"OrderID\""));
        assert!(f3.contains("'CUSTOMER Id' AS \"CUSTOMERId\""));

        let q4 = "SELECT a, b FROM data";
        assert!(suggest_sql_fix(q4).is_none());

        let q5 = "SELECT 'a' AS alpha, 'b' FROM data";
        let f5 = suggest_sql_fix(q5).unwrap();
        assert!(f5.contains("'a' AS alpha"));
        assert!(f5.contains("'b' AS \"b\""));

        let q6 = "SELECT 'ID $', 'ID #' FROM data";
        let f6 = suggest_sql_fix(q6).unwrap();
        assert!(f6.contains("'ID $' AS \"ID\""));
        assert!(f6.contains("'ID #' AS \"ID_1\""));
    }
}
