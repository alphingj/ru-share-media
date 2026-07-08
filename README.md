# ru-share-media

Netflix-style self-hosted media server with session-based authentication.

## Features

- Session-based authentication with Argon2id password hashing
- CSRF protection on all state-changing endpoints
- Admin-managed user accounts with RBAC
- Media scanning with ffprobe metadata extraction
- SQLite database for metadata storage
- Rate limiting per IP
- CORS support with CIDR subnet ranges
- Responsive Netflix-style UI

## Installation

### From GitHub Releases

Download the latest release for your platform from the [Releases](https://github.com/alphingj/ru-share-media/releases) page.

Extract and run:
```bash
tar -xzf ru-share-media.tar.gz
./ru-share-media
```

### From Source

```bash
git clone https://github.com/alphingj/ru-share-media.git
cd ru-share-media
cargo build --release
./target/release/ru-share-media
```

## Configuration

Environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `ADMIN_USER` | Admin username | `admin` |
| `ADMIN_PASS` | Admin password (required; if not set, a random one is generated) | - |
| `RU_SHARE_MEDIA_PATH` | Media storage directory | `./media` |
| `RU_SHARE_LISTEN` | Listen address | `0.0.0.0:8080` |
| `RU_SHARE_CORS_ORIGIN` | CORS origins (comma-separated, supports CIDR) | `http://localhost:8080` |

## Usage

1. Start the server
2. Login with admin credentials
3. Configure CORS origins if needed
4. Upload media files to the media directory
5. Run a scan via the admin panel to index media

## API Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/login` | ❌ | Login with username/password |
| GET | `/api/logout` | ✅ | Invalidate session |
| GET | `/api/library` | ✅ | Browse media library |
| POST | `/api/scan` | ✅+Admin | Trigger media scan |
| POST | `/api/admin/users` | ✅+Admin | Create new user |
| GET | `/api/admin/users` | ✅+Admin | List all users |
| DELETE | `/api/admin/users/:id` | ✅+Admin | Delete user |
| GET | `/api/health` | ❌ | Health check |

## Security

- Passwords hashed with Argon2id
- UUIDs used for session IDs (not predictable)
- CSRF tokens required for state-changing requests
- HttpOnly SameSite cookies for session
- 60 requests/minute rate limiting per IP
- Path traversal protection
- SQL injection protection via parameterized queries