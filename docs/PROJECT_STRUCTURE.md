# Project Structure

This document explains the organization of the LocationSimulator project.

## Directory Layout

```
LocationSimulatorPy/
├── main.py                      # Entry point - starts the Qt application
├── requirements.txt             # Python package dependencies
│
├── scripts/                     # Executable scripts
│   ├── install.sh              # Install dependencies & setup venv
│   ├── run.sh                  # Run app from source
│   ├── build.sh                # Simple build (auto-detects arch)
│   ├── build_all.sh            # Advanced multi-platform builder
│   └── start_tunnel.sh         # Manual RemoteXPC tunnel starter
│
├── build_configs/               # PyInstaller build configurations
│   ├── macos_intel.spec        # macOS x86_64 build config
│   ├── macos_arm64.spec        # macOS ARM64 build config
│   ├── windows_x64.spec        # Windows 64-bit build config
│   └── windows_x86.spec        # Windows 32-bit build config
│
├── docs/                        # Documentation
│   ├── BUILDING.md             # Complete build guide
│   ├── BUILD_INSTRUCTIONS.md   # Legacy build docs
│   └── PROJECT_STRUCTURE.md    # This file
│
├── core/                        # Business logic (no UI)
│   ├── device_manager.py       # Detect & manage iOS devices
│   ├── location_controller.py  # Set/clear device location
│   ├── saved_locations.py      # Manage bookmarks & GPX routes
│   ├── tunnel_manager.py       # iOS 17+ RemoteXPC tunnel
│   └── settings_manager.py     # Load/save app preferences
│
└── ui/                          # User interface (PyQt6)
    ├── main_window.py          # Main window + layout
    ├── map_widget.py           # Web-based map (OSM/Google)
    ├── device_panel.py         # Device list sidebar
    ├── locations_panel.py      # Saved locations & GPX routes
    ├── control_bar.py          # Bottom action bar
    ├── api_key_dialog.py       # Google Maps API key prompt
    └── password_dialog.py      # Sudo password prompt
```

## Module Responsibilities

### Core Layer

**device_manager.py**
- Detects connected iOS devices via pymobiledevice3
- Manages device selection
- Emits signals when device list changes

**location_controller.py**
- Simulates GPS locations on devices
- Handles iOS 16 (direct) and iOS 17+ (via tunnel)
- Manages GPX route playback

**saved_locations.py**
- Saves/loads favorite locations
- Parses and manages GPX route files
- Provides persistence layer

**tunnel_manager.py**
- Starts/stops RemoteXPC tunnel for iOS 17+
- Handles sudo password prompts
- Monitors tunnel process health

**settings_manager.py**
- Loads/saves app settings (QSettings)
- Manages Google Maps API key
- Stores map provider preference

### UI Layer

**main_window.py**
- Main application window
- Assembles all UI components
- Handles menu bar and shortcuts
- Coordinates between UI and core

**map_widget.py**
- Embeds web view with Leaflet (OSM) or Google Maps
- JavaScript ↔ Python bridge (QWebChannel)
- Handles map interactions and search

**device_panel.py**
- Shows connected devices
- Device selection interface
- Auto-refresh functionality

**locations_panel.py**
- Saved locations tab
- GPX routes tab with import
- Route playback controls

**control_bar.py**
- Set/Clear/Save location buttons
- Coordinate display (now removed, shown on map)
- Device-dependent button states

**api_key_dialog.py**
- Prompts user for Google Maps API key
- Provides instructions for obtaining key
- Validates and saves key

**password_dialog.py**
- Prompts for macOS password (sudo)
- Used to start RemoteXPC tunnel
- Security notice about password usage

## Data Flow

```
User Interaction (UI)
    ↓
Main Window (coordinates)
    ↓
Core Layer (business logic)
    ↓
pymobiledevice3 (device communication)
    ↓
iOS Device
```

### Example: Setting Location

1. User clicks on map → `map_widget.py`
2. Map JavaScript calls Python via QWebChannel
3. Coordinates update in `main_window.py`
4. User clicks "Set Location" → `control_bar.py`
5. Main window calls `location_controller.set_location()`
6. Controller checks iOS version
7. For iOS 17+: Uses RemoteXPC tunnel
8. For iOS 16: Uses direct DVT connection
9. pymobiledevice3 sends GPS coordinates
10. Device updates location

## Build Artifacts

After building, the `dist/` directory contains:

**macOS**:
```
dist/
└── LocationSimulator-macOS-Intel.app/     # or -AppleSilicon
    └── Contents/
        ├── MacOS/LocationSimulator        # Executable
        ├── Frameworks/                    # Qt frameworks
        ├── Resources/                     # Resources
        └── Info.plist                     # App metadata
```

**Windows**:
```
dist/
└── LocationSimulator-Windows-x64/         # or -x86
    ├── LocationSimulator.exe              # Main executable
    ├── *.dll                              # Dependencies
    └── ... (other runtime files)
```

## Development Workflow

1. **Install dependencies**
   ```bash
   ./scripts/install.sh
   ```

2. **Run from source** (for development)
   ```bash
   ./scripts/run.sh
   # or
   python main.py
   ```

3. **Make changes** to code

4. **Test changes**
   ```bash
   ./scripts/run.sh
   ```

5. **Build for distribution**
   ```bash
   ./scripts/build_all.sh
   ```

6. **Test built app**
   ```bash
   open dist/LocationSimulator-*.app  # macOS
   ```

## Adding New Features

### Adding a new UI component

1. Create file in `ui/` (e.g., `ui/new_widget.py`)
2. Import in `ui/main_window.py`
3. Add to layout in `_setup_ui()`
4. Connect signals/slots in `_setup_connections()`

### Adding new business logic

1. Create file in `core/` (e.g., `core/new_feature.py`)
2. Add to `core/__init__.py` exports
3. Import in relevant UI components
4. Use via signals/slots or direct calls

### Adding new build target

1. Create spec file in `build_configs/`
2. Update `scripts/build_all.sh` detection logic
3. Document in `docs/BUILDING.md`

## Configuration Files

- **requirements.txt**: Python dependencies for pip
- **build_configs/*.spec**: PyInstaller build configurations
- **~/.config/LocationSimulator/**: User settings (QSettings)
- **~/Library/Application Support/LocationSimulator/**: macOS app data
  - `saved_locations.json`: Bookmarked locations
  - `routes/`: GPX route files

## Dependencies

**Runtime**:
- Python 3.10+
- PyQt6 (GUI framework)
- PyQt6-WebEngine (embedded web view)
- pymobiledevice3 (iOS device communication)
- cryptography (required by pymobiledevice3)

**Build-time**:
- PyInstaller (create standalone executables)

**Optional**:
- create-dmg (macOS DMG creation)
- Inno Setup (Windows installer creation)

## Platform Differences

| Feature | macOS | Windows |
|---------|-------|---------|
| Device detection | Native (usbmuxd) | Requires iTunes |
| RemoteXPC tunnel | sudo required | Admin required |
| Build output | .app bundle | .exe folder |
| Code signing | Optional (recommended) | Optional |
| Notarization | Required for distribution | N/A |

## Notes

- The app uses Qt WebEngine for maps (Chromium-based)
- JavaScript console logs are forwarded to Python console
- QWebChannel enables JavaScript ↔ Python communication
- Settings are stored using Qt's QSettings (platform-native)
- GPX routes are stored as separate files in user's data directory
