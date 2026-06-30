# Deploy operator runbook

This runbook is the canonical reference when the self-heal chain does not resolve an incident. Reach for it when:

- GitHub Actions is unreachable / returning 5xx
- The self-hosted runner is offline or its token is expired
- A deploy proof fails for an undocumented reason
- The daemon is up but `/v1/health` is consistently hung
- The audit trail is failing to validate

## Self-heal chain reminder

Before reading further, confirm the four self-heal layers are not the fix:

1. **Runner** — systemd `MemoryMax=2G Restart=always` (see `scripts/install-self-hosted-runner.sh`).
2. **Script** — `install-daemon.sh` `watchdog_check`, `binary_version`, `patch_service_unit_execstart`.
3. **Workflow** — `auto-retry-deploy.yml` re-dispatches once on failure.
4. **Audit** — `auto-heal-audit.py` synthesizes `self_heal` rows.

See [`docs/self-heal-chain.md`](self-heal-chain.md) for the layered diagram.

## Symptom → action

### Deploy Live Daemon is failing every run

1. **Tail the deploy run log** for the actual error.
2. If the error is `health verification failed for http://127.0.0.1:8787/v1/health`:
   - **Wait 30s** — musl cold start takes ~10–30s to seed Mem0.
   - If still hung after 30s: `systemctl restart focusa-daemon.service && sleep 30 && curl -fsS --max-time 5 http://127.0.0.1:8787/v1/health`.
   - If still hung: file a daemon-side bug; the deploy automation's behavior is correct.
3. If the error is `binary version mismatch`:
   - `binary_version` should parse the version from filename. If it returns empty for a valid asset, file a bug.
4. If the error is `service ExecStart mismatch`:
   - This should auto-patch in `install-daemon.sh` via `patch_service_unit_execstart`. If it doesn't, run `validate_service_execstart` manually:
     ```bash
     sudo -u github-runner bash scripts/install-daemon.sh --no-restart --no-verify ...
     ```

### GitHub Actions is down / 502 / 5xx

The deploy workflow depends on GitHub. If GitHub is unreachable for an extended period, you have two options:

**Option A — Manual deploy from VPS as the runner user:**

```bash
# As the github-runner user on the VPS
sudo -u github-runner bash
cd /opt/actions-runner-focusa/_work/focusa/focusa
git fetch --tags --force --quiet origin
git checkout v0.9.42-dev
bash scripts/install-daemon.sh \
  --binary target/release/focusa-daemon \
  --service-name focusa-daemon \
  --health-url http://127.0.0.1:8787/v1/health \
  --expected-version 0.9.42-dev
curl -fsS http://127.0.0.1:8787/v1/health
```

**Option B — Wait for GitHub to recover.** The audit recorder and self-heal chain are designed to be safe to retry once GitHub is back. The auto-retry workflow re-dispatches one retry per upstream failure; if that also failed, you will need to re-dispatch manually.

### Self-hosted runner is offline (systemctl inactive)

```bash
# Diagnose
systemctl status actions.runner.Startempire-Wire-focusa.host-focusa-deploy.service -n 30

# Common causes:
# 1. Runner config token expired. Re-register:
sudo -u github-runner /opt/actions-runner-focusa/config.sh remove --token <fresh-token>
sudo -u github-runner /opt/actions-runner-focusa/config.sh --url https://github.com/Startempire-Wire/focusa \
  --token <fresh-token> \
  --name host-focusa-deploy \
  --labels self-hosted,linux,x64,focusa-deploy,production \
  --work _work \
  --replace --unattended
systemctl restart actions.runner.Startempire-Wire-focusa.host-focusa-deploy.service

# 2. Disk full. Run safe cleanup as root:
bash scripts/safe-disk-cleanup.sh --apply

# 3. Kernel OOM-killed and Restart=always not set. Verify drop-in:
ls -la /etc/systemd/system/actions.runner.*.service.d/
cat /etc/systemd/system/actions.runner.*.service.d/override.conf
# Should contain: MemoryMax=2G, Restart=always, RestartSec=15
# If missing, run scripts/install-self-hosted-runner.sh again.
```

### Runner token is expired

The runner config token is short-lived (1h). Re-issue:

