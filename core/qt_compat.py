"""
Qt Compatibility Layer
Allows core modules to run with standard Python when PyQt6 is not installed or in daemon mode.
"""
import os
import sys
import json
import threading
import time
from pathlib import Path

try:
    if os.environ.get("LOCATION_SIM_NO_QT") == "1":
        raise ImportError("Forced pure-python daemon mode (no Qt event loop)")
    from PyQt6.QtCore import QObject, pyqtSignal, QTimer, QStandardPaths
    from PyQt6.QtCore import QSettings
    HAS_PYQT6 = True
except ImportError:
    HAS_PYQT6 = False

    class QObject:
        def __init__(self, *args, **kwargs):
            pass

    class SignalInstance:
        def __init__(self):
            self._callbacks = []

        def connect(self, callback):
            if callback not in self._callbacks:
                self._callbacks.append(callback)

        def disconnect(self, callback=None):
            if callback is None:
                self._callbacks.clear()
            elif callback in self._callbacks:
                self._callbacks.remove(callback)

        def emit(self, *args, **kwargs):
            for cb in list(self._callbacks):
                try:
                    cb(*args, **kwargs)
                except Exception as e:
                    print(f"Error in signal handler: {e}", file=sys.stderr)

    class pyqtSignal:
        """Descriptor that provides a per-instance SignalInstance"""
        def __init__(self, *types):
            self.types = types

        def __set_name__(self, owner, name):
            self.name = f"__signal_{name}"

        def __get__(self, instance, owner=None):
            if instance is None:
                return self
            if not hasattr(instance, self.name):
                setattr(instance, self.name, SignalInstance())
            return getattr(instance, self.name)

    class QTimer:
        def __init__(self):
            self.timeout = SignalInstance()
            self._running = False
            self._thread = None
            self._interval = 1.0

        def start(self, interval_ms: int = 1000):
            self.stop()
            self._interval = interval_ms / 1000.0
            self._running = True
            self._thread = threading.Thread(target=self._run, daemon=True)
            self._thread.start()

        def stop(self):
            self._running = False

        def _run(self):
            while self._running:
                time.sleep(self._interval)
                if self._running:
                    self.timeout.emit()

    class QStandardPaths:
        class StandardLocation:
            AppDataLocation = 1

        @staticmethod
        def writableLocation(loc):
            if sys.platform == 'darwin':
                return str(Path.home() / 'Library' / 'Application Support')
            elif sys.platform == 'win32':
                return os.environ.get('APPDATA', str(Path.home() / 'AppData' / 'Roaming'))
            else:
                return str(Path.home() / '.local' / 'share')

    class QSettings:
        def __init__(self):
            self._file = Path(QStandardPaths.writableLocation(1)) / "LocationSimulator" / "settings.json"
            self._file.parent.mkdir(parents=True, exist_ok=True)
            self._data = {}
            if self._file.exists():
                try:
                    with open(self._file, 'r') as f:
                        self._data = json.load(f)
                except Exception:
                    self._data = {}

        def value(self, key, default=None, type=None):
            val = self._data.get(key, default)
            if type is not None and val is not None:
                try:
                    return type(val)
                except (ValueError, TypeError):
                    return default
            return val

        def setValue(self, key, value):
            self._data[key] = value
            self._save()

        def remove(self, key):
            if key in self._data:
                del self._data[key]
                self._save()

        def _save(self):
            try:
                with open(self._file, 'w') as f:
                    json.dump(self._data, f, indent=2)
            except Exception as e:
                print(f"Error saving settings: {e}", file=sys.stderr)
