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

After installation, the FilePi service will be automatically enabled and started.

- **Web Interface**: Access at `http://<your-server-ip>:8080`
- **Default Media Directory**: `/var/lib/filepi/media`
- **Default Port**: `8080`
- **Service Name**: `filepi.service`

To verify the service is running:
```bash
sudo systemctl status filepi.service
```

To find your server's IP address:
```bash
hostname -I
```

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

### Changing the server port

1. Edit the systemd service file:
   ```bash
   sudo systemctl edit filepi.service
   ```

2. Add the following lines:
   ```ini
   [Service]
   Environment=FILE_PI_PORT=8012
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

### Multiple configuration options

You can combine multiple environment variables in a single override:

```bash
sudo systemctl edit filepi.service
```

```ini
[Service]
Environment=FILE_PI_ROOT_DIR=/your/preferred/path
Environment=FILE_PI_PORT=8012
Environment=FILE_PI_LOGLEVEL=INFO
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

To remove FilePi while preserving user data:

```bash
sudo systemctl stop filepi.service
sudo apt remove filepi
```

To completely remove FilePi including all data and configuration:

```bash
sudo systemctl stop filepi.service
sudo apt remove --purge filepi
```

**Note:** Using `--purge` will permanently delete all files in `/var/lib/filepi/media/` including your media files. Use with caution!