"""
Settings Manager - Manages application settings and preferences
"""

from PyQt6.QtCore import QSettings


class SettingsManager:
    """Manages application settings using QSettings"""

    def __init__(self):
        self.settings = QSettings()

    # Map Provider Settings
    def get_map_provider(self) -> str:
        """Get the selected map provider ('osm' or 'google')"""
        return self.settings.value('map/provider', 'osm', type=str)

    def set_map_provider(self, provider: str):
        """Set the selected map provider"""
        self.settings.setValue('map/provider', provider)

    def get_google_api_key(self) -> str:
        """Get the stored Google Maps API key"""
        return self.settings.value('map/google_api_key', '', type=str)

    def set_google_api_key(self, api_key: str):
        """Set the Google Maps API key"""
        self.settings.setValue('map/google_api_key', api_key)

    def clear_google_api_key(self):
        """Clear the stored Google Maps API key"""
        self.settings.remove('map/google_api_key')
