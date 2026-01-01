"""
Device Panel - Sidebar panel showing connected iOS devices
"""

from PyQt6.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QLabel, QPushButton,
    QListWidget, QListWidgetItem, QFrame
)
from PyQt6.QtCore import Qt, pyqtSlot
from PyQt6.QtGui import QFont

from core import DeviceManager, iOSDevice


class DevicePanel(QWidget):
    """Panel showing connected iOS devices"""
    
    def __init__(self, device_manager: DeviceManager):
        super().__init__()
        self.device_manager = device_manager
        
        self._setup_ui()
        self._setup_connections()
        
    def _setup_ui(self):
        """Setup the UI"""
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(0)
        
        # Header
        header = QWidget()
        header.setStyleSheet("""
            background-color: white;
            border-bottom: 2px solid #E3F2FD;
        """)
        header_layout = QHBoxLayout(header)
        header_layout.setContentsMargins(16, 12, 16, 12)

        title = QLabel("Devices")
        title.setFont(QFont("", 15, QFont.Weight.Bold))
        title.setStyleSheet("color: #1565C0;")
        header_layout.addWidget(title)

        header_layout.addStretch()

        self.refresh_btn = QPushButton("⟳")
        self.refresh_btn.setFixedSize(28, 28)
        self.refresh_btn.setStyleSheet("""
            QPushButton {
                border: none;
                font-size: 18px;
                color: #1565C0;
                background-color: transparent;
                border-radius: 14px;
            }
            QPushButton:hover {
                background-color: #E3F2FD;
            }
            QPushButton:pressed {
                background-color: #BBDEFB;
            }
        """)
        self.refresh_btn.setToolTip("Refresh devices")
        self.refresh_btn.clicked.connect(self.device_manager.refresh_devices)
        header_layout.addWidget(self.refresh_btn)

        layout.addWidget(header)
        
        # Device list
        self.device_list = QListWidget()
        self.device_list.setStyleSheet("""
            QListWidget {
                border: none;
                background: #FAFAFA;
                color: #000;
                outline: none;
            }
            QListWidget::item {
                padding: 4px;
                border: none;
                background: transparent;
            }
            QListWidget::item:selected {
                background: transparent;
            }
            QListWidget::item:hover {
                background: transparent;
            }
        """)
        self.device_list.setSpacing(8)
        self.device_list.itemClicked.connect(self._on_device_clicked)
        layout.addWidget(self.device_list)
        
        # Empty state
        self.empty_label = QLabel("No devices connected\n\nConnect an iOS device\nwith Developer Mode enabled")
        self.empty_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.empty_label.setStyleSheet("""
            color: #9E9E9E;
            padding: 24px;
            font-size: 13px;
            line-height: 1.6;
            background: transparent;
        """)
        layout.addWidget(self.empty_label)
        
        self.setMinimumHeight(180)
        self.setMaximumHeight(220)
        
    def _setup_connections(self):
        """Setup signal connections"""
        self.device_manager.devices_updated.connect(self._on_devices_updated)
        
    @pyqtSlot(list)
    def _on_devices_updated(self, devices: list):
        """Handle devices list update"""
        self.device_list.clear()

        if devices:
            self.device_list.show()
            self.empty_label.hide()

            for device in devices:
                item = QListWidgetItem()
                widget = DeviceItemWidget(device)
                item.setSizeHint(widget.sizeHint())
                item.setData(Qt.ItemDataRole.UserRole, device)
                self.device_list.addItem(item)
                self.device_list.setItemWidget(item, widget)

                # Select if this is the selected device
                is_selected = (self.device_manager.selected_device and
                              device.udid == self.device_manager.selected_device.udid)
                if is_selected:
                    self.device_list.setCurrentItem(item)
                    widget.setSelected(True)
                else:
                    widget.setSelected(False)
        else:
            self.device_list.hide()
            self.empty_label.show()
            
    def _on_device_clicked(self, item: QListWidgetItem):
        """Handle device item clicked"""
        device = item.data(Qt.ItemDataRole.UserRole)
        if device:
            self.device_manager.selected_device = device

            # Update selection state for all device widgets
            for i in range(self.device_list.count()):
                list_item = self.device_list.item(i)
                widget = self.device_list.itemWidget(list_item)
                if widget:
                    item_device = list_item.data(Qt.ItemDataRole.UserRole)
                    widget.setSelected(item_device.udid == device.udid)


