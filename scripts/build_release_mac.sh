#!/bin/bash
# ==============================================================================
# Location Simulator - Professional macOS Release Build Script
# Creates a standalone, self-contained LocationSimulator.app and .dmg
# ==============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$SCRIPT_DIR"

echo "=================================================="
echo "🚀 Building Location Simulator Release for macOS"
echo "=================================================="

# 1. Compile Rust native UI in release mode
echo "🦀 Compiling Rust Native UI (Release Profile)..."
cargo build --release --manifest-path rust_ui/Cargo.toml

RUST_BIN="rust_ui/target/release/rust_ui"
if [ ! -f "$RUST_BIN" ]; then
    echo "❌ Error: Rust binary not found at $RUST_BIN"
    exit 1
fi

# 2. Prepare .app bundle directory structure
APP_NAME="LocationSimulator"
DIST_DIR="$SCRIPT_DIR/dist"
APP_BUNDLE="$DIST_DIR/${APP_NAME}.app"
CONTENTS="$APP_BUNDLE/Contents"
MACOS_DIR="$CONTENTS/MacOS"
RESOURCES_DIR="$CONTENTS/Resources"

echo "🧹 Preparing application bundle..."
rm -rf "$APP_BUNDLE"
mkdir -p "$MACOS_DIR"
mkdir -p "$RESOURCES_DIR"

# 3. Copy executable & Python daemon sidecar
echo "📦 Copying binaries and daemon components..."
cp "$RUST_BIN" "$MACOS_DIR/LocationSimulator"
chmod +x "$MACOS_DIR/LocationSimulator"

cp "core_daemon.py" "$MACOS_DIR/"
cp -R "core" "$MACOS_DIR/"

# 4. Copy app icon
if [ -f "icons/icon.icns" ]; then
    cp "icons/icon.icns" "$RESOURCES_DIR/icon.icns"
fi

# 5. Link / Bundle Python Virtualenv
if [ -d "venv" ]; then
    echo "🐍 Linking Python environment..."
    cp -R "venv" "$RESOURCES_DIR/venv"
fi

# 6. Generate Info.plist
echo "📝 Generating Info.plist..."
cat << 'EOF' > "$CONTENTS/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>LocationSimulator</string>
    <key>CFBundleDisplayName</key>
    <string>Location Simulator</string>
    <key>CFBundleIdentifier</key>
    <string>com.alphax.locationsimulator</string>
    <key>CFBundleVersion</key>
    <string>1.0.0</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>CFBundleExecutable</key>
    <string>LocationSimulator</string>
    <key>CFBundleIconFile</key>
    <string>icon.icns</string>
    <key>LSMinimumSystemVersion</key>
    <string>12.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
    <key>NSRequiresAquaSystemAppearance</key>
    <false/>
</dict>
</plist>
EOF

# 7. Write PkgInfo
echo "APPL????" > "$CONTENTS/PkgInfo"

# 8. Ad-hoc codesign bundle so macOS Gatekeeper allows execution
echo "🔏 Ad-hoc code signing bundle..."
codesign --force --deep --sign - "$APP_BUNDLE"

# 9. Create DMG installer
DMG_PATH="$DIST_DIR/${APP_NAME}-macOS-$(uname -m).dmg"
echo "💿 Creating DMG installer at $DMG_PATH..."
rm -f "$DMG_PATH"
hdiutil create -volname "LocationSimulator" -srcfolder "$APP_BUNDLE" -ov -format UDZO "$DMG_PATH" > /dev/null

echo "=================================================="
echo "✅ macOS Release Build Completed Successfully!"
echo "📱 App Bundle: $APP_BUNDLE"
echo "💿 DMG File:   $DMG_PATH"
echo "=================================================="
