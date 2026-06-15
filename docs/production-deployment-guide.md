# Production deployment guide

## Scope

This document covers a supported production deployment pattern for the Focusa daemon and operator tools on a Linux VPS behind HTTPS.

## 1) Install or install directory

Use system binaries from a release tag, then run as a dedicated user (example: `focusa`):

```bash
mkdir -p /opt/focusa/bin
cp focusa focusa-daemon /opt/focusa/bin/
chown -R focusa:focusa /opt/focusa
```

Set `FOCUSA_PROJECT_ROOT` and `FOCUSA_CONTINUITY_ID` in service env:

```bash
export FOCUSA_PROJECT_ROOT=/opt/focusa/repo
export FOCUSA_CONTINUITY_ID=focusa-prod-01
export FOCUSA_LOG_LEVEL=info
export FOCUSA_PUBLIC_STREAM=1
export FOCUSA_PAIRING_URL=https://focusa.example.com
```

## 2) systemd daemon service

Create `/etc/systemd/system/focusa.service`:

```ini
[Unit]
Description=Focusa daemon
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=focusa
Group=focusa
WorkingDirectory=/opt/focusa
Environment=FOCUSA_PROJECT_ROOT=/opt/focusa/repo
Environment=FOCUSA_CONTINUITY_ID=focusa-prod-01
Environment=FOCUSA_LOG_LEVEL=info
Environment=FOCUSA_API_BASE_URL=http://127.0.0.1:8787
Environment=FOCUSA_PAIRING_URL=https://focusa.example.com
ExecStart=/opt/focusa/bin/focusa-daemon
Restart=on-failure
RestartSec=3
NoNewPrivileges=yes
PrivateTmp=yes
ProtectSystem=full
ProtectHome=true

[Install]
WantedBy=multi-user.target
```

Enable:

```bash
systemctl daemon-reload
systemctl enable --now focusa
systemctl status focusa
journalctl -u focusa -f
```

## 3) Reverse proxy + TLS

Use Nginx/Caddy reverse proxy to expose HTTPS on 443 only and keep daemon on localhost-only port.

Example Nginx block:

```nginx
server {
    listen 80;
    server_name focusa.example.com;
    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl;
    server_name focusa.example.com;

    ssl_certificate     /etc/letsencrypt/live/focusa.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/focusa.example.com/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:8787;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        # Keep long-lived websocket/event style calls safe
        proxy_read_timeout 3600s;
    }
}
```

TLS: issue certs with certbot

```bash
certbot --nginx -d focusa.example.com
```

## 4) Rate limiting (nginx)

Basic per-IP burst/rate controls:

```nginx
limit_req_zone $binary_remote_addr zone=api_limit:10m rate=5r/s;

server {
    location / {
        limit_req zone=api_limit burst=20 nodelay;
        limit_req_status 429;
    }
}
```

## 5) Log rotation

`journalctl` defaults are often enough, but persistent compact logs are preferred:

```ini
# /etc/systemd/journald.conf
SystemMaxUse=200M
SystemKeepFree=10%
MaxRetentionSec=2week
```

For file logs, rotate with `/etc/logrotate.d/focusa`:

```text
/var/log/focusa/*.log {
    daily
    rotate 14
    compress
    missingok
    notifempty
    copytruncate
}
```

## 6) Health checks

From the VPS:

```bash
curl -sS https://focusa.example.com/v1/health | jq .
curl -sS https://focusa.example.com/v1/workpoint/current?project_root=/opt/focusa/repo&continuity_id=focusa-cont-01 | jq .
```

## 7) Release asset provenance

Production installs should pull artifacts from the signed GitHub release tag for the target version and verify checksums/signature policy in your org process.

Recommended checklist:

- Deploy daemon + CLI/TUI artifacts from latest tag page
- Verify `FOCUSA_PAIRING_URL` resolves to public host used by menubar pairing paths
- Confirm reverse proxy/TLS and rate limit are active before opening the service
- Record deployment evidence in runtime/beads workflow as routine evidence
