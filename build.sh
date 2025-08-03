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
PKG_VERSION="1.0.0"
PKG_ARCH=$(dpkg --print-architecture 2>/dev/null || echo "amd64")

while [[ $# -gt 0 ]]; do
    case $1 in
        --type)
            BUILD_TYPE="$2"
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
        -h|--help)
            echo "Usage: $0 [options]"
            echo "Options:"
            echo "  --type [all|go|blazor|deb]    What to build (default: all)"
            echo "  --version VERSION             Package version (default: 1.0.0)"
            echo "  --arch ARCH                   Package architecture (default: auto-detect)"
            echo "  -h, --help                    Show this help"
            echo ""
            echo "Examples:"
            echo "  $0                            # Build everything"
            echo "  $0 --type blazor              # Build only Blazor frontend"
            echo "  $0 --type deb --version 1.2.0 # Build only Debian package with version 1.2.0"
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
echo "Version: $PKG_VERSION"
echo "Architecture: $PKG_ARCH"
echo ""

# Function to build Blazor WebAssembly
build_blazor() {
    print_step "Building Blazor WebAssembly frontend..."
    
    if [ ! -d "FilePiWeb" ]; then
        print_error "FilePiWeb directory not found. Please create the Blazor project first."
        return 1
    fi
    
    # Clean previous build
    rm -rf webdeploy/ temp-publish/
    
    # Build Blazor WebAssembly
    cd FilePiWeb
    
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
    dotnet publish -c Release -o ../temp-publish
    cd ..
    
    # Copy only the wwwroot contents to webdeploy
    mkdir -p webdeploy
    cp -r temp-publish/wwwroot/* webdeploy/
    rm -rf temp-publish/
    
    print_success "Blazor WebAssembly build completed"
    echo "Output: ./webdeploy/"
}

# Function to build Go application
build_go() {
    print_step "Building Go application..."
    
    # Clean previous build
    rm -f filepi filepi.exe
    
    # Build Go application
    go mod tidy
    go build -ldflags "-s -w" -o filepi .
    
    print_success "Go application build completed"
    echo "Output: ./filepi"
}

# Function to create Debian package
build_deb() {
    print_step "Creating Debian package..."
    
    # Check prerequisites
    if [ ! -f "filepi" ]; then
        print_error "filepi binary not found. Run with --type go first."
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
    "go")
        build_go
        ;;
    "deb")
        build_deb
        ;;
    "all")
        build_blazor
        build_go
        build_deb
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
    echo "  - Go executable: ./filepi"
fi
if [ -d "webdeploy" ]; then
    echo "  - Blazor UI: ./webdeploy/"
fi
if [ -f "outputs/filepi_${PKG_VERSION}_${PKG_ARCH}.deb" ]; then
    echo "  - Debian package: outputs/filepi_${PKG_VERSION}_${PKG_ARCH}.deb"
fi

if [ "$BUILD_TYPE" = "all" ] || [ "$BUILD_TYPE" = "deb" ]; then
    echo ""
    echo "🚀 To install the package:"
    echo "  sudo dpkg -i outputs/filepi_${PKG_VERSION}_${PKG_ARCH}.deb"
    echo ""
    echo "🌐 After installation, access FilePi at:"
    echo "  http://[your-server-ip]:8080"
fi