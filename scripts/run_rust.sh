#!/bin/bash
# LocationSimulator Rust UI Launch Script

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$SCRIPT_DIR"

# Activate virtual environment if present
if [[ -d "venv" ]]; then
    source venv/bin/activate
fi

# Check if pymobiledevice3 is installed
if ! python3 -c "import pymobiledevice3" 2>/dev/null; then
    echo "⚠️  pymobiledevice3 is not installed in current Python environment."
    echo "   Running: pip3 install pymobiledevice3"
    pip3 install pymobiledevice3
fi

echo "🚀 Building and running LocationSimulator (Rust UI)..."
cargo run --manifest-path rust_ui/Cargo.toml
