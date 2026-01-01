"""
Tunnel Manager - Manages RemoteXPC tunnel for iOS 17+ devices
"""

import subprocess
import logging
import threading
import sys
import time
import shutil
from typing import Optional

from PyQt6.QtCore import QObject, pyqtSignal

logger = logging.getLogger(__name__)


def _get_python_executable():
    """
    Get the correct Python executable.
    When running from PyInstaller bundle, sys.executable points to the app bundle.
    """
    if getattr(sys, 'frozen', False) and hasattr(sys, '_MEIPASS'):
        # Running in PyInstaller bundle - use system python3
        python_exe = shutil.which('python3')
        if not python_exe:
            raise RuntimeError("Python 3 not found in PATH")
        return python_exe
    else:
        # Running from source
        return sys.executable


class TunnelManager(QObject):
    """Manages the RemoteXPC tunnel process for iOS 17+ devices"""

    # Signals
    tunnel_started = pyqtSignal()
    tunnel_stopped = pyqtSignal()
    tunnel_error = pyqtSignal(str)
    password_required = pyqtSignal()  # Request password from GUI

    def __init__(self):
        super().__init__()
        self._process: Optional[subprocess.Popen] = None
        self._monitor_thread: Optional[threading.Thread] = None
        self._is_running = False
        self._is_starting = False  # Track if we're currently trying to start

    @property
    def is_running(self) -> bool:
        """Check if tunnel is running"""
        return self._is_running and self._process is not None and self._process.poll() is None

    def start_tunnel(self):
        """Start the RemoteXPC tunnel in background"""
        if self.is_running:
            logger.info("Tunnel already running")
            return

        if self._is_starting:
            logger.info("Tunnel start already in progress, skipping...")
            return

        self._is_starting = True

        try:
            logger.info("Starting RemoteXPC tunnel...")

            # Get python executable
            python_exe = _get_python_executable()

            # Check if we can run sudo without password (for better UX)
            # If not, the tunnel will fail and we'll show instructions
            cmd = ['sudo', '-n', python_exe, '-m', 'pymobiledevice3', 'remote', 'tunneld']

            # Start the process
            self._process = subprocess.Popen(
                cmd,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                stdin=subprocess.DEVNULL,
                text=True
            )

            # Give it a moment to start
            time.sleep(2)

            # Check if it's still running
            if self._process.poll() is None:
                self._is_running = True
                self._is_starting = False
                logger.info("Tunnel started successfully")
                self.tunnel_started.emit()

                # Start monitor thread
                self._monitor_thread = threading.Thread(target=self._monitor_tunnel, daemon=True)
                self._monitor_thread.start()
            else:
                # Process died immediately - likely needs password
                stderr = self._process.stderr.read() if self._process.stderr else ""
                logger.warning(f"Tunnel process exited immediately: {stderr}")

                # Request password from GUI
                logger.info("Requesting password from user...")
                self.password_required.emit()
                # Note: _is_starting will remain True until password is provided or cancelled

        except Exception as e:
            self._is_starting = False
            logger.error(f"Failed to start tunnel: {str(e)}")
            self.tunnel_error.emit(f"Failed to start tunnel: {str(e)}")

    def _start_tunnel_with_password(self, password: str):
        """Start tunnel with provided sudo password"""
        try:
            logger.info("Starting tunnel with password...")

            python_exe = _get_python_executable()

            # Build the command
            cmd = [python_exe, '-m', 'pymobiledevice3', 'remote', 'tunneld']

            # Use sudo with password via stdin
            sudo_cmd = ['sudo', '-S'] + cmd

            # Start the process
            self._process = subprocess.Popen(
                sudo_cmd,
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True
            )

            # Send password to sudo
            if self._process.stdin:
                self._process.stdin.write(password + '\n')
                self._process.stdin.flush()

            # Give it a moment to start
            time.sleep(2)

            # Check if it's still running
            if self._process.poll() is None:
                self._is_running = True
                self._is_starting = False
                logger.info("Tunnel started successfully with password")
                self.tunnel_started.emit()

                # Start monitor thread
                self._monitor_thread = threading.Thread(target=self._monitor_tunnel, daemon=True)
                self._monitor_thread.start()
            else:
                # Process died - likely wrong password or other error
                stderr = self._process.stderr.read() if self._process.stderr else ""
                self._is_starting = False
                logger.error(f"Tunnel failed to start: {stderr}")

                if "Sorry, try again" in stderr or "incorrect password" in stderr.lower():
                    self.tunnel_error.emit("Incorrect password. Please try again.")
                else:
                    self.tunnel_error.emit(f"Failed to start tunnel: {stderr}")

        except Exception as e:
            self._is_starting = False
            logger.error(f"Failed to start tunnel with password: {str(e)}")
            self.tunnel_error.emit(f"Failed to start tunnel: {str(e)}")

    def provide_password(self, password: str):
        """Called by GUI when user provides password"""
        if not self._is_starting:
            logger.warning("provide_password called but tunnel not starting")
            return

        self._start_tunnel_with_password(password)

    def cancel_password(self):
        """Called by GUI when user cancels password prompt"""
        self._is_starting = False
        logger.info("Tunnel start cancelled by user")
        self.tunnel_error.emit("Tunnel start cancelled")

    def _monitor_tunnel(self):
        """Monitor tunnel process in background"""
        if not self._process:
            return

        try:
            # Wait for process to exit
            self._process.wait()

            logger.info("Tunnel process exited")
            self._is_running = False
            self.tunnel_stopped.emit()

        except Exception as e:
            logger.error(f"Error monitoring tunnel: {str(e)}")

    def stop_tunnel(self):
        """Stop the tunnel process"""
        if not self._process:
            return

        try:
            logger.info("Stopping tunnel...")

            if self._process.poll() is None:
                self._process.terminate()

                # Wait for graceful shutdown
                try:
                    self._process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    logger.warning("Tunnel didn't stop gracefully, killing...")
                    self._process.kill()

            self._is_running = False
            self._process = None
            self.tunnel_stopped.emit()

            logger.info("Tunnel stopped")

        except Exception as e:
            logger.error(f"Error stopping tunnel: {str(e)}")

    def check_tunnel_accessibility(self) -> bool:
        """Check if tunnel is accessible via HTTP"""
        try:
            import urllib.request
            urllib.request.urlopen('http://127.0.0.1:49151/', timeout=2)
            return True
        except:
            return False