```bash
# From a host with `gh` authenticated
gh api -X POST repos/Startempire-Wire/focusa/actions/runners/registration-token --jq '.token'

# On the VPS, as root
RUNNER_TOKEN="<token-from-above>"
sudo -u github-runner /opt/actions-runner-focusa/config.sh remove --token "$(gh api -X POST repos/Startempire-Wire/focusa/actions/runners/remove-token --jq '.token')" || true
sudo -u github-runner /opt/actions-runner-focusa/config.sh --url https://github.com/Startempire-Wire/focusa \
  --token "$RUNNER_TOKEN" \
  --name host-focusa-deploy \
  --labels self-hosted,linux,x64,focusa-deploy,production \
  --work _work \
  --replace --unattended
systemctl restart actions.runner.Startempire-Wire-focusa.host-focusa-deploy.service
```

### Daemon is up but /v1/health is consistently hung

1. **Verify listening:**
   ```bash
   ss -tlnp | grep 8787
   ```
2. **Probe raw TCP:**
   ```bash
   timeout 3 bash -c 'cat </dev/tcp/127.0.0.1/8787 && echo OPEN || echo CLOSED'
   ```
3. **Tail journal:**
   ```bash
   journalctl -u focusa-daemon.service -n 50 --no-pager
   ```
4. **Restart:**
   ```bash
   systemctl restart focusa-daemon.service
   sleep 30  # wait for mem0 seed
   curl -fsS --max-time 5 http://127.0.0.1:8787/v1/health
   ```
5. **If still hung after restart:** the upstream daemon has a bug. Roll back:
   ```bash
   ls -lat /usr/local/lib/focusa/backups/*.bak | head -1
   sudo install -m 0755 /usr/local/lib/focusa/backups/focusa-daemon.<timestamp>.bak /usr/local/bin/focusa-daemon
   systemctl restart focusa-daemon.service
   ```

### Audit trail fails to validate

```bash
python3 scripts/audit-schema.py validate release-proof/audit/audit.jsonl
```

The output lists each malformed row by line number. Common fixes:

- Missing `event` field → add `event: "failure"` (or `"addition"` / `"self_heal"`).
- Missing `ts` field → use `_date_to_ts` helper or add a synthetic ts from `date`.
- Duplicate `id` → de-duplicate by removing the older row (migrate handles this).
- Unknown `category` → add to `VALID_CATEGORIES` in `scripts/audit-schema.py`.

If the audit file is corrupt, regenerate via `auto-heal-audit.py`:

```bash
python3 scripts/auto-heal-audit.py
python3 scripts/audit-schema.py validate release-proof/audit/audit.jsonl
```

### Beads CLI rejecting a close (`bd-evidence` push hook)

The push hook requires `Evidence citations:` in the close reason.

```bash
# Correct
bd close focusa-X --reason "Evidence citations: <github-run-url> | <curl /v1/health output>"

# Wrong
bd close focusa-X --reason "done"
```

## Acceptance criteria for "always newest version live"

The deploy automation is healthy when:

- `/v1/health` returns `{"ok":true,"status":"ok","version":"<tag>"}` (intermittent hangs are tolerated; persistent hangs require a daemon fix).
- `gh release view <tag>` shows all expected assets (gnu + musl + macOS).
- `git describe --tags` on the VPS checkout matches the deployed tag.
- `audit.jsonl` validates via `scripts/audit-schema.py validate`.
- All 5 backup files in `/usr/local/lib/focusa/backups/` are within the last 30 days.
- The runner service is `active` with `MemoryMax=2G Restart=always`.

## Spec 104 phase gate (between implementation phases)

Between P0→P1, P1→P2, and P2→P3, run the phase gate to verify no omissions or deferrals:

```bash
# Full phase gate (static audit + annex coverage + bead closure)
bash scripts/spec104-phase-gate.sh <P0|P1|P2|P3>

# Or just the static surface audit
python3 scripts/audit-schema.py spec104 release-proof/audit/audit.jsonl
```

The gate fails if:

- Any bead in the completed phase is still **OPEN**.
- A new `OnceLock`/`LazyLock`/static mutable global exists without an Annex A remediation row.
- A source file exists in the repo but is absent from the Spec 104 Annex B surface inventory.
- The static audit itself crashes or times out.

**The gate must pass before any implementation starts on the next phase. No exceptions.**

If a new singleton was introduced during the phase (not covered by a previous Annex A row), it must get a new Annex A row before the gate passes — no deferral to a later phase.

## Escalation

If the self-heal chain has not resolved the incident within 30 minutes:

1. Open an issue in the repo with the run URL, the audit row, and the symptom.
2. Tag the issue with `deploy-automation` and link the bead.
3. Run `bd create "Investigate <symptom> in <subsystem>" -p 1` to track the work.
4. Do not silence a failure by removing the audit row or the guard.