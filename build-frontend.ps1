# build-frontend.ps1
# Helper script to build the frontend and diagnose issues with npm/Node.js

$ErrorActionPreference = "Stop"

Write-Host "--- Beefcake Frontend Build Helper ---" -ForegroundColor Cyan

# 1. Check if npm is in PATH
$npmPath = Get-Command npm -ErrorAction SilentlyContinue
if (-not $npmPath) {
    Write-Host "Error: 'npm' was not found in your system's PATH." -ForegroundColor Red
    Write-Host "`nTroubleshooting steps:" -ForegroundColor Yellow
    Write-Host "1. Ensure Node.js is installed (https://nodejs.org/)"
    Write-Host "2. Restart your terminal (PowerShell/CMD) after installation."
    Write-Host "3. If you just installed it, you might need to restart VS Code or your IDE."
    Write-Host "4. Check if 'C:\Program Files\nodejs\' is in your system environment variables (PATH)."
    
    # Try common locations just in case
    $commonPaths = @(
        "$env:ProgramFiles\nodejs\npm.cmd",
        "${env:ProgramFiles(x86)}\nodejs\npm.cmd",
        "$env:APPDATA\npm\npm.cmd"
    )
    
    $found = $false
    foreach ($p in $commonPaths) {
        if (Test-Path $p) {
            Write-Host "`nFound npm at: $p" -ForegroundColor Green
            Write-Host "You can try running the build using the full path:" -ForegroundColor Gray
            Write-Host "& '$p' run build" -ForegroundColor Gray
            $found = $true
            break
        }
    }
    
    if (-not $found) {
        Write-Host "`nCould not locate npm in common installation folders." -ForegroundColor Red
    }
    
    exit 1
}

Write-Host "Found npm at: $($npmPath.Source)" -ForegroundColor Green

# 2. Run the build
Write-Host "`nRunning 'npm run build'..." -ForegroundColor Cyan
try {
    npm run build
    Write-Host "`nBuild completed successfully!" -ForegroundColor Green
} catch {
    Write-Host "`nBuild failed." -ForegroundColor Red
    Write-Error $_
    exit 1
}
