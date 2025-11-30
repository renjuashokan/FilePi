# FilePi

FilePi is a lightweight, self-hosted file manager application built with a Rust backend and a Blazor WebAssembly frontend. It allows you to easily manage and serve files from your server.

## Prerequisites

Before building and running FilePi, ensure you have the following installed:

*   **Rust**: [Install Rust](https://www.rust-lang.org/tools/install) (latest stable version)
*   **.NET SDK**: [Install .NET SDK](https://dotnet.microsoft.com/download) (version 10.0)

## Building

To build the project, simply run the build script:

*   **Linux/macOS**: `./build.sh`
*   **Windows**: `./build.ps1`

> [!NOTE]
> If you have a Syncfusion license key, set the `SYNCFUSION_LICENSE_KEY` environment variable before building to inject it into the application.

## Running

After building, you can run the application using the generated binary.

### Environment Variables

The application can be configured using the following environment variables:

| Variable | Description | Default |
| :--- | :--- | :--- |
| `FILE_PI_ROOT_DIR` | The root directory to serve files from. | `.` (Current Directory) |
| `FILE_PI_PORT` | The HTTP port the server will listen on. | `8080` |
| `FILE_PI_LOGLEVEL` | The logging level (e.g., `info`, `debug`, `error`). | `info` |
| `FILE_PI_LOG_DIR` | The directory where logs will be stored. | `./logs` |

### Example Usage

Run the server serving files from your home directory on port 3000:

```bash
export FILE_PI_ROOT_DIR=$HOME
export FILE_PI_PORT=3000
./filepi
```

Or inline:

```bash
FILE_PI_ROOT_DIR=/path/to/files FILE_PI_PORT=9090 ./filepi
```

## Project Structure

*   `filepi-server/`: Rust backend source code.
*   `frontend/`: Blazor WebAssembly frontend source code.
*   `build.sh`: Main build script.
*   `webdeploy/`: Directory where frontend assets are deployed (served by the backend).
