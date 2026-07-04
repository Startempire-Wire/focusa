# Live release automation

## Goal

Always run the newest tagged Focusa daemon in production without duplicate daemons, while keeping rollback fast and explicit.

The live deploy hook now assumes a **self-hosted GitHub Actions runner on the VPS** with labels:

- `self-hosted`
- `linux`
- `x64`
- `focusa-deploy`
- `production`

## Mandatory single path

All build/deploy work uses the **full live release pipeline only**. Do not build artifacts locally, do not deploy from `target/release`, and do not run only `Deploy Live Daemon` as a shortcut. See [`docs/canonical-live-release-pipeline.md`](canonical-live-release-pipeline.md).

## Release model

Focusa now uses a tag-driven release/deploy model:

1. Core edits land on `main`.
2. CI proves build/tests/clippy/static deploy automation.
3. `scripts/create-dev-release-tag.sh --base 0.9 --push` creates the next tag, stamps version surfaces, and pushes `main` + tag.
4. `Release` workflow builds artifacts from the tag and verifies stamped version surfaces.
5. `Deploy Live Daemon` workflow runs only on a self-hosted VPS runner after a successful GitHub `CI` run for the exact target commit, performs a safe disk-cleanup preflight, installs the daemon, restarts the service, verifies `/v1/health`, and rolls back automatically on failure.

## Canonical commands

Create/push a new release tag and wait for CI + release + live deploy:

```bash
scripts/create-dev-release-tag.sh --base 0.9 --push
```

Do not manually run only `Deploy Live Daemon` for normal releases. Redeploy/retry is owned by Auto Heal + Watchdog. If rollback is required, file/fix the underlying pipeline state and drive it through the same audited release/deploy automation; do not install local binaries by hand.

Rollback is a pipeline-managed redeploy of an earlier release tag.

## Required GitHub configuration

### Self-hosted runner

Install with:

```bash
sudo scripts/install-self-hosted-runner.sh
```

This creates the `github-runner` user, installs the runner under `/opt/actions-runner-focusa`, configures the `focusa-deploy,production` labels, and writes a narrow sudoers rule so the runner can execute only the privileged deploy/cleanup scripts.

### Repository variables

Optional, with defaults shown:

- `FOCUSA_DEPLOY_INSTALL_ROOT` = `/usr/local`
- `FOCUSA_DEPLOY_SERVICE_NAME` = `focusa-daemon`
- `FOCUSA_DEPLOY_HEALTH_URL` = `http://127.0.0.1:8787/v1/health`
- `FOCUSA_DEPLOY_ASSET_SUFFIX` = `x86_64-unknown-linux-musl` (AlmaLinux 8 ships glibc 2.28; the Ubuntu-built gnu binary requires glibc >= 2.39, so the musl static-pie artifact is canonical)
- `FOCUSA_DEPLOY_REQUIRE_SERVICE` = `1`
- `FOCUSA_DEPLOY_USE_SUDO` = `1`
- `FOCUSA_DEPLOY_AUDIT_LOG` = `/var/log/focusa/deploy-audit.jsonl`
- `FOCUSA_DEPLOY_MIN_FREE_GB` = `15`
- `FOCUSA_DEPLOY_MAX_USAGE_PCT` = `92`

## VPS install/restart safeguards

`scripts/install-daemon.sh` now enforces:

- deploy lock via `flock` so two deploys cannot overlap
- backup of the current binary before replacement
- service `ExecStart` validation so systemd points at the canonical install path
- service stop + stray process cleanup before install
- restart through systemd
- `/v1/health` verification after restart
- version check against the expected release tag version
- checksum capture for old/new binaries in the audit trail
- automatic rollback to the previous binary if start/health/version checks fail
- duplicate-daemon guard using `pgrep -x focusa-daemon`
- append-only deploy audit log entries for deploy start, preflight, completion, and rollback
+
+`scripts/safe-disk-cleanup.sh` runs before deploy to reclaim only scoped, rebuildable Focusa cruft such as:
+
+- repo `target/`
+- repo `.tmp/`
+- old `/tmp/focusa-release-*` and `/tmp/focusa-deploy-*`
+- stale deploy backups past retention
+
+It fails closed if free space or disk usage remains below configured thresholds.

## Recommended systemd unit

Use `focusa-daemon.service` as the canonical service name.

The deploy workflow assumes one systemd-managed daemon instance, not ad-hoc background launches.

## Version truth

The release tag is the source of truth.

`Release` workflow stamps and then verifies these surfaces against the tag:

