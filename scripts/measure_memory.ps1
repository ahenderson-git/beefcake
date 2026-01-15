# Memory measurement script for Beefcake analysis
# Usage: .\scripts\measure_memory.ps1 -ProcessName "beefcake" -TestFile "path\to\test.csv"

param(
    [string]$ProcessName = "beefcake",
    [string]$TestFile = "",
    [int]$IntervalSeconds = 1
)

Write-Host "=== Beefcake Memory Profiler ===" -ForegroundColor Cyan
Write-Host ""

if ($TestFile) {
    Write-Host "Test file: $TestFile" -ForegroundColor Yellow
    $fileSize = (Get-Item $TestFile).Length / 1MB
    Write-Host "File size: $([math]::Round($fileSize, 2)) MB" -ForegroundColor Yellow
    Write-Host ""
}

Write-Host "Waiting for process '$ProcessName' to start..." -ForegroundColor Yellow
Write-Host "Press Ctrl+C to stop monitoring" -ForegroundColor Gray
Write-Host ""

# Wait for process to start
while (-not (Get-Process -Name $ProcessName -ErrorAction SilentlyContinue)) {
    Start-Sleep -Milliseconds 500
}

Write-Host "Process detected! Starting memory monitoring..." -ForegroundColor Green
Write-Host ""
Write-Host "Time (s) | Working Set (MB) | Peak Working Set (MB) | Private Bytes (MB)" -ForegroundColor Cyan
Write-Host "---------|------------------|----------------------|--------------------" -ForegroundColor Cyan

$startTime = Get-Date
$peakMemory = 0

try {
    while ($true) {
        $process = Get-Process -Name $ProcessName -ErrorAction SilentlyContinue

        if (-not $process) {
            Write-Host ""
            Write-Host "Process terminated." -ForegroundColor Red
            break
        }

        $elapsed = [math]::Round(((Get-Date) - $startTime).TotalSeconds, 1)
        $workingSet = [math]::Round($process.WorkingSet64 / 1MB, 2)
        $peakWorkingSet = [math]::Round($process.PeakWorkingSet64 / 1MB, 2)
        $privateBytes = [math]::Round($process.PrivateMemorySize64 / 1MB, 2)

        if ($workingSet -gt $peakMemory) {
            $peakMemory = $workingSet
        }

        Write-Host ("{0,8} | {1,16} | {2,20} | {3,18}" -f $elapsed, $workingSet, $peakWorkingSet, $privateBytes)

        Start-Sleep -Seconds $IntervalSeconds
    }
}
finally {
    Write-Host ""
    Write-Host "=== Summary ===" -ForegroundColor Cyan
    Write-Host "Peak Memory Usage: $peakMemory MB" -ForegroundColor Yellow

    if ($TestFile) {
        $ratio = [math]::Round($peakMemory / $fileSize, 2)
        Write-Host "Memory to File Size Ratio: ${ratio}x" -ForegroundColor Yellow
    }
}
