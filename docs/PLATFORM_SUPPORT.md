# Platform Support

Complete guide to supported platforms and architectures for LocationSimulator.

## Summary

LocationSimulator supports **5 different build targets** across macOS and Windows:

| Platform | Architecture | Build Config | Notes |
|----------|--------------|--------------|-------|
| macOS | Intel (x86_64) | `macos_intel.spec` | Intel-based Macs |
| macOS | Apple Silicon (ARM64) | `macos_arm64.spec` | M1, M2, M3, M4 Macs |
| Windows | 64-bit (x64) | `windows_x64.spec` | Most Windows PCs |
| Windows | 32-bit (x86) | `windows_x86.spec` | Older Windows PCs |
| Windows | ARM64 | `windows_arm64.spec` | Surface Pro X, Copilot+ PCs |

## macOS Support

### macOS Intel (x86_64)

**Devices**:
- All Intel-based Macs (2006-2020)
- Mac Pro, iMac, MacBook Pro (pre-2020)

**Requirements**:
- macOS 10.13+ (High Sierra or later)
- No special setup required

**Build**:
```bash
pyinstaller build_configs/macos_intel.spec
```

### macOS Apple Silicon (ARM64)

**Devices**:
- M1 Macs (2020)
- M2 Macs (2022)
- M3 Macs (2023)
- M4 Macs (2024+)

**Requirements**:
- macOS 11.0+ (Big Sur or later)
- Native ARM64 Python recommended

**Build**:
```bash
pyinstaller build_configs/macos_arm64.spec
```

**Rosetta 2 Compatibility**:
- Intel builds will run on Apple Silicon via Rosetta 2
- But native ARM64 builds are **much faster**

### Universal Binary

To create a "fat" binary that runs on both architectures:

1. Build both versions
2. Use `lipo` to combine them
3. See [docs/BUILDING.md](BUILDING.md) for detailed instructions

## Windows Support

### Windows 64-bit (x64)

**Devices**:
- Most modern Windows PCs (2010+)
- Intel/AMD 64-bit processors

**Requirements**:
- Windows 10/11 (64-bit)
- Python 3.10+ (64-bit)
- iTunes or Apple Mobile Device Support

**Build**:
```bash
pyinstaller build_configs\windows_x64.spec
```

**Most Common**: This is the standard Windows build that works on most PCs.

### Windows 32-bit (x86)

**Devices**:
- Older Windows PCs
- 32-bit Windows installations

**Requirements**:
- Windows 7/8/10/11 (32-bit)
- Python 3.10+ (32-bit)
- iTunes or Apple Mobile Device Support

**Build**:
```bash
pyinstaller build_configs\windows_x86.spec
```

**Note**: 32-bit Windows is becoming rare. Most users should use x64 build.

### Windows ARM64

**Devices**:
- **Surface Pro X** (Qualcomm SQ1/SQ2)
- **Surface Pro 9 5G** (Qualcomm SQ3)
- **Surface Laptop 7** (Snapdragon X Elite)
- **Copilot+ PCs** (Snapdragon X series)
- Other Windows ARM devices

**Requirements**:
- Windows 10/11 ARM64
- Python 3.10+ ARM64 (from python.org or Microsoft Store)
- iTunes ARM64 or Apple Mobile Device Support

**Build**:
```bash
pyinstaller build_configs\windows_arm64.spec
```

## Windows ARM64: Native vs Emulated

Windows ARM64 devices can run apps in two ways:

### 1. Native ARM64 (Recommended)

**Pros**:
- ✅ Best performance
- ✅ Better battery life
- ✅ No emulation overhead

**Cons**:
- ⚠️ Must build on ARM64 Windows with ARM64 Python
- ⚠️ Requires ARM64 version of all dependencies

**How to build**:
```bash
# On Windows ARM64 device
# 1. Install ARM64 Python from python.org
# 2. Install ARM64 dependencies
pip install pymobiledevice3 PyQt6 PyQt6-WebEngine pyinstaller

# 3. Build
pyinstaller build_configs\windows_arm64.spec
```

### 2. x64 Emulation (Easier)

**Pros**:
- ✅ No special build needed - use x64 build
- ✅ Works transparently
- ✅ Can build on any x64 Windows machine

**Cons**:
- ⚠️ Slower performance (20-40% overhead)
- ⚠️ Higher power consumption
- ⚠️ Some compatibility issues possible

**How to use**:
```bash
# Just run the x64 build - it will work via emulation
dist\LocationSimulator-Windows-x64\LocationSimulator.exe
```

### Which to Choose?

**Use Native ARM64 if**:
- You have an ARM64 Windows device
- You want best performance
- You're distributing to ARM64 users

