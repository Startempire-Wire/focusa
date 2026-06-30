# Live release automation

## Goal

Always run the newest tagged Focusa daemon in production without duplicate daemons, while keeping rollback fast and explicit.

## Release model

Focusa now uses a tag-driven release/deploy model:

1. Core edits land on `main`.
2. CI proves build/tests/clippy/static deploy automation.
3. `scripts/create-dev-release-tag.sh --push` creates the next tag, stamps version surfaces, and pushes `main` + tag.
4. `Release` workflow builds artifacts from the tag and verifies stamped version surfaces.
5. `Deploy Live Daemon` workflow downloads the daemon asset from that GitHub release, uploads it to the VPS, installs it, restarts the service, verifies `/v1/health`, and rolls back automatically on failure.

## Canonical commands

Create/push a new release tag and wait for CI + release + live deploy:

```bash
scripts/create-dev-release-tag.sh --push
```

Redeploy or roll back to an older tag from GitHub Actions:

- open **Actions → Deploy Live Daemon → Run workflow**
- set `release_tag` to the target tag, e.g. `v0.9.40-dev`

Rollback is just a redeploy of an earlier release tag.

## Required GitHub configuration

### Secrets

- `FOCUSA_DEPLOY_HOST`
- `FOCUSA_DEPLOY_USER`
- `FOCUSA_DEPLOY_SSH_KEY`

### Repository variables

Optional, with defaults shown:

- `FOCUSA_DEPLOY_PORT` = `22`
- `FOCUSA_DEPLOY_INSTALL_ROOT` = `/usr/local`
- `FOCUSA_DEPLOY_SERVICE_NAME` = `focusa-daemon`
- `FOCUSA_DEPLOY_HEALTH_URL` = `http://127.0.0.1:8787/v1/health`
- `FOCUSA_DEPLOY_ASSET_SUFFIX` = `x86_64-unknown-linux-gnu`
- `FOCUSA_DEPLOY_REQUIRE_SERVICE` = `1`
- `FOCUSA_DEPLOY_USE_SUDO` = `1`

## VPS install/restart safeguards

`scripts/install-daemon.sh` now enforces:

- deploy lock via `flock` so two deploys cannot overlap
- backup of the current binary before replacement
- service stop + stray process cleanup before install
- restart through systemd
- `/v1/health` verification after restart
- version check against the expected release tag version
- automatic rollback to the previous binary if start/health/version checks fail
- duplicate-daemon guard using `pgrep -x focusa-daemon`

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

- Do not tag manually if you want version surfaces committed cleanly; use `scripts/create-dev-release-tag.sh`.
- Do not run ad-hoc `focusa-daemon &` alongside systemd.
- Use GitHub Actions as the canonical deploy path so build/release/live state stay aligned.
