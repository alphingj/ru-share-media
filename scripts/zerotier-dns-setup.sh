#!/bin/bash

echo "=== Zerotier DNS Setup ==="

# Check if device is connected to Zerotier
if ! command -v zerotier-cli &> /dev/null; then
  echo "Zerotier not installed. Run zerotier-setup.sh first."
  exit 1
fi

# Get current Zerotier addresses
echo "Current Zerotier addresses:"
zerotier-cli listnetworks 2>/dev/null | grep -v "^Name" | while read line; do
  if [ -n "$line" ]; then
    echo "  $line"
  fi
done

# Get local IP info
echo ""
echo "Network interfaces:"
ip -4 addr show | grep -oP '(?<=inet\s)\d+(\.\d+){3}' | head -5

# Configure DNS settings
read -p "Enter device hostname (e.g., media-server): " HOSTNAME
read -p "Enter Zerotier network subnet (e.g., 10.147.17): " SUBNET

if [ -n "$HOSTNAME" ] && [ -n "$SUBNET" ]; then
  # Add to /etc/hosts for local resolution
  echo ""
  echo "Adding to /etc/hosts..."
  CURRENT_IP=$(ip -4 addr show | grep -oP '(?<=inet\s)\d+(\.\d+){3}' | grep "^$SUBNET" | head -1)
  
  if [ -z "$CURRENT_IP" ]; then
    # Try to get from Zerotier
    CURRENT_IP=$(zerotier-cli info | grep -oP '(?<=address\s)\d+(\.\d+){3}' | head -1)
  fi
  
  if [ -n "$CURRENT_IP" ]; then
    echo "$CURRENT_IP $HOSTNAME.local" | sudo tee -a /etc/hosts > /dev/null
    echo "Added $CURRENT_IP $HOSTNAME.local to /etc/hosts"
  fi
fi

# Show dnsmasq config template
echo ""
echo "=== DNSMasq Configuration Template ==="
echo "Add this to /etc/dnsmasq.conf for network-wide DNS:"
cat << 'EOF'
# Zerotier DNS configuration
address=/media-server/10.147.17.2
address=/nas/10.147.17.3
address=/laptop/10.147.17.4
address=/zerotier.local/10.147.17.1

# Reverse DNS
ptr-address=2.17.147.10.in-addr.arpa
EOF

echo ""
echo "=== mDNS/Avahi Configuration ==="
echo "For .local domain resolution, install avahi:"
echo "  sudo apt install avahi-daemon avahi-utils  # Debian/Ubuntu"
echo "  sudo systemctl enable --now avahi-daemon"
echo ""
echo "Then configure:"
cat << 'EOF'
# /etc/avahi/avahi-daemon.conf
[hostalias]
host-alias=media-server
host-alias=zerotier
EOF

echo ""
echo "To find your device's Zerotier IP:"
echo "  zerotier-cli info"
echo ""
echo "To check if hostname resolves:"
echo "  ping media-server.local"