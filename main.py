#!/usr/bin/env python3
"""
LocationSimulator - iPhone Location Simulation Tool
Uses pymobiledevice3 for reliable location simulation on iOS devices.
"""

import sys
import os

# Enable Qt WebEngine debugging
os.environ['QTWEBENGINE_REMOTE_DEBUGGING'] = '9222'
os.environ['QTWEBENGINE_CHROMIUM_FLAGS'] = '--enable-logging --log-level=0'

# Ensure we can find our modules
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from PyQt6.QtWidgets import QApplication
from PyQt6.QtCore import Qt
from PyQt6.QtGui import QIcon

from ui.main_window import MainWindow


def main():
    # Enable high DPI scaling
    QApplication.setHighDpiScaleFactorRoundingPolicy(
        Qt.HighDpiScaleFactorRoundingPolicy.PassThrough
    )
    
    app = QApplication(sys.argv)
    app.setApplicationName("LocationSimulator")
    app.setApplicationDisplayName("Location Simulator")
    app.setOrganizationName("LocationSimulator")
    app.setOrganizationDomain("locationsimulator.local")

    # Set app style
    app.setStyle("Fusion")

    # Set global stylesheet for white backgrounds and black text
    app.setStyleSheet("""
        QMainWindow, QDialog, QWidget {
            background-color: white;
            color: black;
        }
        QLabel {
            color: black;
        }
        QLineEdit, QTextEdit, QPlainTextEdit {
            background-color: white;
            color: black;
        }
        QMessageBox {
            background-color: white;
        }
        QMessageBox QLabel {
            color: black;
        }
    """)
    
    # Create and show main window
    window = MainWindow()
    window.show()
    
    sys.exit(app.exec())


if __name__ == "__main__":
    main()
