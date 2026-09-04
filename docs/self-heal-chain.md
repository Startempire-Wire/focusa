# Self-heal chain

This document describes how the Focusa deploy pipeline self-heals at every layer. Read this before debugging a deploy failure or adding a new self-heal branch.


## DRY failure classification

All release-path self-heal decisions share one classifier:

```text
GitHub failed logs
  └─ scripts/classify-ci-failure.py
       ├─ JSON for agents/audit/debugging
       └─ KEY=value env output via .github/scripts/classify-release-failure.sh
```

`Auto Heal Release Pipeline` and `Release Pipeline Watchdog` consume the same
`failure_class`, `retry_policy`, `source_refs`, and `remediation_template`.
Transient classes may rerun once. Deterministic classes such as
`rust_compile_api_drift`, `ci_clippy_failure`, and `ci_test_failure` stop the
rerun loop and require a patch plus a fresh GitHub CI run. Local Rust output is
advisory only; GitHub CI/Release/Deploy remains the canonical proof path.



## Beads ownership hygiene

Self-heal proof depends on normal `bd sync`, evidence policy, and pre-push gates.
A Beads daemon for this repo must run as the project owner (`wirebot` here), not
root; otherwise `.beads/issues.jsonl` can be rewritten as root and block
operator recovery. `tests/bd_sync_ownership_policy_test.sh` checks the JSONL
owners and any running `.beads/daemon.pid` process owner.




## Self-heal telemetry

`scripts/self-heal-telemetry.py` summarizes audit health for operators and future
automation: class counts, retry-policy counts, repeated failure classes,
open repair-needed deterministic failures, stale unhealed failures, and the latest
self-heal timestamp. Use it when deciding whether the release path is looping,
stuck, or healthy.


## Historical classifier backfill

`scripts/backfill-audit-classifier-fields.py` enriches historical failure rows
without rewriting them. `--dry-run` reports candidate `addition` rows; `--apply`
appends explicit `add-backfill-classifier-*` rows containing
`classifier_schema`, `failure_class`, `retry_policy`, `deterministic`,
`safe_to_rerun_unchanged`, `source_refs`, `remediation_template`, and
`classifier_signals`. `scripts/audit-failure-summary.py` overlays those rows by
`derived_from` so triage output shows enriched classes while the original
failure rows remain immutable.

```bash
python3 scripts/backfill-audit-classifier-fields.py --audit release-proof/audit/audit.jsonl --dry-run
python3 scripts/backfill-audit-classifier-fields.py --audit release-proof/audit/audit.jsonl --apply
python3 scripts/audit-schema.py validate release-proof/audit/audit.jsonl
python3 scripts/audit-failure-summary.py --class unknown_process_failure --limit 5
```

## Audit failure triage CLI

`scripts/audit-failure-summary.py` is the first-line read-only operator CLI for
recent release-path failures:

```bash
python3 scripts/audit-failure-summary.py --limit 10
python3 scripts/audit-failure-summary.py --class ci_clippy_failure --limit 5
python3 scripts/audit-failure-summary.py --class ci_clippy_failure --limit 5 --json
```

Human output shows retry policy, remediation, source refs, run id, and log URL;
JSON output preserves the selected failure rows for handoff or automation.

## Deploy self-heal proof drill

`scripts/deploy-self-heal-proof-drill.py` focuses on the live deploy path without
mutating production. It proves `deploy_health_failure` permits exactly one
bounded redeploy, a deterministic self-heal/process class stops for repair,
audit failure/self_heal rows are synthesized, summaries render remediation, and
(optional) the live `/v1/health` endpoint remains OK. The manual workflow
`.github/workflows/deploy-self-heal-proof-drill.yml` runs the same proof with
`health_url=skip` by default for GitHub-hosted safety.

## Failure injection drill

`scripts/self-heal-decision-drill.py` is a safe dry-run harness that feeds the
classifier fixtures through classification, audit recording, self-heal synthesis,
and audit summary rendering. It proves deterministic classes choose
`repair_required_no_rerun` and transient classes choose `rerun_once_allowed`
without dispatching production deploy or mutating `release-proof/audit/audit.jsonl`.
The manual workflow `.github/workflows/self-heal-failure-injection.yml` runs the
same drill via `workflow_dispatch` for GitHub-hosted proof.

## Classifier fixture suite

Classifier behavior is locked by `tests/self_heal_classifier_fixture_test.py` and
fixtures in `tests/fixtures/self-heal-classifier/`. Every core self-heal class
needs a `.log` sample plus `.expected.json` so changes to retry policy,
source refs, signals, or remediation are intentional. The release automation
static gate runs this suite before CI can pass.

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
│   .github/workflows/auto-retry-deploy.yml is quarantined.          │
│   Retry requires exact failure and rollback evidence; there is no  │
│   automatic redispatch authority.                                 │
└────────────────────────────────────────────────────────────────────┘
                              ▲
┌────────────────────────────────────────────────────────────────────┐
│ Layer 2 — Rust system-install transaction                         │
│   nonblocking deploy lock + operator-halt gate                    │
│   exact systemd MainPID/executable and one-process invariant      │
│   atomic full-release + unit promotion                            │
│   bounded health/CallGraph acceptance + complete rollback         │
│   install-daemon.sh only adapts legacy arguments                  │
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
| Stale systemd ExecStart pointing at deleted binary | Layer 2 — atomically render the canonical unit; restore the prior unit on failure | `fail-2026-06-30-stale-execstart` |
| glibc mismatch (Ubuntu-built gnu binary on AlmaLinux 8) | Workflow — signed musl release-set selection | `fail-2026-06-30-stale-execstart` |
| Candidate `--version` crashes or disagrees | Layer 2 — Rust process status/version gate before activation | `fail-2026-06-30-binary-version-zombie` |
| Unmanaged or duplicate daemon exists | Layer 2 — reject exact process inventory without signalling it | `fail-2026-06-30-deploy-oom-killed` |
| /v1/health times out | Layer 2 — bounded readiness check and full transaction rollback | `fail-2026-06-30-health-attempts-30` |
| /v1/health hangs intermittently | Layer 2 rolls back; investigate the daemon bug before retry | `fail-2026-06-30-health-intermittent` |
| Deploy workflow reports failure | Layer 3 records failure; retry stays quarantined pending exact diagnosis | verified run `28435325539` |
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
5. **Rollback**: use `focusa update rollback --dry-run=false --yes`; never hand-copy one binary or bypass the full release transaction.

The audit trail captures every step regardless of outcome. No silent fixes.