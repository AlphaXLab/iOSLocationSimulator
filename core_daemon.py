#!/usr/bin/env python3
"""
Core Daemon - JSON-RPC sidecar process for LocationSimulator.
Communicates with the Rust UI over stdin/stdout using line-delimited JSON.
"""

import sys
import os
import json
import traceback
import threading

# Force pure-python signals and timers (no PyQt6 event loop required in daemon)
os.environ["LOCATION_SIM_NO_QT"] = "1"

# Ensure unbuffered output
sys.stdout.reconfigure(line_buffering=True)
sys.stderr.reconfigure(line_buffering=True)

# Add project root to sys.path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from core import (
    DeviceManager,
    LocationController,
    SavedLocationsManager,
    TunnelManager,
    SettingsManager,
)
from core.device_manager import iOSDevice


def emit_event(event_name: str, data: any):
    """Emit an asynchronous event to the Rust UI over stdout"""
    try:
        msg = json.dumps({"event": event_name, "data": data})
        sys.stdout.write(msg + "\n")
        sys.stdout.flush()
    except Exception as e:
        sys.stderr.write(f"[Daemon] Error emitting event {event_name}: {e}\n")


def serialize_device(device: iOSDevice) -> dict:
    return {
        "udid": device.udid,
        "name": device.name,
        "model": device.model,
        "ios_version": device.ios_version,
        "connection_type": device.connection_type.value,
        "display_name": device.display_name,
        "icon_name": device.icon_name,
    }


