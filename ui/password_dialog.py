"""
Password Dialog - Secure password input for sudo
"""

from PyQt6.QtWidgets import (
    QDialog, QVBoxLayout, QHBoxLayout, QLabel,
    QPushButton, QLineEdit, QFrame
)
from PyQt6.QtCore import Qt
from PyQt6.QtGui import QFont


class PasswordDialog(QDialog):
    """Dialog for entering sudo password"""

    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowTitle("Administrator Access Required")
        self.setModal(True)
        self.setMinimumWidth(450)

        self._setup_ui()

    def _setup_ui(self):
        """Setup the dialog UI"""
        # Set white background for the entire dialog
        self.setStyleSheet("""
            QDialog {
                background-color: white;
            }
            QLabel {
                color: black;
            }
        """)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(24, 24, 24, 24)
        layout.setSpacing(16)

        # Icon and title
        title_label = QLabel("🔐 Administrator Password Required")
        title_font = QFont()
        title_font.setPointSize(14)
        title_font.setWeight(QFont.Weight.Bold)
        title_label.setFont(title_font)
        title_label.setStyleSheet("color: black;")
        layout.addWidget(title_label)

        # Explanation
        explanation = QLabel(
            "Location Simulator needs to start a RemoteXPC tunnel for iOS 17+ devices.\n\n"
            "This requires administrator (sudo) access to create a network interface.\n\n"
            "Please enter your macOS password:"
        )
        explanation.setWordWrap(True)
        explanation.setStyleSheet("color: #555555;")
        layout.addWidget(explanation)

        # Password input
        password_label = QLabel("Password:")
        password_label.setStyleSheet("font-weight: 500; color: black;")
        layout.addWidget(password_label)

        self.password_input = QLineEdit()
        self.password_input.setEchoMode(QLineEdit.EchoMode.Password)
        self.password_input.setPlaceholderText("Enter your macOS password")
        self.password_input.setMinimumHeight(36)
        self.password_input.setStyleSheet("""
            QLineEdit {
                padding: 8px;
                border: 2px solid #E0E0E0;
                border-radius: 6px;
                font-size: 14px;
                background-color: white;
                color: black;
            }
            QLineEdit:focus {
                border-color: #1565C0;
            }
        """)
        self.password_input.returnPressed.connect(self.accept)
        layout.addWidget(self.password_input)

        # Security note
        security_note = QLabel(
            "🔒 Your password is only used to start the tunnel and is never stored."
        )
        security_note.setStyleSheet("color: #666666; font-size: 12px;")
        layout.addWidget(security_note)

        # Spacer
        layout.addStretch()

        # Buttons
        button_layout = QHBoxLayout()
        button_layout.setSpacing(12)

        cancel_button = QPushButton("Cancel")
        cancel_button.setMinimumHeight(36)
        cancel_button.setStyleSheet("""
            QPushButton {
                padding: 8px 16px;
                border: 1px solid #E0E0E0;
                border-radius: 6px;
                background-color: white;
                color: black;
                font-size: 14px;
            }
            QPushButton:hover {
                background-color: #F5F5F5;
            }
        """)
        cancel_button.clicked.connect(self.reject)

        ok_button = QPushButton("Start Tunnel")
        ok_button.setMinimumHeight(36)
        ok_button.setStyleSheet("""
            QPushButton {
                padding: 8px 16px;
                border: none;
                border-radius: 6px;
                background-color: #1565C0;
                color: white;
                font-size: 14px;
                font-weight: 500;
            }
            QPushButton:hover {
                background-color: #0D47A1;
            }
        """)
        ok_button.setDefault(True)
        ok_button.clicked.connect(self.accept)

        button_layout.addStretch()
        button_layout.addWidget(cancel_button)
        button_layout.addWidget(ok_button)

        layout.addLayout(button_layout)

        # Focus on password input
        self.password_input.setFocus()

    def get_password(self):
        """Get the entered password"""
        return self.password_input.text()

    @staticmethod
    def get_password_from_user(parent=None):
        """Static method to show dialog and get password"""
        dialog = PasswordDialog(parent)
        if dialog.exec() == QDialog.DialogCode.Accepted:
            return dialog.get_password()
        return None
