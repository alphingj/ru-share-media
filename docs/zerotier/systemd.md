# Systemd Deployment

## Install Service

```bash
sudo cp scripts/ru-share-media.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable ru-share-media
```

## Configure CORS

Edit `Environment=RU_SHARE_CORS_ORIGIN` in the service file:

```ini
Environment=RU_SHARE_CORS_ORIGIN=http://localhost:8080,http://10.147.17.0/24
```

## Start

```bash
sudo systemctl start ru-share-media
sudo systemctl status ru-share-media
```