#!/bin/bash

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

# Change to the root directory
PROJECT_DIR="$SCRIPT_DIR/.."

pip freeze | grep -v "file:///" > "$PROJECT_DIR/requirements.txt"