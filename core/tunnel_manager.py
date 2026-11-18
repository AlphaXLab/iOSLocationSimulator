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

from .qt_compat import QObject, pyqtSignal

logger = logging.getLogger(__name__)


def _get_keychain_password() -> Optional[str]:
    """Retrieve saved tunnel password from macOS Keychain"""
    if sys.platform != 'darwin':
        return None
    try:
        res = subprocess.run(
            ['/usr/bin/security', 'find-generic-password', '-s', 'LocationSimulator-Tunnel', '-w'],
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
            timeout=2
        )
        if res.returncode == 0:
            pwd = res.stdout.strip()
            if pwd:
                return pwd
    except Exception:
        pass
    return None


def _save_keychain_password(password: str):
    """Store tunnel password securely in macOS Keychain"""
    if sys.platform != 'darwin' or not password:
        return
    import getpass
    user = getpass.getuser()
    try:
        subprocess.run(
            ['/usr/bin/security', 'add-generic-password', '-U', '-s', 'LocationSimulator-Tunnel', '-a', user, '-w', password],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False
        )
    except Exception as e:
        logger.warning(f"Failed to save password to keychain: {e}")


def _clear_keychain_password():
    """Remove saved tunnel password from macOS Keychain"""
    if sys.platform != 'darwin':
        return
    try:
        subprocess.run(
            ['/usr/bin/security', 'delete-generic-password', '-s', 'LocationSimulator-Tunnel'],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False
        )
    except Exception:
        pass


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


def is_tunneld_listening(port: int = 49151) -> bool:
    """Check if pymobiledevice3 tunneld is already listening on local port"""
    import socket
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.settimeout(0.5)
    try:
        return s.connect_ex(('127.0.0.1', port)) == 0
    except Exception:
        return False
    finally:
        s.close()


