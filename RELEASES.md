# ru-share-media Releases

Pre-built binaries for all supported platforms are available on GitHub Releases.

## Latest Version: v3.0.0

### Binary Downloads

| Architecture | Binary Name | Target | Pi Compatible |
|-------------|-----------|--------|---------------|
| x86_64 Linux | `ru-share-media-x86_64-linux` | x86_64-unknown-linux-gnu | No |
| x86_64 Musl | `ru-share-media-x86_64-musl-linux` | x86_64-unknown-linux-musl | No |
| ARM64 (Pi 3/4/5) | `ru-share-media-aarch64-linux` | aarch64-unknown-linux-gnu | No (Pi 3B+ limited) |
| ARM v7 (Pi 2/3) | `ru-share-media-armv7-linux` | armv7-unknown-linux-gnueabihf | Yes |
| ARM v7 Musl | `ru-share-media-armv7-musl-linux` | armv7-unknown-linux-musleabihf | Yes |
| ARM v6 (Pi Zero/1) | `ru-share-media-armv6-linux` | arm-unknown-linux-gnueabihf | **Yes** |
| ARM v6 Musl (Pi Zero/1) | `ru-share-media-armv6-musl-linux` | arm-unknown-linux-musleabihf | **Yes** |

## Quick Install

```bash
# Auto-detect architecture and install
curl -fsSL https://raw.githubusercontent.com/alphingj/ru-share-media/main/start.sh | bash
```

## Manual Download

```bash
# Replace VERSION with actual tag
wget -O ru-share-media https://github.com/alphingj/ru-share-media/releases/download/v3.0.0/ru-share-media-armv6-linux
chmod +x ru-share-media
./ru-share-media
```

## Architecture Detection Script

The `start.sh` script automatically detects:
- `x86_64` - Intel/AMD 64-bit PCs
- `aarch64` - ARM 64-bit (Pi 3/4/5, Apple Silicon)
- `armv7l` - ARM 32-bit (Pi 2/3)
- `armv6l` - ARM 32-bit (Pi Zero/1)

## Pi Zero W Optimization

For Pi Zero W (ARMv6, 512MB RAM):
- Use `-musl` builds for static linking
- Disable hardware transcoding (use ffprobe metadata only)
- Recommended swap: 512MB+

## System Requirements

- **Minimum**: 512MB RAM (Pi Zero W), 100MB disk
- **Recommended**: 1GB RAM, 2GB+ disk for cache
- **FFmpeg**: Required for media scanning (ffprobe)
- **ZeroTier**: Optional, for remote access