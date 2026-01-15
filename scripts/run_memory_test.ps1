# Complete memory test workflow
# Generates test data, runs memory profiler, and launches app

param(
    [int]$Rows = 5000000,
    [int]$Cols = 100,
    [string]$TestFile = "test_5M_100cols.csv"
)

Write-Host "=== Beefcake Memory Test Suite ===" -ForegroundColor Cyan
Write-Host ""

# Step 1: Check if test file exists
if (Test-Path $TestFile) {
    Write-Host "Test file already exists: $TestFile" -ForegroundColor Green
    $fileSize = (Get-Item $TestFile).Length / 1MB
    Write-Host "File size: $([math]::Round($fileSize, 2)) MB" -ForegroundColor Yellow
    Write-Host ""

    $response = Read-Host "Regenerate test data? (y/N)"
    if ($response -ne 'y' -and $response -ne 'Y') {
        Write-Host "Using existing test file." -ForegroundColor Green
        Write-Host ""
        $generate = $false
    } else {
        $generate = $true
    }
} else {
    $generate = $true
}

# Step 2: Generate test data if needed
if ($generate) {
    Write-Host "Generating test data: $Rows rows × $Cols columns..." -ForegroundColor Yellow
    Write-Host ""

    python scripts\generate_test_data.py $Rows $Cols $TestFile

    if ($LASTEXITCODE -ne 0) {
        Write-Host ""
        Write-Host "ERROR: Failed to generate test data." -ForegroundColor Red
        Write-Host "Make sure Python is installed and in PATH." -ForegroundColor Red
        exit 1
    }

    Write-Host ""
}

# Step 3: Instructions
Write-Host "=== Test Instructions ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "1. In another terminal, run your Beefcake app:" -ForegroundColor Yellow
Write-Host "   cargo run --release" -ForegroundColor White
Write-Host ""
Write-Host "2. In the app, load the test file:" -ForegroundColor Yellow
Write-Host "   $TestFile" -ForegroundColor White
Write-Host ""
Write-Host "3. This script will monitor memory usage automatically." -ForegroundColor Yellow
Write-Host ""
Write-Host "Press Enter when you're ready to start monitoring..." -ForegroundColor Green
Read-Host

# Step 4: Start memory monitoring
Write-Host ""
Write-Host "Starting memory monitor..." -ForegroundColor Cyan
Write-Host ""

.\scripts\measure_memory.ps1 -ProcessName "beefcake" -TestFile $TestFile -IntervalSeconds 1
