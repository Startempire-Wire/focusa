# `install-daemon.sh` self-heal contract

This document is the canonical API spec for the self-heal functions in `scripts/install-daemon.sh`. The static test `tests/release_deploy_automation_static_test.sh` pins every guard literal in this document, so any change must be reflected here.

## Entry point

```
bash scripts/install-daemon.sh \
  --binary <path-to-binary> \
  --service-name focusa-daemon \
  --health-url http://127.0.0.1:8787/v1/health \
  --expected-version <x.y.z[-suffix]>
```

Optional flags: `--no-restart`, `--no-verify`, `--dry-run`.

Required environment:

- `FOCUSA_DEPLOY_LOCK_FILE=/tmp/focusa-daemon-deploy.lock` (default)
- `FOCUSA_STATE_DIR=/usr/local/lib/focusa` (default)
- `FOCUSA_BACKUP_DIR=/usr/local/lib/focusa/backups` (default)
- `FOCUSA_DEPLOY_AUDIT_LOG=/var/log/focusa/deploy-audit.jsonl` (default)
- `FOCUSA_DEPLOY_WALL_CLOCK_SEC=600` (default; RSS-budget guard)
- `FOCUSA_DEPLOY_RSS_LIMIT_MB=768` (default; memory-budget guard)
- `FOCUSA_CALLGRAPH_VALIDATOR_URL` (optional override; defaults to the canonical URL derived from `HEALTH_URL`)
- GitHub context: `FOCUSA_GITHUB_RUN_ID`, `FOCUSA_GITHUB_SHA`, `FOCUSA_GITHUB_TAG`, `FOCUSA_GITHUB_WORKFLOW`.

## Self-heal functions

### `binary_version(path)`

Returns the version string for a binary.

Contract:

1. Parse the filename first. For canonical asset names like `focusa-daemon-v0.9.42-dev-x86_64-unknown-linux-musl`, return `0.9.42-dev`. (The leading `v` is stripped.)
2. If filename parsing fails, run `timeout 3 <path> --version` and parse `[0-9]+\.[0-9]+\.[0-9]+[-+.]?[0-9A-Za-z._-]*` from the first line.
3. If both fail, return empty string and `return 0`.

Never wedges for more than 3 seconds. Never segfaults on glibc-broken binaries (because it never executes them when filename parsing succeeds).

### `validate_service_execstart()`

Validates the systemd unit's `ExecStart` line.

Contract:

1. If the unit file does not exist, return success.
2. Read `ExecStart` via `sudo -n systemctl show -p ExecStart --value <unit>`.
3. If `ExecStart` does not contain the canonical install path, call `patch_service_unit_execstart()` instead of `die`-ing.
4. After a successful patch, audit `deploy_preflight=patched` and `sudo -n systemctl daemon-reload`.

### `patch_service_unit_execstart()`

Patches a stale systemd unit.

Contract:

1. Wrapped in `set +e` so a single failure cannot abort the deploy.
2. For `/etc/systemd/system/<unit>`:
   - Replace any existing `^[[:space:]]*ExecStart=.*` with `ExecStart=${INSTALL_PATH}`.
   - Replace any existing `^[[:space:]]*WorkingDirectory=.*` with `WorkingDirectory=${STATE_DIR}`.
3. Use `sudo -n sed` first, fall back to user `sed` if sudo not allowed (no-op for the runner).
4. Always `return 0` on success; returns 1 if the unit file is missing.

### `watchdog_check()`

Wall clock and RSS budget enforcer.

Contract:

1. Wrapped in `set +e` so a failing check returns instead of `die`-ing.
2. If `elapsed > WALL_CLOCK_SEC`, audit `deploy_oom_killed=timeout` and `die`.
3. If our own RSS exceeds `RSS_LIMIT_MB`, audit `deploy_oom_killed=rss_exceeded` and `die`.
4. Returns 0 normally; non-zero only when the caller (in `watchdog_loop`) should escalate to TERM the parent.

### `watchdog_loop()`

Background watchdog runner.

Contract:

1. Runs in a subshell `> /dev/null 2>&1 &`.
2. Polls every 5s via `sleep 5` then `watchdog_check`.
3. On `watchdog_check` non-zero: audit `deploy_oom_killed=watchdog_exit` then `kill -TERM $$` then `exit 1` so the parent shell dies fast.
4. Trap on EXIT ensures the background process is cleaned up.

### `wait_for_health(attempts, expected_version)`

Polls the health endpoint.

Contract:

1. Each iteration performs a bounded TCP and HTTP health probe through Python's standard library.
2. If the response body is JSON and `expected_version` is non-empty, it retries until the reported version matches.
3. An empty or unavailable health response always triggers rollback; an active service is not sufficient acceptance.
4. After exhausting `attempts` (default 60), audit `deploy_health=timeout` and return 1.

### Installed capability verification

After exact health/version acceptance, `scripts/verify-callgraph-validator.py` posts a deterministic, side-effect-free golden graph to `POST /v1/callgraphs/validate`. Deployment succeeds only when the response is HTTP 200 with `canonical=true`, `valid=true`, `status=valid`, and an empty issue list. Missing routes, transport failures, malformed envelopes, and structural rejection trigger rollback. `--no-verify` remains the explicit operator bypass for the complete verification phase.

## Audit events emitted

Every self-heal function emits an audit row to `FOCUSA_DEPLOY_AUDIT_LOG` (default `/var/log/focusa/deploy-audit.jsonl`) via `scripts/audit-schema.py validate`-compatible shape. Required keys:

- `ts` (ISO 8601 UTC)
- `event` (`failure` | `addition` | `self_heal`)
- `id` (only for `failure` and `addition`)
- `category` (one of `VALID_CATEGORIES` in `scripts/audit-schema.py`)
- `subsystem` (one of `VALID_SUBSYSTEMS`)
- `scope`, `symptom`, `root_cause`, `fix`, `guard`, `test`, `linked_run`

If you add a new audit-emitting branch, you must:

1. Add the new `category` to `VALID_CATEGORIES` in `scripts/audit-schema.py`.
2. Add a row to `release-proof/audit/categories.md`.
3. Add a static test assertion in `tests/release_deploy_automation_static_test.sh`.

## Failure codes

Self-heal branches emit these specific audit outcomes:

| Outcome | Trigger |
|---|---|
| `deploy_preflight=patched` | `patch_service_unit_execstart` succeeded |
| `deploy_oom_killed=timeout` | wall clock exceeded `WALL_CLOCK_SEC` |
| `deploy_oom_killed=rss_exceeded` | RSS exceeded `RSS_LIMIT_MB` |
| `deploy_oom_killed=watchdog_exit` | `watchdog_loop` killed the parent |
| `deploy_health=timeout` | `wait_for_health` exhausted attempts |
| `deploy_capability=verified` | installed CallGraph validator returned the canonical valid envelope |
| `deploy_rollback=failed` | rollback restart failed; daemon unhealthy |
| `deploy_complete=success` | full deploy succeeded |
| `deploy_install=completed` | installed without restart (--no-restart) |

## Adding new self-heal branches

To add a new branch:

1. Implement the function in `install-daemon.sh`.
2. Document it in this file.
3. Add a literal `assert_grep` in `tests/release_deploy_automation_static_test.sh`.
4. Add a row to `release-proof/audit/categories.md`.
5. Update `CHANGELOG.md` by running `python3 scripts/changelog-gen.py`.

A self-heal branch without all five anchors is not merged.