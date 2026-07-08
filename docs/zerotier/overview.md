# Zerotier Overview

[Zerotier](https://www.zerotier.com) creates a secure mesh VPN network allowing devices to connect as if on the same LAN—no port forwarding or public IP needed.

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

```bash
# 1. Join network
sudo zerotier-cli join YOUR_NETWORK_ID

# 2. Approve device at https://my.zerotier.com/network/YOUR_NETWORK_ID

# 3. Configure CORS for Zerotier
RU_SHARE_CORS_ORIGIN="http://10.147.17.0/24"

# 4. Access from any device
http://media-server.local:8080
```