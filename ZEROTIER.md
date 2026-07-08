# Zerotier Setup for ru-share-media

This guide explains how to set up Zerotier mesh VPN for remote access to your ru-share media server.

## Architecture

```
                    Zerotier Network (10.147.17.0/24)
                           |
         ┌─────────────────┼─────────────────┐
         │                 │                 │
    media-server        dns-server       other-devices
    10.147.17.2      10.147.17.1      10.147.17.x
         │
    ru-share-media
    Port 8080
```

## Quick Start

### 1. Create Zerotier Network

1. Sign up at https://my.zerotier.com
2. Create new network
3. Enable "Auto-assign from range" for IP assignments
4. Note the Network ID

### 2. Install Zerotier on Media Server

```bash
# Run the setup script
sudo ./scripts/zerotier-setup.sh
```

Or manually:
```bash
curl -s https://install.zerotier.com | bash
sudo systemctl enable --now zerotier-one
sudo zerotier-cli join YOUR_NETWORK_ID
```

### 3. Approve Device

1. Go to https://my.zerotier.com/network/YOUR_NETWORK_ID
2. Add your server's Node ID (found via `zerotier-cli info`)
3. Authorize the device

### 4. Configure DNS

Run the DNS setup script:
```bash
./scripts/zerotier-dns-setup.sh
```

This will:
- Add hostname to `/etc/hosts`
- Show dnsmasq configuration template
- Configure mDNS/Avahi for .local domains

### 5. Docker Deployment (Optional)

Use the provided `docker-compose.yml`:
```bash
# Set environment variables
export ZEROTIER_NETWORK_ID=your_network_id
export ADMIN_PASS=secure_password

# Start services
docker-compose up -d
```

## Domain Names

After setup, you can access the media server by:

| Hostname | IP Address | Description |
|----------|------------|-------------|
| media-server.local | 10.147.17.2 | Main media server |
| media-server:8080 | 10.147.17.2:8080 | Direct port access |
| zt:8080 | 10.147.17.2:8080 | Alternative |

## Accessing from Remote Devices

### From Linux/macOS

```bash
# Install Zerotier
curl -s https://install.zerotier.com | bash

# Join network
sudo zerotier-cli join YOUR_NETWORK_ID

# Access media server
http://media-server.local:8080
```

### From Windows

1. Download Zerotier from https://www.zerotier.com/download.shtml
2. Install and run
3. In the app, click "Join Network" and enter your Network ID
4. Access via browser: `http://media-server.local:8080`

### From Android/iOS

1. Install "Zerotier" app from app store
2. Open app and join your network
3. Access via browser: `http://media-server.local:8080`

## Network Configuration in Zerotier Central

Set these options in your Zerotier network settings:

### Managed Routes
```
Prefix: 10.147.17.0/24
Quality: 1
```

### DNS Servers
```
10.147.17.1
```

### Enable Options
- Allow Managed Routes
- Allow Assign IP4
- Allow DHCP

## Security Considerations

1. **Firewall**: Restrict ru-share-media to only listen on Zerotier interface:
   ```bash
   # In ru-share-media.service
   Environment=RU_SHARE_LISTEN=10.147.17.2:8080
   ```

2. **Use HTTPS**: Set up reverse proxy with Caddy or Nginx:
   ```bash
   # Example with Caddy
   media-server.local {
       reverse_proxy 10.147.17.2:8080
       tls internal
   }
   ```

3. **Strong Passwords**: Change default admin password immediately

## Troubleshooting

```bash
# Check Zerotier status
zerotier-cli info

# List networks
zerotier-cli listnetworks

# Check IP address
ip addr show

# Test DNS resolution
nslookup media-server.local
ping media-server.local

# Check ru-share-media logs
journalctl -u ru-share-media -f
```

## Files

| File | Purpose |
|------|---------|
| `scripts/zerotier-setup.sh` | Install and join Zerotier network |
| `scripts/zerotier-dns-setup.sh` | Configure DNS for devices |
| `docker-compose.yml` | Docker deployment with Zerotier |
| `dnsmasq.conf` | DNS configuration for network |
| `scripts/ru-share-media.service` | Systemd service file |