class TunnelManager(QObject):
    """Manages the RemoteXPC tunnel process for iOS 17+ devices"""

    # Signals
    tunnel_starting = pyqtSignal()
    tunnel_started = pyqtSignal()
    tunnel_stopped = pyqtSignal()
    tunnel_error = pyqtSignal(str)
    password_required = pyqtSignal()  # Request password from GUI

    def __init__(self):
        super().__init__()
        self._process: Optional[subprocess.Popen] = None
        self._monitor_thread: Optional[threading.Thread] = None
        self._is_running = is_tunneld_listening()
        self._is_starting = False  # Track if we're currently trying to start
        self._device_connected = False
        self._should_run = False
        self._restart_timer: Optional[threading.Timer] = None
        self._restart_lock = threading.Lock()

    @property
    def is_running(self) -> bool:
        """Check if tunnel is running"""
        if self._is_running and self._process is not None and self._process.poll() is None:
            return True
        return is_tunneld_listening()

    def set_device_connected(self, connected: bool):
        """Update device connection status. Stop tunnel when disconnected; start when connected."""
        with self._restart_lock:
            prev_connected = self._device_connected
            self._device_connected = connected

            if not connected:
                logger.info("No iOS device connected. Stopping RemoteXPC tunnel...")
                self._should_run = False
                self._cancel_restart_timer()
                if self.is_running or self._is_starting or self._process is not None:
                    threading.Thread(target=self._stop_internal, daemon=True).start()
            else:
                self._should_run = True
                if not self.is_running and not self._is_starting:
                    logger.info("iOS device connected. Starting RemoteXPC tunnel...")
                    threading.Thread(target=self.start_tunnel, daemon=True).start()

    def _cancel_restart_timer(self):
        """Cancel any scheduled auto-restart timer"""
        if self._restart_timer:
            try:
                self._restart_timer.cancel()
            except Exception:
                pass
            self._restart_timer = None

    def _schedule_auto_restart(self, delay: float = 2.0):
        """Auto restart tunnel when an issue occurs, instead of displaying error"""
        with self._restart_lock:
            self._cancel_restart_timer()
            if not self._device_connected or not self._should_run:
                self._is_starting = False
                return

            self._is_starting = True
            self.tunnel_starting.emit()

            def _do_restart():
                with self._restart_lock:
                    if not self._device_connected or not self._should_run:
                        self._is_starting = False
                        return
                    logger.info("Executing auto-restart of RemoteXPC tunnel...")
                    self._cleanup_process()
                    time.sleep(1.0)
                    self.start_tunnel()

            self._restart_timer = threading.Timer(delay, _do_restart)
            self._restart_timer.daemon = True
            self._restart_timer.start()

    def _cleanup_process(self):
        """Terminate active tunnel process and kill any orphaned tunneld"""
        if self._process:
            try:
                if self._process.poll() is None:
                    self._process.terminate()
                    self._process.wait(timeout=2)
            except Exception:
                try:
                    self._process.kill()
                except Exception:
                    pass
        self._process = None

        if is_tunneld_listening():
            saved_pwd = _get_keychain_password()
            if saved_pwd:
                subprocess.run(
                    ['sudo', '-S', 'pkill', '-f', 'pymobiledevice3 remote tunneld'],
                    input=saved_pwd + '\n',
                    text=True,
                    capture_output=True,
                    timeout=4
                )
            else:
                subprocess.run(['pkill', '-f', 'pymobiledevice3 remote tunneld'], capture_output=True, timeout=4)

    def _format_clean_error(self, stderr: str) -> str:
        """Extract a concise single-line error message from process stderr"""
        if not stderr:
            return "Tunnel failed to start"
        lines = [line.strip() for line in stderr.splitlines() if line.strip()]
        error_lines = [
            l for l in lines 
            if l.startswith("ERROR:") or "Error" in l or "Errno" in l or "Exception" in l or "failed" in l.lower()
        ]
        if error_lines:
            last_err = error_lines[-1]
            if last_err.startswith("ERROR:"):
                last_err = last_err[6:].strip()
            return last_err
        for l in reversed(lines):
            if not l.startswith("INFO:") and not l.startswith("WARNING:") and "predates altIRK" not in l:
                return l
        return lines[-1] if lines else "Tunnel failed to start"

    def start_tunnel(self):
        """Start the RemoteXPC tunnel in background"""
        if not self._device_connected:
            logger.info("Cannot start tunnel: No iOS device connected")
            self._is_starting = False
            return

        self._should_run = True

        if self.is_running or is_tunneld_listening():
            logger.info("Tunnel already running on localhost:49151")
            self._is_running = True
            self._is_starting = False
            self.tunnel_started.emit()
            return

        if self._is_starting:
            logger.info("Tunnel start already in progress, skipping...")
            return

        self._is_starting = True
        self.tunnel_starting.emit()

        try:
            logger.info("Starting RemoteXPC tunnel...")

            # Get python executable
            python_exe = _get_python_executable()

            # Check if we can run sudo without password (for better UX)
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
                logger.info("Tunnel started successfully (passwordless)")
                self.tunnel_started.emit()

                # Start monitor thread
                self._monitor_thread = threading.Thread(target=self._monitor_tunnel, daemon=True)
                self._monitor_thread.start()
                return

            # If died, check if it failed because tunneld is ALREADY listening on port 49151
            stderr = self._process.stderr.read() if self._process.stderr else ""
            if "address already in use" in stderr.lower() or "errno 48" in stderr.lower() or is_tunneld_listening():
                self._is_running = True
                self._is_starting = False
                logger.info("Tunnel daemon already running on port 49151")
                self.tunnel_started.emit()
                return

            # Check if a password is saved in macOS Keychain
            saved_pwd = _get_keychain_password()
            if saved_pwd:
                logger.info("Found saved password in macOS Keychain, trying auto-authentication...")
                if self._start_tunnel_with_password(saved_pwd):
                    return

            # If passwordless sudo failed and no valid saved password - request password from GUI
            self._is_starting = False
            logger.info("Requesting password from user...")
            self.password_required.emit()

        except Exception as e:
            logger.error(f"Failed to start tunnel: {str(e)}")
            if self._device_connected and self._should_run:
                logger.info("Tunnel startup encountered an issue. Auto-restarting...")
                self._schedule_auto_restart(delay=2.0)
            else:
                self._is_starting = False
                self.tunnel_stopped.emit()

    def _start_tunnel_with_password(self, password: str) -> bool:
        """Start tunnel with provided sudo password. Returns True on success, False on failure."""
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
                return True
            else:
                stderr = self._process.stderr.read() if self._process.stderr else ""
                logger.warning(f"Tunnel process failed with password: {stderr}")

                if "address already in use" in stderr.lower() or "errno 48" in stderr.lower() or is_tunneld_listening():
                    self._is_running = True
                    self._is_starting = False
                    logger.info("Tunnel daemon already running on port 49151")
                    self.tunnel_started.emit()
                    return True

                if "Sorry, try again" in stderr or "incorrect password" in stderr.lower():
                    # Invalid password: user must re-enter
                    self._is_starting = False
                    _clear_keychain_password()
                    self.password_required.emit()
                    return False

                # Any other issue: auto-restart instead of displaying error
                if self._device_connected and self._should_run:
                    logger.info("Tunnel encountered an issue with password. Auto-restarting...")
                    self._schedule_auto_restart(delay=2.0)
                    return False
                else:
                    self._is_starting = False
                    self.tunnel_stopped.emit()
                    return False

        except Exception as e:
            logger.error(f"Failed to start tunnel with password: {str(e)}")
            if self._device_connected and self._should_run:
                self._schedule_auto_restart(delay=2.0)
            else:
                self._is_starting = False
                self.tunnel_stopped.emit()
            return False

    def provide_password(self, password: str, remember: bool = True):
        """Called by GUI when user provides password"""
        self._is_starting = True
        self.tunnel_starting.emit()
        success = self._start_tunnel_with_password(password)
        if success and remember:
            _save_keychain_password(password)

    def cancel_password(self):
        """Called by GUI when user cancels password prompt"""
        self._is_starting = False
        self._should_run = False
        self._cancel_restart_timer()
        logger.info("Tunnel start cancelled by user")
        self.tunnel_stopped.emit()

    def _monitor_tunnel(self):
        """Monitor tunnel process in background"""
        if not self._process:
            return

        try:
            # Wait for process to exit
            self._process.wait()

            logger.info("Tunnel process exited")
            self._is_running = False

            if self._device_connected and self._should_run:
                logger.info("Tunnel exited unexpectedly while device is connected. Auto-restarting...")
                self._schedule_auto_restart(delay=2.0)
            else:
                self.tunnel_stopped.emit()

        except Exception as e:
            logger.error(f"Error monitoring tunnel: {str(e)}")
            if self._device_connected and self._should_run:
                self._schedule_auto_restart(delay=2.0)
            else:
                self.tunnel_stopped.emit()

    def _stop_internal(self):
        """Internal helper to stop the tunnel process and cleanup"""
        self._cancel_restart_timer()
        self._cleanup_process()
        self._is_running = False
        self._is_starting = False
        self.tunnel_stopped.emit()
        logger.info("RemoteXPC tunnel stopped")

    def stop_tunnel(self, manual: bool = False):
        """Stop the tunnel process"""
        try:
            logger.info("Stopping RemoteXPC tunnel...")
            if manual:
                self._should_run = False
            self._stop_internal()
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
