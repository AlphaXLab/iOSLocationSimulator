#!/bin/bash

# LocationSimulator Launch Script

# Change to project root
SCRIPT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$SCRIPT_DIR"

# Activate virtual environment if it exists
if [[ -d "venv" ]]; then
    source venv/bin/activate
fi

# Check if dependencies are installed
if ! python3 -c "import pymobiledevice3, PyQt6" 2>/dev/null; then
    echo "❌ Dependencies not installed. Run ./scripts/install.sh first."
    exit 1
fi

# Run the application
echo "🚀 Starting LocationSimulator..."
python3 main.py
