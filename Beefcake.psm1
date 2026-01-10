# ==============================================================================
# BEEFCAKE CLI WRAPPERS
# ==============================================================================
# These functions wrap the 'beefcake.exe' CLI for high-performance data operations.

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
