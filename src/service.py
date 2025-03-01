import argparse
import os
import platform
import shutil
import subprocess
import sys


def install_service():
    """
    Install the filepi service on the system with a custom data directory.
    Usage: filepi-install-service --data-dir /path/to/data
    """
    # Check if the platform is Linux
    if platform.system() != "Linux":
        print("Error: This script is only supported on Linux systems.")
        sys.exit(1)

    # Set up argument parser
    parser = argparse.ArgumentParser(description="Install FilePi systemd service")
    parser.add_argument("--data-dir", type=str, required=True, help="Path to the data directory for FilePi")
    parser.add_argument("--filepi-path", type=str, help="Optional: Path to filepi executable (defaults to auto-detect)")

    # Parse arguments
    args = parser.parse_args()
    data_dir = args.data_dir
    filepi_path = args.filepi_path

    # Validate data directory
    if not os.path.isdir(data_dir):
        print(f"Error: Data directory '{data_dir}' does not exist or is not a directory")
        print("Please create the directory first or specify a valid directory")
        sys.exit(1)

    # Get absolute path to data directory
    data_dir = os.path.abspath(data_dir)

    # Find filepi executable path if not provided
    if not filepi_path:
        try:
            # Use 'which' to find the executable path
            result = subprocess.run(["which", "filepi"], capture_output=True, text=True, check=True)
            filepi_path = result.stdout.strip()
            print(f"Automatically detected filepi path: {filepi_path}")
        except subprocess.CalledProcessError:
            print("Error: Could not find filepi executable. Please install it or specify the path with --filepi-path")
            sys.exit(1)
    else:
        # Validate provided path
        if not os.path.isfile(filepi_path) or not os.access(filepi_path, os.X_OK):
            print(f"Error: The specified filepi path '{filepi_path}' is not a valid executable")
            sys.exit(1)

        # Ensure it's an absolute path
        filepi_path = os.path.abspath(filepi_path)

    # Create service file content directly
    service_content = f"""[Unit]
Description=FilePi Server
After=network.target

[Service]
Type=simple
ExecStart={filepi_path}
Environment="FILE_PI_ROOT_DIR={data_dir}"
Restart=always
RestartSec=10
KillSignal=SIGINT
SyslogIdentifier=filepi
StandardOutput=append:/var/log/filepi.log
StandardError=append:/var/log/filepi_error.log

[Install]
WantedBy=multi-user.target
"""

    # Temporary file
    temp_service_file = "/tmp/filepi.service"

    # Write content to temp file
    with open(temp_service_file, "w") as file:
        file.write(service_content)

    # Check if the user has sufficient privileges
    if os.geteuid() != 0:
        print("This script needs to be run with sudo or as root")
        sys.exit(1)

    # Copy to systemd directory
    dest = "/etc/systemd/system/filepi.service"
    try:
        shutil.copy(temp_service_file, dest)
        os.chmod(dest, 0o644)

        # Clean up temp file
        os.remove(temp_service_file)

        # Reload systemd and enable service
        print("Reloading systemd daemon...")
        subprocess.run(["systemctl", "daemon-reload"], check=True)

        print("Enabling filepi service...")
        subprocess.run(["systemctl", "enable", "filepi.service"], check=True)

        print("\nService installed successfully.")
        print(f"Data directory set to: {data_dir}")
        print(f"Using filepi executable: {filepi_path}")
        print("\nTo start the service, run: sudo systemctl start filepi.service")
        print("To check service status: sudo systemctl status filepi.service")

    except Exception as e:
        print(f"Error installing service: {e}")
        sys.exit(1)


if __name__ == "__main__":
    install_service()
