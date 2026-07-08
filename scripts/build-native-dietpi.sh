#!/bin/bash
set -euo pipefail

# Native build script for DietPi / ARM
# Installs dependencies and builds ru-share-media locally on the device.

SUDO=${SUDO:-sudo}

info() { printf '→ %s\n' "$*"; }
err() { printf 'ERROR: %s\n' "$*" >&2; exit 1; }

[ "$(id -u)" -eq 0 ] || err "Run with sudo or as root"

info "Updating package lists..."
${SUDO} apt-get update

info "Installing build dependencies..."
${SUDO} apt-get install -y --no-install-recommends \
  build-essential \
  pkg-config \
  libssl-dev \
  sqlite3 \
  libsqlite3-dev \
  cmake \
  curl \
  ca-certificates

info "Installing rustup + stable toolchain if missing..."
if ! command -v rustup >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
  # shellcheck disable=SC1091
  source "$HOME/.cargo/env"
fi

rustup default stable
rustc --version
cargo --version

info "Building ru-share-media in release mode (native ARM)..."
cargo build --release

info "Build complete: target/release/ru-share-media"
ls -lh target/release/ru-share-media
