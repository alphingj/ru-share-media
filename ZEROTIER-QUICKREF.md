# Zerotier Quick Reference

## After Setup - Commands to Run

```bash
# Check if connected to Zerotier
zerotier-cli info
# Expected output: 200 <NODE_ID> <STATUS> 10.147.17.2

# Find your device's Zerotier IP
ip addr show | grep 10.147

# Test DNS resolution
ping media-server.local

# Access the media server
# Browser: http://media-server.local:8080
# Or: http://10.147.17.2:8080
```

## Common Network IDs

Your Zerotier Network ID looks like:
```
d21ab3a123456789
```

Find it at: https://my.zerotier.com/network

## Device Hostnames

| Device | Hostname | IP |
|--------|----------|-----|
| Media Server | media-server.local | 10.147.17.2 |
| DNS Server | dns-server.local | 10.147.17.1 |
| NAS | nas.local | 10.147.17.3 |
| Laptop | laptop.local | 10.147.17.4 |

## One-liner Setup

```bash
# Install Zerotier and join network
curl -s https://install.zerotier.com | bash && \
sudo systemctl enable --now zerotier-one && \
sudo zerotier-cli join YOUR_NETWORK_ID
```

## Uninstall Zerotier

```bash
sudo systemctl stop zerotier-one
sudo systemctl disable zerotier-one
sudo zerotier-cli leave YOUR_NETWORK_ID
sudo apt remove zerotier-one
sudo rm -rf /var/lib/zerotier-one
```