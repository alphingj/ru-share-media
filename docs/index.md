# ru-share-media Documentation

Netflix-style self-hosted media server with session-based authentication.

## Quick Links

- [Installation](installation/releases.md)
- [Configuration](configuration/env.md)
- [Zerotier Setup](zerotier/overview.md)
- [API Reference](api.md)

## Features

- Session-based authentication with Argon2id password hashing
- CSRF protection on all state-changing endpoints
- Admin-managed user accounts with RBAC
- Media scanning with ffprobe metadata extraction
- SQLite database for metadata storage
- Rate limiting per IP
- CORS support with CIDR subnet ranges
- Responsive Netflix-style UI

## Quick Start

```bash
# From GitHub Releases
wget https://github.com/alphingj/ru-share-media/releases/latest/download/ru-share-media.tar.gz
tar -xzf ru-share-media.tar.gz
./ru-share-media

# Or with Docker
docker run -d \
  -p 8080:8080 \
  -v ./media:/app/media \
  -e ADMIN_PASS=yourpassword \
  ghcr.io/alphingj/ru-share-media:latest
```

Access at `http://localhost:8080`

## Secure Remote Access with Zerotier

For secure access from anywhere without port forwarding, see the [Zerotier Setup Guide](zerotier/overview.md).

## Links

- [GitHub Repository](https://github.com/alphingj/ru-share-media)
- [Releases](https://github.com/alphingj/ru-share-media/releases)
- [Issues](https://github.com/alphingj/ru-share-media/issues)