- root `Cargo.toml`
- root `Cargo.lock` Focusa package entries
- `apps/menubar/package.json`
- `apps/menubar/src-tauri/Cargo.toml`
- `apps/menubar/src-tauri/Cargo.lock`
- `apps/menubar/src-tauri/tauri.conf.json`
- visible Settings version

Verifier:

```bash
python3 scripts/verify-version-surfaces.py v0.9.41-dev
```

## Fast fallback paths

### Automatic fallback

If the newly deployed daemon:

- fails to start
- returns unhealthy `/v1/health`
- reports the wrong version
- leaves duplicate daemon processes running

then `scripts/install-daemon.sh` restores the backed-up binary and restarts the prior version.

### Manual fallback

If a release is functionally bad but technically healthy:

1. go to **Deploy Live Daemon** workflow
2. choose previous good tag
3. run deploy again

This gives a quick tag-based rollback without editing the VPS manually.

## Operator guidance

- Do not tag manually if you want version surfaces committed cleanly; use `scripts/create-dev-release-tag.sh`. (Manual `git tag -d && git push :refs/tags/<t> && git tag -a` is acceptable only for re-pointing an existing tag during fast iterations; the script is preferred.)
- Do not run ad-hoc `focusa-daemon &` alongside systemd.
- Use GitHub Actions as the canonical deploy path so build/release/live state stay aligned.

## Self-healing safety net

The deploy pipeline self-heals at three layers; operators do not need to intervene for any of these.

### Runner layer (kernel OOM protection)

`scripts/install-self-hosted-runner.sh` writes a systemd drop-in for the runner unit:

- `MemoryMax=2G` (overridable via `FOCUSA_RUNNER_MEMORY_MAX`)
- `Restart=always`
- `RestartSec=15`

If the runner is kernel OOM-killed, systemd restarts it within 15s and the runner reconnects to GitHub. Without this, a transient memory spike would silently kill the runner and the next deploy would fail with no audit trail.

### Script layer (wall clock + RSS + binary version)

`scripts/install-daemon.sh` ships a `watchdog_check()` that runs in a background loop (`watchdog_loop`):

- wall clock budget: `WALL_CLOCK_SEC` (default 600s)
- RSS budget: `RSS_LIMIT_MB` (default 768MB)

On breach it audit-logs `deploy_oom_killed` and `TERM`s the parent shell. The script also has:

- `binary_version()` that parses version from filename first (e.g. `focusa-daemon-v0.9.42-dev-x86_64-unknown-linux-musl` → `0.9.42-dev`) and only falls back to `timeout 3 ... --version`
- `curl --max-time 5` on health probes
- `patch_service_unit_execstart()` that auto-rewrites a stale systemd `ExecStart` and reloads systemd, so a unit pointing at a deleted in-tree build artifact self-heals without operator action

### Workflow layer (auto-retry)

`.github/workflows/auto-retry-deploy.yml` listens on `workflow_run` completion of `Deploy Live Daemon`. If the upstream failed AND was triggered by `release` or `workflow_dispatch`, it re-dispatches the workflow once with the same tag + musl asset. Never retries `workflow_run`-triggered deploys (no infinite loops).

### Audit layer

`scripts/auto-heal-audit.py` is idempotent and is invoked on every CI / Release / Deploy workflow run + on `workflow_run` + on an hourly schedule. It scans `release-proof/audit/audit.jsonl` and synthesizes a `self_heal` row for every failure row that lacks one.

## Intermittent health hang recovery (operator recipe)

If `/v1/health` is responding 200 sometimes and hanging other times (TCP accept then no response), the upstream daemon is likely in a mem0-seeding race. Symptoms in the journal:

```
focusa_core::runtime::daemon: Focusa daemon starting (version ...)
focusa_core::server: Listening on 127.0.0.1:8787
focusa_core::runtime::daemon: Startup: Mem0 memories seeded count=5
```

then nothing for >30s, then a restart.

**Recovery:**

1. `systemctl restart focusa-daemon.service`
2. Wait 30s for mem0 seed.
3. `curl -fsS --max-time 5 http://127.0.0.1:8787/v1/health`
4. If still hung, `journalctl -u focusa-daemon.service -n 50 --no-pager` and file an upstream daemon bug; the deploy automation's behavior is correct (rollback on persistent failure).

## Audit trail

Every addition and every failure during release/CI/deploy must be captured in:

- `release-proof/audit/audit.jsonl` — append-only ledger
- `release-proof/audit/categories.md` — failure category playbook
- `docs/failures-playbook.md` — human-readable index

Rules:

- no silent fixes
- each failure pairs with a category fix and a regression guard
- commit evidence citations are required on closed beads to pass the
  `bd-evidence` push policy
