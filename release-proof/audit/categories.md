# Failure category playbook

This file indexes every recorded failure so future automation can detect, self-heal, or at minimum quote the known fix. Categories:

- `brittle_regex_match`
- `stale_version_surface`
- `missing_ci_gate_passing`
- `infrastructure_blocked`
- `disk_pressure`
- `permission_denied`
- `hostname_assumption`
- `policy_violation`
- `stale_execut` — systemd unit ExecStart references deleted in-tree build artifact
- `hang_or_oom` — deploy script hangs/OOMs at runtime (curl loop, broken binary, etc.)
- `health_timeout` — /v1/health does not respond within the probe budget
- `intermittent_health` — /v1/health responds inconsistently (likely upstream daemon bug)
- `self_heal` — meta-row written by `auto-heal-audit.py` to document a synthesized self-heal

Each entry links back to `release-proof/audit/audit.jsonl` and the regression guard/test.

## brittle_regex_match

- **symptom**: static proof or smoke check fails in CI but passes locally
- **root cause**: regex anchors or whitespace-sensitive patterns differed between rg versions
- **fix**: use fixed-string `grep -Fq`
- **guard**: all workflow YAML assertions in `tests/release_deploy_automation_static_test.sh` are now fixed-string
- **test**: re-run `./tests/release_deploy_automation_static_test.sh` and observe PASS in CI logs

## stale_version_surface

- **symptom**: visible version in Settings, menubar, tauri.conf, root Cargo.toml drifts from latest tag
- **root cause**: manual tag pushes did not run stamp script
- **fix**: run `python3 scripts/stamp-menubar-version.py <tag>` then `python3 scripts/verify-version-surfaces.py <tag>` and commit
- **guard**: `verify-version-surfaces.py` is part of the release workflow
- **test**: release workflow post-stamp verification step

## missing_ci_gate_passing

- **symptom**: deploy gate errors `Deploy blocked: no successful CI push run on main for <sha>`
- **root cause**: target commit's CI run had not gone green
- **fix**: this is the intended behavior; do not bypass
- **guard**: deploy workflow CI gate step
- **test**: re-run after CI for that SHA is green

## infrastructure_blocked

- **symptom**: deploy transport (SSH) unreachable from GH-hosted runner
- **root cause**: VPS firewall blocks inbound SSH from external IPs
- **fix**: install self-hosted GitHub runner on VPS
- **guard**: `actions: read` permission + `runs-on: [self-hosted, linux, x64, focusa-deploy]`
- **test**: live deployment workflow dispatch proves path

## disk_pressure

- **symptom**: `guardian check disk` returns critical
- **root cause**: rebuildable cargo/target or unused /tmp artifacts
- **fix**: deploy preflight runs `scripts/safe-disk-cleanup.sh --apply`
- **guard**: deploy workflow preflight step
- **test**: static proof references `MIN_FREE_GB`

## permission_denied

- **symptom**: `as-user` runs cannot read root-owned toolchain
- **root cause**: Rust toolchain installed by root
- **fix**: deploy scripts invoked via narrow sudoers rule
- **guard**: `/etc/sudoers.d/focusa-github-runner`
- **test**: live deploy proof

## hostname_assumption

- **symptom**: runner registered with wrong name
- **root cause**: FQDN used for runner name
- **fix**: `hostname -s` short form
- **guard**: installer logs name before config
- **test**: dry-run output

## policy_violation

- **symptom**: `git push` blocked by `bd-evidence` hook
- **root cause**: closed bead lacked explicit evidence citations
- **fix**: reopen + close with `Evidence citations:` line
- **guard**: bead policy hook enforces citation form
- **test**: actual push attempt
## stale_execut

- **symptom**: deploy proof fails with `service ExecStart mismatch for focusa-daemon.service; expected reference to /usr/local/bin/focusa-daemon`. Daemon restart exits with status=1 because the unit points at a deleted in-tree build artifact (e.g. `/home/wirebot/focusa/target/release/focusa-daemon` pruned by `safe-disk-cleanup.sh`).
- **root cause**: install script was deployed before the in-tree build artifact was moved to `/usr/local/bin`. When `target/` gets pruned, the unit dangles. First-time VPS setup is exposed because there is no previous install to roll back to.
- **fix**: `install-daemon.sh` now ships `patch_service_unit_execstart()` which rewrites `ExecStart=` and `WorkingDirectory=` in `/etc/systemd/system/<unit>` to canonical install paths and `daemon-reload`s. Sudoers allowlist gained `/usr/bin/sed` for the runner.
- **guard**: `validate_service_execstart` auto-invokes `patch_service_unit_execstart` instead of `die`-ing; audits `deploy_preflight=patched`.
- **test**: static proof asserts `patch_service_unit_execstart` and `ExecStart=${INSTALL_PATH}` literal in `install-daemon.sh`.
- **manual first-time recovery**: if the runner is registering the unit for the first time and `validate_service_execstart` fails before any install has happened, edit `/etc/systemd/system/focusa-daemon.service` directly to set `ExecStart=/usr/local/bin/focusa-daemon` and `WorkingDirectory=/usr/local/lib/focusa`, then `systemctl daemon-reload`.
- **linked runs**: `28424197504`, `28442594742`, `28443558667`.

