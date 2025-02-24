#!/bin/bash
set -ex

export FILE_PI_ROOT_DIR="/media/renju/Seagate Backup Plus Drive"

# Get current directory
DIR="$(realpath "${0%/*}")"

# Get Parent dir
PARENTDIR="$(dirname "$DIR")"
cd $PARENTDIR
# Create virtual environment
python3 -m venv venv

# Activate virtual environment
source venv/bin/activate
pip install -r requirements.txt


python3 -m src.main

