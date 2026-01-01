# Building LocationSimulator

Complete guide for building LocationSimulator as standalone executables for different platforms.

## Table of Contents

- [Quick Start](#quick-start)
- [Platform-Specific Builds](#platform-specific-builds)
  - [macOS](#macos)
  - [Windows](#windows)
- [Build Configurations](#build-configurations)
- [Distribution](#distribution)
- [Troubleshooting](#troubleshooting)

## Quick Start

The easiest way to build for your current platform:

```bash
./scripts/build_all.sh
```

This automatically detects your platform and architecture and builds the appropriate version.

## Platform-Specific Builds

### macOS

LocationSimulator supports both Intel and Apple Silicon Macs.

#### macOS Apple Silicon (ARM64)

```bash
# Using automatic detection
./scripts/build_all.sh

# Or manually
pyinstaller build_configs/macos_arm64.spec
```

**Output**: `dist/LocationSimulator-macOS-AppleSilicon.app`

#### macOS Intel (x86_64)

```bash
# Using automatic detection (on Intel Mac)
./scripts/build_all.sh

# Or manually
pyinstaller build_configs/macos_intel.spec
```

**Output**: `dist/LocationSimulator-macOS-Intel.app`

#### macOS Universal Binary

To create a Universal binary that runs on both architectures:

1. Build on Apple Silicon:
   ```bash
   pyinstaller build_configs/macos_arm64.spec
   mv dist/LocationSimulator.app dist/LocationSimulator-arm64.app
   ```

2. Build on Intel:
   ```bash
   pyinstaller build_configs/macos_intel.spec
   mv dist/LocationSimulator.app dist/LocationSimulator-x86_64.app
   ```

3. Create Universal app:
   ```bash
   lipo -create \
     dist/LocationSimulator-arm64.app/Contents/MacOS/LocationSimulator \
     dist/LocationSimulator-x86_64.app/Contents/MacOS/LocationSimulator \
     -output dist/LocationSimulator-Universal.app/Contents/MacOS/LocationSimulator
   ```

### Windows

LocationSimulator can be built for Windows, but **requires iTunes or Apple Mobile Device Support** to be installed.

#### Windows 64-bit (x64)

```bash
# On Windows 64-bit system
pyinstaller build_configs\windows_x64.spec
```

**Output**: `dist\LocationSimulator-Windows-x64\`

#### Windows 32-bit (x86)

```bash
# On Windows 32-bit system or with 32-bit Python
pyinstaller build_configs\windows_x86.spec
```

**Output**: `dist\LocationSimulator-Windows-x86\`

#### Windows ARM64

For Windows on ARM devices (Surface Pro X, Copilot+ PCs, etc.):

```bash
# On Windows ARM64 system with ARM64 Python
pyinstaller build_configs\windows_arm64.spec
```

**Output**: `dist\LocationSimulator-Windows-ARM64\`

**Note about ARM64 Windows**:
- **Native ARM64 build** (recommended): Build on ARM64 Windows with ARM64 Python for best performance
- **x64 emulation**: The x64 build will run on ARM64 Windows through emulation (slower but works)
- Most Windows ARM64 devices can run x64 apps transparently via emulation

#### Windows Requirements

- **Python 3.10+** (32-bit or 64-bit depending on target)
- **PyInstaller** (`pip install pyinstaller`)
- **iTunes** or **Apple Mobile Device Support** (for device communication)

**Note**: pymobiledevice3 on Windows requires Apple's USB drivers, which are installed with iTunes.

## Build Configurations

All build configurations are in the `build_configs/` directory:

| File | Platform | Architecture | Output |
|------|----------|--------------|--------|
| `macos_intel.spec` | macOS | Intel (x86_64) | .app bundle |
| `macos_arm64.spec` | macOS | Apple Silicon (ARM64) | .app bundle |
| `windows_x64.spec` | Windows | 64-bit (x86_64) | .exe folder |
| `windows_x86.spec` | Windows | 32-bit (x86) | .exe folder |
| `windows_arm64.spec` | Windows | ARM64 | .exe folder |

### Customizing Builds

Edit the `.spec` files to customize:

- **Icon**: Change `icon=None` to `icon='path/to/icon.icns'` (macOS) or `icon='path/to/icon.ico'` (Windows)
- **App Name**: Modify `name='LocationSimulator'`
- **Version**: Update `CFBundleVersion` and `CFBundleShortVersionString` (macOS)
- **Hidden Imports**: Add missing modules to `hiddenimports` list
- **Bundle Size**: Use `upx=True` to compress binaries (enabled by default)

## Distribution

### macOS Distribution

#### Code Signing

For distribution outside the App Store, you need to sign the app:

```bash
# Sign the app
codesign --deep --force --verify --verbose \
  --sign "Developer ID Application: Your Name (TEAMID)" \
  --options runtime \
  dist/LocationSimulator.app

# Verify signature
codesign --verify --deep --strict --verbose=2 \
  dist/LocationSimulator.app
```

#### Notarization

Required for macOS 10.15+ to avoid security warnings:

```bash
# Create a ZIP for notarization
ditto -c -k --keepParent \
  dist/LocationSimulator.app \
  LocationSimulator.zip

# Submit for notarization
xcrun notarytool submit LocationSimulator.zip \
  --apple-id "your@email.com" \
  --team-id "TEAMID" \
  --password "app-specific-password" \
  --wait

# Staple the ticket
xcrun stapler staple dist/LocationSimulator.app
```

#### Creating DMG Installer

```bash
# Install create-dmg
brew install create-dmg

# Create DMG
create-dmg \
  --volname "LocationSimulator" \
  --window-pos 200 120 \
  --window-size 800 400 \
  --icon-size 100 \
  --icon "LocationSimulator.app" 200 190 \
  --hide-extension "LocationSimulator.app" \
  --app-drop-link 600 185 \
  "LocationSimulator-Installer.dmg" \
  "dist/"
```

### Windows Distribution

#### Creating Installer

Use **Inno Setup** to create a Windows installer:

1. Install Inno Setup: https://jrsoftware.org/isinfo.php

2. Create `installer.iss`:
   ```inno
   [Setup]
   AppName=LocationSimulator
   AppVersion=1.0
   DefaultDirName={pf}\LocationSimulator
   DefaultGroupName=LocationSimulator
   OutputDir=dist
   OutputBaseFilename=LocationSimulator-Setup

   [Files]
   Source: "dist\LocationSimulator-Windows-x64\*"; DestDir: "{app}"; Flags: recursesubdirs

   [Icons]
   Name: "{group}\LocationSimulator"; Filename: "{app}\LocationSimulator.exe"
   Name: "{commondesktop}\LocationSimulator"; Filename: "{app}\LocationSimulator.exe"
   ```

3. Compile: Run Inno Setup and compile the `.iss` file

## Troubleshooting

### Build Issues

#### "No module named 'PyQt6'"

Install dependencies first:
```bash
./scripts/install.sh
```

#### "Unable to find Qt platform plugin"

Qt WebEngine is missing. Reinstall:
```bash
pip install --upgrade --force-reinstall PyQt6-WebEngine
```

#### Build succeeds but app crashes

Check hidden imports in the `.spec` file. Add missing modules to `hiddenimports` list.

### macOS Issues

#### "App is damaged and can't be opened"

Remove quarantine attribute:
```bash
xattr -cr dist/LocationSimulator.app
```

Or allow in System Preferences:
1. System Preferences → Security & Privacy
2. Click "Open Anyway"

#### App crashes on launch

Run from terminal to see error:
```bash
./dist/LocationSimulator.app/Contents/MacOS/LocationSimulator
```

### Windows Issues

#### "iTunes not found"

Install iTunes or Apple Mobile Device Support:
- Full iTunes: https://www.apple.com/itunes/download/
- Or extract Apple Mobile Device Support from iTunes installer

#### Missing DLL errors

Run dependency walker or use `dumpbin` to find missing DLLs:
```cmd
dumpbin /dependents LocationSimulator.exe
```

### Size Optimization

The app bundle can be large (~150-200MB). To reduce:

1. **Enable UPX compression** (already enabled in `.spec` files)

2. **Exclude unnecessary modules** in `.spec`:
   ```python
   excludes=['tkinter', 'matplotlib', 'numpy', 'pandas']
   ```

3. **Use one-file mode** (slower startup but smaller distribution):
   ```python
   exe = EXE(
       ...
       a.binaries,  # Include binaries
       a.zipfiles,
       a.datas,
       ...
   )
   ```

## Build Environment

### Recommended Setup

**macOS**:
- macOS 10.15+ (for building)
- Xcode Command Line Tools
- Python 3.10+
- Virtual environment

**Windows**:
- Windows 10/11
- Python 3.10+ (32-bit or 64-bit)
- Visual C++ Redistributable
- iTunes or Apple Mobile Device Support

### Dependencies

All platforms:
```bash
pip install pyinstaller pymobiledevice3 PyQt6 PyQt6-WebEngine
```

## CI/CD

### GitHub Actions Example

See `.github/workflows/build.yml` for automated builds on multiple platforms.

### Building on CI

```yaml
- name: Build macOS app
  run: |
    pip install pyinstaller
    ./scripts/build_all.sh

- name: Upload artifact
  uses: actions/upload-artifact@v3
  with:
    name: LocationSimulator-macOS
    path: dist/*.app
```

## Support

For build issues:
1. Check this documentation
2. Review PyInstaller logs in `build/`
3. Test with `python main.py` first to ensure code works
4. Open an issue on GitHub with build logs
