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

    @property
    def is_running(self) -> bool:
        """Check if tunnel is running"""
        if self._is_running and self._process is not None and self._process.poll() is None:
            return True
        return is_tunneld_listening()

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
                else:
                    logger.warning("Saved Keychain password failed, clearing from Keychain...")
                    _clear_keychain_password()

            # Process died immediately and no valid saved password - request password from GUI
            self._is_starting = False
            logger.info("Requesting password from user...")
            self.password_required.emit()

        except Exception as e:
            self._is_starting = False
            logger.error(f"Failed to start tunnel: {str(e)}")
            self.tunnel_error.emit(f"Failed to start tunnel: {str(e)}")

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
                # Process died - likely wrong password or other error
                stderr = self._process.stderr.read() if self._process.stderr else ""
                self._is_starting = False
                logger.error(f"Tunnel failed to start: {stderr}")

                if "address already in use" in stderr.lower() or "errno 48" in stderr.lower() or is_tunneld_listening():
                    self._is_running = True
                    logger.info("Tunnel daemon already running on port 49151")
                    self.tunnel_started.emit()
                    return True

                if "Sorry, try again" in stderr or "incorrect password" in stderr.lower():
                    self.tunnel_error.emit("Incorrect password. Please try again.")
                else:
                    self.tunnel_error.emit(self._format_clean_error(stderr))
                return False

        except Exception as e:
            self._is_starting = False
            logger.error(f"Failed to start tunnel with password: {str(e)}")
            self.tunnel_error.emit(f"Failed to start tunnel: {str(e)}")
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
        try:
            logger.info("Stopping tunnel...")

            if self._process and self._process.poll() is None:
                self._process.terminate()
                try:
                    self._process.wait(timeout=3)
                except subprocess.TimeoutExpired:
                    logger.warning("Tunnel didn't stop gracefully, killing...")
                    self._process.kill()

            if is_tunneld_listening():
                saved_pwd = _get_keychain_password()
                if saved_pwd:
                    subprocess.run(
                        ['sudo', '-S', 'pkill', '-f', 'pymobiledevice3 remote tunneld'],
                        input=saved_pwd + '\n',
                        text=True,
                        capture_output=True,
                        timeout=5
                    )
                else:
                    subprocess.run(['pkill', '-f', 'pymobiledevice3 remote tunneld'], capture_output=True, timeout=5)

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