class DeviceItemWidget(QWidget):
    """Widget representing a device in the list"""

    def __init__(self, device: iOSDevice):
        super().__init__()
        self.device = device
        self.is_selected = False

        # Main container with card styling
        self.setStyleSheet("""
            DeviceItemWidget {
                background-color: white;
                border-radius: 8px;
                border: 1px solid #E0E0E0;
            }
        """)

        layout = QHBoxLayout(self)
        layout.setContentsMargins(16, 14, 16, 14)
        layout.setSpacing(14)

        # Device icon with background circle
        icon_container = QWidget()
        icon_container.setFixedSize(44, 44)
        icon_container.setStyleSheet("""
            background-color: #E3F2FD;
            border-radius: 22px;
        """)
        icon_layout = QVBoxLayout(icon_container)
        icon_layout.setContentsMargins(0, 0, 0, 0)
        icon_layout.setAlignment(Qt.AlignmentFlag.AlignCenter)

        icon_label = QLabel("📱")
        icon_label.setFont(QFont("", 22))
        icon_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        icon_layout.addWidget(icon_label)

        layout.addWidget(icon_container)

        # Device info (3 lines)
        info_layout = QVBoxLayout()
        info_layout.setSpacing(3)
        info_layout.setContentsMargins(0, 0, 0, 0)

        # Line 1: Device name
        name_label = QLabel(device.name)
        name_label.setFont(QFont("", 14, QFont.Weight.DemiBold))
        name_label.setStyleSheet("color: #212121; background: transparent;")
        name_label.setWordWrap(False)
        info_layout.addWidget(name_label)

        # Line 2: Model and iOS version
        model_details = f"{device.model} • iOS {device.ios_version}"
        model_label = QLabel(model_details)
        model_label.setStyleSheet("color: #757575; font-size: 12px; background: transparent;")
        model_label.setWordWrap(False)
        info_layout.addWidget(model_label)

        # Line 3: Connection type with icon
        connection_text = "USB" if device.connection_type.value == "USB" else "Wi-Fi"
        conn_icon = "🔌" if device.connection_type.value == "USB" else "📶"
        connection_line = f"{conn_icon} Connected via {connection_text}"
        conn_label = QLabel(connection_line)
        conn_label.setStyleSheet("color: #666666; font-size: 11px; background: transparent;")
        conn_label.setWordWrap(False)
        info_layout.addWidget(conn_label)

        layout.addLayout(info_layout, 1)

        # Set minimum height to fit 3 lines
        self.setMinimumHeight(90)

    def setSelected(self, selected: bool):
        """Update styling when selection changes"""
        self.is_selected = selected
        if selected:
            self.setStyleSheet("""
                DeviceItemWidget {
                    background-color: #E3F2FD;
                    border-radius: 8px;
                    border: 2px solid #1565C0;
                }
            """)
        else:
            self.setStyleSheet("""
                DeviceItemWidget {
                    background-color: white;
                    border-radius: 8px;
                    border: 1px solid #E0E0E0;
                }
            """)

    def enterEvent(self, event):
        """Handle mouse enter"""
        if not self.is_selected:
            self.setStyleSheet("""
                DeviceItemWidget {
                    background-color: #F5F5F5;
                    border-radius: 8px;
                    border: 1px solid #BDBDBD;
                }
            """)
        super().enterEvent(event)

    def leaveEvent(self, event):
        """Handle mouse leave"""
        if not self.is_selected:
            self.setStyleSheet("""
                DeviceItemWidget {
                    background-color: white;
                    border-radius: 8px;
                    border: 1px solid #E0E0E0;
                }
            """)
        super().leaveEvent(event)
