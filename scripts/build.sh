#!/bin/bash
# Build script for LocationSimulator macOS app
# This is a simple wrapper - use build_all.sh for automatic platform detection

set -e  # Exit on error

# Change to project root
cd "$(dirname "$0")/.."

echo "🔨 Building LocationSimulator macOS Application..."
echo ""
echo "ℹ️  TIP: Use ./scripts/build_all.sh for automatic platform detection"
echo ""

# Check if virtual environment is activated
if [ -z "$VIRTUAL_ENV" ]; then
    echo "⚠️  Virtual environment not activated. Activating..."
    source venv/bin/activate
fi

# Install PyInstaller if not already installed
echo "📦 Installing/Updating PyInstaller..."
pip install --upgrade pyinstaller

# Clean previous builds
echo "🧹 Cleaning previous builds..."
rm -rf build dist

# Detect architecture and use appropriate spec file
ARCH=$(uname -m)
if [ "$ARCH" = "arm64" ]; then
    SPEC_FILE="build_configs/macos_arm64.spec"
    echo "🍎 Detected Apple Silicon (ARM64)"
else
    SPEC_FILE="build_configs/macos_intel.spec"
    echo "💻 Detected Intel (x86_64)"
fi

# Build the application
echo "🚀 Building application bundle..."
pyinstaller "$SPEC_FILE"

# Check if build was successful
if [ -d "dist/LocationSimulator.app" ]; then
    echo ""
    echo "✅ Build successful!"
    echo ""
    echo "📱 Application location: dist/LocationSimulator.app"
    echo ""
    echo "You can now:"
    echo "  1. Run the app: open dist/LocationSimulator.app"
    echo "  2. Copy to Applications: cp -r dist/LocationSimulator.app /Applications/"
    echo ""
    echo "⚠️  NOTE: The app requires administrator (sudo) access to start the RemoteXPC tunnel."
    echo "          Make sure you have proper permissions to use iOS device location simulation."
    echo ""
else
    echo ""
    echo "❌ Build failed! Check the output above for errors."
    exit 1
fi
