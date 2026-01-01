"""Core modules for LocationSimulator"""

from .device_manager import DeviceManager, iOSDevice, ConnectionType
from .location_controller import LocationController, Coordinate, RouteSimulationState
from .saved_locations import SavedLocationsManager, SavedLocation, GPXRoute, GPXWaypoint
from .tunnel_manager import TunnelManager
from .settings_manager import SettingsManager

__all__ = [
    'DeviceManager',
    'iOSDevice',
    'ConnectionType',
    'LocationController',
    'Coordinate',
    'RouteSimulationState',
    'SavedLocationsManager',
    'SavedLocation',
    'GPXRoute',
    'GPXWaypoint',
    'TunnelManager',
    'SettingsManager',
]
