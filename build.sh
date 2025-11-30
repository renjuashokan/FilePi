#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color


print_step() {
    echo -e "${BLUE}📦 $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}


# Parse command line arguments
BUILD_TYPE="all"
BUILD_MODE="debug"
PKG_VERSION="1.0.0"
PKG_ARCH=$(dpkg --print-architecture 2>/dev/null || echo "amd64")
CLEAN_BUILD="false"

while [[ $# -gt 0 ]]; do
    case $1 in
        --type)
            BUILD_TYPE="$2"
            shift 2
            ;;
        --mode)
            BUILD_MODE="$2"
            shift 2
            ;;
        --version)
            PKG_VERSION="$2"
            shift 2
            ;;
        --arch)
            PKG_ARCH="$2"
            shift 2
            ;;
        --clean)
            CLEAN_BUILD="true"
            shift 1
            ;;
        -h|--help)
            echo "Usage: $0 [options]"
            echo "Options:"
            echo "  --type [all|rust|blazor|deb]  What to build (default: all)"
            echo "  --mode [debug|release]        Build mode (default: release)"
            echo "  --version VERSION             Package version (default: 1.0.0)"
            echo "  --arch ARCH                   Package architecture (default: auto-detect)"
            echo "  --clean                       Clean build artifacts before building"
            echo "  -h, --help                    Show this help"
            echo ""
            echo "Examples:"
            echo "  $0                            # Build everything in release mode"
            echo "  $0 --type blazor              # Build only Blazor frontend"
            echo "  $0 --mode debug               # Build in debug mode"
            echo "  $0 --type deb --version 1.2.0 # Build only Debian package"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

print_step "Starting FilePi build process..."
echo "Build type: $BUILD_TYPE"
echo "Build mode: $BUILD_MODE"
echo "Version: $PKG_VERSION"
echo "Architecture: $PKG_ARCH"
echo ""

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
FILEPI_WEB_DIR="$SCRIPT_DIR/frontend/FilePiWeb"
WEBDEPLOY_DIR="$SCRIPT_DIR/webdeploy"

# Function to build Blazor WebAssembly
build_blazor() {
    print_step "Building Blazor WebAssembly frontend..."
    TEMP_PUBLISH_DIR="$SCRIPT_DIR/temp-publish"
    WEB_PROJECT="FilePiWeb.csproj"
    
    if [ ! -d "$FILEPI_WEB_DIR" ]; then
        print_error "FilePiWeb directory not found. Please create the Blazor project first."
        return 1
    fi
    
    # Clean previous build
    rm -rf $WEBDEPLOY_DIR $TEMP_PUBLISH_DIR
    
    # Build Blazor WebAssembly
    cd $FILEPI_WEB_DIR
    
    # Restore LibMan packages if libman.json exists
    if [ -f "libman.json" ]; then
        print_step "Restoring client-side libraries..."
        if command -v libman &> /dev/null; then
            libman restore
        else
            print_warning "libman not found, skipping client library restore"
        fi
    fi
    
    # Build and publish Blazor
    dotnet restore $WEB_PROJECT
    dotnet build $WEB_PROJECT -c Release
    dotnet publish $WEB_PROJECT -c Release -o $TEMP_PUBLISH_DIR
    cd ..
    
    # Copy only the wwwroot contents to webdeploy
    mkdir -p $WEBDEPLOY_DIR
    cp -r $TEMP_PUBLISH_DIR/wwwroot/* $WEBDEPLOY_DIR/
    rm -rf $TEMP_PUBLISH_DIR
    
    print_success "Blazor WebAssembly build completed"
    echo "Output: $WEBDEPLOY_DIR"
}


# Function to build Rust application
build_rust() {
    print_step "Building Rust application..."
    
    # Clean previous build
    if [ "$CLEAN_BUILD" = "true" ]; then
        if [ "$BUILD_MODE" = "release" ]; then
            cargo clean --release
        else
            cargo clean
        fi
    fi
    
    # Build Rust application
    if [ "$BUILD_MODE" = "release" ]; then
        print_step "Building in release mode (optimized)..."
        cargo build --release
        
        # Copy binary to root for easier access
        cp target/release/filepi-rust ./filepi || cp target/release/filepi-rust.exe ./filepi.exe 2>/dev/null || true
        
        print_success "Rust application build completed (release)"
        echo "Output: ./target/release/filepi-rust or ./filepi"
    else
        print_step "Building in debug mode..."
        cargo build
        
        # Copy binary to root for easier access
        cp target/debug/filepi-rust ./filepi || cp target/debug/filepi-rust.exe ./filepi.exe 2>/dev/null || true
        
        print_success "Rust application build completed (debug)"
        echo "Output: ./target/debug/filepi-rust or ./filepi"
    fi
}

# Function to create Debian package
build_deb() {
    print_step "Creating Debian package..."
    
    # Check prerequisites
    if [ ! -f "filepi" ] && [ ! -f "target/release/filepi-rust" ]; then
        print_error "filepi binary not found. Run with --type rust first."
        return 1
    fi
    
    if [ ! -d "webdeploy" ] || [ ! -f "webdeploy/index.html" ]; then
        print_error "webdeploy directory not found or incomplete. Run with --type blazor first."
        return 1
    fi
    
    # Run the Debian package build
    ./build-deb.sh "$PKG_VERSION" "$PKG_ARCH"
    
    print_success "Debian package build completed"
    echo "Output: outputs/filepi_${PKG_VERSION}_${PKG_ARCH}.deb"
}

# Execute based on build type
case $BUILD_TYPE in
    "blazor")
        build_blazor
        ;;
    "rust")
        build_rust
        ;;
    # "deb")
    #     build_deb
    #     ;;
    "all")
        build_blazor
        build_rust
        ;;
    *)
        print_error "Invalid build type: $BUILD_TYPE"
        exit 1
        ;;
esac

print_success "Build process completed!"

# Show final outputs
echo ""
echo "📁 Generated files:"
if [ -f "filepi" ]; then
    echo "  - Rust executable: ./filepi"
fi
if [ -f "target/release/filepi-rust" ]; then
    echo "  - Rust executable: ./target/release/filepi-rust"
fi
if [ -f "target/debug/filepi-rust" ]; then
    echo "  - Rust executable: ./target/debug/filepi-rust"
fi
if [ -d "webdeploy" ]; then
    echo "  - Blazor UI: ./webdeploy/"
fi
if [ -f "outputs/filepi_${PKG_VERSION}_${PKG_ARCH}.deb" ]; then
    echo "  - Debian package: outputs/filepi_${PKG_VERSION}_${PKG_ARCH}.deb"
fi
