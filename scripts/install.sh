#!/bin/bash
# ru-share-media Install Script
# Sets up the media server with ZeroTier networking

set -euo pipefail

REPO="alphingj/ru-share-media"
VERSION="${RU_SHARE_VERSION:-latest}"
DATA_DIR="${RU_SHARE_DATA_PATH:-/opt/ru-share-media}"
LISTEN="${RU_SHARE_LISTEN:-0.0.0.0:8080}"

err() { printf 'ERROR: %s\n' "$*" >&2; exit 1; }
info() { printf '→ %s\n' "$*"; }

detect_version() {
  if [ "$VERSION" = "latest" ]; then
    VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep -oP '"tag_name"\s*:\s*"\K[^"]+' || err "Failed to detect version")
  fi
  info "Version: $VERSION"
}

download_binary() {
  local arch
  case "$(uname -m)" in
    x86_64|amd64)
      if ldd --version 2>/dev/null | grep -q musl; then
        arch="ru-share-media-x86_64-musl-linux"
      else
        arch="ru-share-media-x86_64-linux"
      fi
      ;;
    aarch64|arm64) arch="ru-share-media-aarch64-linux" ;;
    armv7l*)
      if ldd --version 2>/dev/null | grep -q musl; then
        arch="ru-share-media-armv7-musl-linux"
      else
        arch="ru-share-media-armv7-linux"
      fi
      ;;
    armv6l*)
      if ldd --version 2>/dev/null | grep -q musl; then
        arch="ru-share-media-armv6-musl-linux"
      else
        arch="ru-share-media-armv6-linux"
      fi
      ;;
    *) err "Unsupported architecture" ;;
  esac
  
  info "Downloading ${arch}..."
  curl -fsSL "https://github.com/${REPO}/releases/download/${VERSION}/${arch}" -o /usr/local/bin/ru-share-media
  chmod +x /usr/local/bin/ru-share-media
  info "Installed to /usr/local/bin/ru-share-media"
}

setup_zerotier() {
  if ! command -v zerotier-cli &>/dev/null; then
    info "Installing ZeroTier..."
    curl -s https://install.zerotier.com | sudo bash || err "ZeroTier install failed"
  fi
  
  if [ -z "${ZT_NETWORK_ID:-}" ]; then
    echo "ZeroTier Network ID (find at https://my.zerotier.com):"
    read -r ZT_NETWORK_ID
  fi
  
  if [ -n "${ZT_NETWORK_ID:-}" ]; then
    info "Joining ZeroTier network ${ZT_NETWORK_ID}..."
    sudo zerotier-cli join "$ZT_NETWORK_ID" || true
  fi
}

create_config() {
  mkdir -p /etc/ru-share-media "$DATA_DIR"
  
  cat > /etc/ru-share-media/config.env << EOF
RU_SHARE_DATA_PATH=${DATA_DIR}
RU_SHARE_LISTEN=${LISTEN}
RU_SHARE_CORS_ORIGIN=http://localhost:8080,http://192.168.1.0/24,http://10.0.0.0/8
ADMIN_USER=admin
ADMIN_PASS=${ADMIN_PASS:-changeme}
EOF
  
  if [ -n "${ADMIN_PASS:-}" ]; then
    chmod 600 /etc/ru-share-media/config.env
  fi
}

create_systemd() {
  cat > /etc/systemd/system/ru-share-media.service << 'EOF'
[Unit]
Description=ru-share Media Server
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
EnvironmentFile=/etc/ru-share-media/config.env
ExecStart=/usr/local/bin/ru-share-media
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF
  systemctl daemon-reload
  systemctl enable ru-share-media
}

main() {
  [ "$(id -u)" -eq 0 ] || err "Must run as root (use sudo)"
  detect_version
  setup_zerotier
  create_config
  download_binary
  create_systemd
  echo
  echo "Installation complete!"
  echo "Config: /etc/ru-share-media/config.env"
  echo "Start:  systemctl start ru-share-media"
}

main "$@"