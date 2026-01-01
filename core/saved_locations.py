"""
Saved Locations Manager - Handles persistence of saved locations and GPX routes
"""

import json
import os
import xml.etree.ElementTree as ET
from dataclasses import dataclass, asdict, field
from typing import List, Optional
from datetime import datetime
from pathlib import Path

from PyQt6.QtCore import QObject, pyqtSignal, QStandardPaths

from .location_controller import Coordinate


@dataclass
class SavedLocation:
    """A saved location bookmark"""
    id: str
    name: str
    latitude: float
    longitude: float
    category: str = "other"
    created_at: str = field(default_factory=lambda: datetime.now().isoformat())
    
    @property
    def coordinate(self) -> Coordinate:
        return Coordinate(self.latitude, self.longitude)
    
    def to_dict(self) -> dict:
        return asdict(self)
    
    @classmethod
    def from_dict(cls, data: dict) -> 'SavedLocation':
        return cls(**data)


@dataclass
class GPXWaypoint:
    """A waypoint from a GPX file"""
    latitude: float
    longitude: float
    elevation: Optional[float] = None
    name: Optional[str] = None
    
    @property
    def coordinate(self) -> Coordinate:
        return Coordinate(self.latitude, self.longitude)


@dataclass
class GPXRoute:
    """A route imported from a GPX file"""
    id: str
    name: str
    waypoints: List[GPXWaypoint]
    imported_at: str = field(default_factory=lambda: datetime.now().isoformat())
    
    def to_dict(self) -> dict:
        return {
            "id": self.id,
            "name": self.name,
            "waypoints": [
                {"latitude": w.latitude, "longitude": w.longitude, 
                 "elevation": w.elevation, "name": w.name}
                for w in self.waypoints
            ],
            "imported_at": self.imported_at
        }
    
    @classmethod
    def from_dict(cls, data: dict) -> 'GPXRoute':
        waypoints = [
            GPXWaypoint(**wp) for wp in data.get("waypoints", [])
        ]
        return cls(
            id=data["id"],
            name=data["name"],
            waypoints=waypoints,
            imported_at=data.get("imported_at", datetime.now().isoformat())
        )


