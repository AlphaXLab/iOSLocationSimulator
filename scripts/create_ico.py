#!/usr/bin/env python3
"""
Create Windows ICO file from PNG
"""
import sys
from pathlib import Path

try:
    from PIL import Image
except ImportError:
    print("❌ Pillow not installed. Installing in venv...")
    import subprocess
    subprocess.check_call([sys.executable, '-m', 'pip', 'install', 'Pillow', '--quiet'])
    from PIL import Image

def create_ico(png_path, ico_path):
    """Convert PNG to multi-resolution ICO"""
    print(f"📝 Loading {png_path}...")
    img = Image.open(png_path)

    # Convert to RGBA if needed
    if img.mode != 'RGBA':
        img = img.convert('RGBA')

    # Create different sizes
    sizes = [(256, 256), (128, 128), (64, 64), (48, 48), (32, 32), (16, 16)]

    print(f"🔄 Creating multi-resolution ICO with sizes: {sizes}")

    # Save as ICO with multiple sizes
    img.save(
        ico_path,
        format='ICO',
        sizes=sizes
    )

    print(f"✅ Created: {ico_path}")

if __name__ == '__main__':
    project_root = Path(__file__).parent.parent
    png_path = project_root / 'img' / 'pin.png'
    ico_path = project_root / 'icons' / 'icon.ico'

    # Create icons directory if needed
    ico_path.parent.mkdir(exist_ok=True)

    if not png_path.exists():
        print(f"❌ Error: {png_path} not found!")
        sys.exit(1)

    create_ico(png_path, ico_path)
