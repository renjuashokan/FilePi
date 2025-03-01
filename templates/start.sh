#!/bin/bash
set -ex

# Install the prerequisite
sudo apt install python3-venv

# Create a virtual environment
python3 -m venv ~/filepi_env

# Activate the virtual environment
source ~/filepi_env/bin/activate

# Now install your package
pip install filepi-*.whl

# You can run your package from the virtual environment
filepi

# Enable as service

sudo filepi-install-service --data-dir /path/to/data