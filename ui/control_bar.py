"""
Control Bar - Bottom bar with coordinates and action buttons
"""

from PyQt6.QtWidgets import (
    QWidget, QHBoxLayout, QLabel, QPushButton, QFrame
)
from PyQt6.QtCore import pyqtSignal
from PyQt6.QtGui import QFont

from core import DeviceManager, LocationController, SavedLocationsManager


class ControlBar(QWidget):
    """Bottom control bar with coordinates and action buttons"""
    
    set_location_clicked = pyqtSignal()
    clear_location_clicked = pyqtSignal()
    save_location_clicked = pyqtSignal()
    
    def __init__(self, device_manager: DeviceManager,
                 location_controller: LocationController,
                 saved_locations: SavedLocationsManager):
        super().__init__()
        
        self.device_manager = device_manager
        self.location_controller = location_controller
        self.saved_locations = saved_locations
        
        self._setup_ui()
        self._setup_connections()
        
    def _setup_ui(self):
        """Setup the UI"""
        self.setStyleSheet("""
            QWidget {
                background-color: white;
                border-top: 1px solid #ddd;
            }
        """)
        
        layout = QHBoxLayout(self)
        layout.setContentsMargins(16, 12, 16, 12)
        layout.setSpacing(16)

        # Note: Coordinates are displayed on the map itself, so we don't need to show them here
        # Just add a stretch to push buttons to the right
        layout.addStretch()
        
        # Action buttons
        self.save_btn = QPushButton("💾 Save")
        self.save_btn.setStyleSheet("""
            QPushButton {
                background-color: #E8F5E9;
                color: #2E7D32;
                border: 1px solid #4CAF50;
                padding: 6px 12px;
                border-radius: 4px;
                font-size: 13px;
                font-weight: 500;
            }
            QPushButton:hover {
                background-color: #C8E6C9;
            }
            QPushButton:disabled {
                background-color: #F5F5F5;
                color: #999;
                border-color: #DDD;
            }
        """)
        self.save_btn.clicked.connect(self.save_location_clicked.emit)
        layout.addWidget(self.save_btn)

        self.clear_btn = QPushButton("✖️ Clear")
        self.clear_btn.setStyleSheet("""
            QPushButton {
                background-color: #FFEBEE;
                color: #C62828;
                border: 1px solid #EF5350;
                padding: 6px 12px;
                border-radius: 4px;
                font-size: 13px;
                font-weight: 500;
            }
            QPushButton:hover {
                background-color: #FFCDD2;
            }
            QPushButton:disabled {
                background-color: #F5F5F5;
                color: #999;
                border-color: #DDD;
            }
        """)
        self.clear_btn.clicked.connect(self.clear_location_clicked.emit)
        layout.addWidget(self.clear_btn)

        self.set_btn = QPushButton("📍 Set Location")
        self.set_btn.setStyleSheet("""
            QPushButton {
                background-color: #1565C0;
                color: white;
                border: none;
                padding: 8px 16px;
                border-radius: 4px;
                font-weight: bold;
                font-size: 14px;
            }
            QPushButton:hover {
                background-color: #0D47A1;
            }
            QPushButton:disabled {
                background-color: #BDBDBD;
                color: #757575;
            }
        """)
        self.set_btn.clicked.connect(self.set_location_clicked.emit)
        layout.addWidget(self.set_btn)
        
        self.setFixedHeight(60)
        
    def _setup_connections(self):
        """Setup signal connections"""
        self.device_manager.devices_updated.connect(self._update_button_state)
        
    def update_coordinates(self, lat: float, lon: float):
        """Update coordinates (no longer displayed, but kept for compatibility)"""
        # Coordinates are now only shown on the map itself
        pass
        
    def _update_button_state(self, devices: list):
        """Update button enabled state based on device selection"""
        has_device = bool(self.device_manager.selected_device)
        self.set_btn.setEnabled(has_device)
        self.clear_btn.setEnabled(has_device)
