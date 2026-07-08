# API Reference

All endpoints require authentication unless noted.

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/login` | No | Login |
| GET | `/api/logout` | Yes | Logout |
| GET | `/api/library` | Yes | Browse media |
| POST | `/api/scan` | Admin | Trigger scan |
| POST | `/api/admin/users` | Admin | Create user |
| GET | `/api/admin/users` | Admin | List users |
| DELETE | `/api/admin/users/:id` | Admin | Delete user |

## Login

```bash
curl -X POST http://10.147.17.2:8080/api/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"yourpass"}'
```

Returns CSV cookie `session_id` and CSRF token.