"""
Main Window - Primary application window with map and controls
"""

from PyQt6.QtWidgets import (
    QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QSplitter,
    QLabel, QPushButton, QLineEdit, QStatusBar, QToolBar,
    QMessageBox, QInputDialog, QFileDialog, QFrame, QCompleter
)
from PyQt6.QtCore import Qt, QUrl, pyqtSlot, QStringListModel, QTimer
from PyQt6.QtGui import QAction, QIcon, QKeySequence
from PyQt6.QtWebEngineWidgets import QWebEngineView
import json
import urllib.request
import urllib.parse

from core import (
    DeviceManager, LocationController, SavedLocationsManager,
    Coordinate, TunnelManager, SettingsManager
)
from .device_panel import DevicePanel
from .locations_panel import LocationsPanel
from .map_widget import MapWidget
from .control_bar import ControlBar
from .password_dialog import PasswordDialog
from .api_key_dialog import ApiKeyDialog


class MainWindow(QMainWindow):
    """Main application window"""
    
    def __init__(self):
        super().__init__()

        # Initialize managers
        self.device_manager = DeviceManager()
        self.location_controller = LocationController()
        self.saved_locations = SavedLocationsManager()
        self.tunnel_manager = TunnelManager()
        self.settings_manager = SettingsManager()

        # Current selected coordinates
        self.current_lat = 37.7749
        self.current_lon = -122.4194

        # Track if we've already attempted to start tunnel
        self._tunnel_start_attempted = False

        self._setup_ui()
        self._setup_menu()
        self._setup_connections()

        # Start device auto-refresh
        self.device_manager.start_auto_refresh()

        # Start tunnel automatically (will check if needed for iOS 17+)
        QTimer.singleShot(2000, self._check_and_start_tunnel)
        
    def _setup_ui(self):
        """Setup the user interface"""
        self.setWindowTitle("Location Simulator")
        self.setMinimumSize(1100, 750)
        self.resize(1200, 800)
        
        # Central widget
        central = QWidget()
        self.setCentralWidget(central)
        
        # Main layout with splitter
        layout = QHBoxLayout(central)
        layout.setContentsMargins(0, 0, 0, 0)
        
        splitter = QSplitter(Qt.Orientation.Horizontal)
        layout.addWidget(splitter)
        
        # Left sidebar
        sidebar = QWidget()
        sidebar.setMaximumWidth(300)
        sidebar.setMinimumWidth(250)
        sidebar_layout = QVBoxLayout(sidebar)
        sidebar_layout.setContentsMargins(0, 0, 0, 0)
        sidebar_layout.setSpacing(0)
        
        # Device panel
        self.device_panel = DevicePanel(self.device_manager)
        sidebar_layout.addWidget(self.device_panel)
        
        # Separator
        separator = QFrame()
        separator.setFrameShape(QFrame.Shape.HLine)
        sidebar_layout.addWidget(separator)
        
        # Locations panel
        self.locations_panel = LocationsPanel(
            self.saved_locations,
            self.device_manager,
            self.location_controller
        )
        sidebar_layout.addWidget(self.locations_panel, 1)
        
        splitter.addWidget(sidebar)
        
        # Right side - map and controls
        right_widget = QWidget()
        right_layout = QVBoxLayout(right_widget)
        right_layout.setContentsMargins(0, 0, 0, 0)
        right_layout.setSpacing(0)
        
        # Search bar
        search_bar = self._create_search_bar()
        right_layout.addWidget(search_bar)

        # Map widget - initialize with saved settings
        map_provider = self.settings_manager.get_map_provider()
        google_api_key = self.settings_manager.get_google_api_key()
        self.map_widget = MapWidget(map_provider=map_provider, google_api_key=google_api_key)
        right_layout.addWidget(self.map_widget, 1)
        
        # Control bar
        self.control_bar = ControlBar(
            self.device_manager,
            self.location_controller,
            self.saved_locations
        )
        right_layout.addWidget(self.control_bar)
        
        splitter.addWidget(right_widget)
        
        # Set splitter proportions
        splitter.setSizes([280, 920])
        
        # Status bar
        self.statusBar().showMessage("Ready")
        
    def _create_search_bar(self) -> QWidget:
        """Create the location search bar"""
        widget = QWidget()
        widget.setStyleSheet("background-color: #f5f5f5; border-bottom: 1px solid #ddd;")
        layout = QHBoxLayout(widget)
        layout.setContentsMargins(10, 8, 10, 8)

        # Search icon label (will change to loading spinner)
        self.search_icon_label = QLabel("🔍")
        self.search_icon_label.setStyleSheet("font-size: 18px;")
        layout.addWidget(self.search_icon_label)
        
        # Search input with autocomplete
        self.search_input = QLineEdit()
        self.search_input.setPlaceholderText("Search location or enter coordinates (lat, lon)...")
        self.search_input.setStyleSheet("""
            QLineEdit {
                border: none;
                background: transparent;
                padding: 5px;
                font-size: 15px;
                color: #000;
            }
            QLineEdit::placeholder {
                color: #888;
            }
        """)

        # Setup autocomplete with UnfilteredPopupCompletion
        # This shows ALL suggestions without filtering by what user typed
        self.search_completer = QCompleter()
        self.search_completer.setCaseSensitivity(Qt.CaseSensitivity.CaseInsensitive)
        self.search_completer.setCompletionMode(QCompleter.CompletionMode.UnfilteredPopupCompletion)
        self.search_completer.setMaxVisibleItems(7)
        self.search_completer_model = QStringListModel()
        self.search_completer.setModel(self.search_completer_model)
        self.search_input.setCompleter(self.search_completer)

        # Style the completer popup
        self.search_completer.popup().setStyleSheet("""
            QListView {
                background-color: white;
                color: black;
                border: 2px solid #1565C0;
                border-radius: 4px;
                padding: 4px;
                font-size: 13px;
                min-width: 400px;
            }
            QListView::item {
                padding: 8px;
                border-bottom: 1px solid #eee;
                color: black;
            }
            QListView::item:selected {
                background-color: #E3F2FD;
                color: #1565C0;
            }
            QListView::item:hover {
                background-color: #f5f5f5;
            }
        """)

        # Debounce timer for search suggestions
        self.search_timer = QTimer()
        self.search_timer.setSingleShot(True)
        self.search_timer.timeout.connect(self._fetch_search_suggestions)

        self.search_input.textChanged.connect(self._on_search_text_changed)
        self.search_input.returnPressed.connect(self._on_search)
        self.search_completer.activated.connect(self._on_suggestion_selected)
        self.search_completer.highlighted.connect(self._on_suggestion_highlighted)

        # Store location data for selected suggestions
        self.location_cache = {}

        layout.addWidget(self.search_input, 1)
        
        # Search button
        search_btn = QPushButton("Search")
        search_btn.setStyleSheet("""
            QPushButton {
                background-color: #E3F2FD;
                color: #1565C0;
                border: 1px solid #1565C0;
                padding: 6px 16px;
                border-radius: 4px;
                font-size: 13px;
                font-weight: 500;
            }
            QPushButton:hover {
                background-color: #BBDEFB;
            }
        """)
        search_btn.clicked.connect(self._on_search)
        layout.addWidget(search_btn)
        
        return widget
        
    def _setup_menu(self):
        """Setup menu bar"""
        menubar = self.menuBar()
        
        # File menu
        file_menu = menubar.addMenu("File")
        
        import_action = QAction("Import GPX...", self)
        import_action.setShortcut(QKeySequence("Ctrl+I"))
        import_action.triggered.connect(self._import_gpx)
        file_menu.addAction(import_action)
        
        file_menu.addSeparator()
        
        quit_action = QAction("Quit", self)
        quit_action.setShortcut(QKeySequence.StandardKey.Quit)
        quit_action.triggered.connect(self.close)
        file_menu.addAction(quit_action)
        
        # Device menu
        device_menu = menubar.addMenu("Device")
        
        refresh_action = QAction("Refresh Devices", self)
        refresh_action.setShortcut(QKeySequence("Ctrl+R"))
        refresh_action.triggered.connect(self.device_manager.refresh_devices)
        device_menu.addAction(refresh_action)
        
        device_menu.addSeparator()
        
        clear_action = QAction("Clear Simulated Location", self)
        clear_action.setShortcut(QKeySequence("Ctrl+K"))
        clear_action.triggered.connect(self._clear_location)
        device_menu.addAction(clear_action)
        
        # Location menu
        location_menu = menubar.addMenu("Location")
        
        set_action = QAction("Set Current Location", self)
        set_action.setShortcut(QKeySequence("Ctrl+Return"))
        set_action.triggered.connect(self._set_location)
        location_menu.addAction(set_action)
        
        save_action = QAction("Save Current Location", self)
        save_action.setShortcut(QKeySequence("Ctrl+Shift+S"))
        save_action.triggered.connect(self._save_location)
        location_menu.addAction(save_action)

        # View menu
        view_menu = menubar.addMenu("View")

        map_provider_menu = view_menu.addMenu("Map Provider")

        osm_action = QAction("OpenStreetMap (Free)", self)
        osm_action.setCheckable(True)
        osm_action.setChecked(self.settings_manager.get_map_provider() == 'osm')
        osm_action.triggered.connect(self._switch_to_osm)
        map_provider_menu.addAction(osm_action)

        google_action = QAction("Google Maps (Requires API Key)", self)
        google_action.setCheckable(True)
        google_action.setChecked(self.settings_manager.get_map_provider() == 'google')
        google_action.triggered.connect(self._switch_to_google_maps)
        map_provider_menu.addAction(google_action)

        # Store references for later updates
        self.osm_action = osm_action
        self.google_action = google_action

        # Help menu
        help_menu = menubar.addMenu("Help")
        
        about_action = QAction("About", self)
        about_action.triggered.connect(self._show_about)
        help_menu.addAction(about_action)
        
    def _setup_connections(self):
        """Setup signal connections"""
        # Map signals
        self.map_widget.coordinates_changed.connect(self._on_coordinates_changed)

        # Location controller signals
        self.location_controller.simulation_result.connect(self._on_simulation_result)

        # Device manager signals
        self.device_manager.error_occurred.connect(self._on_error)
        # Don't auto-start tunnel on every device update - causes multiple terminal windows
        # self.device_manager.devices_updated.connect(lambda devices: self._check_and_start_tunnel())

        # Tunnel manager signals
        self.tunnel_manager.tunnel_started.connect(self._on_tunnel_started)
        self.tunnel_manager.tunnel_error.connect(self._on_tunnel_error)
        self.tunnel_manager.password_required.connect(self._on_password_required)

        # Locations panel signals
        self.locations_panel.location_selected.connect(self._on_location_selected)

        # Control bar signals
        self.control_bar.set_location_clicked.connect(self._set_location)
        self.control_bar.clear_location_clicked.connect(self._clear_location)
        self.control_bar.save_location_clicked.connect(self._save_location)
        
    @pyqtSlot(float, float)
    def _on_coordinates_changed(self, lat: float, lon: float):
        """Handle coordinates changed from map"""
        print(f"[MainWindow] _on_coordinates_changed: lat={lat}, lon={lon}")
        self.current_lat = lat
        self.current_lon = lon
        self.control_bar.update_coordinates(lat, lon)
        print(f"[MainWindow] Control bar updated with new coordinates")
        
    @pyqtSlot(float, float)
    def _on_location_selected(self, lat: float, lon: float):
        """Handle location selected from sidebar"""
        self.current_lat = lat
        self.current_lon = lon
        self.map_widget.set_location(lat, lon)
        self.control_bar.update_coordinates(lat, lon)
        
    @pyqtSlot(bool, str)
    def _on_simulation_result(self, success: bool, message: str):
        """Handle simulation result"""
        self.statusBar().showMessage(message)  # Display permanently

        # Show detailed error in message box for failures
        if not success:
            QMessageBox.critical(
                self,
                "Location Simulation Error",
                message
            )
        
    @pyqtSlot(str)
    def _on_error(self, error: str):
        """Handle error from device manager"""
        self.statusBar().showMessage(f"Error: {error}", 5000)
        
    def _on_search_text_changed(self, text: str):
        """Handle search text changes with debouncing"""
        # Stop previous timer
        self.search_timer.stop()

        # Reset to search icon if text is too short
        if len(text.strip()) < 3:
            self.search_icon_label.setText("🔍")
            return

        # Show loading indicator
        self.search_icon_label.setText("⏳")

        # Start new timer (800ms debounce - wait for user to stop typing)
        self.search_timer.start(800)

    def _fetch_search_suggestions(self):
        """Fetch location suggestions from Nominatim API"""
        query = self.search_input.text().strip()
        if len(query) < 3:
            self.search_icon_label.setText("🔍")
            return

        try:
            # Use Nominatim (OpenStreetMap) geocoding API
            encoded_query = urllib.parse.quote(query)
            url = f"https://nominatim.openstreetmap.org/search?q={encoded_query}&format=json&limit=7&addressdetails=1"

            req = urllib.request.Request(url)
            req.add_header('User-Agent', 'LocationSimulator/1.0')

            with urllib.request.urlopen(req, timeout=3) as response:
                data = json.loads(response.read().decode())

                suggestions = []
                self.location_cache.clear()

                for item in data:
                    display_name = item.get('display_name', '')
                    lat = item.get('lat')
                    lon = item.get('lon')

                    if display_name and lat and lon:
                        suggestions.append(display_name)
                        self.location_cache[display_name] = (float(lat), float(lon))

                # Update completer model
                self.search_completer_model.setStringList(suggestions)

                # Reset icon back to search
                self.search_icon_label.setText("🔍")

        except Exception as e:
            # Reset icon on error
            self.search_icon_label.setText("🔍")
            # Silently fail - don't interrupt user experience
            print(f"Search suggestion error: {e}")

    def _on_suggestion_highlighted(self, suggestion: str):
        """Handle highlighting of a suggestion (for preview)"""
        # Store the currently highlighted suggestion
        self._highlighted_suggestion = suggestion

    def _on_suggestion_selected(self, suggestion: str):
        """Handle selection of a suggestion from dropdown"""
        if suggestion in self.location_cache:
            lat, lon = self.location_cache[suggestion]
            self.map_widget.set_location(lat, lon)
            self.current_lat = lat
            self.current_lon = lon
            self.control_bar.update_coordinates(lat, lon)
            self.statusBar().showMessage(f"Location set to: {suggestion[:50]}...", 3000)

    def _on_search(self):
        """Handle search input"""
        query = self.search_input.text().strip()
        if not query:
            return

        # Check if the query is in cache (from suggestions)
        if query in self.location_cache:
            lat, lon = self.location_cache[query]
            self.map_widget.set_location(lat, lon)
            self.current_lat = lat
            self.current_lon = lon
            self.control_bar.update_coordinates(lat, lon)
            return

        # Check if it's coordinates
        if ',' in query:
            parts = query.split(',')
            if len(parts) == 2:
                try:
                    lat = float(parts[0].strip())
                    lon = float(parts[1].strip())
                    self.map_widget.set_location(lat, lon)
                    self.current_lat = lat
                    self.current_lon = lon
                    self.control_bar.update_coordinates(lat, lon)
                    return
                except ValueError:
                    pass

        # Otherwise treat as address search
        self.map_widget.search_location(query)
        
    def _set_location(self):
        """Set location on selected device"""
        device = self.device_manager.selected_device
        if not device:
            QMessageBox.warning(self, "No Device", "No device selected")
            return

        # Get the actual current coordinates from the map (in case QWebChannel didn't sync)
        lat, lon = self.map_widget.get_current_coordinates()
        print(f"[MainWindow] Setting location from map center: {lat}, {lon}")

        # Update our internal state
        self.current_lat = lat
        self.current_lon = lon

        self.location_controller.set_location(
            device, lat, lon
        )
        self.statusBar().showMessage(f"Setting location to {lat:.6f}, {lon:.6f}...")
        
    def _clear_location(self):
        """Clear simulated location"""
        device = self.device_manager.selected_device
        if not device:
            QMessageBox.warning(self, "No Device", "No device selected")
            return
            
        self.location_controller.clear_location(device)
        self.statusBar().showMessage("Clearing location...")
        
    def _save_location(self):
        """Save current location"""
        name, ok = QInputDialog.getText(
            self, "Save Location", "Enter location name:"
        )
        if ok and name:
            self.saved_locations.add_location(
                name, self.current_lat, self.current_lon
            )
            self.statusBar().showMessage(f"Saved location: {name}")
            
    def _import_gpx(self):
        """Import a GPX file"""
        file_path, _ = QFileDialog.getOpenFileName(
            self, "Import GPX File", "",
            "GPX Files (*.gpx);;All Files (*)"
        )
        if file_path:
            try:
                route = self.saved_locations.import_gpx(file_path)
                self.statusBar().showMessage(
                    f"Imported route: {route.name} ({len(route.waypoints)} points)"
                )
            except Exception as e:
                QMessageBox.critical(self, "Import Error", str(e))
                
    def _switch_to_osm(self):
        """Switch to OpenStreetMap"""
        # Update checkboxes
        self.osm_action.setChecked(True)
        self.google_action.setChecked(False)

        # Save preference
        self.settings_manager.set_map_provider('osm')

        # Switch map
        self.map_widget.switch_map_provider('osm')
        self.statusBar().showMessage("Switched to OpenStreetMap")

    def _switch_to_google_maps(self):
        """Switch to Google Maps (requires API key)"""
        # Check if we have an API key
        api_key = self.settings_manager.get_google_api_key()

        if not api_key:
            # No API key - show dialog
            api_key = ApiKeyDialog.get_api_key_from_user(self)

            if not api_key:
                # User cancelled - revert checkbox
                self.osm_action.setChecked(True)
                self.google_action.setChecked(False)
                self.statusBar().showMessage("Google Maps requires an API key")
                return

            # Save the API key
            self.settings_manager.set_google_api_key(api_key)

        # Update checkboxes
        self.osm_action.setChecked(False)
        self.google_action.setChecked(True)

        # Save preference
        self.settings_manager.set_map_provider('google')

        # Switch map
        self.map_widget.switch_map_provider('google', api_key)
        self.statusBar().showMessage("Switched to Google Maps")

    def _show_about(self):
        """Show about dialog"""
        QMessageBox.about(
            self,
            "About Location Simulator",
            """<h2>Location Simulator</h2>
            <p>Version 1.0</p>
            <p>Simulate iPhone location using pymobiledevice3</p>
            <p><b>Requirements:</b></p>
            <ul>
                <li>iOS device with Developer Mode enabled</li>
                <li>Device paired with this Mac</li>
            </ul>
            """
        )
        
    def _check_and_start_tunnel(self):
        """Check if any iOS 17+ devices connected and start tunnel if needed"""
        # Only attempt to start tunnel once per session
        if self._tunnel_start_attempted:
            return

        devices = self.device_manager.devices

        # Check if any device is iOS 17+
        needs_tunnel = False
        for device in devices:
            try:
                ios_version = device.ios_version
                ios_major = int(ios_version.split('.')[0])
                if ios_major >= 17:
                    needs_tunnel = True
                    break
            except:
                pass

        if needs_tunnel:
            # Mark that we've attempted to start the tunnel
            self._tunnel_start_attempted = True

            # Check if tunnel is already running
            if not self.tunnel_manager.check_tunnel_accessibility():
                self.statusBar().showMessage("Starting RemoteXPC tunnel for iOS 17+ device...", 3000)
                self.tunnel_manager.start_tunnel()
            else:
                self.statusBar().showMessage("RemoteXPC tunnel already running", 2000)

    def _on_tunnel_started(self):
        """Handle tunnel started signal"""
        self.statusBar().showMessage("✅ RemoteXPC tunnel started successfully", 3000)

    def _on_password_required(self):
        """Handle password required signal from tunnel manager"""
        self.statusBar().showMessage("Administrator password required to start tunnel...", 5000)

        # Show password dialog
        password = PasswordDialog.get_password_from_user(self)

        if password:
            # User provided password
            self.statusBar().showMessage("Starting tunnel with provided password...", 3000)
            self.tunnel_manager.provide_password(password)
        else:
            # User cancelled
            self.statusBar().showMessage("Tunnel start cancelled", 3000)
            self.tunnel_manager.cancel_password()

    def _on_tunnel_error(self, error: str):
        """Handle tunnel error signal"""
        self.statusBar().showMessage(f"⚠️ {error}", 10000)

        # Show dialog for errors (but not for cancellation)
        if "cancelled" not in error.lower():
            QMessageBox.warning(
                self,
                "RemoteXPC Tunnel Error",
                f"{error}\n\n"
                "You can try again or start the tunnel manually:\n./start_tunnel.sh"
            )

    def closeEvent(self, event):
        """Handle window close"""
        self.device_manager.stop_auto_refresh()
        self.location_controller.stop_route()
        self.location_controller._stop_ios17_simulation()  # Clean up iOS 17+ simulation process
        self.tunnel_manager.stop_tunnel()
        super().closeEvent(event)
