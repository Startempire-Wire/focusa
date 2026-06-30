# Production deployment guide

## Scope

Supported Linux VPS deployment pattern for the Focusa daemon behind HTTPS, with systemd service management and GitHub Actions live deploy automation.

See also:

- [`docs/live-release-automation.md`](live-release-automation.md) — tag-driven release/deploy pipeline
- [`docs/self-heal-chain.md`](self-heal-chain.md) — 4-layer self-heal architecture
- [`docs/failures-playbook.md`](failures-playbook.md) — human-readable failure index
- [`docs/deploy-runbook.md`](deploy-runbook.md) — operator incident playbook
- [`release-proof/audit/categories.md`](../release-proof/audit/categories.md) — failure category playbook
- [`CHANGELOG.md`](../CHANGELOG.md) — auto-generated changelog from the audit trail

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

### Re-tagging a published tag

When iterating on a release (e.g. the tag already points at an older SHA after a follow-up commit), the canonical pattern is:

```bash
git tag -d v0.9.42-dev
git push origin :refs/tags/v0.9.42-dev
git tag -a v0.9.42-dev -m "v0.9.42-dev" <sha>
git push origin v0.9.42-dev
```

Re-tagging a tag that has already been shipped to users is forbidden by semver; this is only for dev tags (`v0.9.42-dev`) before the corresponding release is finalized.

Do not run ad-hoc background daemon instances in parallel with systemd-managed production.

## 5) Rollback

Four self-heal layers exist:

1. **Runner layer** — systemd `MemoryMax=2G Restart=always` (`scripts/install-self-hosted-runner.sh`)
2. **Script layer** — `install-daemon.sh` `watchdog_check` + auto-rollback on health failure
3. **Workflow layer** — `auto-retry-deploy.yml` re-dispatches once on `workflow_run` failure
4. **Audit layer** — `auto-heal-audit.py` synthesizes `self_heal` rows for every failure missing one

See [`docs/self-heal-chain.md`](self-heal-chain.md) for the layered diagram.

When the chain does not resolve the incident, follow [`docs/deploy-runbook.md`](deploy-runbook.md).

Manual rollback recipe:

```bash
ls -lat /usr/local/lib/focusa/backups/*.bak | head -1
sudo install -m 0755 /usr/local/lib/focusa/backups/focusa-daemon.<timestamp>.bak /usr/local/bin/focusa-daemon
sudo systemctl restart focusa-daemon.service
```

## 6) First-install from a clean VPS

If the VPS has never run the Focusa daemon:

```bash
# 1. Install daemon binary
sudo install -m 0755 <built-or-downloaded-binary> /usr/local/bin/focusa-daemon
sudo mkdir -p /usr/local/lib/focusa/backups /var/log/focusa
sudo chown github-runner:github-runner /usr/local/lib/focusa /var/log/focusa

# 2. Write a fresh systemd unit (install-daemon.sh will auto-patch this on first deploy)
sudo tee /etc/systemd/system/focusa-daemon.service >/dev/null <<'EOF'
[Unit]
Description=Focusa Metacognition Daemon (Rust)
After=network.target
Wants=network.target
[Service]
Type=simple
ExecStart=/usr/local/bin/focusa-daemon
WorkingDirectory=/usr/local/lib/focusa
Restart=on-failure
RestartSec=5
Environment=FOCUSA_BIND=127.0.0.1:8787
Environment=FOCUSA_HOME=/usr/local/lib/focusa
[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable --now focusa-daemon.service
sleep 30
curl -fsS --max-time 5 http://127.0.0.1:8787/v1/health

# 3. Install self-hosted runner
sudo bash scripts/install-self-hosted-runner.sh

# 4. Configure GH repo variables (optional, defaults shown):
#    FOCUSA_DEPLOY_ASSET_SUFFIX=x86_64-unknown-linux-musl
#    FOCUSA_DEPLOY_HEALTH_URL=http://127.0.0.1:8787/v1/health

# 5. Trigger the first deploy via GitHub Actions
gh workflow run 'Deploy Live Daemon' --ref main -f release_tag=v0.9.42-dev
```

## 7) Menubar releases

The menubar app is released from the same Git tag through GitHub Actions. The release assets include the latest macOS DMGs and `.app.tar.gz` archives built from the tagged version.
