#!/bin/bash

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

# Change to the root directory
PROJECT_DIR="$SCRIPT_DIR/.."

# Format code
echo "Formatting code with Black..."
black $PROJECT_DIR

# Sort imports
echo "Sorting imports with isort..."
isort $PROJECT_DIR

# Lint code
echo "Linting code with Ruff..."
ruff check $PROJECT_DIR