# ==============================================================================
# BEEFCAKE CLI WRAPPERS
# ==============================================================================
# These functions wrap the 'beefcake.exe' CLI for high-performance data operations.

function make {
    <#
    .SYNOPSIS
    A 'make' alias for Windows users to bridge the gap with the Makefile.
    #>
    param(
        [Parameter(Mandatory=$false, Position=0)]
        [string]$Target = "help"
    )
    
    $ShimPath = Join-Path $PSScriptRoot "make.ps1"
    if (Test-Path $ShimPath) {
        # Import-Module Beefcake already happened to call this function, 
        # but make.ps1 also tries to import it. To avoid recursion or double warnings,
        # we can just call the shim.
        & $ShimPath $Target
    } else {
        Write-Warning "make.ps1 not found in $PSScriptRoot"
    }
}

function Invoke-Beefcake {
    <#
    .SYNOPSIS
        Core execution wrapper for the Beefcake binary.
    #>
    [CmdletBinding()]
    param(
        [Parameter(ValueFromRemainingArguments = $true)]
        [string[]]$Arguments
    )
    
    # Try to find the binary in PATH, then local directory, then PSScriptRoot
    $Binary = "beefcake.exe"
    if (-not (Get-Command $Binary -ErrorAction SilentlyContinue)) {
        $Binary = Join-Path $PSScriptRoot "beefcake.exe"
        if (-not (Test-Path $Binary)) {
            $Binary = Join-Path (Get-Location) "beefcake.exe"
        }
    }

    if (-not (Test-Path $Binary) -and -not (Get-Command $Binary -ErrorAction SilentlyContinue)) {
        Write-Error "beefcake.exe not found. Please ensure it is in your PATH or the module directory."
        return
    }

    & $Binary @Arguments
}

function Import-BeefcakeData {
    <#
    .SYNOPSIS
        Imports a file (CSV, Parquet, JSON) directly into a PostgreSQL table.
    .EXAMPLE
        Import-BeefcakeData -Path "./logs.csv" -Table "system_logs" -Schema "analytics"
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true, HelpMessage = "Path to the source data file")]
        [string]$Path,

        [Parameter(Mandatory = $true, HelpMessage = "Target table name in Postgres")]
        [string]$Table,

        [string]$Schema = "public",

        [string]$DatabaseUrl = $env:DATABASE_URL,

        [switch]$Clean
    )

    $Args = @("import", "--file", $Path, "--table", $Table, "--schema", $Schema)
    
    if ($DatabaseUrl) {
        $Args += "--db-url", $DatabaseUrl
    }

    if ($Clean) {
        $Args += "--clean"
    }

    Invoke-Beefcake -Arguments $Args
}

function Convert-BeefcakeData {
    <#
    .SYNOPSIS
        Uses Beefcake's Polars engine to convert between data formats at high speed.
    .EXAMPLE
        Convert-BeefcakeData -InputPath "raw_data.csv" -OutputPath "optimized_data.parquet"
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string]$InputPath,

        [Parameter(Mandatory = $true)]
        [string]$OutputPath,

        [switch]$Clean
    )

    $Args = @("export", "--input", $InputPath, "--output", $OutputPath)
    if ($Clean) {
        $Args += "--clean"
    }

    Invoke-Beefcake -Arguments $Args
}

function Clean-BeefcakeData {
    <#
    .SYNOPSIS
        Uses Beefcake's heuristic engine to automatically clean a data file.
    .EXAMPLE
        Clean-BeefcakeData -Path "dirty.csv" -OutputPath "clean.parquet"
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path,

        [Parameter(Mandatory = $true)]
        [string]$OutputPath
    )

    Invoke-Beefcake -Arguments @("clean", "--file", $Path, "--output", $OutputPath)
}

# ==============================================================================
# DEVELOPMENT COMMANDS
# ==============================================================================

function Invoke-BeefcakeQuality {
    <#
    .SYNOPSIS
        Runs all quality checks (lint, format, type-check).
    #>
    [CmdletBinding()]
    param()
    
    $ScriptPath = Join-Path $PSScriptRoot "scripts\quality.ps1"
    if (Test-Path $ScriptPath) {
        & $ScriptPath
    } else {
        Write-Host "Running all quality checks via npm..." -ForegroundColor Cyan
        npm run quality
    }
}

function Invoke-BeefcakeTest {
    <#
    .SYNOPSIS
        Runs all tests (Rust + TypeScript).
    #>
    [CmdletBinding()]
    param(
        [switch]$RustOnly,
        [switch]$TsOnly
    )
    
    if (-not $TsOnly) {
        Write-Host "Cleaning problematic PDB files..." -ForegroundColor Cyan
        npm run clean-pdbs
        Write-Host "Running Rust tests..." -ForegroundColor Cyan
        cargo test
    }
    
    if (-not $RustOnly) {
        Write-Host "Running TypeScript tests..." -ForegroundColor Cyan
        npm test
    }
}

function Invoke-BeefcakeDocs {
    <#
    .SYNOPSIS
        Generates documentation.
    #>
    [CmdletBinding()]
    param(
        [switch]$Open
    )
    
    Write-Host "Generating documentation..." -ForegroundColor Cyan
    npm run docs
    
    if ($Open) {
        Start-Process "target\doc\beefcake\index.html"
    }
}

function Invoke-BeefcakeDev {
    <#
    .SYNOPSIS
        Starts the development environment.
    #>
    [CmdletBinding()]
    param()
    
    npm run tauri dev
}

Export-ModuleMember -Function *-Beefcake*, make
