# Beefcake Documentation Generation Script
#
# This script generates both Rust and TypeScript documentation.
# Run: .\scripts\docs.ps1

param(
    [Parameter(Position=0)]
    [ValidateSet('all', 'rust', 'ts', 'open')]
    [string]$Target = 'all'
)

function Write-Header {
    param([string]$Message)
    Write-Host "`n===================================" -ForegroundColor Cyan
    Write-Host $Message -ForegroundColor Cyan
    Write-Host "===================================`n" -ForegroundColor Cyan
}

function Write-Success {
    param([string]$Message)
    Write-Host "✓ $Message" -ForegroundColor Green
}

function Write-Error-Message {
    param([string]$Message)
    Write-Host "✗ $Message" -ForegroundColor Red
}

function Generate-RustDocs {
    Write-Header "Generating Rust Documentation"

    try {
        cargo doc --document-private-items --no-deps 2>&1 | Out-String | Write-Host
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Rust documentation generated successfully!"
            if (Test-Path "target\doc\beefcake\index.html") {
                $docPath = Resolve-Path "target\doc\beefcake\index.html"
                Write-Host "Location: $docPath" -ForegroundColor Gray
            }
        } else {
            Write-Error-Message "Failed to generate Rust documentation"
            exit 1
        }
    } catch {
        Write-Error-Message "Error generating Rust docs: $_"
        exit 1
    }
}

function Generate-TypeScriptDocs {
    Write-Header "Generating TypeScript Documentation"

    try {
        npm run docs:ts
        if ($LASTEXITCODE -eq 0) {
            Write-Success "TypeScript documentation generated successfully!"
            if (Test-Path "docs\typescript\index.html") {
                $docPath = Resolve-Path "docs\typescript\index.html"
                Write-Host "Location: $docPath" -ForegroundColor Gray
            }
        } else {
            Write-Error-Message "Failed to generate TypeScript documentation"
            exit 1
        }
    } catch {
        Write-Error-Message "Error generating TypeScript docs: $_"
        exit 1
    }
}

function Open-RustDocs {
    Write-Header "Opening Rust Documentation"

    $docPath = "target\doc\beefcake\index.html"
    if (Test-Path $docPath) {
        Start-Process $docPath
        Write-Success "Opened Rust documentation in browser"
    } else {
        Write-Host "Generating docs first..." -ForegroundColor Yellow
        Generate-RustDocs
        Start-Process (Resolve-Path $docPath)
    }
}

# Main execution
Write-Host @"

  ____             __            _
 |  _ \           / _|          | |
 | |_) | ___  ___| |_ ___  __ _| | _____
 |  _ < / _ \/ _ \  _/ __|/ _\` | |/ / _ \
 | |_) |  __/  __/ || (__| (_| |   <  __/
 |____/ \___|\___|_| \___|\__,_|_|\_\___|

  Documentation Generator

"@ -ForegroundColor Cyan

switch ($Target) {
    'all' {
        Generate-RustDocs
        Generate-TypeScriptDocs

        Write-Host "`n" -NoNewline
        Write-Header "Documentation Summary"
        Write-Host "  Rust docs:       target\doc\beefcake\index.html" -ForegroundColor White
        Write-Host "  TypeScript docs: docs\typescript\index.html" -ForegroundColor White
        Write-Host "`nTo open Rust docs, run:" -ForegroundColor Gray
        Write-Host "  .\scripts\docs.ps1 open" -ForegroundColor Yellow
    }
    'rust' {
        Generate-RustDocs
    }
    'ts' {
        Generate-TypeScriptDocs
    }
    'open' {
        Open-RustDocs
    }
}

Write-Host "`n"
