# Beefcake Quality Check Script for Windows
Write-Host "Running all quality checks..." -ForegroundColor Cyan

Write-Host "1/4: Running ESLint..." -ForegroundColor Yellow
npm run lint
if ($LASTEXITCODE -ne 0) { Write-Error "ESLint failed"; exit $LASTEXITCODE }

Write-Host "2/4: Checking formatting..." -ForegroundColor Yellow
npm run format:check
if ($LASTEXITCODE -ne 0) { Write-Error "Formatting check failed"; exit $LASTEXITCODE }

Write-Host "3/4: Type checking..." -ForegroundColor Yellow
npm run type-check
if ($LASTEXITCODE -ne 0) { Write-Error "Type checking failed"; exit $LASTEXITCODE }

Write-Host "4/4: Type coverage..." -ForegroundColor Yellow
npm run type-coverage
if ($LASTEXITCODE -ne 0) { Write-Error "Type coverage failed"; exit $LASTEXITCODE }

Write-Host "Quality checks passed!" -ForegroundColor Green
