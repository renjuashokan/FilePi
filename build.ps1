param (
    [string]$Type = "all",
    [string]$Mode = "debug",
    [string]$Version = "1.0.0",
    [string]$Arch = "auto",
    [switch]$Clean = $false,
    [switch]$Help = $false
)

if ($Help) {
    Write-Host "Usage: .\build.ps1 [options]"
    Write-Host "Options:"
    Write-Host "  -Type [all|rust|blazor]       What to build (default: all)"
    Write-Host "  -Mode [debug|release]         Build mode (default: debug)"
    Write-Host "  -Version VERSION              Package version (default: 1.0.0)"
    Write-Host "  -Arch ARCH                    Package architecture (default: auto)"
    Write-Host "  -Clean                        Clean build artifacts before building"
    Write-Host "  -Help                         Show this help"
    Write-Host ""
    Write-Host "Examples:"
    Write-Host "  .\build.ps1                                   # Build everything in debug mode"
    Write-Host "  .\build.ps1 -Type blazor                      # Build only Blazor frontend"
    Write-Host "  .\build.ps1 -Mode release                     # Build in release mode"
    exit 0
}

$ErrorActionPreference = "Stop"

# Colors
function Write-Step { param([string]$Message) Write-Host "📦 $Message" -ForegroundColor Cyan }
function Write-Success { param([string]$Message) Write-Host "✅ $Message" -ForegroundColor Green }
function Write-Warning { param([string]$Message) Write-Host "⚠️  $Message" -ForegroundColor Yellow }
function Write-Error { param([string]$Message) Write-Host "❌ $Message" -ForegroundColor Red }

Write-Step "Starting FilePi build process..."
Write-Host "Build type: $Type"
Write-Host "Build mode: $Mode"
Write-Host "Version: $Version"
Write-Host "Architecture: $Arch"
Write-Host ""

$ScriptDir = $PSScriptRoot
$FilePiWebDir = Join-Path $ScriptDir "frontend\FilePiWeb"
$WebDeployDir = Join-Path $ScriptDir "webdeploy"

function Build-Blazor {
    Write-Step "Building Blazor WebAssembly frontend..."
    $TempPublishDir = Join-Path $ScriptDir "temp-publish"
    $WebProject = "FilePiWeb.csproj"

    if (-not (Test-Path $FilePiWebDir)) {
        Write-Error "FilePiWeb directory not found. Please create the Blazor project first."
        return
    }

    # Clean previous build
    if (Test-Path $WebDeployDir) { Remove-Item -Recurse -Force $WebDeployDir }
    if (Test-Path $TempPublishDir) { Remove-Item -Recurse -Force $TempPublishDir }

    Push-Location $FilePiWebDir

    # Restore LibMan packages if libman.json exists
    if (Test-Path "libman.json") {
        Write-Step "Restoring client-side libraries..."
        if (Get-Command libman -ErrorAction SilentlyContinue) {
            libman restore
        }
        else {
            Write-Warning "libman not found, skipping client library restore"
        }
    }

    # Build and publish Blazor
    dotnet restore $WebProject
    dotnet build $WebProject -c Release
    dotnet publish $WebProject -c Release -o $TempPublishDir
    
    Pop-Location

    # Copy only the wwwroot contents to webdeploy
    New-Item -ItemType Directory -Force -Path $WebDeployDir | Out-Null
    Copy-Item -Recurse -Force "$TempPublishDir\wwwroot\*" $WebDeployDir
    Remove-Item -Recurse -Force $TempPublishDir

    Write-Success "Blazor WebAssembly build completed"
    Write-Host "Output: $WebDeployDir"
}

function Build-Rust {
    Write-Step "Building Rust application..."

    # Clean previous build
    if ($Clean) {
        if ($Mode -eq "release") {
            cargo clean --release
        }
        else {
            cargo clean
        }
    }

    # Build Rust application
    if ($Mode -eq "release") {
        Write-Step "Building in release mode (optimized)..."
        cargo build --release

        # Copy binary to root
        $Target = "target\release\filepi.exe"
        if (Test-Path $Target) {
            Copy-Item -Force $Target ".\filepi.exe"
            Write-Success "Rust application build completed (release)"
            Write-Host "Output: $Target or .\filepi.exe"
        }
        else {
            Write-Error "Build failed: $Target not found"
        }
    }
    else {
        Write-Step "Building in debug mode..."
        cargo build

        # Copy binary to root
        $Target = "target\debug\filepi.exe"
        if (Test-Path $Target) {
            Copy-Item -Force $Target ".\filepi.exe"
            Write-Success "Rust application build completed (debug)"
            Write-Host "Output: $Target or .\filepi.exe"
        }
        else {
            Write-Error "Build failed: $Target not found"
        }
    }
}

switch ($Type) {
    "blazor" { Build-Blazor }
    "rust" { Build-Rust }
    "all" {
        Build-Blazor
        Build-Rust
    }
    default {
        Write-Error "Invalid build type: $Type"
        exit 1
    }
}

Write-Success "Build process completed!"
