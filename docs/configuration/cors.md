# CORS Configuration

## Supported Formats

1. **Exact origins**: `http://localhost:8080`
2. **CIDR subnets**: `http://10.147.17.0/24`
3. **Multiple**: `http://localhost:8080,http://10.147.17.0/24`

## Examples

```bash
# Zerotier only - most restrictive
RU_SHARE_CORS_ORIGIN="http://10.147.17.0/24"

# Development + Zerotier
RU_SHARE_CORS_ORIGIN="http://localhost:8080,http://127.0.0.1:8080,http://10.147.17.0/24"

# Multiple subnets
RU_SHARE_CORS_ORIGIN="http://localhost:8080,http://10.147.17.0/24,http://192.168.1.0/24"

# With HTTPS reverse proxy
RU_SHARE_CORS_ORIGIN="https://media.yourdomain.com"
```

## Testing

```bash
curl -H "Origin: http://10.147.17.5" -I http://10.147.17.2:8080/api/health
```