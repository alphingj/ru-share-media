# Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `ADMIN_USER` | Admin username | `admin` |
| `ADMIN_PASS` | Admin password (required; random generated if unset) | - |
| `RU_SHARE_MEDIA_PATH` | Media storage directory | `./media` |
| `RU_SHARE_LISTEN` | Listen address | `0.0.0.0:8080` |
| `RU_SHARE_CORS_ORIGIN` | Initial CORS origins (comma-separated, supports CIDR). Can later be updated from admin panel. | `http://localhost:8080` |

## CORS Examples

```bash
# Local only
RU_SHARE_CORS_ORIGIN=http://localhost:8080

# Zerotier network (10.147.17.0/24)
RU_SHARE_CORS_ORIGIN=http://10.147.17.0/24

# Multiple origins
RU_SHARE_CORS_ORIGIN=http://localhost:8080,http://10.147.17.0/24
```