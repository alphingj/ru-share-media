# ru-share-media v3.0.0

Netflix-style self-hosted media server with DLNA/UPnP support. Stream your media library to any device.

## Features

- **Netflix-style UI**: Dark theme with hero banners, horizontal rows, and responsive design
- **DLNA/UPnP Discovery**: Automatic discovery on local network
- **HLS Streaming**: Adaptive bitrate playback (1080p, 720p, 480p, 360p)
- **Media Scanning**: Automatic metadata extraction via ffprobe
- **ZeroTier Support**: Secure remote access via VPN
- **Pi Zero Compatible**: Optimized builds for ARM devices

## Quick Start

```bash
# Download and run
curl -fsSL https://github.com/alphingj/ru-share-media/releases/latest/download/ru-share-media-x86_64-linux -o ru-share-media
chmod +x ru-share-media
./ru-share-media
```

Visit `http://localhost:8080` to start watching.

## Configuration

```bash
export RU_SHARE_MEDIA_PATH=/path/to/media
export RU_SHARE_LISTEN=0.0.0.0:8080
export RU_SHARE_CORS_ORIGIN="http://localhost:8080,http://192.168.1.0/24,http://10.215.169.0/24"
export ADMIN_USER=admin
export ADMIN_PASS=securepassword
```

## API Endpoints

- `GET /api/library` - Browse media
- `POST /api/scan` - Rescan media directory
- `POST /api/login` - Authentication
- `GET /hls/:id` - HLS stream
- `GET /api/health` - Health check

## Install with ZeroTier

```bash
sudo curl -fsSL https://raw.githubusercontent.com/alphingj/ru-share-media/main/scripts/install.sh | bash
```

## Supported Formats

- Video: MP4, MKV, AVI, MOV, WMV, FLV, WebM, M4V
- Audio: MP3, FLAC, WAV, AAC, OGG, M4A
- Images: JPG, PNG, GIF, WebP