# Spec 125 — Focusa Over-the-Air Auto-Update and Dev Mode License

## Status

Draft — operator-directed after `v0.9.80-dev` release showed server daemon was current but local CLI/TUI were stale.

## Problem

A Focusa host can have multiple installed parts at different versions. The daemon may be latest while the CLI, TUI, Pi extension, installer, or menubar package is stale. Operators should not need to remember manual binary sync after every passing release.

## Goals

1. Keep Focusa installs on the latest approved release automatically when policy allows.
2. Provide an explicit auto-update option that can be turned on or off.
3. Add a first-party `dev_mode` license level for core Focusa developers.
4. Let `dev_mode` installs track the cutting edge by default.
5. Update every Focusa part from tagged releases that passed required CI/proof gates.
6. Preserve rollback, checksums, Context Authority, local data ownership, and license boundaries.

## Non-goals

- No automatic update from untagged commits by default.
- No silent overwrite of data directories, `.env`, license files, Workpoints, evidence, or project state.
- No retagging or mutation of published releases.
- No cloud-memory dependency for local updates.

## Current server inventory model

| Part | Current server location | Update cadence | Notes |
| --- | --- | --- | --- |
| Focusa daemon | `/usr/local/bin/focusa-daemon` | every accepted release | systemd service binary |
| Focusa CLI | `/usr/local/bin/focusa` | every accepted release | operator command surface |
| Focusa TUI | `/usr/local/bin/focusa-tui` | every accepted release | needs version/self-test surface |
| systemd unit | `/etc/systemd/system/focusa-daemon.service` | only on service contract change | not replaced by normal binary update |
| systemd drop-ins | `/etc/systemd/system/focusa-daemon.service.d/*.conf` | only on policy/runtime change | preserve local overrides |
| Focusa runtime home | `/usr/local/lib/focusa` | never wholesale replaced | state, rollback, update history |
| Focusa env | `/home/wirebot/focusa/.env` or configured env file | never auto-overwritten | secrets/local config boundary |
| License files | `/root/.config/focusa/license*.json` | refresh/validate, never overwrite with eval | entitlement boundary |
| Repo checkout | `/home/wirebot/focusa` | git-managed source | live build/proof host |
| Release assets | GitHub release `vX.Y.Z-dev` | source of tagged binaries | must pass CI/proof gates |
| Menubar app | release `.dmg` / `.app.tar.gz` | client update channel | Mac-side updater, not server daemon |
| Pi extension | `apps/pi-extension` / package install path | package/update channel | must match daemon tool contracts |
| Public installer | install.focusa.dev installer path | only on installer release | bootstrap surface |

## License levels

| Level | Intended user | Update behavior | Entitlements |
| --- | --- | --- | --- |
| `evaluation` | trial/local exploration | check + notify only | no unattended binary replacement |
| `community_source` | source users | repo/build guidance, no unattended release asset install | source build, manual update |
| `pro_local` | paid local operators | opt-in auto-update for CLI/TUI; daemon update requires safe preflight | `packaged_installer`, `ota_check`, `ota_apply_manual` |
| `team_self_hosted` | teams/self-hosted deployments | staged updates, maintenance windows, team notification | pro + `ota_scheduled`, `team_update_orchestration` |
| `enterprise` | regulated/self-hosted orgs | pinned channels, approval gates, audit export, optional airgap bundles | team + `ota_policy_admin`, `airgap_update_bundle` |
| `dev_mode` | Focusa core developers | default auto-update ON for local Focusa parts, cutting-edge dev channel allowed | all product features plus `ota_auto_update`, `official_release_bundle`, `packaged_installer`, `developer_channel` |

### `dev_mode` clarification

Current installer code treats registry `status=dev_mode` as a test fixture. Spec 125 changes the product model:

- `dev_mode` becomes a real first-party developer license level.
- Test fixtures must use a different internal status such as `fixture_dev_key` or `test_mode`.
- `dev_mode` is not a public paid tier.
- `dev_mode` is allowed to auto-update all local Focusa developer parts by default.

## Auto-update switch

Auto-update must be explicit, inspectable, and reversible.

Suggested policy file:

```json
{
  "schema": "focusa.update_policy.v1",
  "enabled": true,
  "channel": "dev",
  "mode": "automatic",
  "license_level": "dev_mode",
  "parts": {
    "cli": true,
    "daemon": true,
    "tui": true,
    "pi_extension": true,
    "menubar": false,
    "installer": false
  },
  "maintenance_window": "always",
  "require_ci_success": true,
  "require_release_success": true,
  "require_deploy_success_for_daemon_hosts": true,
  "require_checksums": true,
  "rollback": true,
  "notify_before_restart": false
}
```

Recommended server path:

- global policy: `/usr/local/lib/focusa/update-policy.json`
- update history: `/usr/local/lib/focusa/updates/history/*.json`
- staging: `/usr/local/lib/focusa/updates/staging/<tag>/`
- rollback: `/usr/local/lib/focusa/updates/rollback/<timestamp>/`

## CLI/API surface

```bash
focusa update status --json
focusa update check --channel dev --json
focusa update plan --tag latest --json
focusa update apply --tag latest --json
focusa update rollback --json
focusa update policy show --json
focusa update policy set --enabled true --channel dev --mode automatic
focusa update policy set --enabled false
```