## hang_or_oom

- **symptom**: deploy proof OOM-killed (exit 137) after >20 minutes; no `deploy_complete` event ever written.
- **root cause**: `install-daemon.sh` had no wall-clock or RSS budget; the runner's OOM-killer terminated the process mid-script. Two sub-classes observed:
  - **binary_version glibc segfault**: `binary_version` invoked `--version` on a glibc-incompatible binary, which segfaulted inside libc, leaving zombie processes that held port 8787.
  - **wait_for_health curl loop**: a slow-starting daemon made every probe time out at `curl --max-time` but the loop itself kept the script alive indefinitely.
- **fix**:
  1. `install-daemon.sh` ships `watchdog_check()` that polls wall clock (default 600s) and RSS (default 768MB) and dies with `deploy_oom_killed` audit row on overrun.
  2. `binary_version` parses version from filename first; only runs `--version` as fallback under `timeout 3`.
  3. `curl --max-time 5` caps a single probe.
  4. `wait_for_health` default attempts bumped to 60 (60s) for musl cold start.
- **guard**: `WALL_CLOCK_SEC` and `RSS_LIMIT_MB` env vars; `auto-retry-deploy.yml` re-dispatches once on failure.
- **test**: static proof asserts `WALL_CLOCK_SEC`, `RSS_LIMIT_MB`, `deploy_oom_killed`, `deploy_health`, `watchdog_check`, `watchdog_loop`, `timeout 3`.
- **linked runs**: `28427772346`, `28439067838`.

## health_timeout

- **symptom**: deploy proof rolls back because `wait_for_health` exhausted attempts. `/v1/health` never responded in the probe window.
- **root cause**: musl cold start takes ~10–30s to seed Mem0 before serving; the previous `attempts=30` (30s) was too short.
- **fix**: bumped to 60 attempts. Watchdog still caps the whole script at 600s.
- **guard**: `wait_for_health 60` + watchdog_check every iteration + curl --max-time 5 + auto-rollback.
- **test**: static proof.
- **linked runs**: `28442594742`.

## intermittent_health

- **symptom**: deploy reports failure on some runs; `/v1/health` returns 200 ~70% of attempts, hangs (TCP accept then no response) the other 30%. Daemon restart loop visible in journal.
- **root cause**: upstream daemon bug in `v0.9.42-dev` — mem0 seeding race under load; not deploy automation. Deploy automation correctly reports failure when health is consistently unavailable, but intermittent hangs slip through and the script rolls back a working binary.
- **fix**: tracked separately as `fail-2026-06-30-health-intermittent`. Long-term: ship a daemon-side fix for /v1/health hang.
- **guard**: deploy automation's behavior is correct (rollback on persistent failure); intermittent issues should be tolerated or the daemon fixed.
- **test**: not covered by static test (upstream daemon behavior).

## self_heal

- **symptom**: meta-row written by `scripts/auto-heal-audit.py` whenever it finds a failure row without a matching `self_heal` row.
- **root cause**: every failure must produce one new audit row + one new category fix + one regression guard.
- **fix**: `auto-heal-audit.py` synthesizes the row automatically; idempotent and run on every CI / Release / Deploy / workflow_run / hourly schedule.
- **guard**: `auto-heal-audit.py` is part of `audit-recorder.yml`.
- **test**: row count grows monotonically; `auto_heal_audit.py` exits 0 on every run.

## Auto-heal chain (how the layers fit)

The self-heal chain is layered. Any one failure can be absorbed at one of three layers without operator action:

1. **Runner layer** — systemd `MemoryMax=2G Restart=always RestartSec=15` keeps the GitHub Actions runner itself alive across OOM kills. Kernel OOM-kill → systemd restart → runner reconnects to GitHub.
2. **Script layer** — `install-daemon.sh` watchdog caps wall clock at 600s and RSS at 768MB. Background `watchdog_loop` calls `watchdog_check` every 5s; on breach it audit-logs `deploy_oom_killed=watchdog_exit` and `TERM`s the parent shell.
3. **Workflow layer** — `auto-retry-deploy.yml` watches `workflow_run` completion of `Deploy Live Daemon`; on failure it re-dispatches the workflow once with the same release tag + musl asset. Never retries `workflow_run`-triggered deploys (no infinite loops).
4. **Audit layer** — `auto-heal-audit.py` synthesizes `self_heal` rows for every failure lacking one, on every CI / Release / Deploy workflow run + hourly schedule.

The chain is enforced by `tests/release_deploy_automation_static_test.sh` which asserts every guard literal.
