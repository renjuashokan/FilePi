#!/bin/bash

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

# Change to the root directory
PROJECT_DIR="$SCRIPT_DIR/.."

cd $PROJECT_DIR

# Set the root directory of the project
python setup.py sdist bdist_wheel
