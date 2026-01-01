"""
API Key Dialog - For entering Google Maps API key
"""

from PyQt6.QtWidgets import (
    QDialog, QVBoxLayout, QHBoxLayout, QLabel,
    QPushButton, QLineEdit
)
from PyQt6.QtCore import Qt
from PyQt6.QtGui import QFont


class ApiKeyDialog(QDialog):
    """Dialog for entering Google Maps API key"""

    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowTitle("Google Maps API Key")
        self.setModal(True)
        self.setMinimumWidth(500)

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
        title_label = QLabel("🗺️ Google Maps API Key Required")
        title_font = QFont()
        title_font.setPointSize(14)
        title_font.setWeight(QFont.Weight.Bold)
        title_label.setFont(title_font)
        title_label.setStyleSheet("color: black;")
        layout.addWidget(title_label)

        # Explanation with detailed steps
        explanation = QLabel(
            "To use Google Maps, you need an API key from Google Cloud Console.\n\n"
            "Google Maps offers $200/month free credit (approximately 28,000 map loads).\n\n"
            "Steps to get your API key:\n"
            "1. Go to: <a href='https://console.cloud.google.com/google/maps-apis'>Google Cloud Console</a>\n"
            "2. Create a new project or select an existing one\n"
            "3. Click 'Enable APIs and Services'\n"
            "4. Search for 'Maps JavaScript API' and enable it\n"
            "5. Go to 'Credentials' in the left sidebar\n"
            "6. Click 'Create Credentials' → 'API Key'\n"
            "7. Copy the API key and paste it below\n\n"
            "Optional: Set API restrictions to 'HTTP referrers' for security\n\n"
            "Please enter your Google Maps API key:"
        )
        explanation.setWordWrap(True)
        explanation.setTextFormat(Qt.TextFormat.RichText)
        explanation.setOpenExternalLinks(True)
        explanation.setStyleSheet("color: #555555; line-height: 1.4;")
        layout.addWidget(explanation)

        # API key input
        api_key_label = QLabel("API Key:")
        api_key_label.setStyleSheet("font-weight: 500; color: black;")
        layout.addWidget(api_key_label)

        self.api_key_input = QLineEdit()
        self.api_key_input.setPlaceholderText("Enter your Google Maps API key")
        self.api_key_input.setMinimumHeight(36)
        self.api_key_input.setStyleSheet("""
            QLineEdit {
                padding: 8px;
                border: 2px solid #E0E0E0;
                border-radius: 6px;
                font-size: 14px;
                background-color: white;
                color: black;
                font-family: monospace;
            }
            QLineEdit:focus {
                border-color: #1565C0;
            }
        """)
        self.api_key_input.returnPressed.connect(self.accept)
        layout.addWidget(self.api_key_input)

        # Info note
        info_note = QLabel(
            "ℹ️ Your API key will be stored locally and used only for loading Google Maps."
        )
        info_note.setStyleSheet("color: #666666; font-size: 12px;")
        layout.addWidget(info_note)

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

        ok_button = QPushButton("Save API Key")
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

        # Focus on API key input
        self.api_key_input.setFocus()

    def get_api_key(self):
        """Get the entered API key"""
        return self.api_key_input.text().strip()

    @staticmethod
    def get_api_key_from_user(parent=None):
        """Static method to show dialog and get API key"""
        dialog = ApiKeyDialog(parent)
        if dialog.exec() == QDialog.DialogCode.Accepted:
            api_key = dialog.get_api_key()
            if api_key:  # Only return if not empty
                return api_key
        return None
