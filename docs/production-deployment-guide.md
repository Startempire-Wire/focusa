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

Only supported path:

- release from `scripts/create-dev-release-tag.sh --base 0.9 --push`
- GitHub `CI` proves tests/clippy/static gates
- GitHub `Release` workflow builds/tag-stamps artifacts
- GitHub `Deploy Live Daemon` workflow installs the new daemon on the VPS
- installer restarts systemd, verifies `/v1/health`, and auto-rolls back on failure
- Auto Heal + Watchdog detect and retry failures without manual deploy intervention

Local release builds and partial deploy workflow shortcuts are forbidden. See [`docs/canonical-live-release-pipeline.md`](canonical-live-release-pipeline.md).

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
2. **Rust lifecycle layer** — exact process ownership, atomic full-release/unit promotion, bounded acceptance, and rollback
3. **Workflow layer** — `auto-retry-deploy.yml` is quarantined pending exact failure/rollback evidence
4. **Audit layer** — `auto-heal-audit.py` synthesizes `self_heal` rows for every failure missing one

See [`docs/self-heal-chain.md`](self-heal-chain.md) for the layered diagram.

When the chain does not resolve the incident, follow [`docs/deploy-runbook.md`](deploy-runbook.md).

Rollback uses the signed, full-release journal:

```bash
focusa update rollback --dry-run=false --yes
```

Hand-copying one daemon binary is forbidden because it creates mixed-version
surfaces and bypasses unit/process/health settlement.

## 6) First-install from a clean VPS

If the VPS has never run the Focusa daemon:

1. Install the self-hosted deployment runner through
   `scripts/install-self-hosted-runner.sh`.
2. Configure the repository deployment variables, including the musl target and
   loopback health URL.
3. Trigger the full release/deploy pipeline.
4. Let `focusa install --system-install` create the state root, promote all four
   signed binaries, render systemd, activate exactly one daemon, and verify
   health plus CallGraph capability.

Do not preinstall a checkout binary or hand-write the unit; doing so creates a
second lifecycle authority before the canonical transaction runs.

## 7) Menubar releases

The menubar app is released from the same Git tag through GitHub Actions. The release assets include the latest macOS DMGs and `.app.tar.gz` archives built from the tagged version.
