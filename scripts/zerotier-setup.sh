#!/bin/bash
set -e

echo "=== Zerotier Setup Script ==="

# Check if running as root
if [ "$EUID" -ne 0 ]; then
  echo "Please run with sudo or as root"
  exit 1
fi

# Install Zerotier
if ! command -v zerotier-one &> /dev/null; then
  echo "Installing Zerotier..."
  curl -s https://install.zerotier.com | bash
else
  echo "Zerotier already installed"
fi

# Enable and start Zerotier service
systemctl enable zerotier-one
systemctl start zerotier-one

# Join network
read -p "Enter your Zerotier Network ID: " NETWORK_ID
zerotier-cli join "$NETWORK_ID"

echo ""
echo "=== Next Steps ==="
echo "1. Go to https://my.zerotier.com and approve this device for network '$NETWORK_ID'"
echo "2. Once approved, the device will get a 10.x.x.x address"
echo "3. Run: ./scripts/zerotier-dns-setup.sh to configure DNS"
echo ""
echo "Device ID: $(zerotier-cli info | awk '{print $2}')"