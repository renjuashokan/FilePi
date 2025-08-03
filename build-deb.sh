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
  PKG_ARCH=$(dpkg --print-architecture)
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
    echo "Please build the filepi binary first using: go build -o filepi ."
    exit 1
fi

# Check if filepi.service exists
if [ ! -f "filepi.service" ]; then
    echo "Error: filepi.service not found in current directory"
    exit 1
fi

# Check if webdeploy directory exists
if [ ! -d "webdeploy" ]; then
    echo "Error: webdeploy directory not found"
    echo "Please build the Blazor frontend first using: ./build.sh"
    echo "Or manually run:"
    echo "  cd FilePiWeb"
    echo "  dotnet publish -c Release -o ../temp-publish"
    echo "  cd .."
    echo "  mkdir -p webdeploy"
    echo "  cp -r temp-publish/wwwroot/* webdeploy/"
    echo "  rm -rf temp-publish/"
    exit 1
fi

# Verify webdeploy has content
if [ ! -f "webdeploy/index.html" ]; then
    echo "Error: webdeploy directory exists but appears to be empty or incomplete"
    echo "Expected to find webdeploy/index.html"
    exit 1
fi

# Set package details
PKG_NAME="filepi"
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
Maintainer: Renju Ashokan <renjuashokan@gmail.com>
Description: Lightweight network file browser with web UI
 FilePi is a lightweight network file browser designed
 primarily for Raspberry Pi and other resource-constrained
 devices. It allows you to browse, stream, and manage files
 on your device from any web browser through a modern
 Blazor WebAssembly interface.
 .
 Features include file browsing with sorting and pagination,
 video streaming with thumbnails, file search functionality,
 file upload capabilities, and a responsive web interface
 accessible from any device on the network.
EOF

# Create postinst script
cat > "${PKG_DIR}/DEBIAN/postinst" << EOF
#!/bin/sh
set -e

# Set permissions
chmod 755 /opt/filepi/filepi

# Set ownership and permissions for webdeploy
chown -R root:root /opt/filepi/webdeploy
find /opt/filepi/webdeploy -type f -exec chmod 644 {} \;
find /opt/filepi/webdeploy -type d -exec chmod 755 {} \;

# Create the cache directory
mkdir -p /var/lib/filepi/media/.cache

# Set ownership for media directory
chown -R root:root /var/lib/filepi

# Reload systemd to recognize the new service
systemctl daemon-reload

# Enable and start the service
systemctl enable filepi.service
systemctl start filepi.service || true

echo "FilePi has been installed and started!"
echo "Access the web interface at: http://\$(hostname -I | awk '{print \$1}'):8080"
echo "Media files should be placed in: /var/lib/filepi/media/"
echo "Check service status with: systemctl status filepi"

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
        # Remove the application directory
        rm -rf /opt/filepi
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
echo "Copying binary and service file..."
cp filepi "${PKG_DIR}/opt/filepi/"
cp filepi.service "${PKG_DIR}/lib/systemd/system/"

# Copy webdeploy directory
echo "Copying Blazor WebAssembly files..."
cp -r webdeploy "${PKG_DIR}/opt/filepi/"

# Calculate installed size for the control file
INSTALLED_SIZE=$(du -sk "${PKG_DIR}" | cut -f1)
echo "Installed-Size: ${INSTALLED_SIZE}" >> "${PKG_DIR}/DEBIAN/control"

# Build the package
echo "Building Debian package..."
dpkg-deb --build "${PKG_DIR}"

# Move the .deb file to the outputs directory
mv "${PKG_DIR}.deb" "${OUTPUT_DIR}/"

rm -rf "${PKG_DIR}"

echo "Package built successfully: ${OUTPUT_DIR}/${PKG_DIR}.deb"
echo ""
echo "To install the package:"
echo "  sudo dpkg -i ${OUTPUT_DIR}/${PKG_DIR}.deb"
echo ""
echo "After installation, access FilePi at: http://[your-server-ip]:8080"