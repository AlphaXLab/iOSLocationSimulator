#!/bin/bash
# Script to convert PNG icon to ICNS (macOS) and ICO (Windows) formats

set -e

# Change to project root
cd "$(dirname "$0")/.."

echo "🎨 Creating application icons from img/pin.png..."
echo ""

# Check if source image exists
if [ ! -f "img/pin.png" ]; then
    echo "❌ Error: img/pin.png not found!"
    exit 1
fi

# Create icons directory if it doesn't exist
mkdir -p icons

echo "📱 Creating macOS icon (ICNS)..."

# For macOS, we need to create an iconset with multiple sizes
mkdir -p icons/icon.iconset

# Generate all required sizes for macOS
sips -z 16 16     img/pin.png --out icons/icon.iconset/icon_16x16.png
sips -z 32 32     img/pin.png --out icons/icon.iconset/icon_16x16@2x.png
sips -z 32 32     img/pin.png --out icons/icon.iconset/icon_32x32.png
sips -z 64 64     img/pin.png --out icons/icon.iconset/icon_32x32@2x.png
sips -z 128 128   img/pin.png --out icons/icon.iconset/icon_128x128.png
sips -z 256 256   img/pin.png --out icons/icon.iconset/icon_128x128@2x.png
sips -z 256 256   img/pin.png --out icons/icon.iconset/icon_256x256.png
sips -z 512 512   img/pin.png --out icons/icon.iconset/icon_256x256@2x.png
sips -z 512 512   img/pin.png --out icons/icon.iconset/icon_512x512.png
sips -z 1024 1024 img/pin.png --out icons/icon.iconset/icon_512x512@2x.png

# Convert iconset to icns
iconutil -c icns icons/icon.iconset -o icons/icon.icns

# Clean up iconset folder
rm -rf icons/icon.iconset

echo "✅ Created: icons/icon.icns"
echo ""

echo "💻 Creating Windows icon (ICO)..."

# Check if ImageMagick is installed for ICO creation
if command -v convert &> /dev/null; then
    # Use ImageMagick to create multi-resolution ICO
    convert img/pin.png -define icon:auto-resize=256,128,64,48,32,16 icons/icon.ico
    echo "✅ Created: icons/icon.ico"
elif command -v sips &> /dev/null; then
    # Fallback: Use sips to create a simple ICO (macOS only)
    echo "⚠️  ImageMagick not found. Creating basic ICO..."
    sips -s format ico img/pin.png --out icons/icon.ico
    echo "✅ Created: icons/icon.ico (basic)"
else
    echo "⚠️  Cannot create ICO file. Install ImageMagick:"
    echo "    brew install imagemagick"
    echo ""
    echo "Skipping Windows icon creation..."
fi

echo ""
echo "╔════════════════════════════════════════╗"
echo "║   ✅ Icon creation complete!           ║"
echo "╚════════════════════════════════════════╝"
echo ""
echo "Created icons:"
echo "  • macOS: icons/icon.icns"
if [ -f "icons/icon.ico" ]; then
    echo "  • Windows: icons/icon.ico"
fi
echo ""
echo "Icons will be automatically used when building."
echo ""
