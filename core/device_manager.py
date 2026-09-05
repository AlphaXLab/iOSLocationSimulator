"""
Device Manager - Handles iOS device detection and management using pymobiledevice3
"""

import asyncio
from dataclasses import dataclass
from typing import List, Optional, Callable
from enum import Enum
import threading
import time

from PyQt6.QtCore import QObject, pyqtSignal, QTimer


class ConnectionType(Enum):
    USB = "USB"
    WIFI = "WiFi"
    UNKNOWN = "Unknown"


@dataclass
class iOSDevice:
    """Represents a connected iOS device"""
    udid: str
    name: str
    model: str
    ios_version: str
    connection_type: ConnectionType
    
    @property
    def display_name(self) -> str:
        return f"{self.name} ({self.model})"
    
    @property
    def icon_name(self) -> str:
        if "ipad" in self.model.lower():
            return "ipad"
        return "iphone"


class DeviceManager(QObject):
    """Manages connected iOS devices using pymobiledevice3"""
    
    # Signals
    devices_updated = pyqtSignal(list)  # List[iOSDevice]
    error_occurred = pyqtSignal(str)
    
    def __init__(self):
        super().__init__()
        self._devices: List[iOSDevice] = []
        self._selected_device: Optional[iOSDevice] = None
        self._refresh_timer = QTimer()
        self._refresh_timer.timeout.connect(self.refresh_devices)
        self._is_refreshing = False
        
    @property
    def devices(self) -> List[iOSDevice]:
        return self._devices
    
    @property
    def selected_device(self) -> Optional[iOSDevice]:
        return self._selected_device
    
    @selected_device.setter
    def selected_device(self, device: Optional[iOSDevice]):
        self._selected_device = device
        
    def start_auto_refresh(self, interval_ms: int = 5000):
        """Start automatic device refresh"""
        self._refresh_timer.start(interval_ms)
        self.refresh_devices()
        
    def stop_auto_refresh(self):
        """Stop automatic device refresh"""
        self._refresh_timer.stop()
        
    def refresh_devices(self):
        """Refresh the list of connected devices"""
        if self._is_refreshing:
            return
            
        self._is_refreshing = True
        
        # Run in background thread
        thread = threading.Thread(target=self._fetch_devices_thread)
        thread.daemon = True
        thread.start()
        
    def _fetch_devices_thread(self):
        """Background thread to fetch devices"""
        try:
            devices = self._fetch_devices()
            self._devices = devices
            
            # Auto-select first device if none selected
            if self._selected_device is None and devices:
                self._selected_device = devices[0]
            
            # Verify selected device is still connected
            if self._selected_device:
                connected_udids = [d.udid for d in devices]
                if self._selected_device.udid not in connected_udids:
                    self._selected_device = devices[0] if devices else None
            
            self.devices_updated.emit(devices)
            
        except Exception as e:
            self.error_occurred.emit(str(e))
        finally:
            self._is_refreshing = False
            
    def _fetch_devices(self) -> List[iOSDevice]:
        """Fetch connected devices using pymobiledevice3"""
        try:
            return asyncio.run(self._fetch_devices_async())
        except ImportError:
            self.error_occurred.emit(
                "pymobiledevice3 not installed. Run: pip3 install pymobiledevice3"
            )
            return []
        except Exception as e:
            print(f"Error listing devices: {e}")
            return []

    async def _fetch_devices_async(self) -> List[iOSDevice]:
        """Async fetch of connected devices (pymobiledevice3 11+ APIs are async)"""
        from pymobiledevice3.usbmux import list_devices
        from pymobiledevice3.lockdown import create_using_usbmux

        devices = []

        for device in await list_devices():
            lockdown = None
            try:
                lockdown = await create_using_usbmux(serial=device.serial)
                all_values = lockdown.all_values

                name = all_values.get('DeviceName', 'Unknown')
                model = all_values.get('ProductType', 'iPhone')
                ios_version = all_values.get('ProductVersion', 'Unknown')
                udid = device.serial

                conn_type = ConnectionType.USB
                connection_type = str(getattr(device, 'connection_type', '')).lower()
                if 'network' in connection_type or 'wifi' in connection_type:
                    conn_type = ConnectionType.WIFI

                model = self._format_model_name(model)

                devices.append(iOSDevice(
                    udid=udid,
                    name=name,
                    model=model,
                    ios_version=ios_version,
                    connection_type=conn_type
                ))

            except Exception as e:
                print(f"Error reading device {device.serial}: {e}")
                continue
            finally:
                if lockdown is not None:
                    try:
                        await lockdown.close()
                    except Exception:
                        pass

        return devices
    
    def _format_model_name(self, identifier: str) -> str:
        """Convert device identifier to friendly name"""
        model_map = {
            "iPhone16,1": "iPhone 15 Pro",
            "iPhone16,2": "iPhone 15 Pro Max",
            "iPhone15,4": "iPhone 15",
            "iPhone15,5": "iPhone 15 Plus",
            "iPhone15,2": "iPhone 14 Pro",
            "iPhone15,3": "iPhone 14 Pro Max",
            "iPhone14,7": "iPhone 14",
            "iPhone14,8": "iPhone 14 Plus",
            "iPhone14,2": "iPhone 13 Pro",
            "iPhone14,3": "iPhone 13 Pro Max",
            "iPhone14,5": "iPhone 13",
            "iPhone14,4": "iPhone 13 mini",
            "iPhone13,2": "iPhone 12",
            "iPhone13,1": "iPhone 12 mini",
            "iPhone13,3": "iPhone 12 Pro",
            "iPhone13,4": "iPhone 12 Pro Max",
            "iPhone12,1": "iPhone 11",
            "iPhone12,3": "iPhone 11 Pro",
            "iPhone12,5": "iPhone 11 Pro Max",
        }
        return model_map.get(identifier, identifier)
