# FilePi - Lightweight Network File Browser

FilePi is a lightweight network file browser designed primarily for Raspberry Pi and other resource-constrained devices. It allows you to browse, stream, and manage files on your device from any web browser or through the dedicated [Pi View mobile app](https://github.com/renjuashokan/pi_view).

## Features

- 📁 File browsing with sorting and pagination
- 🎬 Video streaming support with thumbnails
- 🔍 File search functionality
- 📤 File upload capabilities
- 📱 Compatible with Pi View mobile app

## Requirements

- FFmpeg (required for video thumbnail generation)


## Quick Start

### 1. Install FFmpeg

#### On Raspberry Pi / Debian / Ubuntu:
```bash
sudo apt update
sudo apt install ffmpeg
```

#### On macOS:
```bash
brew install ffmpeg
```

### 2. Set Environment Variables

```bash
# Set the root directory for file browsing
export FILE_PI_ROOT_DIR=/path/to/your/files

# Optional: Set custom port (default: 8080)
export FILE_PI_PORT=3000

# Optional: Set log level (DEBUG, INFO, WARN, ERROR)
export FILE_PI_LOGLEVEL=INFO
```

### 3. Run the Server

#### Using the Go implementation:
```bash
# Download the latest release from the releases section
./filepi
```


### 4. Access the File Browser

- Web interface: Open `http://[device-ip]:[port]` in your browser (default port: 8080)
- Mobile: Install the [Pi View app](https://github.com/renjuashokan/pi_view) and connect to your device

## API Endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /api/v1/files` | List files in a directory |
| `GET /api/v1/videos` | List video files (with recursive option) |
| `GET /api/v1/search` | Search for files by name |
| `GET /api/v1/file/{path}` | Serve a file for download |
| `GET /api/v1/stream/{path}` | Stream a video file |
| `GET /api/v1/thumbnail/{path}` | Get a thumbnail for a video file |
| `POST /api/v1/createfolder` | Create a new folder |
| `POST /api/v1/uploadfile` | Upload a file |

## Configuration Options

| Parameter | Default | Description |
|-----------|---------|-------------|
| `FILE_PI_ROOT_DIR` | Current directory | Root directory for file browsing |
| `FILE_PI_PORT` | 8080 | Port number for the HTTP server |
| `FILE_PI_LOGLEVEL` | INFO | Log level (DEBUG, INFO, WARN, ERROR) |

## Examples

### Running on different ports

```bash
# Run on port 3000
FILE_PI_PORT=3000 ./filepi

# Run on port 9090 with custom directory
FILE_PI_PORT=9090 FILE_PI_ROOT_DIR=/home/user/media ./filepi

# Run with all custom settings
FILE_PI_PORT=8888 FILE_PI_ROOT_DIR=/srv/files FILE_PI_LOGLEVEL=DEBUG ./filepi
```

## Mobile App

FilePi works seamlessly with the [Pi View mobile app](https://github.com/renjuashokan/pi_view), which provides a user-friendly interface for browsing and streaming files from your server.

## Development

### Project Structure

- Go version: Uses Fiber framework
  - `fileserver.go`: Core file operations
  - `router.go`: API endpoint definitions
  - `main.go`: Server initialization

### Building from Source

#### Go version:
```bash
go build -o filepi
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgements

- [Fiber](https://gofiber.io/) - Go web framework
- [FFmpeg](https://ffmpeg.org/) - Used for video thumbnail generation