Daemon/API routes:

- `GET /v1/update/status`
- `POST /v1/update/check`
- `POST /v1/update/plan`
- `POST /v1/update/apply`
- `POST /v1/update/rollback`
- `GET /v1/update/policy`
- `POST /v1/update/policy`

## Update algorithm

1. Resolve installed inventory: binary paths, hashes, versions, service PID, daemon health, policy, license level.
2. Resolve latest eligible release for the configured channel.
3. Verify release provenance:
   - tag exists,
   - CI succeeded,
   - Release workflow succeeded,
   - Deploy workflow succeeded for daemon hosts,
   - required assets exist,
   - checksums/signatures exist and verify.
4. Compare installed hash/version to release manifest.
5. Build an update plan by part.
6. Snapshot current binaries and service metadata.
7. Download to staging.
8. Verify staged checksums/signatures.
9. Install CLI and TUI first.
10. Install daemon last.
11. Restart daemon only when daemon binary changed and policy allows restart.
12. Verify health and versions.
13. Notify local clients and append update history.
14. Roll back automatically if health/proof fails.

## Release manifest

The release workflow should publish a machine-readable manifest:

```json
{
  "schema": "focusa.release_manifest.v1",
  "tag": "v0.9.80-dev",
  "commit": "8fa6452d...",
  "channel": "dev",
  "ci_run": 29084865466,
  "release_run": 29084866662,
  "deploy_run": 29085699876,
  "assets": {
    "focusa": {
      "platform": "x86_64-unknown-linux-gnu",
      "name": "focusa-v0.9.80-dev-x86_64-unknown-linux-gnu",
      "sha256": "..."
    },
    "focusa-daemon": {
      "platform": "x86_64-unknown-linux-gnu",
      "name": "focusa-daemon-v0.9.80-dev-x86_64-unknown-linux-gnu",
      "sha256": "..."
    },
    "focusa-tui": {
      "platform": "x86_64-unknown-linux-gnu",
      "name": "focusa-tui-v0.9.80-dev-x86_64-unknown-linux-gnu",
      "sha256": "..."
    }
  },
  "requires_license_features": ["packaged_installer"],
  "dev_mode_features": ["ota_auto_update", "official_release_bundle", "developer_channel"],
  "rollback_supported": true
}
```

## Part-specific rules

### CLI

- Replace `/usr/local/bin/focusa` from verified release asset.
- Verify `focusa --version` equals target tag.
- No daemon restart required.

### Daemon

- Replace `/usr/local/bin/focusa-daemon` from verified release asset.
- Snapshot old daemon binary before replacement.
- Restart `focusa-daemon` only after binary replacement.
- Verify `/v1/health.version` equals target tag.
- Never invoke `focusa-daemon --version` while the service is running unless the binary supports a safe version flag.

### TUI

- Replace `/usr/local/bin/focusa-tui` from verified release asset.
- Add or require a safe `focusa-tui --version` or `focusa-tui --headless-self-test --json` version surface.

### Pi extension

- Update only through a package/version channel or local repo build policy.
- Must verify daemon tool-contract version compatibility.
- Notify Pi sessions that a new extension is available or installed.

### Menubar

- Mac client updater should use signed `.dmg` / `.app.tar.gz` assets.
- Server should expose update metadata but not install Mac bundles locally.

### Installer

- Public installer updates require separate release proof and static installer tests.
- Auto-update must not overwrite installer scripts without release approval.

## Notifications

Update events should be visible in:

- daemon `/v1/update/status`,
- CLI `focusa update status`,
- TUI status view,
- Mac menubar cockpit,
- Pi extension tool doctor,
- Focusa evidence/history ledger.

## Acceptance criteria

1. `focusa update check --json` reports installed vs latest for daemon, CLI, TUI.
2. `focusa update policy set --enabled true|false` toggles auto-update without editing code.
3. `dev_mode` license enables default automatic updates for local developer installs.
4. Non-dev licenses require explicit opt-in and license features before unattended update.
5. Update apply verifies CI/release/deploy status before binary replacement.
6. Update apply verifies checksums/signatures before install.
7. Update apply snapshots rollback binaries before replacement.
8. Daemon restart happens only when daemon binary changed and policy allows restart.
9. Failed health/version proof triggers rollback.
10. Update history records old version, new version, assets, checksums, workflow proof, and rollback location.
11. No update writes over `.env`, license files, Workpoints, Evidence, or runtime data.
12. All Focusa local server parts can be brought to the latest approved tag without manual binary copying.

## First implementation slice

Implement read-only inventory first:

```bash
focusa update check --channel dev --json
```

It must report:

- latest eligible tag,
- installed daemon/CLI/TUI versions or hashes,
- stale parts,
- license level,
- auto-update enabled/disabled,
- exact update plan preview,
- no mutations.

## Second implementation slice

Implement policy toggle:

```bash
focusa update policy set --enabled true --channel dev --mode automatic
focusa update policy set --enabled false
```

## Third implementation slice

Implement guarded apply for `dev_mode` only, then generalize to paid tiers after license and rollback proof pass.