**Use x64 Emulation if**:
- You don't have ARM64 Windows device to build on
- You want one build for all Windows users
- Performance is acceptable

## Cross-Platform Compatibility

### Can I run macOS builds on Windows?
❌ No. macOS builds (.app) only run on macOS.

### Can I run Windows builds on macOS?
❌ No. Windows builds (.exe) only run on Windows.

### Can I run Intel builds on Apple Silicon?
✅ Yes, via Rosetta 2 (automatic, transparent, but slower).

### Can I run Apple Silicon builds on Intel?
❌ No. Intel Macs cannot run ARM64 binaries.

### Can I run x64 builds on Windows ARM64?
✅ Yes, via emulation (automatic, transparent, but slower).

### Can I run x86 builds on Windows x64?
✅ Yes, via WoW64 (automatic, fast, recommended).

### Can I run ARM64 builds on Windows x64?
❌ No. x64 Windows cannot run ARM64 binaries.

## Linux Support

**Current Status**: ❌ Not officially supported

**Why not?**:
- iOS device detection requires Apple's usbmuxd
- pymobiledevice3 has limited Linux support
- Device pairing is complex on Linux
- Most iOS users don't use Linux

**Workarounds**:
- Use a macOS or Windows VM
- Use Windows Subsystem for Linux (WSL) with USB passthrough
- Contribute Linux support to the project!

## Recommended Build Strategy

### For Personal Use

Build for your current platform/architecture:
```bash
./scripts/build_all.sh  # Auto-detects
```

### For Distribution

Build for all platforms:

**On macOS Intel**:
```bash
pyinstaller build_configs/macos_intel.spec
```

**On macOS Apple Silicon**:
```bash
pyinstaller build_configs/macos_arm64.spec
```

**On Windows x64**:
```bash
pyinstaller build_configs\windows_x64.spec
pyinstaller build_configs\windows_x86.spec  # 32-bit
```

**On Windows ARM64** (optional):
```bash
pyinstaller build_configs\windows_arm64.spec
```

**Result**: 5 builds covering all platforms and architectures

## Testing Builds

### macOS

**On Intel Mac**:
- ✅ Test Intel build (native)
- ✅ Test ARM64 build (via Rosetta 2)

**On Apple Silicon Mac**:
- ✅ Test ARM64 build (native)
- ✅ Test Intel build (via Rosetta 2)

### Windows

**On x64 Windows**:
- ✅ Test x64 build (native)
- ✅ Test x86 build (via WoW64)

**On ARM64 Windows**:
- ✅ Test ARM64 build (native)
- ✅ Test x64 build (via emulation)
- ✅ Test x86 build (via emulation)

## File Sizes

Approximate sizes of built applications:

| Platform | Architecture | Size | Notes |
|----------|--------------|------|-------|
| macOS | Intel | ~180 MB | Includes Qt frameworks |
| macOS | Apple Silicon | ~170 MB | Slightly smaller |
| Windows | x64 | ~150 MB | Includes all DLLs |
| Windows | x86 | ~140 MB | Smaller than x64 |
| Windows | ARM64 | ~150 MB | Similar to x64 |

**Note**: Sizes include Python runtime, PyQt6, QtWebEngine, and all dependencies.

## Distribution Recommendations

### For macOS Users

Provide **both** Intel and Apple Silicon builds:
- `LocationSimulator-macOS-Intel.app`
- `LocationSimulator-macOS-AppleSilicon.app`

Or create a Universal Binary (larger but works on both).

### For Windows Users

**Minimum**:
- `LocationSimulator-Windows-x64.zip` (covers most users)

**Recommended**:
- `LocationSimulator-Windows-x64.zip` (most users)
- `LocationSimulator-Windows-x86.zip` (legacy systems)
- `LocationSimulator-Windows-ARM64.zip` (ARM devices)

### For All Users

Create a **download page** with:
```
macOS:
- Intel Macs: Download Intel version
- Apple Silicon (M1/M2/M3): Download ARM64 version

Windows:
- Most PCs: Download x64 version
- ARM PCs (Surface Pro X, Copilot+): Download ARM64 version
- Older PCs: Download x86 version
```

## Future Platform Support

**Potential additions**:
- **Linux** support (if community requests it)
- **macOS Universal Binary** (one build for both architectures)
- **Windows Universal** (one installer for all architectures)

## Getting Help

If you have issues with a specific platform:

1. Check [docs/BUILDING.md](BUILDING.md) for build instructions
2. Verify you have the correct Python architecture
3. Test on the target platform if possible
4. Open an issue on GitHub with:
   - Your platform and architecture
   - Build logs
   - Error messages
