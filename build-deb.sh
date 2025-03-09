#!/bin/bash
set -e

# Check if version argument is provided
if [ "$1" != "" ]; then
  PKG_VERSION="$1"
else
  PKG_VERSION="1.0.0"
fi

# Replace underscores with hyphens
PKG_VERSION=$(echo "$PKG_VERSION" | sed 's/_/-/g')

# Create outputs directory if it doesn't exist
OUTPUT_DIR="outputs"
mkdir -p "${OUTPUT_DIR}"


# This script builds a .deb package for FilePi without requiring debhelper
echo "Building FilePi Debian package..."

# Check if filepi binary exists
if [ ! -f "filepi" ]; then
    echo "Error: filepi binary not found in current directory"
    echo "Please build the filepi binary first and place it in the current directory"
    exit 1
fi

# Check if filepi.service exists
if [ ! -f "filepi.service" ]; then
    echo "Error: filepi.service not found in current directory"
    exit 1
fi

# Set package details
PKG_NAME="filepi"
PKG_ARCH=$(dpkg --print-architecture)
PKG_DIR="${PKG_NAME}_${PKG_VERSION}_${PKG_ARCH}"

echo "Creating package structure for ${PKG_DIR}..."

# Create package directory structure
mkdir -p "${PKG_DIR}/DEBIAN"
mkdir -p "${PKG_DIR}/opt/filepi"
mkdir -p "${PKG_DIR}/lib/systemd/system"
mkdir -p "${PKG_DIR}/var/lib/filepi/media"

# Create control file
cat > "${PKG_DIR}/DEBIAN/control" << EOF
Package: filepi
Version: ${PKG_VERSION}
Section: net
Priority: optional
Architecture: ${PKG_ARCH}
Depends: ffmpeg
Maintainer: Renju Ashokan <renju@example.com>
Description: Lightweight network file browser
 FilePi is a lightweight network file browser designed
 primarily for Raspberry Pi and other resource-constrained
 devices. It allows you to browse, stream, and manage files
 on your device from any web browser or through the
 dedicated Pi View mobile app.
EOF

# Create postinst script
cat > "${PKG_DIR}/DEBIAN/postinst" << EOF
#!/bin/sh
set -e

# Set permissions
chmod 755 /opt/filepi/filepi

# Create the cache directory
mkdir -p /var/lib/filepi/media/.cache

# Reload systemd to recognize the new service
systemctl daemon-reload

# Enable and start the service
systemctl enable filepi.service
systemctl start filepi.service || true

exit 0
EOF

# Create postrm script
cat > "${PKG_DIR}/DEBIAN/postrm" << EOF
#!/bin/sh
set -e

case "\$1" in
    purge)
        # Remove the data directory
        rm -rf /var/lib/filepi
    ;;
    
    remove|upgrade|failed-upgrade|abort-install|abort-upgrade|disappear)
        # Stop the service if it's running
        systemctl stop filepi.service || true
        systemctl disable filepi.service || true
        systemctl daemon-reload
    ;;
esac

exit 0
EOF

# Make scripts executable
chmod 755 "${PKG_DIR}/DEBIAN/postinst"
chmod 755 "${PKG_DIR}/DEBIAN/postrm"

# Copy files
cp filepi "${PKG_DIR}/opt/filepi/"
cp filepi.service "${PKG_DIR}/lib/systemd/system/"

# Build the package
echo "Building Debian package..."
dpkg-deb --build "${PKG_DIR}"

# Move the .deb file to the outputs directory
mv "${PKG_DIR}.deb" "${OUTPUT_DIR}/"

rm -rf "${PKG_DIR}"

echo "Package built successfully: ${PKG_DIR}.deb"