"""
Location Controller - Handles location simulation using pymobiledevice3
"""

import threading
import time
import logging
import traceback
import asyncio
import subprocess
import json
import sys
import os
from dataclasses import dataclass
from typing import Optional, Callable, List, Tuple
from enum import Enum

from PyQt6.QtCore import QObject, pyqtSignal, QTimer

from .device_manager import iOSDevice


def _get_python_executable():
    """
    Get the correct Python executable.

    When running from PyInstaller bundle, sys.executable points to the app bundle,
    not Python. We need to use the bundled Python or call pymobiledevice3 directly.
    """
    # Check if running in PyInstaller bundle
    if getattr(sys, 'frozen', False) and hasattr(sys, '_MEIPASS'):
        # Running in PyInstaller bundle
        # Don't use sys.executable (it's the app), use python3 from system
        # or better yet, import and call pymobiledevice3 directly
        return None  # Signal to use direct import instead of subprocess
    else:
        # Running from source - use current Python
        return sys.executable

# Setup logging
logging.basicConfig(
    level=logging.DEBUG,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


@dataclass
class Coordinate:
    """Represents a geographic coordinate"""
    latitude: float
    longitude: float
    
    def __str__(self):
        return f"{self.latitude:.6f}, {self.longitude:.6f}"


@dataclass 
class SimulationResult:
    """Result of a location simulation operation"""
    success: bool
    message: str


class RouteSimulationState(Enum):
    IDLE = "idle"
    RUNNING = "running"
    PAUSED = "paused"
    COMPLETED = "completed"


class LocationController(QObject):
    """Controls location simulation on iOS devices"""
    
    # Signals
    simulation_result = pyqtSignal(bool, str)  # success, message
    location_updated = pyqtSignal(float, float)  # lat, lon
    route_progress = pyqtSignal(int, int)  # current, total
    route_state_changed = pyqtSignal(str)  # state name
    
    def __init__(self):
        super().__init__()
        self._is_simulating = False
        self._current_location: Optional[Coordinate] = None
        self._route_state = RouteSimulationState.IDLE
        self._route_thread: Optional[threading.Thread] = None
        self._stop_route = False
        self._pause_route = False
        self._speed = 1.0

        # For iOS 17+ - keep track of the subprocess that maintains the connection
        self._ios17_process: Optional[subprocess.Popen] = None
        self._ios17_udid: Optional[str] = None
        
    @property
    def is_simulating(self) -> bool:
        return self._is_simulating
    
    @property
    def current_location(self) -> Optional[Coordinate]:
        return self._current_location
    
    @property
    def route_state(self) -> RouteSimulationState:
        return self._route_state
        
    def set_location(self, device: iOSDevice, latitude: float, longitude: float):
        """Set simulated location on device"""
        thread = threading.Thread(
            target=self._set_location_thread,
            args=(device, latitude, longitude)
        )
        thread.daemon = True
        thread.start()
        
    def _set_location_thread(self, device: iOSDevice, latitude: float, longitude: float):
        """Background thread to set location"""
        try:
            logger.info(f"Setting location on device {device.name} ({device.udid}): {latitude}, {longitude}")
            result = self._simulate_location(device.udid, latitude, longitude)

            if result.success:
                self._current_location = Coordinate(latitude, longitude)
                self._is_simulating = True
                self.location_updated.emit(latitude, longitude)
                logger.info(f"Location successfully set to {latitude}, {longitude}")
            else:
                logger.error(f"Failed to set location: {result.message}")

            self.simulation_result.emit(result.success, result.message)

        except Exception as e:
            error_trace = traceback.format_exc()
            logger.error(f"Exception in _set_location_thread: {str(e)}\n{error_trace}")
            self.simulation_result.emit(False, f"Error: {str(e)}")
            
    def clear_location(self, device: iOSDevice):
        """Clear simulated location on device"""
        thread = threading.Thread(
            target=self._clear_location_thread,
            args=(device,)
        )
        thread.daemon = True
        thread.start()
        
    def _clear_location_thread(self, device: iOSDevice):
        """Background thread to clear location"""
        try:
            result = self._clear_simulated_location(device.udid)
            
            if result.success:
                self._current_location = None
                self._is_simulating = False
                
            self.simulation_result.emit(result.success, result.message)
            
        except Exception as e:
            self.simulation_result.emit(False, f"Error: {str(e)}")
    
    def _simulate_location(self, udid: str, latitude: float, longitude: float) -> SimulationResult:
        """Simulate location using pymobiledevice3"""
        try:
            logger.debug(f"Importing pymobiledevice3 modules...")
            from pymobiledevice3.lockdown import create_using_usbmux
            from pymobiledevice3.services.dvt.dvt_secure_socket_proxy import DvtSecureSocketProxyService
            from pymobiledevice3.services.dvt.instruments.location_simulation import LocationSimulation

            # Connect to device
            logger.debug(f"Connecting to device with UDID: {udid}")
            lockdown = create_using_usbmux(serial=udid)
            logger.debug(f"Lockdown connection established: {lockdown}")

            # Get iOS version to determine approach
            ios_version_str = lockdown.all_values.get('ProductVersion', '0.0')
            ios_major_version = int(ios_version_str.split('.')[0])
            logger.debug(f"iOS version detected: {ios_version_str} (major: {ios_major_version})")

            # For iOS 17+, use RemoteXPC tunnel approach
            if ios_major_version >= 17:
                logger.info(f"iOS 17+ detected - using RemoteXPC tunnel approach")

                # Get tunnel info
                tunnel_info = self._get_tunnel_info(udid)
                if not tunnel_info:
                    return SimulationResult(
                        success=False,
                        message=f"❌ RemoteXPC Tunnel Not Running\n\n"
                                f"iOS {ios_version_str} requires a RemoteXPC tunnel.\n\n"
                                f"Please start the tunnel first:\n"
                                f"1. Open a terminal window\n"
                                f"2. Run: ./start_tunnel.sh\n"
                                f"3. Keep that terminal running\n"
                                f"4. Try setting location again\n\n"
                                f"The tunnel must stay running while using location simulation."
                    )

                tunnel_addr, tunnel_port = tunnel_info

                # Stop any existing iOS 17+ simulation first
                self._stop_ios17_simulation()

                # Start the location simulation process (keeps connection open)
                result = self._start_ios17_simulation(udid, tunnel_addr, tunnel_port, latitude, longitude)
                return result

            # For iOS 16 and below, use direct lockdown approach
            logger.debug(f"Using direct lockdown approach for iOS {ios_major_version}")

            # Start location simulation
            logger.debug(f"Creating DvtSecureSocketProxyService...")
            with DvtSecureSocketProxyService(lockdown) as dvt:
                logger.debug(f"DVT service created, simulating location...")
                LocationSimulation(dvt).simulate_location(latitude, longitude)
                logger.debug(f"Location simulation command sent successfully")

            return SimulationResult(
                success=True,
                message=f"✅ Location set to {latitude:.4f}, {longitude:.4f}"
            )

        except ImportError as e:
            logger.error(f"Import error: {str(e)}\n{traceback.format_exc()}")
            return SimulationResult(
                success=False,
                message=f"❌ pymobiledevice3 not installed or import failed: {str(e)}"
            )
        except Exception as e:
            error_msg = str(e)
            error_trace = traceback.format_exc()
            logger.error(f"Exception in _simulate_location: {error_msg}\n{error_trace}")

            # Handle common errors
            if "DeveloperMode" in error_msg or "developer" in error_msg.lower():
                return SimulationResult(
                    success=False,
                    message=f"❌ Developer Mode not enabled on device\nDetails: {error_msg}"
                )
            elif "pairing" in error_msg.lower():
                return SimulationResult(
                    success=False,
                    message=f"❌ Device not paired. Trust this computer on your iPhone\nDetails: {error_msg}"
                )
            elif "not found" in error_msg.lower():
                return SimulationResult(
                    success=False,
                    message=f"❌ Device not found. Reconnect and try again\nDetails: {error_msg}"
                )
            elif "InvalidService" in error_msg or "Invalid Service" in error_msg:
                return SimulationResult(
                    success=False,
                    message=f"❌ Invalid Service - Developer Mode may not be enabled or device is not in Developer Mode\nDetails: {error_msg}"
                )
            else:
                return SimulationResult(
                    success=False,
                    message=f"❌ {error_msg}\n\nFull error: {error_trace}"
                )
    
    def _clear_simulated_location(self, udid: str) -> SimulationResult:
        """Clear simulated location using pymobiledevice3"""
        try:
            logger.debug(f"Clearing location simulation for device: {udid}")
            from pymobiledevice3.lockdown import create_using_usbmux
            from pymobiledevice3.services.dvt.dvt_secure_socket_proxy import DvtSecureSocketProxyService
            from pymobiledevice3.services.dvt.instruments.location_simulation import LocationSimulation

            # Connect to device
            logger.debug(f"Connecting to device with UDID: {udid}")
            lockdown = create_using_usbmux(serial=udid)

            # Get iOS version to determine approach
            ios_version_str = lockdown.all_values.get('ProductVersion', '0.0')
            ios_major_version = int(ios_version_str.split('.')[0])
            logger.debug(f"iOS version detected: {ios_version_str} (major: {ios_major_version})")

            # For iOS 17+, use RemoteXPC tunnel approach
            if ios_major_version >= 17:
                logger.info(f"iOS 17+ detected - using RemoteXPC tunnel approach")

                # Get tunnel info
                tunnel_info = self._get_tunnel_info(udid)
                if not tunnel_info:
                    return SimulationResult(
                        success=False,
                        message=f"❌ RemoteXPC Tunnel Not Running\n\n"
                                f"iOS {ios_version_str} requires a RemoteXPC tunnel.\n\n"
                                f"Please start the tunnel first:\n"
                                f"1. Open a terminal window\n"
                                f"2. Run: ./start_tunnel.sh\n"
                                f"3. Keep that terminal running\n"
                                f"4. Try clearing location again\n\n"
                                f"The tunnel must stay running while using location simulation."
                    )

                # For iOS 17+, just stop the running simulation process
                self._stop_ios17_simulation()

                logger.info(f"iOS 17+ location cleared by stopping simulation process")
                return SimulationResult(
                    success=True,
                    message="✅ Simulated location cleared"
                )

            # For iOS 16 and below, use direct lockdown approach
            logger.debug(f"Using direct lockdown approach for iOS {ios_major_version}")

            # Clear location simulation
            logger.debug(f"Creating DvtSecureSocketProxyService...")
            with DvtSecureSocketProxyService(lockdown) as dvt:
                logger.debug(f"Clearing location simulation...")
                LocationSimulation(dvt).clear()
                logger.debug(f"Location simulation cleared successfully")

            return SimulationResult(
                success=True,
                message="✅ Simulated location cleared"
            )

        except Exception as e:
            error_trace = traceback.format_exc()
            logger.error(f"Failed to clear location: {str(e)}\n{error_trace}")

            # Handle InvalidService error specifically
            error_msg = str(e)
            if "InvalidService" in error_msg or "Invalid Service" in error_msg:
                return SimulationResult(
                    success=False,
                    message=f"❌ Invalid Service - This likely means:\n\n"
                            f"1. For iOS 17+: You need to start RemoteXPC tunnel (see instructions above)\n"
                            f"2. Developer Mode is not enabled on device\n"
                            f"3. Device needs to be re-paired with this computer\n\n"
                            f"Details: {error_msg}"
                )

            return SimulationResult(
                success=False,
                message=f"❌ Failed to clear location: {str(e)}\n\nFull error: {error_trace}"
            )
    
    # Route simulation methods
    
    def start_route(self, device: iOSDevice, waypoints: List[Coordinate], speed: float = 1.0):
        """Start simulating movement along a route"""
        self.stop_route()  # Stop any existing route
        
        self._speed = speed
        self._stop_route = False
        self._pause_route = False
        self._route_state = RouteSimulationState.RUNNING
        self.route_state_changed.emit(self._route_state.value)
        
        self._route_thread = threading.Thread(
            target=self._run_route_thread,
            args=(device, waypoints)
        )
        self._route_thread.daemon = True
        self._route_thread.start()
        
    def _run_route_thread(self, device: iOSDevice, waypoints: List[Coordinate]):
        """Background thread for route simulation"""
        total = len(waypoints)
        
        for i, waypoint in enumerate(waypoints):
            if self._stop_route:
                break
                
            # Handle pause
            while self._pause_route and not self._stop_route:
                time.sleep(0.1)
                
            if self._stop_route:
                break
                
            # Simulate location
            result = self._simulate_location(device.udid, waypoint.latitude, waypoint.longitude)
            
            if result.success:
                self._current_location = waypoint
                self.location_updated.emit(waypoint.latitude, waypoint.longitude)
                self.route_progress.emit(i + 1, total)
            else:
                self.simulation_result.emit(False, result.message)
                
            # Wait between points (adjusted by speed)
            time.sleep(1.0 / self._speed)
            
        # Completed or stopped
        if not self._stop_route:
            self._route_state = RouteSimulationState.COMPLETED
        else:
            self._route_state = RouteSimulationState.IDLE
            
        self.route_state_changed.emit(self._route_state.value)
        
    def pause_route(self):
        """Pause route simulation"""
        if self._route_state == RouteSimulationState.RUNNING:
            self._pause_route = True
            self._route_state = RouteSimulationState.PAUSED
            self.route_state_changed.emit(self._route_state.value)
            
    def resume_route(self):
        """Resume route simulation"""
        if self._route_state == RouteSimulationState.PAUSED:
            self._pause_route = False
            self._route_state = RouteSimulationState.RUNNING
            self.route_state_changed.emit(self._route_state.value)
            
    def stop_route(self):
        """Stop route simulation"""
        self._stop_route = True
        self._pause_route = False
        
        if self._route_thread and self._route_thread.is_alive():
            self._route_thread.join(timeout=1.0)
            
        self._route_state = RouteSimulationState.IDLE
        self.route_state_changed.emit(self._route_state.value)
        
    def set_speed(self, speed: float):
        """Set route simulation speed"""
        self._speed = max(0.1, min(20.0, speed))

    # iOS 17+ helper methods

    def _get_tunnel_info(self, udid: str) -> Optional[Tuple[str, int]]:
        """Get tunnel address and port for a device from tunneld service"""
        try:
            import urllib.request

            # Query tunneld service
            response = urllib.request.urlopen('http://127.0.0.1:49151/', timeout=5)
            data = json.loads(response.read().decode('utf-8'))

            # Find device tunnel info
            if udid in data:
                device_tunnels = data[udid]
                if device_tunnels and len(device_tunnels) > 0:
                    tunnel_info = device_tunnels[0]
                    tunnel_addr = tunnel_info.get('tunnel-address')
                    tunnel_port = tunnel_info.get('tunnel-port')

                    if tunnel_addr and tunnel_port:
                        logger.debug(f"Found tunnel for {udid}: {tunnel_addr}:{tunnel_port}")
                        return (tunnel_addr, tunnel_port)

            logger.error(f"No tunnel found for device {udid}")
            return None

        except Exception as e:
            logger.error(f"Failed to get tunnel info: {str(e)}")
            return None

    def _run_pymobiledevice3_command(self, args: List[str], timeout: int = 3) -> Tuple[bool, str]:
        """Run a pymobiledevice3 command using subprocess

        Note: The location simulation commands keep connection open, so we start them
        and kill after a short delay. The location stays set even after killing the process.
        """
        try:
            # Get the python executable
            python_exe = _get_python_executable()

            if python_exe is None:
                # Running in PyInstaller bundle - use system python3
                # Look for python3 in PATH
                import shutil
                python_exe = shutil.which('python3')
                if not python_exe:
                    raise RuntimeError("Python 3 not found in PATH. Please install Python 3.")

            # Build command
            cmd = [python_exe, '-m', 'pymobiledevice3'] + args

            logger.debug(f"Running command: {' '.join(cmd)}")

            # Start the process
            process = subprocess.Popen(
                cmd,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True
            )

            # Give it time to execute the command (location simulation happens quickly)
            # but the process stays connected, so we'll kill it after a delay
            time.sleep(timeout)

            # Check if there was an immediate error
            if process.poll() is not None:
                # Process already exited (likely an error)
                stdout, stderr = process.communicate()
                logger.debug(f"Command exit code: {process.returncode}")
                logger.debug(f"Command stdout: {stdout}")
                logger.debug(f"Command stderr: {stderr}")

                if process.returncode == 0:
                    return (True, stdout)
                else:
                    return (False, stderr or stdout)

            # Process is still running (normal for location simulation)
            # The command has executed, so terminate the process
            logger.debug("Command still running (normal), terminating connection...")
            process.terminate()

            try:
                stdout, stderr = process.communicate(timeout=2)
            except subprocess.TimeoutExpired:
                process.kill()
                stdout, stderr = process.communicate()

            logger.debug(f"Command output: {stdout}")
            if stderr:
                # Check if stderr contains actual errors or just warnings
                if "InvalidService" in stderr or "Error" in stderr or "Failed" in stderr:
                    logger.error(f"Command stderr: {stderr}")
                    return (False, stderr)
                else:
                    logger.debug(f"Command stderr (non-critical): {stderr}")

            # If we got here, the command executed successfully
            return (True, stdout)

        except Exception as e:
            logger.error(f"Failed to run command: {str(e)}\n{traceback.format_exc()}")
            return (False, str(e))

    def _start_ios17_simulation(self, udid: str, tunnel_addr: str, tunnel_port: int,
                                latitude: float, longitude: float) -> SimulationResult:
        """Start iOS 17+ location simulation and keep the process running"""
        try:
            # Get python executable
            python_exe = _get_python_executable()

            if python_exe is None:
                # Running in PyInstaller bundle - use system python3
                import shutil
                python_exe = shutil.which('python3')
                if not python_exe:
                    raise RuntimeError("Python 3 not found in PATH. Please install Python 3.")

            # Build command
            # Use '--' to separate options from positional args (tunnel_addr contains ::1 which looks like -1)
            cmd = [
                python_exe, '-m', 'pymobiledevice3',
                'developer', 'dvt', 'simulate-location', 'set',
                '--rsd', tunnel_addr, str(tunnel_port),
                '--',
                str(latitude), str(longitude)
            ]

            logger.info(f"Starting iOS 17+ location simulation process...")
            logger.debug(f"Command: {' '.join(cmd)}")

            # Start process and keep it running
            self._ios17_process = subprocess.Popen(
                cmd,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True
            )

            self._ios17_udid = udid

            # Give it a moment to start and check for immediate errors
            time.sleep(2)

            if self._ios17_process.poll() is not None:
                # Process died immediately - error occurred
                stdout, stderr = self._ios17_process.communicate()
                logger.error(f"iOS 17+ simulation failed: {stderr or stdout}")
                self._ios17_process = None
                self._ios17_udid = None
                return SimulationResult(
                    success=False,
                    message=f"❌ Failed to start location simulation\n\n{stderr or stdout}"
                )

            # Process is running - simulation is active
            logger.info(f"iOS 17+ location simulation active (process running in background)")
            self._is_simulating = True
            self._current_location = Coordinate(latitude, longitude)

            return SimulationResult(
                success=True,
                message=f"✅ Location set to {latitude:.4f}, {longitude:.4f}\n"
                        f"(Connection kept open - location will stay set until cleared)"
            )

        except Exception as e:
            logger.error(f"Failed to start iOS 17+ simulation: {str(e)}\n{traceback.format_exc()}")
            self._ios17_process = None
            self._ios17_udid = None
            return SimulationResult(
                success=False,
                message=f"❌ Failed to start simulation: {str(e)}"
            )

    def _stop_ios17_simulation(self):
        """Stop iOS 17+ location simulation by terminating the process"""
        if self._ios17_process is None:
            return

        try:
            logger.info("Stopping iOS 17+ location simulation...")

            if self._ios17_process.poll() is None:
                # Process is still running, terminate it
                self._ios17_process.terminate()

                try:
                    self._ios17_process.wait(timeout=3)
                except subprocess.TimeoutExpired:
                    logger.warning("Process didn't terminate, killing it...")
                    self._ios17_process.kill()
                    self._ios17_process.wait()

            logger.info("iOS 17+ location simulation stopped")

        except Exception as e:
            logger.error(f"Error stopping iOS 17+ simulation: {str(e)}")

        finally:
            self._ios17_process = None
            self._ios17_udid = None
            self._is_simulating = False
            self._current_location = None
