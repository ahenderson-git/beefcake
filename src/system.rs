use anyhow::{Result, anyhow};
use std::process::Command;

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
    std::fs::read_to_string(path).map_err(|e| anyhow!("Failed to read file {path}: {e}"))
}

pub fn write_text_file(path: &str, contents: &str) -> Result<()> {
    std::fs::write(path, contents).map_err(|e| anyhow!("Failed to write file {path}: {e}"))
}
