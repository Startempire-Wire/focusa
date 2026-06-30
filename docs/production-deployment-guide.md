# Production deployment guide

## Scope

Supported Linux VPS deployment pattern for the Focusa daemon behind HTTPS, with systemd service management and GitHub Actions live deploy automation.

See also: `docs/live-release-automation.md`

## 1) Install layout

Use a stable install root and keep the daemon system-managed:

```bash
sudo mkdir -p /opt/focusa /opt/focusa/repo
sudo mkdir -p /usr/local/bin
```

Recommended runtime env:

```bash
export FOCUSA_PROJECT_ROOT=/opt/focusa/repo
export FOCUSA_CONTINUITY_ID=focusa-prod-01
export FOCUSA_LOG_LEVEL=info
export FOCUSA_PUBLIC_STREAM=1
export FOCUSA_PAIRING_URL=https://focusa.example.com
```

## 2) systemd service

Use `focusa-daemon.service` as the canonical service name.

Create `/etc/systemd/system/focusa-daemon.service`:

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
Environment=FOCUSA_PUBLIC_STREAM=1
Environment=FOCUSA_PAIRING_URL=https://focusa.example.com
ExecStart=/usr/local/bin/focusa-daemon
Restart=always
RestartSec=2

[Install]
WantedBy=multi-user.target
```

Enable it:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now focusa-daemon.service
```

## 3) Health proof

```bash
curl -sS http://127.0.0.1:8787/v1/health | jq .
```

Expected fields include:

- `ok: true`
- `status: ok`
- `version: <release version>`

## 4) Live release automation

Preferred path:

- release from `scripts/create-dev-release-tag.sh --push`
- GitHub `Release` workflow builds/tag-stamps artifacts
- GitHub `Deploy Live Daemon` workflow installs the new daemon on the VPS
- installer restarts systemd, verifies `/v1/health`, and auto-rolls back on failure

Do not run ad-hoc background daemon instances in parallel with systemd-managed production.

## 5) Rollback

Two rollback layers exist:

1. **automatic rollback** inside `scripts/install-daemon.sh` if start/health/version checks fail
2. **manual rollback** by re-running the `Deploy Live Daemon` workflow with an older release tag

## 6) Menubar releases

The menubar app is released from the same Git tag through GitHub Actions. The release assets include the latest macOS DMGs and `.app.tar.gz` archives built from the tagged version.
