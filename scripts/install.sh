#!/bin/bash

# LocationSimulator Installation Script

set -e

# Change to project root
cd "$(dirname "$0")/.."

echo "🚀 LocationSimulator Installer"
echo "=============================="
echo ""

# Check Python version
PYTHON_CMD=""
if command -v python3 &> /dev/null; then
    PYTHON_VERSION=$(python3 -c 'import sys; print(f"{sys.version_info.major}.{sys.version_info.minor}")')
    echo "✅ Python $PYTHON_VERSION found"
    PYTHON_CMD="python3"
else
    echo "❌ Python 3 not found. Please install Python 3.10 or later."
    exit 1
fi

# Check if we're in a virtual environment
if [[ -z "$VIRTUAL_ENV" ]]; then
    echo ""
    echo "📦 Creating virtual environment..."
    $PYTHON_CMD -m venv venv
    source venv/bin/activate
    echo "✅ Virtual environment created and activated"
else
    echo "✅ Already in virtual environment: $VIRTUAL_ENV"
fi

# Upgrade pip
echo ""
echo "📦 Upgrading pip..."
pip install --upgrade pip --quiet

# Install dependencies
echo ""
echo "📦 Installing dependencies..."
echo "   - pymobiledevice3 (iOS device communication)"
echo "   - PyQt6 (GUI framework)"
echo "   - PyQt6-WebEngine (Map display)"
echo ""

pip install pymobiledevice3 PyQt6 PyQt6-WebEngine --quiet

echo ""
echo "✅ Installation complete!"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "To run the application:"
echo ""
echo "  ./scripts/run.sh"
echo ""
echo "Or manually:"
echo ""
echo "  source venv/bin/activate"
echo "  python main.py"
echo ""
echo "To build standalone application:"
echo ""
echo "  ./scripts/build_all.sh"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📱 Before using, make sure to:"
echo "   1. Enable Developer Mode on your iPhone"
echo "   2. Connect via USB and tap 'Trust' on the device"
echo ""
