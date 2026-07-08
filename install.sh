#!/bin/bash
# ru-share-media System Installer
# Installs to /usr/local/bin and sets up systemd service

set -euo pipefail

REPO="alphingj/ru-share-media"
VERSION="${RU_SHARE_VERSION:-latest}"
DATA_DIR="${RU_SHARE_DATA_PATH:-/opt/ru-share-media}"
LISTEN_ADDR="${RU_SHARE_LISTEN:-0.0.0.0:8080}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info() { printf "${GREEN}→${NC} %s\n" "$*"; }
warn() { printf "${YELLOW}!${NC} %s\n" "$*"; }
err() { printf "${RED}✗${NC} %s\n" "$*" >&2; exit 1; }

[[ "$(id -u)" -eq 0 ]] || err "Must run as root (use sudo)"

detect_arch() {
  case "$(uname -m)" in
    x86_64|amd64) echo "ru-share-media-x86_64-linux" ;;
    aarch64|arm64) echo "ru-share-media-aarch64-linux" ;;
    armv7l*) echo "ru-share-media-armv7-linux" ;;
    armv6l*) echo "ru-share-media-armv6-musl-linux" ;;
    *) err "Unsupported architecture" ;;
  esac
}

get_version() {
  if [[ "$VERSION" == "latest" ]]; then
    curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep -oP '"tag_name"\s*:\s*"\K[^"]+' || echo "v3.0.0"
  else
    echo "$VERSION"
  fi
}

install_binary() {
  local arch=$1
  local version=$2
  local url="https://github.com/${REPO}/releases/download/${version}/${arch}"

  info "Downloading ${arch}..."
  if curl -fsSL "$url" -o /tmp/ru-share-media; then
    :
  else
    info "Per-arch binary not found, trying release archive..."
    local archive_url="https://github.com/${REPO}/releases/download/${version}/ru-share-media.tar.gz"
    curl -fsSL "$archive_url" -o /tmp/ru-share-media.tar.gz || {
      err "Failed to download release archive - try building from source"
    }
    tar -xzf /tmp/ru-share-media.tar.gz -C /tmp
    rm -f /tmp/ru-share-media.tar.gz
  fi

  install -m 755 /tmp/ru-share-media /usr/local/bin/ru-share-media
  rm -f /tmp/ru-share-media
  info "Installed to /usr/local/bin/ru-share-media"
}

setup_zerotier() {
  if ! command -v zerotier-cli &>/dev/null; then
    info "Installing ZeroTier..."
    curl -s https://install.zerotier.com | bash || warn "ZeroTier install failed"
  fi
  
  echo ""
  echo "Enter your ZeroTier Network ID (from https://my.zerotier.com):"
  read -rp "Network ID (or press Enter to skip): " ZT_NETWORK
  
  if [[ -n "$ZT_NETWORK" ]]; then
    zerotier-cli join "$ZT_NETWORK" || warn "Failed to join network"
  fi
}

create_config() {
  mkdir -p "$DATA_DIR" /etc/ru-share-media
  
  [[ -z "${ADMIN_USER:-}" ]] && read -rp "Admin username [admin]: " ADMIN_USER
  ADMIN_USER="${ADMIN_USER:-admin}"
  
  [[ -z "${ADMIN_PASS:-}" ]] && read -rsp "Admin password: " ADMIN_PASS
  ADMIN_PASS="${ADMIN_PASS:-changeme}"
  
  cat > /etc/ru-share-media/config.env << EOF
# ru-share-media configuration
RU_SHARE_MEDIA_PATH=${DATA_DIR}
RU_SHARE_LISTEN=${LISTEN_ADDR}
RU_SHARE_CORS_ORIGIN=http://localhost:8080,http://192.168.1.0/24
ADMIN_USER=${ADMIN_USER}
ADMIN_PASS=${ADMIN_PASS}
EOF
  
  chmod 600 /etc/ru-share-media/config.env
  info "Config saved to /etc/ru-share-media/config.env"
}

create_service() {
  cat > /etc/systemd/system/ru-share-media.service << 'EOF'
[Unit]
Description=ru-share-media - Netflix-style Media Server
Documentation=https://github.com/alphingj/ru-share-media
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=nobody
Group=nogroup
EnvironmentFile=/etc/ru-share-media/config.env
ExecStart=/usr/local/bin/ru-share-media
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/ru-share-media
PrivateTmp=true

[Install]
WantedBy=multi-user.target
EOF

  systemctl daemon-reload
  systemctl enable ru-share-media.service
  info "Systemd service created"
}

main() {
  local arch=$(detect_arch)
  local version=$(get_version)
  
  info "Installing ru-share-media v${version}"
  info "Architecture: $(uname -m) → ${arch}"
  
  install_binary "$arch" "$version"
  setup_zerotier
  create_config
  create_service
  
  echo ""
  echo "${GREEN}Installation complete!${NC}"
  echo "Config: /etc/ru-share-media/config.env"
  echo "Data:   ${DATA_DIR}"
  echo ""
  echo "Start:  systemctl start ru-share-media"
  echo "Status: systemctl status ru-share-media"
  echo "Logs:   journalctl -u ru-share-media -f"
}

main "$@"