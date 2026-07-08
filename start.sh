#!/bin/bash
# ru-share-media - Architecture-aware installer and starter
# Downloads appropriate binary for your device and launches server

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RELEASE_REPO="alphingj/ru-share-media"
VERSION="${RU_SHARE_VERSION:-latest}"
DATA_PATH="${RU_SHARE_MEDIA_PATH:-${SCRIPT_DIR}/media}"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

info() { printf "${GREEN}→${NC} %s\n" "$*"; }
warn() { printf "${YELLOW}!${NC} %s\n" "$*"; }
err() { printf "${RED}✗${NC} %s\n" "$*" >&2; exit 1; }

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
    armv7l*) echo "ru-share-media-armv7-linux" ;;
    armv6l*)
      if ldd --version 2>/dev/null | grep -q musl; then
        echo "ru-share-media-armv6-musl-linux"
      else
        echo "ru-share-media-armv6-linux"
      fi
      ;;
    *) err "Unsupported architecture: $(uname -m)" ;;
  esac
}

get_latest_version() {
  if command -v gh &>/dev/null; then
    gh release list --repo "$RELEASE_REPO" --limit 1 | awk '{print $1}'
  elif command -v curl &>/dev/null; then
    curl -fsSL "https://api.github.com/repos/${RELEASE_REPO}/releases/latest" | grep -oP '"tag_name"\s*:\s*"\K[^"]+' 
  else
    echo "v3.0.0"
  fi
}

download_binary() {
  local arch_name="$1"
  local version="$2"
  local url="https://github.com/${RELEASE_REPO}/releases/download/${version}/${arch_name}"
  
  if [[ "$version" == "latest" ]]; then
    version=$(get_latest_version)
  fi
  
  info "Downloading ${arch_name} (v${version})..."
  
  if [[ -x "${SCRIPT_DIR}/ru-share-media" ]]; then
    info "Binary already exists"
    return
  fi
  
  if command -v curl &>/dev/null; then
    curl -fsSL "$url" -o "${SCRIPT_DIR}/ru-share-media" || {
      warn "Pre-built binary not found, falling back to compile..."
      return 1
    }
  elif command -v wget &>/dev/null; then
    wget -q "$url" -O "${SCRIPT_DIR}/ru-share-media" || {
      warn "Pre-built binary not found, falling back to compile..."
      return 1
    }
  else
    err "curl or wget required for download"
  fi
  
  chmod +x "${SCRIPT_DIR}/ru-share-media"
  info "Binary installed successfully"
}

setup_zerotier() {
  if command -v zerotier-cli &>/dev/null; then
    # Get ZeroTier IP
    ZT_IP=$(ip addr show | grep -oP '(?<=inet )10\.\d+\.\d+\.\d+' | head -1 || true)
    if [[ -n "$ZT_IP" ]]; then
      info "ZeroTier detected: IP $ZT_IP"
      RU_SHARE_CORS_ORIGIN="${RU_SHARE_CORS_ORIGIN},http://${ZT_IP}:8080"
    fi
  fi
}

create_config() {
  # Create data directory
  mkdir -p "$DATA_PATH/hls" "$DATA_PATH/thumbnails"
  
  # Create .env if not exists
  if [[ ! -f "${SCRIPT_DIR}/.env" ]]; then
    cat > "${SCRIPT_DIR}/.env" << EOF
# ru-share-media configuration
export RU_SHARE_MEDIA_PATH="${DATA_PATH}"
export RU_SHARE_LISTEN="${RU_SHARE_LISTEN:-0.0.0.0:8080}"
export RU_SHARE_CORS_ORIGIN="${RU_SHARE_CORS_ORIGIN:-http://localhost:8080,http://192.168.1.0/24}"
export ADMIN_USER="${ADMIN_USER:-admin}"
export ADMIN_PASS="${ADMIN_PASS:-changeme}"
EOF
    info "Created .env configuration file"
  fi
}

compile_fallback() {
  if command -v cargo &>/dev/null; then
    info "Compiling from source (this may take a few minutes)..."
    cargo build --release --target "$(detect_target)"
    mv "${SCRIPT_DIR}/target/$(detect_target)/release/ru-share-media" "${SCRIPT_DIR}/ru-share-media" 2>/dev/null || true
  else
    err "No pre-built binary available and cargo not found for compilation"
  fi
}

detect_target() {
  # Map arch to Rust target
  case "$(uname -m)" in
    x86_64|amd64) echo "x86_64-unknown-linux-gnu" ;;
    aarch64|arm64) echo "aarch64-unknown-linux-gnu" ;;
    armv7l*) echo "armv7-unknown-linux-gnueabihf" ;;
    armv6l*)
      if ldd --version 2>/dev/null | grep -q musl; then
        echo "armv6-unknown-linux-gnueabihf"
      else
        echo "arm-unknown-linux-gnueabi"
      fi
      ;;
    *) echo "$(uname -m)-unknown-linux-gnu" ;;
  esac
}

start_server() {
  # Load env if exists
  [[ -f "${SCRIPT_DIR}/.env" ]] && source "${SCRIPT_DIR}/.env"
  
  export RU_SHARE_MEDIA_PATH="${DATA_PATH}"
  
  info "Starting ru-share-media..."
  info "Listen: ${RU_SHARE_LISTEN}"
  info "Data: ${RU_SHARE_MEDIA_PATH}"
  info "CORS: ${RU_SHARE_CORS_ORIGIN:-http://localhost:8080}"
  
  exec "${SCRIPT_DIR}/ru-share-media"
}

# Main
main() {
  local arch=$(detect_arch)
  
  # Try binary download first
  if ! download_binary "$arch" "$VERSION"; then
    compile_fallback
  fi
  
  create_config
  setup_zerotier
  start_server
}

main "$@"