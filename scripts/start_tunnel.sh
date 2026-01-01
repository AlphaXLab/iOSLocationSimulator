#!/bin/bash
# Helper script to start RemoteXPC tunnel for iOS 17+ devices
# This tunnel is required for Developer services on iOS 17 and later

# Get the project root directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." && pwd )"

echo "========================================="
echo "Starting RemoteXPC Tunnel for iOS 17+"
echo "========================================="
echo ""
echo "This tunnel is required for location simulation on iOS 17+ devices."
echo "Keep this window running while using the Location Simulator app."
echo ""
echo "Press Ctrl+C to stop the tunnel when done."
echo ""
echo "========================================="
echo ""

# Activate virtual environment if it exists
if [ -d "$SCRIPT_DIR/venv" ]; then
    echo "Activating virtual environment..."
    source "$SCRIPT_DIR/venv/bin/activate"
fi

# Check if pymobiledevice3 is installed
if ! python3 -c "import pymobiledevice3" 2>/dev/null; then
    echo "❌ Error: pymobiledevice3 not found!"
    echo "Please install it first: pip3 install pymobiledevice3"
    exit 1
fi

# Start the tunnel (requires sudo for network interface creation)
echo "Starting tunnel... (you may be prompted for your password)"
sudo "$(which python3)" -m pymobiledevice3 remote tunneld
