# -*- mode: python ; coding: utf-8 -*-
"""
PyInstaller spec file for LocationSimulator - Windows 32-bit
Note: This app requires iTunes/Apple Mobile Device Support to be installed on Windows
"""

block_cipher = None

a = Analysis(
    ['..\\main.py'],
    pathex=[],
    binaries=[],
    datas=[],
    hiddenimports=[
        'PyQt6.QtCore',
        'PyQt6.QtGui',
        'PyQt6.QtWidgets',
        'PyQt6.QtWebEngineWidgets',
        'PyQt6.QtWebChannel',
        'pymobiledevice3',
        'pymobiledevice3.cli',
        'pymobiledevice3.services',
        'pymobiledevice3.lockdown',
        'pymobiledevice3.remote',
        'pymobiledevice3.usbmux',
        'cryptography',
        'cryptography.hazmat',
        'cryptography.hazmat.primitives',
        'cryptography.hazmat.backends',
        'cryptography.hazmat.backends.openssl',
        'packaging',
        'packaging.version',
        'packaging.specifiers',
        'packaging.requirements',
    ],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[],
    win_no_prefer_redirects=False,
    win_private_assemblies=False,
    cipher=block_cipher,
    noarchive=False,
)

pyz = PYZ(a.pure, a.zipped_data, cipher=block_cipher)

exe = EXE(
    pyz,
    a.scripts,
    [],
    exclude_binaries=True,
    name='LocationSimulator',
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    upx_exclude=[],
    runtime_tmpdir=None,
    console=False,  # No console window
    disable_windowed_traceback=False,
    target_arch='x86',
    codesign_identity=None,
    entitlements_file=None,
    icon="../icons/icon.ico",  # Add icon path: 'icon.ico'
)

coll = COLLECT(
    exe,
    a.binaries,
    a.zipfiles,
    a.datas,
    strip=False,
    upx=True,
    upx_exclude=[],
    name='LocationSimulator',
)
