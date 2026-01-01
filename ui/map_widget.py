"""
Map Widget - Interactive map using OpenStreetMap/Leaflet or Google Maps
"""

from PyQt6.QtWidgets import QWidget, QVBoxLayout
from PyQt6.QtCore import pyqtSignal, QUrl
from PyQt6.QtWebEngineWidgets import QWebEngineView
from PyQt6.QtWebChannel import QWebChannel


class MapWidget(QWidget):
    """Interactive map widget supporting OpenStreetMap and Google Maps"""

    coordinates_changed = pyqtSignal(float, float)  # lat, lon

    def __init__(self, map_provider='osm', google_api_key=''):
        super().__init__()

        self._lat = 37.7749
        self._lon = -122.4194
        self._map_provider = map_provider  # 'osm' or 'google'
        self._google_api_key = google_api_key

        self._setup_ui()
        
    def _setup_ui(self):
        """Setup the web view with map"""
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)

        self.web_view = QWebEngineView()
        layout.addWidget(self.web_view)

        # Forward JavaScript console messages to Python console
        self.web_view.page().javaScriptConsoleMessage = self._js_console_message

        # Setup web channel for JS communication
        self.channel = QWebChannel()
        self.channel.registerObject("mapBridge", self)
        self.web_view.page().setWebChannel(self.channel)
        print(f"[MapWidget] QWebChannel registered for map provider: {self._map_provider}")

        # Load map HTML
        self.web_view.setHtml(self._get_map_html())
        print(f"[MapWidget] Map HTML loaded for provider: {self._map_provider}")

    def _js_console_message(self, level, message, line, source):
        """Forward JavaScript console messages to Python console"""
        print(f"[JS Console] {message}")

    def switch_map_provider(self, provider: str, google_api_key: str = ''):
        """Switch between map providers"""
        print(f"[MapWidget] Switching map provider to: {provider}")
        self._map_provider = provider
        if provider == 'google':
            self._google_api_key = google_api_key

        # Properly disconnect old channel before creating new one to avoid segfault
        if hasattr(self, 'channel') and self.channel:
            print(f"[MapWidget] Cleaning up old QWebChannel")
            # First, disconnect the web channel from the page
            self.web_view.page().setWebChannel(None)
            # Deregister the bridge object
            try:
                self.channel.deregisterObject(self)
            except:
                pass  # May already be deregistered
            # Delete the old channel
            self.channel.deleteLater()
            self.channel = None
            print(f"[MapWidget] Old QWebChannel cleaned up")

        # Create new channel
        print(f"[MapWidget] Creating new QWebChannel")
        self.channel = QWebChannel()
        self.channel.registerObject("mapBridge", self)
        self.web_view.page().setWebChannel(self.channel)
        print(f"[MapWidget] QWebChannel re-registered for provider: {provider}")

        # Reload map with new provider (preserves current lat/lon)
        self.web_view.setHtml(self._get_map_html())
        print(f"[MapWidget] Map HTML reloaded for provider: {provider}")

    def _get_map_html(self) -> str:
        """Generate the HTML for the map based on provider"""
        if self._map_provider == 'google' and self._google_api_key:
            return self._get_google_maps_html()
        else:
            return self._get_osm_html()

    def _get_osm_html(self) -> str:
        """Generate HTML for OpenStreetMap with Leaflet"""
        return f'''
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <link rel="stylesheet" href="https://unpkg.com/leaflet@1.9.4/dist/leaflet.css" />
    <script src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js"></script>
    <script src="qrc:///qtwebchannel/qwebchannel.js"></script>
    <style>
        * {{ margin: 0; padding: 0; }}
        html, body, #map {{ width: 100%; height: 100%; }}
        
        .crosshair {{
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            z-index: 1000;
            pointer-events: none;
        }}
        .crosshair::before,
        .crosshair::after {{
            content: '';
            position: absolute;
            background: rgba(220, 53, 69, 0.8);
        }}
        .crosshair::before {{
            width: 20px;
            height: 2px;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
        }}
        .crosshair::after {{
            width: 2px;
            height: 20px;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
        }}
        
        .coord-display {{
            position: absolute;
            bottom: 10px;
            left: 10px;
            background: rgba(255, 255, 255, 0.9);
            padding: 5px 10px;
            border-radius: 4px;
            font-family: monospace;
            font-size: 12px;
            z-index: 1000;
            box-shadow: 0 1px 3px rgba(0,0,0,0.2);
        }}
    </style>
</head>
<body>
    <div id="map"></div>
    <div class="crosshair"></div>
    <div class="coord-display" id="coords">Loading...</div>
    
    <script>
        // Initialize map
        var map = L.map('map', {{
            center: [{self._lat}, {self._lon}],
            zoom: 13,
            zoomControl: true
        }});
        
        // Add tile layer
        L.tileLayer('https://{{s}}.tile.openstreetmap.org/{{z}}/{{x}}/{{y}}.png', {{
            attribution: '© OpenStreetMap contributors',
            maxZoom: 19
        }}).addTo(map);
        
        // Current marker
        var marker = null;
        
        // Web channel bridge
        var mapBridge = null;
        
        new QWebChannel(qt.webChannelTransport, function(channel) {{
            mapBridge = channel.objects.mapBridge;
        }});
        
        // Update coordinates display
        function updateCoords() {{
            var center = map.getCenter();
            var lat = center.lat.toFixed(6);
            var lon = center.lng.toFixed(6);
            document.getElementById('coords').textContent = lat + ', ' + lon;

            // Notify Python - only if bridge is ready
            if (mapBridge && typeof mapBridge.onMapMoved === 'function') {{
                mapBridge.onMapMoved(center.lat, center.lng);
            }}
        }}
        
        // Map events
        map.on('moveend', updateCoords);
        map.on('zoomend', updateCoords);
        
        // Click to place marker
        map.on('click', function(e) {{
            if (marker) {{
                map.removeLayer(marker);
            }}
            marker = L.marker(e.latlng, {{
                icon: L.divIcon({{
                    className: 'custom-marker',
                    html: '<div style="background: #dc3545; width: 12px; height: 12px; border-radius: 50%; border: 2px solid white; box-shadow: 0 1px 3px rgba(0,0,0,0.3);"></div>',
                    iconSize: [12, 12],
                    iconAnchor: [6, 6]
                }})
            }}).addTo(map);
            
            // Center on clicked location
            map.panTo(e.latlng);
        }});
        
        // Initial update
        updateCoords();
        
        // Functions called from Python
        function setMapLocation(lat, lon) {{
            map.setView([lat, lon], 15);
            
            if (marker) {{
                map.removeLayer(marker);
            }}
            marker = L.marker([lat, lon], {{
                icon: L.divIcon({{
                    className: 'custom-marker',
                    html: '<div style="background: #dc3545; width: 12px; height: 12px; border-radius: 50%; border: 2px solid white; box-shadow: 0 1px 3px rgba(0,0,0,0.3);"></div>',
                    iconSize: [12, 12],
                    iconAnchor: [6, 6]
                }})
            }}).addTo(map);
        }}
        
        function searchLocation(query) {{
            // Use Nominatim for geocoding
            fetch('https://nominatim.openstreetmap.org/search?format=json&q=' + encodeURIComponent(query))
                .then(response => response.json())
                .then(data => {{
                    if (data && data.length > 0) {{
                        var result = data[0];
                        setMapLocation(parseFloat(result.lat), parseFloat(result.lon));
                    }}
                }})
                .catch(error => console.error('Search error:', error));
        }}
    </script>
</body>
</html>
'''

    def _get_google_maps_html(self) -> str:
        """Generate HTML for Google Maps"""
        return f'''
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <script src="qrc:///qtwebchannel/qwebchannel.js"></script>
    <style>
        * {{ margin: 0; padding: 0; }}
        html, body, #map {{ width: 100%; height: 100%; }}

        .crosshair {{
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            z-index: 1000;
            pointer-events: none;
        }}
        .crosshair::before,
        .crosshair::after {{
            content: '';
            position: absolute;
            background: rgba(220, 53, 69, 0.8);
        }}
        .crosshair::before {{
            width: 20px;
            height: 2px;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
        }}
        .crosshair::after {{
            width: 2px;
            height: 20px;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
        }}

        .coord-display {{
            position: absolute;
            bottom: 10px;
            left: 10px;
            background: rgba(255, 255, 255, 0.9);
            padding: 5px 10px;
            border-radius: 4px;
            font-family: monospace;
            font-size: 12px;
            z-index: 1000;
            box-shadow: 0 1px 3px rgba(0,0,0,0.2);
        }}
    </style>
</head>
<body>
    <div id="map"></div>
    <div class="crosshair"></div>
    <div class="coord-display" id="coords">Loading...</div>

    <script src="https://maps.googleapis.com/maps/api/js?key={self._google_api_key}&callback=initMap" async defer></script>
    <script>
        var map;
        var marker = null;
        var mapBridge = null;

        // Web channel bridge - setup first
        new QWebChannel(qt.webChannelTransport, function(channel) {{
            mapBridge = channel.objects.mapBridge;
            console.log('QWebChannel bridge ready');
        }});

        function initMap() {{
            console.log('initMap called');
            // Initialize map
            map = new google.maps.Map(document.getElementById('map'), {{
                center: {{ lat: {self._lat}, lng: {self._lon} }},
                zoom: 13,
                mapTypeControl: true,
                mapTypeControlOptions: {{
                    style: google.maps.MapTypeControlStyle.DROPDOWN_MENU,
                    mapTypeIds: ['roadmap', 'satellite', 'hybrid', 'terrain']
                }}
            }});

            // Map events - debounce center_changed to avoid excessive updates
            var updateTimeout;
            map.addListener('center_changed', function() {{
                clearTimeout(updateTimeout);
                updateTimeout = setTimeout(updateCoords, 100);
            }});

            map.addListener('zoom_changed', function() {{
                updateCoords();
            }});

            // Click to place marker
            map.addListener('click', function(e) {{
                if (marker) {{
                    marker.setMap(null);
                }}
                marker = new google.maps.Marker({{
                    position: e.latLng,
                    map: map,
                    icon: {{
                        path: google.maps.SymbolPath.CIRCLE,
                        fillColor: '#dc3545',
                        fillOpacity: 1,
                        strokeColor: 'white',
                        strokeWeight: 2,
                        scale: 6
                    }}
                }});

                map.panTo(e.latLng);
            }});

            // Initial update
            setTimeout(updateCoords, 500);
        }}

        // Update coordinates display
        function updateCoords() {{
            if (!map) {{
                console.log('updateCoords: map not ready');
                return;
            }}

            var center = map.getCenter();
            var lat = center.lat().toFixed(6);
            var lon = center.lng().toFixed(6);
            document.getElementById('coords').textContent = lat + ', ' + lon;

            console.log('updateCoords: lat=' + lat + ', lon=' + lon + ', bridge=' + (mapBridge ? 'ready' : 'not ready'));

            // Notify Python - only if bridge is ready
            if (mapBridge && typeof mapBridge.onMapMoved === 'function') {{
                console.log('Calling mapBridge.onMapMoved');
                mapBridge.onMapMoved(center.lat(), center.lng());
            }} else {{
                console.log('mapBridge not ready, skipping Python notification');
            }}
        }}

        // Functions called from Python
        function setMapLocation(lat, lon) {{
            if (!map) return;

            var location = {{ lat: lat, lng: lon }};
            map.setCenter(location);
            map.setZoom(15);

            if (marker) {{
                marker.setMap(null);
            }}
            marker = new google.maps.Marker({{
                position: location,
                map: map,
                icon: {{
                    path: google.maps.SymbolPath.CIRCLE,
                    fillColor: '#dc3545',
                    fillOpacity: 1,
                    strokeColor: 'white',
                    strokeWeight: 2,
                    scale: 6
                }}
            }});
        }}

        function searchLocation(query) {{
            if (!map) return;

            var geocoder = new google.maps.Geocoder();
            geocoder.geocode({{ 'address': query }}, function(results, status) {{
                if (status === 'OK' && results[0]) {{
                    var location = results[0].geometry.location;
                    setMapLocation(location.lat(), location.lng());
                }}
            }});
        }}
    </script>
</body>
</html>
'''

    def set_location(self, lat: float, lon: float):
        """Set the map location"""
        self._lat = lat
        self._lon = lon
        self.web_view.page().runJavaScript(f"setMapLocation({lat}, {lon})")

    def search_location(self, query: str):
        """Search for a location"""
        # Escape quotes in query
        safe_query = query.replace("'", "\\'").replace('"', '\\"')
        self.web_view.page().runJavaScript(f"searchLocation('{safe_query}')")

    def onMapMoved(self, lat: float, lon: float):
        """Called from JavaScript when map moves"""
        print(f"[MapWidget] onMapMoved called: lat={lat}, lon={lon}")
        self._lat = lat
        self._lon = lon
        self.coordinates_changed.emit(lat, lon)
        print(f"[MapWidget] coordinates_changed signal emitted")

    def get_current_coordinates(self):
        """Get the current map center coordinates synchronously from JavaScript"""
        print(f"[MapWidget] Getting current coordinates from map...")

        # Execute JavaScript to get current map center
        if self._map_provider == 'google':
            js_code = """
            (function() {
                if (map && map.getCenter) {
                    var center = map.getCenter();
                    return center.lat() + ',' + center.lng();
                }
                return null;
            })();
            """
        else:  # OSM
            js_code = """
            (function() {
                if (map && map.getCenter) {
                    var center = map.getCenter();
                    return center.lat + ',' + center.lng;
                }
                return null;
            })();
            """

        # Run JavaScript synchronously and get result
        result = [None]  # Use list to store result from callback

        def callback(coords_str):
            result[0] = coords_str

        self.web_view.page().runJavaScript(js_code, callback)

        # Process events to wait for callback
        from PyQt6.QtCore import QEventLoop, QTimer
        loop = QEventLoop()
        QTimer.singleShot(100, loop.quit)  # Wait max 100ms
        loop.exec()

        if result[0]:
            try:
                lat_str, lon_str = result[0].split(',')
                lat, lon = float(lat_str), float(lon_str)
                print(f"[MapWidget] Got coordinates from map: {lat}, {lon}")
                # Update internal state
                self._lat = lat
                self._lon = lon
                return lat, lon
            except Exception as e:
                print(f"[MapWidget] Error parsing coordinates: {e}")

        # Fallback to stored coordinates
        print(f"[MapWidget] Falling back to stored coordinates: {self._lat}, {self._lon}")
        return self._lat, self._lon
