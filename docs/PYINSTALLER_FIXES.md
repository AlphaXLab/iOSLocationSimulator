# PyInstaller Compatibility Fixes

This document explains the fixes needed to make LocationSimulator work when built with PyInstaller.

## The Problem

When running Python code normally, `sys.executable` points to the Python interpreter (e.g., `/usr/bin/python3`). However, when PyInstaller creates a bundled application:

- `sys.executable` points to the **application bundle itself** (e.g., `LocationSimulator.app`)
- Calling `subprocess.Popen([sys.executable, '-m', 'pymobiledevice3'])` launches a **new instance of the app** instead of Python!

This caused the app to:
- Open duplicate windows
- Fail to execute Python modules
- Create an infinite loop of app launches

## The Solution

We detect if we're running in a PyInstaller bundle and use the system Python instead:

```python
def _get_python_executable():
    """
    Get the correct Python executable.
    When running from PyInstaller bundle, sys.executable points to the app bundle.
    """
    if getattr(sys, 'frozen', False) and hasattr(sys, '_MEIPASS'):
        # Running in PyInstaller bundle - use system python3
        python_exe = shutil.which('python3')
        if not python_exe:
            raise RuntimeError("Python 3 not found in PATH")
        return python_exe
    else:
        # Running from source
        return sys.executable
```

### How It Works

1. **Check if frozen**: `getattr(sys, 'frozen', False)` is `True` when running in PyInstaller
2. **Check for MEIPASS**: `hasattr(sys, '_MEIPASS')` confirms it's a PyInstaller bundle
3. **Find system Python**: Use `shutil.which('python3')` to find Python in PATH
4. **Fallback**: If not frozen, use `sys.executable` as normal

## Files Modified

### 1. `core/location_controller.py`

**Issue**: Used `sys.executable` to run `pymobiledevice3` commands

**Fixed**: Two methods updated to use `_get_python_executable()`:
- `_run_pymobiledevice3_command()` - For iOS 16 location commands
- `_start_ios17_simulation()` - For iOS 17+ location simulation

**Lines**: Added helper function at top, updated subprocess calls

### 2. `core/tunnel_manager.py`

**Issue**: Used `sys.executable` to start RemoteXPC tunnel

**Fixed**: Two methods updated to use `_get_python_executable()`:
- `start_tunnel()` - Start tunnel without password
- `_start_tunnel_with_password()` - Start tunnel with sudo password

**Lines**: Added helper function at top, updated subprocess calls

## Testing

### Before Fix
```bash
# Built app
open dist/LocationSimulator.app

# Click "Set Location"
# Result: Opens another instance of LocationSimulator.app ❌
```

### After Fix
```bash
# Built app
open dist/LocationSimulator.app

# Click "Set Location"
# Result: Location is set on device ✅
```

## Requirements for Built App

The built application now requires:

1. **Python 3** must be installed on the target system
2. **pymobiledevice3** must be installed in system Python:
   ```bash
   pip3 install pymobiledevice3
   ```

This is because:
- PyInstaller bundles Python modules **for the app itself**
- But we can't bundle Python to run subprocess commands
- So we use system Python for subprocess calls to `pymobiledevice3`

## Alternative Approaches Considered

### 1. Use Direct API Calls (Not subprocess)
**Pros**: No need for external Python
**Cons**:
- pymobiledevice3 CLI wraps complex async code
- Would need to rewrite all location commands
- More maintenance burden

### 2. Bundle Python Executable
**Pros**: Self-contained
**Cons**:
- Significantly increases app size (~50MB+)
- Complex to bundle correctly across platforms
- Security implications

### 3. Current Solution (Use System Python)
**Pros**:
- Simple, works reliably
- Small fix, minimal changes
- Easy to maintain

**Cons**:
- Requires Python 3 on target system
- Requires pymobiledevice3 to be installed

## Distribution Notes

When distributing the built application, users need:

**macOS**:
```bash
# Install Homebrew Python (if not installed)
brew install python3

# Install pymobiledevice3
pip3 install pymobiledevice3
```

**Windows**:
```bash
# Install Python from python.org
# Then install pymobiledevice3
pip install pymobiledevice3
```

Alternatively, include an installer script that:
1. Checks if Python 3 is installed
2. Installs pymobiledevice3 automatically
3. Shows instructions if Python is missing

## Future Improvements

1. **Check Python on Launch**: Show warning if Python 3 or pymobiledevice3 is missing
2. **Include Installer**: Create setup script to install dependencies
3. **Direct API**: Long-term, migrate to direct pymobiledevice3 API calls instead of CLI

## References

- [PyInstaller Runtime Information](https://pyinstaller.org/en/stable/runtime-information.html)
- [sys.frozen documentation](https://docs.python.org/3/library/sys.html#sys.frozen)
- [PyInstaller --onedir mode](https://pyinstaller.org/en/stable/usage.html#cmdoption-D)
