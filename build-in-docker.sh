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

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Parse arguments
PKG_VERSION="1.0.0"
PKG_ARCH="amd64"

while [[ $# -gt 0 ]]; do
    case $1 in
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
            echo "  --version VERSION    Package version (default: 1.0.0)"
            echo "  --arch ARCH          Package architecture (amd64 or arm64, default: amd64)"
            echo "  -h, --help           Show this help"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

print_step "Building Debian package in Docker..."
echo "Version: $PKG_VERSION"
echo "Architecture: $PKG_ARCH"
echo ""

# Check prerequisites
if [ ! -f "filepi" ]; then
    print_error "filepi binary not found. Please build it first with:"
    echo "  ./build.sh --type rust --mode release"
    exit 1
fi

if [ ! -d "webdeploy" ] || [ ! -f "webdeploy/index.html" ]; then
    print_error "webdeploy directory not found. Please build it first with:"
    echo "  ./build.sh --type blazor"
    exit 1
fi

# Build Docker image
print_step "Building Docker image..."
docker build -t filepi-deb-builder .

# Run the build in Docker
print_step "Running Debian package build in container..."
docker run --rm \
    -v "$(pwd):/build" \
    -u "$(id -u):$(id -g)" \
    filepi-deb-builder \
    ./build-deb.sh "$PKG_VERSION" "$PKG_ARCH"

print_success "Debian package build completed!"
echo ""
echo "📁 Output: outputs/filepi_${PKG_VERSION}_${PKG_ARCH}.deb"
echo ""
echo "To verify the package:"
echo "  docker run --rm -v \"\$(pwd)/outputs:/outputs\" debian:bookworm-slim dpkg -c /outputs/filepi_${PKG_VERSION}_${PKG_ARCH}.deb"
