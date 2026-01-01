# LocationSimulator (Python)

A macOS application to simulate iPhone location using **pymobiledevice3**. This works reliably with Xcode 16+ where Apple removed location simulation from `devicectl`.

![Python](https://img.shields.io/badge/Python-3.10+-blue) ![PyQt6](https://img.shields.io/badge/PyQt6-6.5+-green) ![License](https://img.shields.io/badge/license-MIT-green)

## Features

- 🗺️ **Interactive Map** - Click anywhere to select location (OpenStreetMap)
- 📱 **Device Detection** - Auto-detect connected iOS devices via USB
- 📍 **Saved Locations** - Bookmark frequently used locations
- 🛣️ **GPX Routes** - Import and simulate movement along routes
- ⏯️ **Route Playback** - Play, pause, and control simulation speed (1x-10x)
- 🔍 **Location Search** - Search by address or enter coordinates directly
- ⚡ **Reliable** - Uses pymobiledevice3 (works with Xcode 16+)

## Requirements

- **macOS** 12.0 or later
- **Python** 3.10+
- **iOS device** with Developer Mode enabled (iOS 16+)
- Device must be **trusted/paired** with your Mac

## Installation

### Option 1: Standalone Application (Recommended!)

Build as a standalone application for your platform:

```bash
# Clone or download the project
cd LocationSimulatorPy

# Install dependencies
./scripts/install.sh

# Build for your platform (auto-detects architecture)
./scripts/build_all.sh

# Run the app
# macOS:
open dist/LocationSimulator-macOS-*.app

# Windows:
dist\LocationSimulator-Windows-*\LocationSimulator.exe
```

**Supported Platforms**:
- macOS Intel (x86_64)
- macOS Apple Silicon (ARM64)
- Windows 64-bit (x64)
- Windows 32-bit (x86)
- Windows ARM64 (Surface Pro X, Copilot+ PCs)

See [docs/BUILDING.md](docs/BUILDING.md) for detailed build options.

### Option 2: Run from Source

```bash
# Clone or download the project
cd LocationSimulatorPy

# Run the install script
./scripts/install.sh

# Launch the app
./scripts/run.sh
```

### Manual Install

```bash
# Create virtual environment (recommended)
python3 -m venv venv
source venv/bin/activate

# Install dependencies
pip install -r requirements.txt

# Run the application
python main.py
```

### Dependencies

```bash
pip install pymobiledevice3 PyQt6 PyQt6-WebEngine
```

## Device Setup

### 1. Enable Developer Mode (iOS 16+)

1. On iPhone: **Settings → Privacy & Security → Developer Mode**
2. Toggle **Developer Mode** ON
3. Restart when prompted
4. Confirm after restart

### 2. Pair Device with Mac

1. Connect iPhone via **USB cable**
2. Tap **Trust** on iPhone when prompted
3. Enter passcode if asked
4. Verify pairing:
   ```bash
   python3 -m pymobiledevice3 usbmux list
   ```

## Usage

### Launch the App

```bash
python main.py
```

### Set Location

1. Your device appears in the **Devices** panel
2. Click on the map or search for a location
3. Click **Set Location** (or press ⌘+Enter)

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| ⌘R | Refresh devices |
| ⌘K | Clear simulated location |
| ⌘Enter | Set current location |
| ⌘⇧S | Save current location |
| ⌘I | Import GPX file |

### GPX Route Simulation

1. Click **Routes** tab → **Import GPX**
2. Select a `.gpx` file
3. Double-click route to start simulation
4. Use playback controls to pause/stop/change speed

## Command Line Alternative

You can also use pymobiledevice3 directly:

```bash
# Set location
python3 -m pymobiledevice3 developer dvt simulate-location set -- 16.577913 94.903078

# Clear location
python3 -m pymobiledevice3 developer dvt simulate-location clear

# List devices
python3 -m pymobiledevice3 usbmux list
```

## iOS 17+ Special Requirements

**Good News:** The GUI will automatically start the RemoteXPC tunnel when it detects an iOS 17+ device!

### Automatic Tunnel (Seamless!)

1. Simply launch the app:
   ```bash
   ./run.sh
   ```

2. If you have an iOS 17+ device connected, the app will detect it
3. A password dialog will appear asking for your macOS password
4. Enter your password → Tunnel starts in background → Ready to use! ✨

**No Terminal windows, no manual steps!** The tunnel runs completely in the background.

### Manual Tunnel (Alternative)

If you prefer to manage the tunnel yourself:

1. Open Terminal and run the helper script:
   ```bash
   ./start_tunnel.sh
   ```

2. Keep that terminal window running

3. Launch Location Simulator:
   ```bash
   ./run.sh
   ```

**Note:** The RemoteXPC tunnel is a pymobiledevice3 requirement for iOS 17+ Developer services. It creates a network interface for communication with your device. The app automatically manages it in the background!

## Troubleshooting

### "RemoteXPC Tunnel Not Running" Error (iOS 17+)

If you see "RemoteXPC Tunnel Not Running" when trying to set location on iOS 17+:

1. **Start RemoteXPC tunnel**: Run `./start_tunnel.sh` in a separate terminal
2. **Keep tunnel running**: Don't close the tunnel terminal while using the app
3. **Try again**: Once tunnel is running, the GUI will automatically detect and use it
4. **Verify tunnel**: You can check if the tunnel is running by visiting http://127.0.0.1:49151/

### "No devices found"

```bash
# Check if device is detected
python3 -m pymobiledevice3 usbmux list

# If empty, try:
# 1. Reconnect USB cable
# 2. Tap "Trust" on iPhone
# 3. Restart iPhone
```

### "Developer Mode not enabled"

Enable Developer Mode on your iPhone:
1. Settings → Privacy & Security → Developer Mode → ON
2. Restart device

### "Pairing failed"

```bash
# Remove existing pairing and re-pair
python3 -m pymobiledevice3 pair
```

### Location not changing in app

- Force quit and reopen the app on iPhone
- Some apps cache location data
- Restart the target app

### PyQt6 installation issues on Apple Silicon

```bash
# If PyQt6 fails to install, try:
pip install --upgrade pip
pip install PyQt6 --no-cache-dir
```

## Project Structure

```
LocationSimulatorPy/
├── main.py                      # Application entry point
├── requirements.txt             # Python dependencies
├── scripts/                     # Build & utility scripts
│   ├── install.sh              # Installation script
│   ├── run.sh                  # Launch script
│   ├── build.sh                # Simple build script
│   ├── build_all.sh            # Multi-platform build script
│   └── start_tunnel.sh         # iOS 17+ tunnel helper
├── build_configs/               # PyInstaller configurations
│   ├── macos_intel.spec        # macOS Intel build
│   ├── macos_arm64.spec        # macOS Apple Silicon build
│   ├── windows_x64.spec        # Windows 64-bit build
│   └── windows_x86.spec        # Windows 32-bit build
├── docs/                        # Documentation
│   ├── BUILDING.md             # Build instructions
│   └── BUILD_INSTRUCTIONS.md   # Legacy build docs
├── core/                        # Core business logic
│   ├── __init__.py
│   ├── device_manager.py       # iOS device detection
│   ├── location_controller.py  # Location simulation
│   ├── saved_locations.py      # Persistence & GPX parsing
│   ├── tunnel_manager.py       # RemoteXPC tunnel manager
│   └── settings_manager.py     # App settings
└── ui/                          # User interface
    ├── __init__.py
    ├── main_window.py          # Main application window
    ├── map_widget.py           # Interactive map (OSM/Google Maps)
    ├── device_panel.py         # Device list sidebar
    ├── locations_panel.py      # Saved locations & routes
    ├── control_bar.py          # Bottom control bar
    ├── api_key_dialog.py       # Google Maps API key dialog
    └── password_dialog.py      # Sudo password dialog
```

## How It Works

This app uses **pymobiledevice3** to communicate with iOS devices. Unlike `devicectl` which Apple changed in Xcode 16, pymobiledevice3 provides a stable Python API for:

1. **Device Detection** - Lists connected devices via usbmuxd
2. **Location Simulation** - Uses DVT (Developer Tools) services
3. **Developer Services** - Accesses the same APIs Xcode uses

The location simulation works by:
1. Connecting to the device via lockdown service
2. Starting DVT Secure Socket Proxy
3. Sending simulated GPS coordinates

## Credits

- [pymobiledevice3](https://github.com/doronz88/pymobiledevice3) - iOS device communication
- [Leaflet](https://leafletjs.com/) - Interactive maps
- [OpenStreetMap](https://www.openstreetmap.org/) - Map tiles

## License

MIT License - See LICENSE file for details.
