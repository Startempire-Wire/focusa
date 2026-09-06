# Failures playbook

This document indexes **every CI, runtime, release, or deploy failure** that has
been observed during the Focusa release/deploy automation work.

Source of truth ledger:

- `release-proof/audit/audit.jsonl` — append-only machine-readable events
- `release-proof/audit/categories.md` — failure category playbook with fix
  and guard links

How to use this playbook:

1. Capture the failure in `release-proof/audit/audit.jsonl` with:

   ```json
   {"id":"fail-YYYY-MM-DD-<short>","ts":"...","event":"failure","subsystem":"...","scope":"...","category":"...","symptom":"...","root_cause":"...","fix":"...","guard":"...","test":"...","linked_run":"..."}
   ```

2. Map the failure to a category in `categories.md`. If the category is new,
   add it with a fix/guard/test triplet so the next agent can resolve it
   without re-debugging.

3. Every `addition` event (new code in the Rust installer/system-service
   transaction, `scripts/safe-disk-cleanup.sh`, runner provisioning, or the
   deployment workflow) is paired with the categories its guards mitigate.
   `scripts/install-daemon.sh` is a compatibility adapter and must not gain
   lifecycle logic.

## Lessons (do-not-repeat)

- **Static / regex brittleness**: never use `rg -q '^pattern$'` against
  workflow files; use `grep -Fq 'literal'`. Replaced all such checks in
  `tests/release_deploy_automation_static_test.sh`.
- **Version drift**: every tag push must run
  `stamp-menubar-version.py` then `verify-version-surfaces.py`. The release
  workflow enforces this and the static proof references it.
- **GH-hosted transport risk**: do not rely on inbound SSH from GH-hosted
  runners; deploys must run on a self-hosted runner registered to the
  VPS with label `focusa-deploy`.
- **Privileged scripts**: only the Rust-delegating deploy adapter, governed
  cleanup, and exact public-file installation get NOPASSWD routes; direct
  `kill`, `systemctl`, `sed`, `mv`, `rm`, and `ln` grants are forbidden.
- **Disk pressure**: the deploy preflight runs the safe cleanup with a
  threshold; failure causes the deploy to abort instead of silently running
  the daemon on a starved root filesystem.

## Restart loses recently acknowledged API state

If `tests/restart_recovery_test.sh` sees a frame before SIGTERM but reports
`frame_unavailable` after restart, inspect the final shutdown checkpoint before
changing database compatibility or deleting state. A daemon-local snapshot can
lag direct API writes. The shutdown checkpoint must hold the canonical write
lock, adopt the external mutation epoch, and refuse persistence if adoption
fails. Regression: `shutdown_checkpoint_preserves_external_frame_state`; consumer
proof: the isolated restart test above. Source/unit success is not installed
recovery acceptance.

## CI dependency-info artifacts disappear

When Cargo reports a missing dependency-info file during linting, preserve the
exact run/job/SHA and error path before blaming source changes or deleting caches.
Verify runner workspace separation and process ownership; later checkout cleanup
is not evidence that files disappeared concurrently. Source CI uses unfiltered
workspace tests followed by one workspace-wide **all-targets** Clippy pass; focused
Pi/update cases are already in that suite. Keep warnings fatal and the process-health
wrapper. An exact-head rerun must prove recovery; recurring failures require cache/
artifact-layout investigation, never blind deletion of shared targets (issue #573).

## Self-healing hooks (live)

All hooks below are wired into CI, Release, Deploy, and the audit recorder workflow. They run automatically; operators do not invoke them by hand.

- `crates/focusa-cli/src/commands/system_service.rs` — nonblocking deployment lock, operator-halt gate, exact systemd process ownership, atomic unit staging, bounded health/CallGraph acceptance, and binary+unit rollback.
- `scripts/install-daemon.sh` — validates legacy arguments and delegates exactly once to the signed Rust full-release installer; it never patches units or signals processes.
- `.github/workflows/auto-retry-deploy.yml` — quarantined; it records the safe recovery route but has no automatic redispatch authority.
- `scripts/install-self-hosted-runner.sh` systemd drop-in — `MemoryMax=2G Restart=always RestartSec=15` so the runner self-recovers from kernel OOM kills.
- `scripts/auto-heal-audit.py` (via `.github/workflows/audit-recorder.yml`) — synthesizes `self_heal` rows for every failure lacking one; idempotent; runs on `workflow_run` + hourly `schedule`.
- `tests/release_deploy_automation_static_test.sh` — asserts every self-heal branch's literal is present in source; prevents regression.

Deprecated hook list (was planned, now superseded):

- ~~workflow step: parse recent `audit.jsonl`, fail closed if a guard failure is older than expected~~ — superseded by `auto-heal-audit.py` which synthesizes rows on every run.
- ~~agent loop: on `category=missing_ci_gate_passing`, suggest `gh run rerun`~~ — CI gate is now enforced by the deploy workflow itself (deploy blocks when CI is not green).
- ~~agent loop: on `category=brittle_regex_match`, refuse to add new `rg -q` checks~~ — static test now uses fixed-string `grep -Fq` via `assert_grep` helper; CI fails on any regression.

## Redaction

- do not write credential values into `audit.jsonl`
- do not include bearer tokens, SSH key contents, or release-signed URLs
- redact home paths, SSH config, or hostnames to opaque forms

## Operating principle

> Every failure must produce one new audit row, one new category fix,
> and one regression guard. No silent fixes.

## Release controller test missing its imported helper

If the controller-staged OTA contract check raises
`ModuleNotFoundError: install_target_contract`, stage the canonical helper beside
the test from the same `CONTROLLER_SHA`. Do not use the candidate checkout's
helper or duplicate its implementation: controller and candidate revisions can
differ. `tests/spec143_ota_installability_release_gate_test.py` guards the staging
contract. Verify execution from a temporary directory with `FOCUSA_SPEC143_ROOT`
and `FOCUSA_RELEASE_WORKFLOW_PATH` bound explicitly.

Observed in Release run 34055757279 for v0.9.188. The immutable candidate tag is
not a published/installed release. Open release-gating issues and missing canary
inputs remain separate acceptance requirements; this import repair waives neither.
