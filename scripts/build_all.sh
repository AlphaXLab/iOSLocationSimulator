#!/bin/bash
# Universal build script for LocationSimulator
# Builds for the current platform architecture

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Detect platform and architecture
detect_platform() {
    local os_type=$(uname -s)
    local arch_type=$(uname -m)

    case "$os_type" in
        Darwin*)
            PLATFORM="macos"
            case "$arch_type" in
                x86_64*)
                    ARCH="intel"
                    SPEC_FILE="macos_intel.spec"
                    OUTPUT_NAME="LocationSimulator-macOS-Intel.app"
                    ;;
                arm64*)
                    ARCH="arm64"
                    SPEC_FILE="macos_arm64.spec"
                    OUTPUT_NAME="LocationSimulator-macOS-AppleSilicon.app"
                    ;;
                *)
                    echo -e "${RED}Unsupported macOS architecture: $arch_type${NC}"
                    exit 1
                    ;;
            esac
            ;;
        Linux*)
            echo -e "${RED}Linux builds are not officially supported yet${NC}"
            echo -e "${YELLOW}iOS devices cannot be detected on Linux without special setup${NC}"
            exit 1
            ;;
        MINGW*|MSYS*|CYGWIN*)
            PLATFORM="windows"
            case "$arch_type" in
                x86_64*|amd64*)
                    ARCH="x64"
                    SPEC_FILE="windows_x64.spec"
                    OUTPUT_NAME="LocationSimulator-Windows-x64"
                    ;;
                i*86*)
                    ARCH="x86"
                    SPEC_FILE="windows_x86.spec"
                    OUTPUT_NAME="LocationSimulator-Windows-x86"
                    ;;
                aarch64*|arm64*)
                    ARCH="arm64"
                    SPEC_FILE="windows_arm64.spec"
                    OUTPUT_NAME="LocationSimulator-Windows-ARM64"
                    ;;
                *)
                    echo -e "${RED}Unsupported Windows architecture: $arch_type${NC}"
                    exit 1
                    ;;
            esac
            ;;
        *)
            echo -e "${RED}Unsupported operating system: $os_type${NC}"
            exit 1
            ;;
    esac
}

# Print build information
print_build_info() {
    echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║   LocationSimulator Build System      ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${GREEN}Platform:${NC} $PLATFORM"
    echo -e "${GREEN}Architecture:${NC} $ARCH"
    echo -e "${GREEN}Spec File:${NC} $SPEC_FILE"
    echo -e "${GREEN}Output Name:${NC} $OUTPUT_NAME"
    echo ""
}

# Main build function
build() {
    echo -e "${YELLOW}🔨 Starting build process...${NC}"
    echo ""

    # Create icons if they don't exist
    if [ ! -f "icons/icon.icns" ] || [ ! -f "icons/icon.ico" ]; then
        echo -e "${BLUE}🎨 Creating application icons...${NC}"
        if [ -f "scripts/create_icons.sh" ]; then
            ./scripts/create_icons.sh 2>/dev/null || true
        fi
        # Also create ICO using Python if venv exists
        if [ -f "venv/bin/python" ]; then
            ./venv/bin/python scripts/create_ico.py 2>/dev/null || true
        fi
        echo ""
    fi

    # Check if virtual environment is activated
    if [ -z "$VIRTUAL_ENV" ]; then
        echo -e "${YELLOW}⚠️  Virtual environment not activated. Activating...${NC}"
        if [ -f "venv/bin/activate" ]; then
            source venv/bin/activate
        elif [ -f "venv/Scripts/activate" ]; then
            source venv/Scripts/activate
        else
            echo -e "${RED}❌ Virtual environment not found. Run ./scripts/install.sh first${NC}"
            exit 1
        fi
    fi

    # Install PyInstaller if not already installed
    echo -e "${BLUE}📦 Checking PyInstaller installation...${NC}"
    if ! pip show pyinstaller > /dev/null 2>&1; then
        echo -e "${YELLOW}Installing PyInstaller...${NC}"
        pip install pyinstaller
    fi

    # Clean previous builds
    echo -e "${BLUE}🧹 Cleaning previous builds...${NC}"
    rm -rf build dist

    # Build the application
    echo -e "${BLUE}🚀 Building application...${NC}"
    pyinstaller "build_configs/$SPEC_FILE"

    # Rename output
    if [ "$PLATFORM" = "macos" ]; then
        if [ -d "dist/LocationSimulator.app" ]; then
            mv "dist/LocationSimulator.app" "dist/$OUTPUT_NAME"
            echo -e "${GREEN}✅ Renamed to: dist/$OUTPUT_NAME${NC}"
        fi
    elif [ "$PLATFORM" = "windows" ]; then
        if [ -d "dist/LocationSimulator" ]; then
            mv "dist/LocationSimulator" "dist/$OUTPUT_NAME"
            echo -e "${GREEN}✅ Renamed to: dist/$OUTPUT_NAME${NC}"
        fi
    fi

    # Check if build was successful
    if [ "$PLATFORM" = "macos" ]; then
        if [ -d "dist/$OUTPUT_NAME" ]; then
            echo ""
            echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
            echo -e "${GREEN}║   ✅ Build Successful!                 ║${NC}"
            echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
            echo ""
            echo -e "${BLUE}📱 Application:${NC} dist/$OUTPUT_NAME"
            echo ""
            echo -e "${YELLOW}Next steps:${NC}"
            echo -e "  • Run: ${BLUE}open dist/$OUTPUT_NAME${NC}"
            echo -e "  • Install: ${BLUE}cp -r dist/$OUTPUT_NAME /Applications/${NC}"
            echo ""
        else
            echo -e "${RED}❌ Build failed! Check output above for errors.${NC}"
            exit 1
        fi
    elif [ "$PLATFORM" = "windows" ]; then
        if [ -d "dist/$OUTPUT_NAME" ]; then
            echo ""
            echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
            echo -e "${GREEN}║   ✅ Build Successful!                 ║${NC}"
            echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
            echo ""
            echo -e "${BLUE}📱 Application:${NC} dist/$OUTPUT_NAME"
            echo ""
            echo -e "${YELLOW}Next steps:${NC}"
            echo -e "  • Run: ${BLUE}./dist/$OUTPUT_NAME/LocationSimulator.exe${NC}"
            echo ""
            echo -e "${YELLOW}⚠️  Note:${NC} iTunes or Apple Mobile Device Support must be installed"
            echo ""
        else
            echo -e "${RED}❌ Build failed! Check output above for errors.${NC}"
            exit 1
        fi
    fi
}

# Main execution
detect_platform
print_build_info
build
