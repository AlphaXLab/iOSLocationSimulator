# Building LocationSimulator as Standalone Application

This guide explains how to build LocationSimulator as a standalone macOS application (.app bundle).

## Prerequisites

1. **macOS** (10.13 or later)
2. **Python 3.8+** installed
3. **Virtual environment** with all dependencies installed
4. **Xcode Command Line Tools** (for some build dependencies)

## Quick Build

The easiest way to build:

```bash
./build.sh
```

This will:
- Install/update PyInstaller
- Clean previous builds
- Build the application bundle
- Output to `dist/LocationSimulator.app`

## Manual Build Steps

If you prefer to build manually:

### 1. Install PyInstaller

```bash
source venv/bin/activate
pip install pyinstaller
```

### 2. Run PyInstaller

```bash
pyinstaller build.spec
```

### 3. Find Your Application

The built application will be in:
```
dist/LocationSimulator.app
```

## Installing the Application

### Option 1: Run from dist folder
```bash
open dist/LocationSimulator.app
```

### Option 2: Copy to Applications folder
```bash
cp -r dist/LocationSimulator.app /Applications/
```

Then open from Launchpad or Applications folder.

## Troubleshooting

### "LocationSimulator.app is damaged" error

macOS Gatekeeper may block the app. To fix:

```bash
# Remove quarantine attribute
xattr -cr dist/LocationSimulator.app

# Or allow the app in System Preferences
# System Preferences > Security & Privacy > General > "Open Anyway"
```

### App crashes on launch

Check the console for errors:
```bash
# Run from terminal to see error messages
./dist/LocationSimulator.app/Contents/MacOS/LocationSimulator
```

### Missing dependencies

If the app fails to find pymobiledevice3 or other dependencies:

1. Make sure all dependencies are installed in the virtual environment
2. Check `build.spec` for missing `hiddenimports`
3. Rebuild the app

### Bundle is too large

The bundle includes Python interpreter and all dependencies. To reduce size:

1. Use UPX compression (already enabled in `build.spec`)
2. Exclude unnecessary packages in `build.spec`
3. Use `--onefile` mode (creates single executable instead of bundle)

## Advanced Configuration

### Adding an Icon

1. Create or download an `.icns` icon file
2. Update `build.spec`:
   ```python
   icon='path/to/icon.icns'
   ```

### Code Signing (for distribution)

To distribute the app without security warnings:

1. Get an Apple Developer certificate
2. Sign the app:
   ```bash
   codesign --deep --force --verify --verbose --sign "Developer ID Application: Your Name" dist/LocationSimulator.app
   ```

3. Notarize with Apple (required for distribution):
   ```bash
   xcrun notarytool submit dist/LocationSimulator.app --apple-id "your@email.com" --team-id "TEAMID"
   ```

### Creating a DMG installer

```bash
# Install create-dmg
brew install create-dmg

# Create DMG
create-dmg \
  --volname "LocationSimulator" \
  --volicon "icon.icns" \
  --window-pos 200 120 \
  --window-size 800 400 \
  --icon-size 100 \
  --icon "LocationSimulator.app" 200 190 \
  --hide-extension "LocationSimulator.app" \
  --app-drop-link 600 185 \
  "LocationSimulator-Installer.dmg" \
  "dist/"
```

## One-File Executable (Alternative)

If you prefer a single executable file instead of an app bundle:

1. Edit `build.spec` and change the `EXE` section:
   ```python
   exe = EXE(
       pyz,
       a.scripts,
       a.binaries,  # Include binaries
       a.zipfiles,
       a.datas,
       [],
       name='LocationSimulator',
       debug=False,
       bootloader_ignore_signals=False,
       strip=False,
       upx=True,
       upx_exclude=[],
       runtime_tmpdir=None,
       console=False,
       disable_windowed_traceback=False,
       target_arch=None,
       codesign_identity=None,
       entitlements_file=None,
   )
   ```

2. Remove the `BUNDLE` section

3. Rebuild:
   ```bash
   pyinstaller build.spec
   ```

The output will be a single executable at `dist/LocationSimulator`

## Build Output

After successful build:

```
dist/
└── LocationSimulator.app/
    └── Contents/
        ├── MacOS/
        │   └── LocationSimulator  (main executable)
        ├── Frameworks/           (Qt frameworks)
        ├── Resources/            (app resources)
        └── Info.plist           (app metadata)
```

## Notes

- The app bundle includes Python runtime and all dependencies (~150-200 MB)
- First launch may be slow as macOS verifies the app
- The app still requires sudo access for RemoteXPC tunnel
- pymobiledevice3 CLI tools will be bundled inside the app

## Support

If you encounter issues:
1. Check the console output for error messages
2. Verify all dependencies are installed
3. Try rebuilding with `--clean` flag
4. Open an issue on GitHub with build logs
