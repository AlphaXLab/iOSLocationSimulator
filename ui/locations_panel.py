"""
Locations Panel - Sidebar panel for saved locations and GPX routes
"""

from PyQt6.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QLabel, QPushButton,
    QListWidget, QListWidgetItem, QTabWidget, QMenu, QFileDialog,
    QMessageBox, QProgressBar
)
from PyQt6.QtCore import Qt, pyqtSignal, pyqtSlot
from PyQt6.QtGui import QFont, QAction

from core import (
    DeviceManager, LocationController, SavedLocationsManager,
    SavedLocation, GPXRoute, Coordinate
)


class LocationsPanel(QWidget):
    """Panel for saved locations and GPX routes"""
    
    location_selected = pyqtSignal(float, float)  # lat, lon
    
    def __init__(self, saved_locations: SavedLocationsManager,
                 device_manager: DeviceManager,
                 location_controller: LocationController):
        super().__init__()
        
        self.saved_locations = saved_locations
        self.device_manager = device_manager
        self.location_controller = location_controller
        
        self._setup_ui()
        self._setup_connections()
        self._refresh_lists()
        
    def _setup_ui(self):
        """Setup the UI"""
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(0)
        
        # Tab widget for locations and routes
        self.tab_widget = QTabWidget()
        self.tab_widget.setStyleSheet("""
            QTabWidget::pane {
                border: none;
            }
            QTabBar::tab {
                padding: 10px 20px;
                font-size: 14px;
                font-weight: 500;
                color: #333;
            }
            QTabBar::tab:selected {
                font-weight: 600;
                color: #1565C0;
            }
        """)
        
        # Saved locations tab
        locations_widget = QWidget()
        locations_layout = QVBoxLayout(locations_widget)
        locations_layout.setContentsMargins(0, 0, 0, 0)
        locations_layout.setSpacing(0)
        
        self.locations_list = QListWidget()
        self.locations_list.setStyleSheet("""
            QListWidget {
                border: none;
                background: white;
                color: #000;
            }
            QListWidget::item {
                padding: 10px 12px;
                border-bottom: 1px solid #eee;
                font-size: 14px;
                color: #000;
            }
            QListWidget::item:selected {
                background-color: #E3F2FD;
                color: #1565C0;
            }
            QListWidget::item:hover:!selected {
                background-color: #f5f5f5;
            }
        """)
        self.locations_list.itemClicked.connect(self._on_location_clicked)
        self.locations_list.itemDoubleClicked.connect(self._on_location_double_clicked)
        self.locations_list.setContextMenuPolicy(Qt.ContextMenuPolicy.CustomContextMenu)
        self.locations_list.customContextMenuRequested.connect(self._show_location_context_menu)
        locations_layout.addWidget(self.locations_list)
        
        self.tab_widget.addTab(locations_widget, f"📍 Locations")
        
        # GPX routes tab
        routes_widget = QWidget()
        routes_layout = QVBoxLayout(routes_widget)
        routes_layout.setContentsMargins(0, 0, 0, 0)
        routes_layout.setSpacing(0)
        
        # Import button
        import_btn = QPushButton("+ Import GPX")
        import_btn.setStyleSheet("""
            QPushButton {
                background-color: #E3F2FD;
                color: #1565C0;
                border: 1px solid #1565C0;
                padding: 8px 12px;
                border-radius: 4px;
                font-size: 13px;
                font-weight: 500;
            }
            QPushButton:hover {
                background-color: #BBDEFB;
            }
        """)
        import_btn.clicked.connect(self._import_gpx)
        routes_layout.addWidget(import_btn)
        
        self.routes_list = QListWidget()
        self.routes_list.setStyleSheet("""
            QListWidget {
                border: none;
                background: white;
                color: #000;
            }
            QListWidget::item {
                padding: 10px 12px;
                border-bottom: 1px solid #eee;
                font-size: 14px;
                color: #000;
            }
            QListWidget::item:selected {
                background-color: #E3F2FD;
                color: #1565C0;
            }
        """)
        self.routes_list.itemDoubleClicked.connect(self._on_route_double_clicked)
        self.routes_list.setContextMenuPolicy(Qt.ContextMenuPolicy.CustomContextMenu)
        self.routes_list.customContextMenuRequested.connect(self._show_route_context_menu)
        routes_layout.addWidget(self.routes_list)
        
        # Route playback controls
        self.route_controls = RouteControlsWidget(self.location_controller)
        self.route_controls.hide()
        routes_layout.addWidget(self.route_controls)
        
        self.tab_widget.addTab(routes_widget, "🛣️ Routes")
        
        layout.addWidget(self.tab_widget)
        
    def _setup_connections(self):
        """Setup signal connections"""
        self.saved_locations.locations_updated.connect(self._refresh_locations)
        self.saved_locations.routes_updated.connect(self._refresh_routes)
        self.location_controller.route_state_changed.connect(self._on_route_state_changed)
        
    def _refresh_lists(self):
        """Refresh both lists"""
        self._refresh_locations()
        self._refresh_routes()
        
    @pyqtSlot()
    def _refresh_locations(self):
        """Refresh locations list"""
        self.locations_list.clear()
        
        for location in self.saved_locations.locations:
            item = QListWidgetItem()
            
            # Format display
            category_icons = {
                "favorite": "⭐",
                "work": "🏢",
                "home": "🏠",
                "testing": "🔧",
                "other": "📍"
            }
            icon = category_icons.get(location.category, "📍")
            
            item.setText(f"{icon} {location.name}\n   {location.latitude:.4f}, {location.longitude:.4f}")
            item.setData(Qt.ItemDataRole.UserRole, location)
            self.locations_list.addItem(item)
            
        # Update tab title
        self.tab_widget.setTabText(0, f"📍 Locations ({len(self.saved_locations.locations)})")
        
    @pyqtSlot()
    def _refresh_routes(self):
        """Refresh routes list"""
        self.routes_list.clear()
        
        for route in self.saved_locations.routes:
            item = QListWidgetItem()
            item.setText(f"🛣️ {route.name}\n   {len(route.waypoints)} waypoints")
            item.setData(Qt.ItemDataRole.UserRole, route)
            self.routes_list.addItem(item)
            
        # Update tab title
        self.tab_widget.setTabText(1, f"🛣️ Routes ({len(self.saved_locations.routes)})")
        
    def _on_location_clicked(self, item: QListWidgetItem):
        """Handle location item clicked"""
        location = item.data(Qt.ItemDataRole.UserRole)
        if location:
            self.location_selected.emit(location.latitude, location.longitude)
            
    def _on_location_double_clicked(self, item: QListWidgetItem):
        """Handle location item double-clicked - set location on device"""
        location = item.data(Qt.ItemDataRole.UserRole)
        device = self.device_manager.selected_device
        
        if location and device:
            self.location_controller.set_location(
                device, location.latitude, location.longitude
            )
            
    def _on_route_double_clicked(self, item: QListWidgetItem):
        """Handle route item double-clicked - start route simulation"""
        route = item.data(Qt.ItemDataRole.UserRole)
        device = self.device_manager.selected_device
        
        if route and device:
            waypoints = [Coordinate(wp.latitude, wp.longitude) for wp in route.waypoints]
            self.location_controller.start_route(device, waypoints)
            self.route_controls.set_route(route)
            self.route_controls.show()
            
    @pyqtSlot(str)
    def _on_route_state_changed(self, state: str):
        """Handle route state changes"""
        if state == "idle" or state == "completed":
            self.route_controls.hide()
            
    def _show_location_context_menu(self, pos):
        """Show context menu for locations"""
        item = self.locations_list.itemAt(pos)
        if not item:
            return
            
        location = item.data(Qt.ItemDataRole.UserRole)
        
        menu = QMenu(self)
        
        set_action = menu.addAction("📍 Set Location")
        set_action.triggered.connect(lambda: self._set_location(location))
        
        menu.addSeparator()
        
        delete_action = menu.addAction("🗑️ Delete")
        delete_action.triggered.connect(lambda: self._delete_location(location))
        
        menu.exec(self.locations_list.mapToGlobal(pos))
        
    def _show_route_context_menu(self, pos):
        """Show context menu for routes"""
        item = self.routes_list.itemAt(pos)
        if not item:
            return
            
        route = item.data(Qt.ItemDataRole.UserRole)
        
        menu = QMenu(self)
        
        play_action = menu.addAction("▶️ Start Route")
        play_action.triggered.connect(lambda: self._start_route(route))
        
        menu.addSeparator()
        
        delete_action = menu.addAction("🗑️ Delete")
        delete_action.triggered.connect(lambda: self._delete_route(route))
        
        menu.exec(self.routes_list.mapToGlobal(pos))
        
    def _set_location(self, location: SavedLocation):
        """Set location on device"""
        device = self.device_manager.selected_device
        if device:
            self.location_controller.set_location(
                device, location.latitude, location.longitude
            )
            
    def _delete_location(self, location: SavedLocation):
        """Delete a saved location"""
        self.saved_locations.remove_location(location.id)
        
    def _start_route(self, route: GPXRoute):
        """Start route simulation"""
        device = self.device_manager.selected_device
        if device:
            waypoints = [Coordinate(wp.latitude, wp.longitude) for wp in route.waypoints]
            self.location_controller.start_route(device, waypoints)
            self.route_controls.set_route(route)
            self.route_controls.show()
            
    def _delete_route(self, route: GPXRoute):
        """Delete a GPX route"""
        self.saved_locations.remove_route(route.id)
        
    def _import_gpx(self):
        """Import a GPX file"""
        file_path, _ = QFileDialog.getOpenFileName(
            self, "Import GPX File", "",
            "GPX Files (*.gpx);;All Files (*)"
        )
        if file_path:
            try:
                self.saved_locations.import_gpx(file_path)
            except Exception as e:
                QMessageBox.critical(self, "Import Error", str(e))


