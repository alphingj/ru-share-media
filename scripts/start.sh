#!/bin/bash
# ru-share-media Quick Start Script

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN="${SCRIPT_DIR}/ru-share-media"

err() { printf 'ERROR: %s\n' "$*" >&2; exit 1; }
info() { printf '→ %s\n' "$*"; }

detect_arch() {
  case "$(uname -m)" in
    x86_64|amd64)
      if ldd --version 2>/dev/null | grep -q musl; then
        echo "ru-share-media-x86_64-musl-linux"
      else
        echo "ru-share-media-x86_64-linux"
      fi
      ;;
    aarch64|arm64) echo "ru-share-media-aarch64-linux" ;;
    armv7l*)
      if ldd --version 2>/dev/null | grep -q musl; then
        echo "ru-share-media-armv7-musl-linux"
      else
        echo "ru-share-media-armv7-linux"
      fi
      ;;
    armv6l*)
      if ldd --version 2>/dev/null | grep -q musl; then
        echo "ru-share-media-armv6-musl-linux"
      else
        echo "ru-share-media-armv6-linux"
      fi
      ;;
    *) err "Unsupported arch" ;;
  esac
}

download() {
  if [ -x "$BIN" ]; then
    info "Binary exists: $BIN"
    return
  fi
  
  local arch=$(detect_arch)
  local url="https://github.com/alphingj/ru-share-media/releases/download/v3.0.0/${arch}"
  info "Downloading ${arch}..."
  curl -fsSL "$url" -o "$BIN"
  chmod +x "$BIN"
}

setup_zerotier() {
  if command -v zerotier-cli &>/dev/null; then
    local zt_ip=$(ip addr show | grep -oP '(?<=inet )10\.\d+\.\d+\.\d+' | head -1)
    if [ -n "$zt_ip" ]; then
      info "ZeroTier IP: $zt_ip"
      RU_SHARE_CORS_ORIGIN="${RU_SHARE_CORS_ORIGIN},http://${zt_ip}:8080"
    fi
  fi
}

main() {
  download
  setup_zerotier
  
  export RU_SHARE_DATA_PATH="${SCRIPT_DIR}/data"
  export RU_SHARE_LISTEN="${RU_SHARE_LISTEN:-0.0.0.0:8080}"
  export RU_SHARE_CORS_ORIGIN="${RU_SHARE_CORS_ORIGIN:-http://localhost:8080,http://192.168.1.0/24}"
  
  mkdir -p "$RU_SHARE_DATA_PATH/hls" "$RU_SHARE_DATA_PATH/thumbnails"
  
  echo
  echo "==============================================="
  echo "     ru-share Media Server Starting           "
  echo "==============================================="
  echo "Listen: $RU_SHARE_LISTEN"
  echo "Data:   $RU_SHARE_DATA_PATH"
  echo "CORS:   $RU_SHARE_CORS_ORIGIN"
  echo "==============================================="
  echo
  
  exec "$BIN"
}

main "$@"