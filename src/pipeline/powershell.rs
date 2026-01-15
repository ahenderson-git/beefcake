//! `PowerShell` script generation for pipeline automation.
//!
//! Generates `PowerShell` wrapper scripts that invoke the Beefcake CLI with
//! proper error handling, logging, and parameter support.

use super::spec::PipelineSpec;
use std::path::Path;

/// Generate a `PowerShell` script for running a pipeline
pub fn generate_powershell_script(spec: &PipelineSpec, spec_path: &Path) -> String {
    let spec_path_str = spec_path.display().to_string();
    let pipeline_name = &spec.name;

    format!(
        r#"<#
.SYNOPSIS
    Automated data processing pipeline: {pipeline_name}

.DESCRIPTION
    This script runs the Beefcake pipeline spec to process data files.
    Generated automatically from pipeline specification.

.PARAMETER InputPath
    Path to the input data file to process

.PARAMETER OutputPath
    Path where the processed output file will be saved

.PARAMETER SpecPath
    Path to the pipeline spec JSON file (default: {spec_path_str})

.PARAMETER Date
    Date string for path template substitution (default: today's date in YYYY-MM-DD format)

.PARAMETER LogPath
    Path to write execution log file (optional)

.PARAMETER FailOnWarnings
    Exit with error if warnings are generated during processing

.EXAMPLE
    .\run.ps1 -InputPath "C:\data\input.csv" -OutputPath "C:\data\output.parquet"

.EXAMPLE
    .\run.ps1 -InputPath ".\data\today.csv" -OutputPath ".\data\processed\cleaned_today.csv" -FailOnWarnings

.NOTES
    Pipeline: {pipeline_name}
    Spec Version: {spec_version}
    Generated: {timestamp}
#>

param(
    [Parameter(Mandatory=$true)]
    [string]$InputPath,

    [Parameter(Mandatory=$false)]
    [string]$OutputPath,

    [Parameter(Mandatory=$false)]
    [string]$SpecPath = "{spec_path_str}",

    [Parameter(Mandatory=$false)]
    [string]$Date,

    [Parameter(Mandatory=$false)]
    [string]$LogPath,

    [Parameter(Mandatory=$false)]
    [switch]$FailOnWarnings
)

# Stop on any error
$ErrorActionPreference = "Stop"

# Color output functions
function Write-Info {{
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Cyan
}}

function Write-Success {{
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
}}

function Write-Error {{
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}}

function Write-Warning {{
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor Yellow
}}

# Validate input file exists
if (-not (Test-Path $InputPath)) {{
    Write-Error "Input file not found: $InputPath"
    exit 1
}}

# Validate spec file exists
if (-not (Test-Path $SpecPath)) {{
    Write-Error "Pipeline spec not found: $SpecPath"
    exit 1
}}

# Setup logging
if (-not $LogPath) {{
    $timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
    $LogPath = ".\logs\beefcake_$timestamp.log"
}}

# Ensure log directory exists
$logDir = Split-Path -Parent $LogPath
if ($logDir -and -not (Test-Path $logDir)) {{
    New-Item -ItemType Directory -Path $logDir -Force | Out-Null
}}

Write-Info "Starting pipeline: {pipeline_name}"
Write-Info "Spec: $SpecPath"
Write-Info "Input: $InputPath"
if ($OutputPath) {{
    Write-Info "Output: $OutputPath"
}} else {{
    Write-Info "Output: (from spec path_template)"
}}
Write-Info "Log: $LogPath"

# Build beefcake command
$beefcakeArgs = @(
    "run",
    "--spec", $SpecPath,
    "--input", $InputPath
)

if ($OutputPath) {{
    $beefcakeArgs += "--output", $OutputPath
}}

if ($Date) {{
    $beefcakeArgs += "--date", $Date
}}

if ($LogPath) {{
    $beefcakeArgs += "--log", $LogPath
}}

if ($FailOnWarnings) {{
    $beefcakeArgs += "--fail-on-warnings"
}}

# Execute pipeline
try {{
    Write-Info "Executing: beefcake $($beefcakeArgs -join ' ')"

    $startTime = Get-Date

    # Run beefcake and capture output
    $output = & beefcake @beefcakeArgs 2>&1
    $exitCode = $LASTEXITCODE

    $endTime = Get-Date
    $duration = $endTime - $startTime

    # Display output
    $output | ForEach-Object {{ Write-Host $_ }}

    # Check exit code
    if ($exitCode -eq 0) {{
        Write-Success "Pipeline completed successfully in $($duration.TotalSeconds) seconds"
        exit 0
    }} elseif ($exitCode -eq 2) {{
        Write-Error "Pipeline validation failed (exit code 2)"
        Write-Error "Check the pipeline spec and input file schema"
        exit 2
    }} elseif ($exitCode -eq 3) {{
        Write-Error "Pipeline execution failed (exit code 3)"
        Write-Error "Check the log file for details: $LogPath"
        exit 3
    }} else {{
        Write-Error "Pipeline failed with exit code $exitCode"
        exit $exitCode
    }}
}} catch {{
    Write-Error "Failed to execute pipeline: $_"
    exit 1
}}
"#,
        pipeline_name = pipeline_name,
        spec_version = spec.version,
        spec_path_str = spec_path_str.replace('\\', "\\\\"),
        timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
    )
}

/// Generate a `PowerShell` script for scheduling with Task Scheduler
pub fn generate_scheduled_script(
    spec: &PipelineSpec,
    spec_path: &Path,
    input_dir: &str,
    output_dir: &str,
) -> String {
    let base_script = generate_powershell_script(spec, spec_path);

    format!(
        r#"{}

<#
SCHEDULING WITH WINDOWS TASK SCHEDULER:

1. Open Task Scheduler (taskschd.msc)

2. Create New Task:
   - Name: Beefcake - {}
   - Description: Automated data processing pipeline
   - Run whether user is logged on or not: Yes
   - Run with highest privileges: No

3. Triggers:
   - New Trigger
   - Daily at desired time (e.g., 6:00 AM)
   - Repeat task every: (optional, e.g., 1 hour)

4. Actions:
   - New Action
   - Action: Start a program
   - Program/script: powershell.exe
   - Arguments: -ExecutionPolicy Bypass -File "PATH\TO\THIS\run.ps1" -InputPath "{}\input.csv" -OutputPath "{}\output.parquet"

5. Conditions:
   - Start only if computer is on AC power: No
   - Wake computer to run: No

6. Settings:
   - Allow task to be run on demand: Yes
   - Stop task if it runs longer than: 3 hours
   - If task fails, restart every: 10 minutes
   - Attempt to restart up to: 3 times
#>
"#,
        base_script, spec.name, input_dir, output_dir
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::spec::PipelineSpec;
    use std::path::PathBuf;

    #[test]
    fn test_generate_powershell_script() {
        let spec = PipelineSpec::new("test_pipeline");
        let spec_path = PathBuf::from("C:\\pipelines\\test.json");

        let script = generate_powershell_script(&spec, &spec_path);

        assert!(script.contains("test_pipeline"));
        assert!(script.contains("$InputPath"));
        assert!(script.contains("$OutputPath"));
        assert!(script.contains("beefcake"));
        assert!(script.contains("\"run\""));
        assert!(script.contains("--spec"));
        assert!(script.contains("exit"));
    }

    #[test]
    fn test_generate_scheduled_script() {
        let spec = PipelineSpec::new("daily_import");
        let spec_path = PathBuf::from("C:\\pipelines\\daily.json");

        let script =
            generate_scheduled_script(&spec, &spec_path, "C:\\data\\input", "C:\\data\\output");

        assert!(script.contains("TASK SCHEDULER"));
        assert!(script.contains("daily_import"));
        assert!(script.contains("C:\\data\\input"));
    }
}