class SavedLocationsManager(QObject):
    """Manages saved locations and GPX routes with persistence"""
    
    locations_updated = pyqtSignal()
    routes_updated = pyqtSignal()
    
    # Preset locations for first launch
    PRESET_LOCATIONS = [
        SavedLocation("preset-1", "Apple Park", 37.3349, -122.0090, "work"),
        SavedLocation("preset-2", "San Francisco", 37.7749, -122.4194, "other"),
        SavedLocation("preset-3", "New York City", 40.7128, -74.0060, "other"),
        SavedLocation("preset-4", "London", 51.5074, -0.1278, "other"),
        SavedLocation("preset-5", "Tokyo", 35.6762, 139.6503, "other"),
        SavedLocation("preset-6", "Singapore", 1.3521, 103.8198, "other"),
        SavedLocation("preset-7", "Sydney", -33.8688, 151.2093, "other"),
    ]
    
    def __init__(self):
        super().__init__()
        self._locations: List[SavedLocation] = []
        self._routes: List[GPXRoute] = []
        self._data_dir = self._get_data_dir()
        
        self._load_locations()
        self._load_routes()
        
    def _get_data_dir(self) -> Path:
        """Get application data directory"""
        app_data = QStandardPaths.writableLocation(
            QStandardPaths.StandardLocation.AppDataLocation
        )
        data_dir = Path(app_data) / "LocationSimulator"
        data_dir.mkdir(parents=True, exist_ok=True)
        return data_dir
    
    @property
    def locations(self) -> List[SavedLocation]:
        return self._locations
    
    @property
    def routes(self) -> List[GPXRoute]:
        return self._routes
    
    # Location management
    
    def add_location(self, name: str, latitude: float, longitude: float, 
                     category: str = "other") -> SavedLocation:
        """Add a new saved location"""
        import uuid
        location = SavedLocation(
            id=str(uuid.uuid4()),
            name=name,
            latitude=latitude,
            longitude=longitude,
            category=category
        )
        self._locations.insert(0, location)
        self._save_locations()
        self.locations_updated.emit()
        return location
    
    def remove_location(self, location_id: str):
        """Remove a saved location"""
        self._locations = [l for l in self._locations if l.id != location_id]
        self._save_locations()
        self.locations_updated.emit()
        
    def update_location(self, location: SavedLocation):
        """Update an existing location"""
        for i, loc in enumerate(self._locations):
            if loc.id == location.id:
                self._locations[i] = location
                break
        self._save_locations()
        self.locations_updated.emit()
        
    def _save_locations(self):
        """Save locations to disk"""
        file_path = self._data_dir / "locations.json"
        data = [loc.to_dict() for loc in self._locations]
        with open(file_path, 'w') as f:
            json.dump(data, f, indent=2)
            
    def _load_locations(self):
        """Load locations from disk"""
        file_path = self._data_dir / "locations.json"
        
        if file_path.exists():
            try:
                with open(file_path, 'r') as f:
                    data = json.load(f)
                self._locations = [SavedLocation.from_dict(d) for d in data]
            except Exception as e:
                print(f"Error loading locations: {e}")
                self._locations = list(self.PRESET_LOCATIONS)
        else:
            # First launch - use presets
            self._locations = list(self.PRESET_LOCATIONS)
            self._save_locations()
    
    # GPX Route management
    
    def import_gpx(self, file_path: str) -> GPXRoute:
        """Import a GPX file"""
        import uuid
        
        waypoints = self._parse_gpx(file_path)
        
        if not waypoints:
            raise ValueError("No waypoints found in GPX file")
        
        # Get route name from file
        name = Path(file_path).stem
        
        # Try to get name from GPX
        try:
            tree = ET.parse(file_path)
            root = tree.getroot()
            ns = {'gpx': 'http://www.topografix.com/GPX/1/1'}
            
            name_elem = root.find('.//gpx:name', ns) or root.find('.//name')
            if name_elem is not None and name_elem.text:
                name = name_elem.text
        except:
            pass
        
        route = GPXRoute(
            id=str(uuid.uuid4()),
            name=name,
            waypoints=waypoints
        )
        
        self._routes.insert(0, route)
        self._save_routes()
        self.routes_updated.emit()
        
        return route
    
    def remove_route(self, route_id: str):
        """Remove a GPX route"""
        self._routes = [r for r in self._routes if r.id != route_id]
        self._save_routes()
        self.routes_updated.emit()
        
    def _parse_gpx(self, file_path: str) -> List[GPXWaypoint]:
        """Parse a GPX file and extract waypoints"""
        waypoints = []
        
        try:
            tree = ET.parse(file_path)
            root = tree.getroot()
            
            # Handle namespace
            ns = {'gpx': 'http://www.topografix.com/GPX/1/1'}
            
            # Try to find track points, waypoints, or route points
            point_tags = [
                './/gpx:trkpt', './/gpx:wpt', './/gpx:rtept',
                './/trkpt', './/wpt', './/rtept'
            ]
            
            points = []
            for tag in point_tags:
                try:
                    found = root.findall(tag, ns) or root.findall(tag.replace('gpx:', ''))
                    points.extend(found)
                except:
                    pass
                    
            for point in points:
                lat = point.get('lat')
                lon = point.get('lon')
                
                if lat and lon:
                    # Get optional elevation
                    ele = None
                    ele_elem = point.find('gpx:ele', ns) or point.find('ele')
                    if ele_elem is not None and ele_elem.text:
                        try:
                            ele = float(ele_elem.text)
                        except:
                            pass
                    
                    # Get optional name
                    name = None
                    name_elem = point.find('gpx:name', ns) or point.find('name')
                    if name_elem is not None:
                        name = name_elem.text
                    
                    waypoints.append(GPXWaypoint(
                        latitude=float(lat),
                        longitude=float(lon),
                        elevation=ele,
                        name=name
                    ))
                    
        except ET.ParseError as e:
            raise ValueError(f"Invalid GPX file: {e}")
            
        return waypoints
    
    def _save_routes(self):
        """Save routes to disk"""
        file_path = self._data_dir / "routes.json"
        data = [route.to_dict() for route in self._routes]
        with open(file_path, 'w') as f:
            json.dump(data, f, indent=2)
            
    def _load_routes(self):
        """Load routes from disk"""
        file_path = self._data_dir / "routes.json"
        
        if file_path.exists():
            try:
                with open(file_path, 'r') as f:
                    data = json.load(f)
                self._routes = [GPXRoute.from_dict(d) for d in data]
            except Exception as e:
                print(f"Error loading routes: {e}")
                self._routes = []
        else:
            self._routes = []
