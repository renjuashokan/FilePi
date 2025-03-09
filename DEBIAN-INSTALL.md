# Installing FilePi using Debian package

This document provides instructions for installing FilePi using the Debian package (.deb).

## Prerequisites

- Debian-based Linux distribution (Debian, Ubuntu, Raspberry Pi OS, etc.)
- FFmpeg installed (`sudo apt install ffmpeg`)

## Installation

1. Download the appropriate .deb package for your architecture:
   - `filepi_*_amd64.deb` for 64-bit x86 systems
   - `filepi_*_arm64.deb` for 64-bit ARM systems (Raspberry Pi 4 with 64-bit OS)
   - `filepi_*_armhf.deb` for 32-bit ARM systems (older Raspberry Pi models)

2. Install the package:
   ```bash
   sudo dpkg -i filepi_*.deb
   ```

3. If you encounter any dependency issues, run:
   ```bash
   sudo apt-get install -f
   ```

## Post-Installation

- The FilePi service will be automatically enabled and started
- Files are served from `/var/lib/filepi/media` by default

## Configuration

### Changing the media directory

1. Edit the systemd service file:
   ```bash
   sudo systemctl edit filepi.service
   ```

2. Add the following lines:
   ```ini
   [Service]
   Environment=FILE_PI_ROOT_DIR=/your/preferred/path
   ```

3. Restart the service:
   ```bash
   sudo systemctl restart filepi.service
   ```

### Changing log level

1. Edit the systemd service file:
   ```bash
   sudo systemctl edit filepi.service
   ```

2. Add the following lines:
   ```ini
   [Service]
   Environment=FILE_PI_LOGLEVEL=DEBUG
   ```

3. Restart the service:
   ```bash
   sudo systemctl restart filepi.service
   ```

## Service Management

- Check service status:
  ```bash
  sudo systemctl status filepi.service
  ```

- Stop the service:
  ```bash
  sudo systemctl stop filepi.service
  ```

- Start the service:
  ```bash
  sudo systemctl start filepi.service
  ```

- Disable automatic startup:
  ```bash
  sudo systemctl disable filepi.service
  ```

- View logs:
  ```bash
  sudo journalctl -u filepi.service
  ```

## Uninstallation

To completely remove FilePi:

```bash
sudo systemctl stop filepi.service
sudo apt remove --purge filepi
```

This will remove the application and its configuration files. User data in the media directory will be preserved.