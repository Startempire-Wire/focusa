# Self-heal chain

This document describes how the Focusa deploy pipeline self-heals at every layer. Read this before debugging a deploy failure or adding a new self-heal branch.

## Layered defenses

The pipeline has four self-heal layers, each absorbing a different failure class:

```
┌────────────────────────────────────────────────────────────────────┐
│ Layer 4 — Audit                                                    │
│   scripts/auto-heal-audit.py → release-proof/audit/audit.jsonl    │
│   On every CI / Release / Deploy / workflow_run / hourly schedule. │
│   Synthesizes self_heal rows for every failure lacking one.       │
└────────────────────────────────────────────────────────────────────┘
                              ▲
┌────────────────────────────────────────────────────────────────────┐
│ Layer 3 — Workflow                                                 │
│   .github/workflows/auto-retry-deploy.yml                          │
│   On workflow_run failure of Deploy Live Daemon: re-dispatches     │
│   once with the same tag + musl asset. Never retries workflow_run- │
│   triggered deploys (no loops).                                    │
└────────────────────────────────────────────────────────────────────┘
                              ▲
┌────────────────────────────────────────────────────────────────────┐
│ Layer 2 — Script (install-daemon.sh)                               │
│   watchdog_check() + watchdog_loop() — wall clock + RSS budget     │
│   binary_version() — filename-first parser; never wedges          │
│   patch_service_unit_execstart() — auto-rewrites stale unit       │
│   curl --max-time 5 — caps a single health probe                  │
│   wait_for_health 60 — 60s budget for musl cold start             │
└────────────────────────────────────────────────────────────────────┘
                              ▲
┌────────────────────────────────────────────────────────────────────┐
│ Layer 1 — Runner (systemd drop-in)                                 │
│   MemoryMax=2G                                                     │
│   Restart=always                                                   │
│   RestartSec=15                                                    │
│   Set up by scripts/install-self-hosted-runner.sh and applied live │
│   to /etc/systemd/system/actions.runner.*.service.d/override.conf  │
└────────────────────────────────────────────────────────────────────┘
                              ▲
                          GitHub
```

## Failure → layer mapping

| Failure | First layer that absorbs it | Audit row |
|---|---|---|
| Kernel OOM kill of `actions.runner` process | Layer 1 — systemd restarts runner in 15s | `fail-2026-06-30-runner-oom-killed` |
| Stale systemd ExecStart pointing at deleted binary | Layer 2 — `patch_service_unit_execstart` | `fail-2026-06-30-stale-execstart` |
| glibc mismatch (Ubuntu-built gnu binary on AlmaLinux 8) | Workflow — musl default + `binary_version` filename parser | `fail-2026-06-30-stale-execstart` |
| `binary_version --version` segfault hangs script | Layer 2 — filename-first + `timeout 3` | `fail-2026-06-30-binary-version-zombie` |
| Script hangs in curl loop | Layer 2 — watchdog RSS + wall clock | `fail-2026-06-30-deploy-oom-killed` |
| Script OOM-killed by runner kernel | Layer 2 — watchdog exits with audit row | `fail-2026-06-30-deploy-oom-killed` |
| /v1/health times out | Layer 2 — `wait_for_health 60` + auto-rollback | `fail-2026-06-30-health-attempts-30` |
| /v1/health hangs intermittently | (daemon bug — operator recipe in live-release-automation.md) | `fail-2026-06-30-health-intermittent` |
| Deploy workflow reports failure | Layer 3 — auto-retry re-dispatches once | verified run `28435325539` |
| Audit row missing for a failure | Layer 4 — `auto-heal-audit.py` synthesizes | `add-2026-06-30-*` rows |
| Backup list grows without bound | `safe-disk-cleanup.sh` keeps most recent 5 | `add-2026-06-30-bounded-backups` |

## Adding a new self-heal branch

Required minimum:

1. **Guard code** in the appropriate layer (1, 2, 3, or 4).
2. **Static test assertion** in `tests/release_deploy_automation_static_test.sh` so the guard cannot regress.
3. **Audit row** in `release-proof/audit/audit.jsonl`:
   ```json
   {"id":"add-YYYY-MM-DD-<short>","ts":"...","event":"addition","subsystem":"...","scope":"...","category":"...","fix":"...","guard":"...","test":"..."}
   ```
4. **Category entry** in `release-proof/audit/categories.md` if the category is new.
5. **Cross-link** in `docs/failures-playbook.md` "Self-healing hooks (live)" section.

A new self-heal branch without a static test is not accepted; it will regress silently.

## Operator runbook (when self-heal is not enough)

If the chain fails (rare but possible: e.g. GitHub outage, runner uninstalled, etc.):

1. **Verify runner**: `systemctl is-active actions.runner.*.service`
2. **Verify daemon**: `curl -fsS --max-time 5 http://127.0.0.1:8787/v1/health`
3. **Tail audit log**: `tail -20 /var/log/focusa/deploy-audit.jsonl`
4. **Re-run deploy**: `gh workflow run 'Deploy Live Daemon' --ref main -f release_tag=v0.9.42-dev -f asset_suffix=x86_64-unknown-linux-musl`
5. **Manual rollback**: copy a backup to `/usr/local/bin/focusa-daemon` and `systemctl restart focusa-daemon.service`.

The audit trail captures every step regardless of outcome. No silent fixes.