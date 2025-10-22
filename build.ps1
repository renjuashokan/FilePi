# PowerShell Build Script for FilePi (Rust + Blazor)
param(
    [string]$Type = "all",
    [string]$Mode = "release",
    [string]$Version = "1.0.0"
)

$ErrorActionPreference = "Stop"

# Color functions
function Write-Step {
    param([string]$Message)
    Write-Host "📦 $Message" -ForegroundColor Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "✅ $Message" -ForegroundColor Green
}

function Write-Warning-Custom {
    param([string]$Message)
    Write-Host "⚠️  $Message" -ForegroundColor Yellow
}

function Write-Error-Custom {
    param([string]$Message)
    Write-Host "❌ $Message" -ForegroundColor Red
}

# Help function
function Show-Help {
    Write-Host "Usage: .\build.ps1 [options]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -Type [all|rust|blazor]    What to build (default: all)"
    Write-Host "  -Mode [debug|release]      Build mode (default: release)"
    Write-Host "  -Version VERSION           Package version (default: 1.0.0)"
    Write-Host ""
    Write-Host "Examples:"
    Write-Host "  .\build.ps1                      # Build everything in release mode"
    Write-Host "  .\build.ps1 -Type blazor         # Build only Blazor frontend"
    Write-Host "  .\build.ps1 -Mode debug          # Build in debug mode"
    exit 0
}

if ($Type -eq "help" -or $Type -eq "-h") {
    Show-Help
}

Write-Step "Starting FilePi build process..."
Write-Host "Build type: $Type"
Write-Host "Build mode: $Mode"
Write-Host "Version: $Version"
Write-Host ""

# Function to build Blazor WebAssembly
function Build-Blazor {
    Write-Step "Building Blazor WebAssembly frontend..."
    
    if (-not (Test-Path "FilePiWeb")) {
        Write-Error-Custom "FilePiWeb directory not found. Please create the Blazor project first."
        return $false
    }
    
    # Clean previous build
    if (Test-Path "webdeploy") {
        Remove-Item -Recurse -Force "webdeploy"
    }
    if (Test-Path "temp-publish") {
        Remove-Item -Recurse -Force "temp-publish"
    }
    
    # Build Blazor WebAssembly
    Push-Location FilePiWeb
    
    try {
        # Restore LibMan packages if libman.json exists
        if (Test-Path "libman.json") {
            Write-Step "Restoring client-side libraries..."
            if (Get-Command libman -ErrorAction SilentlyContinue) {
                libman restore
            } else {
                Write-Warning-Custom "libman not found, skipping client library restore"
            }
        }
        
        # Build and publish Blazor
        Write-Step "Publishing Blazor project..."
        dotnet publish -c Release -o ..\temp-publish
        if ($LASTEXITCODE -ne 0) {
            throw "Blazor build failed"
        }
    }
    finally {
        Pop-Location
    }
    
    # Copy only the wwwroot contents to webdeploy
    New-Item -ItemType Directory -Force -Path "webdeploy" | Out-Null
    Copy-Item -Path "temp-publish\wwwroot\*" -Destination "webdeploy\" -Recurse -Force
    Remove-Item -Recurse -Force "temp-publish"
    
    Write-Success "Blazor WebAssembly build completed"
    Write-Host "Output: .\webdeploy\"
    return $true
}

# Function to build Rust application
function Build-Rust {
    Write-Step "Building Rust application..."
    
    # Clean previous build
    # if ($Mode -eq "release") {
    #     Write-Step "Cleaning previous release build..."
    #     cargo clean --release
    # } else {
    #     Write-Step "Cleaning previous debug build..."
    #     cargo clean
    # }
    
    # Build Rust application
    try {
        if ($Mode -eq "release") {
            Write-Step "Building in release mode (optimized)..."
            cargo build --release
            if ($LASTEXITCODE -ne 0) {
                throw "Rust build failed"
            }
            
            # Copy binary to root for easier access
            if (Test-Path "target\release\filepi-rust.exe") {
                Copy-Item "target\release\filepi-rust.exe" "filepi.exe" -Force
            }
            
            Write-Success "Rust application build completed (release)"
            Write-Host "Output: .\target\release\filepi-rust.exe or .\filepi.exe"
        } else {
            Write-Step "Building in debug mode..."
            cargo build
            if ($LASTEXITCODE -ne 0) {
                throw "Rust build failed"
            }
            
            # Copy binary to root for easier access
            if (Test-Path "target\debug\filepi-rust.exe") {
                Copy-Item "target\debug\filepi-rust.exe" "filepi.exe" -Force
            }
            
            Write-Success "Rust application build completed (debug)"
            Write-Host "Output: .\target\debug\filepi-rust.exe or .\filepi.exe"
        }
        return $true
    }
    catch {
        Write-Error-Custom "Rust build failed: $_"
        return $false
    }
}

# Execute based on build type
$success = $true

switch ($Type.ToLower()) {
    "blazor" {
        $success = Build-Blazor
    }
    "rust" {
        $success = Build-Rust
    }
    "all" {
        $success = (Build-Blazor) -and (Build-Rust)
    }
    default {
        Write-Error-Custom "Invalid build type: $Type"
        Show-Help
        exit 1
    }
}

if (-not $success) {
    Write-Error-Custom "Build process failed!"
    exit 1
}

Write-Success "Build process completed!"

# Show final outputs
Write-Host ""
Write-Host "📁 Generated files:"
if (Test-Path "filepi.exe") {
    Write-Host "  - Rust executable: .\filepi.exe"
}
if (Test-Path "target\release\filepi-rust.exe") {
    Write-Host "  - Rust executable: .\target\release\filepi-rust.exe"
}
if (Test-Path "target\debug\filepi-rust.exe") {
    Write-Host "  - Rust executable: .\target\debug\filepi-rust.exe"
}
if (Test-Path "webdeploy") {
    Write-Host "  - Blazor UI: .\webdeploy\"
}