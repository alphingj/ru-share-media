# Zerotier Quick Reference

## Check Status

```bash
zerotier-cli info
ip addr show | grep 10.147
ping media-server.local
```

## Join/Leave

```bash
sudo zerotier-cli join NETWORK_ID
sudo zerotier-cli leave NETWORK_ID
```

## Device IPs

| Device | Hostname | IP |
|--------|----------|-----|
| Media Server | media-server.local | 10.147.17.2 |
| DNS Server | dns-server.local | 10.147.17.1 |

## Docker

```bash
export ZEROTIER_NETWORK_ID=your-network-id
docker-compose up -d
```

## Uninstall

```bash
sudo systemctl stop zerotier-one
sudo apt remove zerotier-one
```