class CoreDaemon:
    def __init__(self):
        self.device_manager = DeviceManager()
        self.location_controller = LocationController()
        self.saved_locations = SavedLocationsManager()
        self.tunnel_manager = TunnelManager()
        self.settings_manager = SettingsManager()

        self._setup_signals()

    def _setup_signals(self):
        # Device manager signals
        self.device_manager.devices_updated.connect(self._on_devices_updated)
        self.device_manager.error_occurred.connect(
            lambda err: emit_event("device_error", {"error": str(err)})
        )

        # Location controller signals
        self.location_controller.simulation_result.connect(
            lambda success, message: emit_event("simulation_result", {"success": success, "message": message})
        )
        self.location_controller.location_updated.connect(
            lambda lat, lon: emit_event("location_updated", {"latitude": lat, "longitude": lon})
        )
        self.location_controller.route_progress.connect(
            lambda cur, total: emit_event("route_progress", {"current": cur, "total": total})
        )
        self.location_controller.route_state_changed.connect(
            lambda state: emit_event("route_state_changed", {"state": state})
        )

        # Saved locations & routes signals
        self.saved_locations.locations_updated.connect(
            lambda: emit_event("saved_locations_updated", self._get_saved_locations())
        )
        self.saved_locations.routes_updated.connect(
            lambda: emit_event("gpx_routes_updated", self._get_gpx_routes())
        )

        # Tunnel manager signals
        self.tunnel_manager.tunnel_starting.connect(
            lambda: emit_event("tunnel_status", {"status": "starting", "running": False})
        )
        self.tunnel_manager.tunnel_started.connect(
            lambda: emit_event("tunnel_status", {"status": "started", "running": True})
        )
        self.tunnel_manager.tunnel_stopped.connect(
            lambda: emit_event("tunnel_status", {"status": "stopped", "running": False})
        )
        self.tunnel_manager.password_required.connect(
            lambda: emit_event("tunnel_status", {"status": "password_required", "running": False})
        )
        self.tunnel_manager.tunnel_error.connect(
            lambda err: emit_event("tunnel_status", {"status": "error", "error": str(err), "running": False})
        )

    def _on_devices_updated(self, devices):
        serialized = [serialize_device(d) for d in devices]
        has_devices = len(devices) > 0

        emit_event("devices_updated", {
            "devices": serialized,
            "selected_udid": self.device_manager.selected_device.udid if self.device_manager.selected_device else None
        })

        # Automatically stop tunnel if no device is connected; restart when connected
        self.tunnel_manager.set_device_connected(has_devices)

    def _get_saved_locations(self):
        return [loc.to_dict() for loc in self.saved_locations.locations]

    def _get_gpx_routes(self):
        return [route.to_dict() for route in self.saved_locations.routes]

    def dispatch(self, method: str, params: dict):
        if method == "get_devices":
            return {
                "devices": [serialize_device(d) for d in self.device_manager.devices],
                "selected_udid": self.device_manager.selected_device.udid if self.device_manager.selected_device else None
            }

        elif method == "select_device":
            udid = params.get("udid")
            found = False
            for dev in self.device_manager.devices:
                if dev.udid == udid:
                    self.device_manager.selected_device = dev
                    found = True
                    break
            return {"success": found, "selected_udid": udid if found else None}

        elif method == "refresh_devices":
            self.device_manager.refresh_devices()
            return {"success": True}

        elif method == "set_location":
            lat = float(params["latitude"])
            lon = float(params["longitude"])
            udid = params.get("udid")

            target_device = None
            if udid:
                for d in self.device_manager.devices:
                    if d.udid == udid:
                        target_device = d
                        break
            if not target_device:
                target_device = self.device_manager.selected_device

            if not target_device:
                return {"success": False, "message": "No iOS device selected"}

            # Asynchronously triggers simulation_result signal
            self.location_controller.set_location(target_device, lat, lon)
            return {"success": True, "message": "Simulation requested"}

        elif method == "clear_location":
            udid = params.get("udid")
            target_device = None
            if udid:
                for d in self.device_manager.devices:
                    if d.udid == udid:
                        target_device = d
                        break
            if not target_device:
                target_device = self.device_manager.selected_device

            if not target_device:
                return {"success": False, "message": "No iOS device selected"}

            self.location_controller.clear_location(target_device)
            return {"success": True, "message": "Clear requested"}

        elif method == "start_tunnel":
            emit_event("tunnel_status", {"status": "starting", "running": False})
            self.tunnel_manager._should_run = True
            threading.Thread(target=self.tunnel_manager.start_tunnel, daemon=True).start()
            return {"success": True}

        elif method == "submit_tunnel_password":
            pwd = params.get("password", "")
            remember = params.get("remember", True)
            emit_event("tunnel_status", {"status": "starting", "running": False})
            threading.Thread(target=lambda: self.tunnel_manager.provide_password(pwd, remember=remember), daemon=True).start()
            return {"success": True}

        elif method == "cancel_tunnel_password":
            self.tunnel_manager.cancel_password()
            return {"success": True}

        elif method == "stop_tunnel":
            self.tunnel_manager.stop_tunnel(manual=True)
            return {"success": True}

        elif method == "get_tunnel_status":
            return {
                "running": self.tunnel_manager.is_running,
                "is_starting": self.tunnel_manager._is_starting
            }

        elif method == "get_saved_locations":
            return self._get_saved_locations()

        elif method == "add_saved_location":
            name = params.get("name", "Saved Location")
            lat = float(params["latitude"])
            lon = float(params["longitude"])
            category = params.get("category", "other")
            loc = self.saved_locations.add_location(name, lat, lon, category)
            return loc.to_dict()

        elif method == "remove_saved_location":
            loc_id = params["id"]
            self.saved_locations.remove_location(loc_id)
            return {"success": True}

        elif method == "get_gpx_routes":
            return self._get_gpx_routes()

        elif method == "import_gpx":
            file_path = params["file_path"]
            route = self.saved_locations.import_gpx(file_path)
            return route.to_dict()

        elif method == "remove_gpx_route":
            route_id = params["id"]
            self.saved_locations.remove_route(route_id)
            return {"success": True}

        elif method == "start_route":
            route_id = params["route_id"]
            speed = float(params.get("speed", 1.0))
            loop = bool(params.get("loop", False))
            route = None
            for r in self.saved_locations.routes:
                if r.id == route_id:
                    route = r
                    break
            if not route:
                return {"success": False, "message": "Route not found"}

            target_device = self.device_manager.selected_device
            if not target_device:
                return {"success": False, "message": "No iOS device selected"}

            self.location_controller.set_speed(speed)
            self.location_controller.start_route(target_device, route, loop)
            return {"success": True}

        elif method == "pause_route":
            self.location_controller.pause_route()
            return {"success": True}

        elif method == "resume_route":
            self.location_controller.resume_route()
            return {"success": True}

        elif method == "stop_route":
            self.location_controller.stop_route()
            return {"success": True}

        elif method == "get_settings":
            return {
                "map_provider": self.settings_manager.get_map_provider(),
                "google_api_key": self.settings_manager.get_google_api_key(),
            }

        elif method == "set_setting":
            key = params["key"]
            val = params["value"]
            if key == "map_provider":
                self.settings_manager.set_map_provider(val)
            elif key == "google_api_key":
                self.settings_manager.set_google_api_key(val)
            return {"success": True}

        elif method == "ping":
            return "pong"

        else:
            raise ValueError(f"Unknown method: {method}")

    def run(self):
        # Start device discovery (every 3 seconds)
        self.device_manager.start_auto_refresh(3000)

        # Notify Rust that daemon is ready
        emit_event("ready", {
            "locations": self._get_saved_locations(),
            "routes": self._get_gpx_routes(),
            "settings": {
                "map_provider": self.settings_manager.get_map_provider(),
                "google_api_key": self.settings_manager.get_google_api_key(),
            }
        })

        # Emit initial tunnel status
        if self.tunnel_manager.is_running:
            emit_event("tunnel_status", {"status": "started", "running": True})
        else:
            emit_event("tunnel_status", {"status": "stopped", "running": False})

        # Process commands from stdin
        for line in sys.stdin:
            line = line.strip()
            if not line:
                continue

            try:
                req = json.loads(line)
                req_id = req.get("id")
                method = req.get("method")
                params = req.get("params", {})

                if method == "quit":
                    break

                try:
                    result = self.dispatch(method, params)
                    resp = {"id": req_id, "result": result}
                except Exception as err:
                    resp = {"id": req_id, "error": str(err), "traceback": traceback.format_exc()}

                sys.stdout.write(json.dumps(resp) + "\n")
                sys.stdout.flush()

            except Exception as e:
                err_resp = {"id": None, "error": f"Invalid request: {e}"}
                sys.stdout.write(json.dumps(err_resp) + "\n")
                sys.stdout.flush()

        self.device_manager.stop_auto_refresh()
        self.tunnel_manager.stop_tunnel()


if __name__ == "__main__":
    daemon = CoreDaemon()
    daemon.run()
