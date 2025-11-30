#!/bin/bash
set -e

# Check if version argument is provided
if [ "$1" != "" ]; then
  PKG_VERSION="$1"
else
  PKG_VERSION="1.0.0"
fi

if [ "$2" != "" ]; then
  PKG_ARCH="$2"
else
  PKG_ARCH=$(dpkg --print-architecture 2>/dev/null || echo "amd64")
fi

# Replace underscores with hyphens in version
PKG_VERSION=$(echo "$PKG_VERSION" | sed 's/_/-/g')

# Create outputs directory if it doesn't exist
OUTPUT_DIR="outputs"
mkdir -p "${OUTPUT_DIR}"

echo "Building FilePi Debian package..."
echo "Version: $PKG_VERSION"
echo "Architecture: $PKG_ARCH"

# Check if filepi binary exists
if [ ! -f "filepi" ]; then
    echo "Error: filepi binary not found in current directory"
    echo "Please build the filepi binary first."
    exit 1
fi

# Check if webdeploy directory exists
if [ ! -d "webdeploy" ]; then
    echo "Error: webdeploy directory not found"
    echo "Please build the Blazor frontend first."
    exit 1
fi

# Set package details
PKG_NAME="filepi"
PKG_DIR="${PKG_NAME}_${PKG_VERSION}_${PKG_ARCH}"
STAGING_DIR="build_deb_temp/${PKG_DIR}"
CONTROL_FILE="${STAGING_DIR}/DEBIAN/control"
CONTROL_TMP="${STAGING_DIR}/DEBIAN/control.tmp"

# Clean up previous staging
rm -rf "build_deb_temp"
mkdir -p "${STAGING_DIR}"

echo "Creating package structure in ${STAGING_DIR}..."

# Create directory structure
mkdir -p "${STAGING_DIR}/DEBIAN"
mkdir -p "${STAGING_DIR}/opt/filepi"
mkdir -p "${STAGING_DIR}/lib/systemd/system"
mkdir -p "${STAGING_DIR}/var/lib/filepi/media"

# Copy Debian control files
if [ -d "debian" ]; then
    cp -r debian/* "${STAGING_DIR}/DEBIAN/"
    # Remove filepi.service from DEBIAN if it was copied there (it belongs in systemd)
    rm -f "${STAGING_DIR}/DEBIAN/filepi.service"

    # Extract Maintainer from the source stanza before removing it
    MAINTAINER=$(grep "^Maintainer:" "${STAGING_DIR}/DEBIAN/control" | head -1)

    # Extract only the Package stanza from control file (remove Source stanza)
    sed -n '/^Package:/,$p' "$CONTROL_FILE" > "$CONTROL_TMP"
    mv "$CONTROL_TMP" "$CONTROL_FILE"

    # Add Maintainer field after Package field if it's missing
    if ! grep -q "^Maintainer:" "$CONTROL_FILE"; then
        sed -i.bak "1a\\
$MAINTAINER" "$CONTROL_FILE" 2>/dev/null || sed -i "" "1a\\
$MAINTAINER
" "$CONTROL_FILE"
        rm -f "${CONTROL_FILE}.bak"
    fi

    # Remove debhelper variable placeholders
    sed 's/${shlibs:Depends}, //g' "$CONTROL_FILE" > "$CONTROL_TMP"
    mv "$CONTROL_TMP" "$CONTROL_FILE"

    sed 's/${misc:Depends}, //g' "$CONTROL_FILE" > "$CONTROL_TMP"
    mv "$CONTROL_TMP" "$CONTROL_FILE"

    sed 's/${shlibs:Depends}//g' "$CONTROL_FILE" > "$CONTROL_TMP"
    mv "$CONTROL_TMP" "$CONTROL_FILE"

    sed 's/${misc:Depends}//g' "$CONTROL_FILE" > "$CONTROL_TMP"
    mv "$CONTROL_TMP" "$CONTROL_FILE"

    # Clean up any trailing commas or spaces in Depends
    sed 's/Depends: , /Depends: /g' "$CONTROL_FILE" > "$CONTROL_TMP"
    mv "$CONTROL_TMP" "$CONTROL_FILE"
else
    echo "Error: debian directory not found!"
    exit 1
fi

# Process control file to replace variables
sed "s/Architecture: .*/Architecture: ${PKG_ARCH}/" "$CONTROL_FILE" > "$CONTROL_TMP"
mv "$CONTROL_TMP" "$CONTROL_FILE"

# Add Version field after Package field (since we removed it from the template)
sed -i.bak "2i\\
Version: ${PKG_VERSION}" "$CONTROL_FILE" 2>/dev/null || sed -i "" "2i\\
Version: ${PKG_VERSION}
" "$CONTROL_FILE"
rm -f "${CONTROL_FILE}.bak"

# Ensure there is a newline at the end of the file
echo "" >> "$CONTROL_FILE"

# Copy binary
echo "Copying binary..."
cp filepi "${STAGING_DIR}/opt/filepi/"
chmod 755 "${STAGING_DIR}/opt/filepi/filepi"

# Copy service file
echo "Copying service file..."
if [ -f "debian/filepi.service" ]; then
    cp debian/filepi.service "${STAGING_DIR}/lib/systemd/system/"
else
    echo "Warning: debian/filepi.service not found"
fi

# Copy webdeploy directory
echo "Copying webdeploy files..."
cp -r webdeploy "${STAGING_DIR}/opt/filepi/"

# Set permissions for scripts
chmod 755 "${STAGING_DIR}/DEBIAN/postinst"
chmod 755 "${STAGING_DIR}/DEBIAN/postrm" 2>/dev/null || true
chmod 755 "${STAGING_DIR}/DEBIAN/preinst" 2>/dev/null || true
chmod 755 "${STAGING_DIR}/DEBIAN/prerm" 2>/dev/null || true

# Calculate installed size
INSTALLED_SIZE=$(du -sk "${STAGING_DIR}" | cut -f1)
echo "Installed-Size: ${INSTALLED_SIZE}" >> "$CONTROL_FILE"

# Build the package
echo "Building .deb package..."
if command -v dpkg-deb >/dev/null 2>&1; then
    dpkg-deb --root-owner-group --build "${STAGING_DIR}"
    # Move the .deb file to the outputs directory
    mv "build_deb_temp/${PKG_DIR}.deb" "${OUTPUT_DIR}/"
    echo "Package built successfully: ${OUTPUT_DIR}/${PKG_DIR}.deb"
else
    echo "Warning: dpkg-deb not found, skipping package build."
    echo "Package structure created in ${STAGING_DIR}"
fi

# Clean up
# rm -rf "build_deb_temp"