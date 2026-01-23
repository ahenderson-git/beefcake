use anyhow::{Result, anyhow};
use beefcake::utils;
use std::path::{Path, PathBuf};
use std::process::Command;

const ALLOWED_TEXT_EXTENSIONS: &[&str] =
    &["csv", "json", "md", "parquet", "ps1", "py", "sql", "txt"];

fn resolve_text_path(path: &str, require_file: bool) -> Result<PathBuf> {
    let raw = PathBuf::from(path);
    let absolute = to_absolute(&raw)?;

    let ext = absolute
        .extension()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("File extension required for text operations"))?
        .to_lowercase();

    if !ALLOWED_TEXT_EXTENSIONS.contains(&ext.as_str()) {
        return Err(anyhow!(
            "File extension .{ext} is not allowed for text operations"
        ));
    }

    let candidate = if require_file {
        absolute.clone()
    } else {
        absolute
            .parent()
            .ok_or_else(|| anyhow!("Failed to resolve parent directory"))?
            .to_path_buf()
    };

    if !is_path_allowed(&candidate)? {
        return Err(anyhow!(
            "Access denied: path is outside permitted directories"
        ));
    }

    Ok(absolute)
}

fn to_absolute(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()
            .map_err(|e| anyhow!("Failed to resolve current directory: {e}"))?
            .join(path))
    }
}

fn normalize_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn allowed_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(dir) = dirs::data_local_dir() {
        roots.push(dir.join("beefcake"));
    }
    if let Some(dir) = dirs::config_dir() {
        roots.push(dir.join("beefcake"));
    }

    let config = utils::load_app_config();
    for entry in config.settings.trusted_paths {
        if !entry.trim().is_empty() {
            roots.push(PathBuf::from(entry));
        }
    }

    roots
}

fn is_path_allowed(path: &Path) -> Result<bool> {
    let absolute = normalize_path(&to_absolute(path)?);

    for root in allowed_roots() {
        let root_abs = normalize_path(&root);
        if absolute.starts_with(&root_abs) {
            return Ok(true);
        }
    }

    Ok(false)
}

pub fn open_path(path: &str) -> Result<()> {
    let absolute = to_absolute(Path::new(path))?;
    if !absolute.exists() {
        return Err(anyhow!("Path not found: {}", absolute.display()));
    }
    if !is_path_allowed(&absolute)? {
        return Err(anyhow!(
            "Access denied: path is outside permitted directories"
        ));
    }

    let status = if cfg!(target_os = "windows") {
        Command::new("explorer").arg(&absolute).status()
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(&absolute).status()
    } else {
        Command::new("xdg-open").arg(&absolute).status()
    };

    match status {
        Ok(exit) if exit.success() => Ok(()),
        Ok(exit) => Err(anyhow!("Failed to open path: exit {exit}")),
        Err(e) => Err(anyhow!("Failed to open path: {e}")),
    }
}

pub fn run_powershell(script: &str) -> Result<String> {
    let output = if cfg!(target_os = "windows") {
        Command::new("powershell")
            .arg("-NoProfile")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-Command")
            .arg(script)
            .output()
    } else {
        Command::new("pwsh")
            .arg("-NoProfile")
            .arg("-Command")
            .arg(script)
            .output()
    };

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            if out.status.success() {
                Ok(stdout)
            } else {
                Err(anyhow!("Error: {stdout}\n{stderr}"))
            }
        }
        Err(e) => Err(anyhow!("Failed to execute powershell: {e}")),
    }
}

pub fn install_python_package(package: &str) -> Result<String> {
    let mut cmd = if cfg!(target_os = "windows") {
        Command::new("python")
    } else {
        Command::new("python3")
    };

    let output = cmd
        .arg("-m")
        .arg("pip")
        .arg("install")
        .arg(package)
        .env("PYTHONIOENCODING", "utf-8")
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            if out.status.success() {
                Ok(format!("Successfully installed {package}\n{stdout}"))
            } else {
                Err(anyhow!("Failed to install {package}: {stdout}\n{stderr}"))
            }
        }
        Err(e) => Err(anyhow!("Failed to execute pip: {e}")),
    }
}

pub fn read_text_file(path: &str) -> Result<String> {
    let absolute = resolve_text_path(path, true)?;
    if !absolute.is_file() {
        return Err(anyhow!("File not found: {}", absolute.display()));
    }
    std::fs::read_to_string(&absolute)
        .map_err(|e| anyhow!("Failed to read file {}: {e}", absolute.display()))
}

pub fn write_text_file(path: &str, contents: &str) -> Result<()> {
    let absolute = resolve_text_path(path, false)?;
    if let Some(parent) = absolute.parent()
        && !parent.exists()
    {
        return Err(anyhow!(
            "Parent directory does not exist: {}",
            parent.display()
        ));
    }
    std::fs::write(&absolute, contents)
        .map_err(|e| anyhow!("Failed to write file {}: {e}", absolute.display()))
}

pub fn check_python_environment() -> Result<String> {
    let mut cmd = if cfg!(target_os = "windows") {
        Command::new("python")
    } else {
        Command::new("python3")
    };

    // Check Python version
    let version_output = cmd
        .arg("--version")
        .env("PYTHONIOENCODING", "utf-8")
        .output();

    let version_info = match version_output {
        Ok(out) => {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                // Python 2 writes to stderr, Python 3 to stdout
                if !stdout.is_empty() { stdout } else { stderr }
            } else {
                return Err(anyhow!("Python not found or not working properly"));
            }
        }
        Err(_) => {
            return Err(anyhow!(
                "Python executable not found. Please install Python 3.8+"
            ));
        }
    };

    // Check if Polars is installed
    let mut polars_cmd = if cfg!(target_os = "windows") {
        Command::new("python")
    } else {
        Command::new("python3")
    };

    let polars_check = polars_cmd
        .arg("-c")
        .arg("import polars; print(f'Polars {polars.__version__} installed')")
        .env("PYTHONIOENCODING", "utf-8")
        .output();

    let polars_info = match polars_check {
        Ok(out) => {
            if out.status.success() {
                String::from_utf8_lossy(&out.stdout).to_string()
            } else {
                "❌ Polars not installed. Run: pip install polars".to_owned()
            }
        }
        Err(_) => "❌ Unable to check for Polars".to_owned(),
    };

    Ok(format!(
        "✅ {}\n{}\n\n💡 Beefcake requires Python 3.8+ with Polars installed.",
        version_info.trim(),
        polars_info.trim()
    ))
}