class RouteControlsWidget(QWidget):
    """Widget for route playback controls"""
    
    def __init__(self, location_controller: LocationController):
        super().__init__()
        self.location_controller = location_controller
        self.route = None
        
        self._setup_ui()
        self._setup_connections()
        
    def _setup_ui(self):
        """Setup the UI"""
        self.setStyleSheet("background-color: #f0f0f0; border-top: 1px solid #ddd;")
        
        layout = QVBoxLayout(self)
        layout.setContentsMargins(10, 10, 10, 10)
        layout.setSpacing(8)
        
        # Route name
        self.route_label = QLabel("Route")
        self.route_label.setFont(QFont("", 14, QFont.Weight.Bold))
        layout.addWidget(self.route_label)
        
        # Progress bar
        self.progress_bar = QProgressBar()
        self.progress_bar.setTextVisible(True)
        layout.addWidget(self.progress_bar)
        
        # Controls
        controls_layout = QHBoxLayout()
        
        self.pause_btn = QPushButton("⏸️")
        self.pause_btn.setFixedSize(32, 32)
        self.pause_btn.setStyleSheet("""
            QPushButton {
                background-color: #FFF3E0;
                color: #E65100;
                border: 1px solid #FF9800;
                border-radius: 4px;
                font-size: 14px;
            }
            QPushButton:hover {
                background-color: #FFE0B2;
            }
        """)
        self.pause_btn.clicked.connect(self._toggle_pause)
        controls_layout.addWidget(self.pause_btn)

        self.stop_btn = QPushButton("⏹️")
        self.stop_btn.setFixedSize(32, 32)
        self.stop_btn.setStyleSheet("""
            QPushButton {
                background-color: #FFEBEE;
                color: #C62828;
                border: 1px solid #EF5350;
                border-radius: 4px;
                font-size: 14px;
            }
            QPushButton:hover {
                background-color: #FFCDD2;
            }
        """)
        self.stop_btn.clicked.connect(self.location_controller.stop_route)
        controls_layout.addWidget(self.stop_btn)
        
        controls_layout.addStretch()
        
        # Speed control
        speed_label = QLabel("Speed:")
        speed_label.setStyleSheet("font-size: 13px;")
        controls_layout.addWidget(speed_label)
        
        for speed in [1, 2, 5, 10]:
            btn = QPushButton(f"{speed}x")
            btn.setFixedWidth(40)
            btn.setStyleSheet("""
                QPushButton {
                    background-color: #E3F2FD;
                    color: #1565C0;
                    border: 1px solid #1565C0;
                    border-radius: 4px;
                    font-size: 12px;
                    font-weight: 500;
                    padding: 4px;
                }
                QPushButton:hover {
                    background-color: #BBDEFB;
                }
            """)
            btn.clicked.connect(lambda checked, s=speed: self.location_controller.set_speed(s))
            controls_layout.addWidget(btn)
            
        layout.addLayout(controls_layout)
        
    def _setup_connections(self):
        """Setup signal connections"""
        self.location_controller.route_progress.connect(self._on_progress)
        self.location_controller.route_state_changed.connect(self._on_state_changed)
        
    def set_route(self, route: GPXRoute):
        """Set the current route"""
        self.route = route
        self.route_label.setText(f"🛣️ {route.name}")
        self.progress_bar.setMaximum(len(route.waypoints))
        self.progress_bar.setValue(0)
        
    @pyqtSlot(int, int)
    def _on_progress(self, current: int, total: int):
        """Handle progress update"""
        self.progress_bar.setMaximum(total)
        self.progress_bar.setValue(current)
        self.progress_bar.setFormat(f"{current}/{total}")
        
    @pyqtSlot(str)
    def _on_state_changed(self, state: str):
        """Handle state changes"""
        if state == "paused":
            self.pause_btn.setText("▶️")
        else:
            self.pause_btn.setText("⏸️")
            
    def _toggle_pause(self):
        """Toggle pause/resume"""
        from core import RouteSimulationState
        
        if self.location_controller.route_state == RouteSimulationState.PAUSED:
            self.location_controller.resume_route()
        else:
            self.location_controller.pause_route()
