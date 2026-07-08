#!/bin/bash
# Test script for ru-share-media

set -euo pipefail

echo "Testing ru-share-media..."

# Check if server is running
if curl -sf http://localhost:8080/api/health > /dev/null; then
  echo "✓ Health check passed"
else
  echo "✗ Server not responding"
  exit 1
fi

# Test CORS headers
echo "Testing CORS configuration..."
curl -sf -H "Origin: http://localhost:8080" -I http://localhost:8080/api/library | grep -q "Access-Control-Allow-Origin" && echo "✓ CORS header present" || echo "✗ CORS header missing"

# Check ZeroTier
if command -v zerotier-cli &>/dev/null; then
  ZT_IP=$(ip addr show | grep -oP '(?<=inet )10\.\d+\.\d+\.\d+' | head -1)
  if [ -n "$ZT_IP" ]; then
    echo "✓ ZeroTier IP: $ZT_IP"
  else
    echo "! ZeroTier network not joined"
  fi
else
  echo "! ZeroTier not installed"
fi

# Check FFmpeg
if command -v ffprobe &>/dev/null; then
  echo "✓ FFmpeg available: $(ffprobe -version | head -1)"
else
  echo "✗ FFmpeg not installed"
fi

echo
echo "To join a ZeroTier network:"
echo "  sudo zerotier-cli join <NETWORK_ID>"
echo
echo "Then update CORS:"
echo "  export RU_SHARE_CORS_ORIGIN=\"http://localhost:8080,http://192.168.1.0/24,http://${ZT_IP}:8080\""
echo
echo "Then restart the server with the new config."