# Docker Deployment

Uses three containers: `ru-share`, `zerotier-one`, and `dnsmasq`.

```bash
# Configure
cp .env.example .env
nano .env  # set ZEROTIER_NETWORK_ID and ADMIN_PASS

# Start
docker-compose up -d
```

The `zerotier-one` container joins your Zerotier network automatically. DNS is provided by `dnsmasq` on `10.147.17.1`.