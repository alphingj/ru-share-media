# Docker

```bash
docker run -d \
  -p 8080:8080 \
  -v $(pwd)/media:/app/media \
  -v $(pwd)/data:/app/data \
  -e ADMIN_USER=admin \
  -e ADMIN_PASS=secure_password \
  -e RU_SHARE_MEDIA_PATH=/app/media \
  -e RU_SHARE_LISTEN=0.0.0.0:8080 \
  ghcr.io/alphingj/ru-share-media:latest
```