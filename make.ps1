param([string]$Target = "help")
$ModulePath = Join-Path $PSScriptRoot "Beefcake.psm1"
# Only import if not already loaded or if called directly
if (Test-Path $ModulePath) { 
    if (-not (Get-Command Invoke-BeefcakeQuality -ErrorAction SilentlyContinue)) {
        Import-Module $ModulePath -DisableNameChecking
    }
}
switch ($Target) {
    "quality" { Invoke-BeefcakeQuality }
    "test" { Invoke-BeefcakeTest }
    "dev" { Invoke-BeefcakeDev }
    "docs" { Invoke-BeefcakeDocs }
    "build" { npm run build; npm run tauri build }
    "clean" {
        Write-Host "Cleaning build artifacts..." -ForegroundColor Cyan
        cargo clean
        if (Test-Path "dist") { Remove-Item -Recurse -Force "dist" }
        if (Test-Path "docs/typescript") { Remove-Item -Recurse -Force "docs/typescript" }
        if (Test-Path "target/doc") { Remove-Item -Recurse -Force "target/doc" }
        Write-Host "Done!" -ForegroundColor Green
    }
    "help" {
        Write-Host "Beefcake Windows Shim" -ForegroundColor Green
        Write-Host "Usage: .\make.ps1 target"
        Write-Host "Targets: quality, test, dev, docs, build, clean"
    }
    Default { & $PSCommandPath help }
}
