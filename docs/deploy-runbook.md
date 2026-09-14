# Deploy operator runbook

> **Canonical rule:** live build/deploy uses the full GitHub release pipeline only. See [`docs/canonical-live-release-pipeline.md`](canonical-live-release-pipeline.md). This runbook is for fixing the pipeline/system when automation is unhealthy, not for manual release builds or hand-installs.

This runbook is the canonical reference when the self-heal chain does not resolve an incident. Reach for it when:

- GitHub Actions is unreachable / returning 5xx
- The self-hosted runner is offline or its token is expired
- A deploy proof fails for an undocumented reason
- The daemon is up but `/v1/health` is consistently hung
- The audit trail is failing to validate

## Self-heal chain reminder

Before reading further, confirm the four self-heal layers are not the fix:

1. **Runner** — systemd `MemoryMax=2G Restart=always` (see `scripts/install-self-hosted-runner.sh`).
2. **Rust lifecycle** — deployment lock, operator halt, exact process, full release/unit transaction, bounded acceptance, and rollback.
3. **Workflow** — `auto-retry-deploy.yml` remains quarantined until an exact failure and rollback state are known.
4. **Audit** — `auto-heal-audit.py` synthesizes `self_heal` rows.

See [`docs/self-heal-chain.md`](self-heal-chain.md) for the layered diagram.

## Symptom → action

### Deploy Live Daemon is failing every run

1. **Tail the deploy run log** for the exact Rust transaction error.
2. If health or CallGraph acceptance fails, confirm the receipt says that prior binaries and unit were restored; do not manually restart the rejected candidate.
3. If release version identity fails, verify the immutable tag and all signed release assets; do not substitute a checkout build.
4. If process identity fails, inspect `systemctl show focusa-daemon.service -p MainPID -p ExecStart` and `/proc/<MainPID>/exe`. Do not kill by process name; resolve the exact unmanaged process owner and authority first.
5. If `RefuseManualStart=yes` is reported, the operator halt is working. Do not remove or bypass it without explicit operator approval.

### GitHub Actions is down / 502 / 5xx

The deploy workflow depends on GitHub. If GitHub is unreachable for an extended period, **do not manually build or install a daemon**. Recovery is:

1. Keep the current production daemon running.
2. Fix GitHub access, runner registration, repository permissions, or workflow syntax.
3. Let `Release Pipeline Watchdog` and `Auto Heal Release Pipeline` resume the full chain.
4. If a workflow bug caused the outage, patch the workflow on `main`, then create a fresh tag through:
   ```bash
   scripts/create-dev-release-tag.sh --base 0.9 --push
   ```

Manual `target/release` deploys are forbidden because they bypass CI, Release asset generation, checksum/codesign validation, Deploy health proof, and audit/self-heal records.

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
4. **Do not manually restart through an active operator halt.** If start authority exists, rerun the exact immutable deployment so the Rust lifecycle performs bounded health verification and keeps rollback atomic.
5. **If the accepted runtime later regresses:** inspect the rollback plan, then use `focusa update rollback --dry-run=false --yes`. Never restore only the daemon binary from a mixed-version backup.

### Triage recent self-heal audit failures

Use the read-only operator summary before opening raw JSONL:

```bash
# Recent failures with retry policy, remediation, source refs, run URL
python3 scripts/audit-failure-summary.py --limit 10

# Filter a deterministic class
python3 scripts/audit-failure-summary.py --class ci_clippy_failure --limit 5

# Machine-readable handoff for agents or incident notes
python3 scripts/audit-failure-summary.py --class ci_clippy_failure --limit 5 --json
```

The output includes `failure_class`, `retry_policy`, remediation (`remediation_template`/`fix`), `source_refs`, `linked_run`, and `log_url` when present.

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