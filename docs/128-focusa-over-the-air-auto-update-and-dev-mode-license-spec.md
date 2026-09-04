# Spec 128 — Focusa Over-the-Air Auto-Update, Installer Intelligence, and Dev Mode License

## Status

Release-blocked audit reopened 2026-07-22. Signed assets, guarded apply/rollback, inventory, policy, and schedulers exist; dev auto-update authority and easy Pi controls are being completed and re-proven before release.

## Problem

A Focusa host can have multiple installed parts at different versions. The daemon may be latest while the CLI, TUI, session runner, Pi extension, agent context, distribution manifest, installer, or menubar package is stale. Operators should not need to remember manual binary sync after every passing release.

## Goals

1. Keep Focusa installs on the latest approved release automatically when policy allows.
2. Provide an explicit auto-update option that can be turned on or off.
3. Add a first-party `dev_mode` license level for core Focusa developers.
4. Let `dev_mode` installs track the cutting edge by default.
5. Update every Focusa part from tagged releases that passed required CI/proof gates.
6. Preserve rollback, checksums, Context Authority, local data ownership, and license boundaries.
7. Detect the host system and dependency gaps before install or update.
8. Offer to install missing system dependencies with a clear, reversible prompt.
9. Make the installer feel polished and beginner-friendly, including a short terminal intro animation when the terminal supports it.
10. Eliminate the customer-hostile stale-surface problem where daemon, CLI, TUI, session runner, Pi extension, agent context, distribution manifest, installer, or menubar are on different effective releases.

## Non-goals

- No automatic update from untagged commits by default.
- No silent overwrite of data directories, `.env`, license files, Workpoints, evidence, or project state.
- No retagging or mutation of published releases.
- No cloud-memory dependency for local updates.

## Portable component inventory model

No inventory location may encode an operator username, host, or deployment topology. Resolution order is: explicit `FOCUSA_*` override → observed running executable/current data and workspace roots → existing platform service/app locations. Conventions are discovery candidates only and are never reported as installed unless present. Expensive binary hashes are cold opt-in (`include_hashes=true`), never part of hot status discovery.

| Part | Portable location source | Update cadence | Notes |
| --- | --- | --- | --- |
| Focusa daemon | `FOCUSA_DAEMON_PATH` or observed running executable | every accepted release | service-managed binary on supported platforms |
| Focusa CLI | `FOCUSA_CLI_PATH` or sibling of observed executable | every accepted release | operator command surface |
| Focusa TUI | `FOCUSA_TUI_PATH` or sibling of observed executable | every accepted release | version/self-test surface |
| Service definition | systemd, launchd, or Windows Service capability | only on service contract change | not replaced by normal binary update |
| Service overrides | platform service-manager override mechanism | only on policy/runtime change | preserve local overrides |
| Focusa runtime home | install-prefix runtime directory | never wholesale replaced | state, rollback, update history |
| Focusa env | `FOCUSA_ENV_FILE` or platform config directory | never auto-overwritten | secrets/local config boundary |
| License files | `FOCUSA_CONFIG_DIR` or platform config directory | refresh/validate, never downgrade | entitlement boundary |
| Source checkout | `FOCUSA_SOURCE_ROOT` or current workspace | git-managed source | optional developer surface |
| Release assets | configured signed release channel | source of tagged binaries | must pass CI/proof gates |
| Desktop app | `FOCUSA_DESKTOP_APP_PATH` or platform app mechanism | client update channel | macOS, Windows, and supported Linux packaging |
| Agent extension | `FOCUSA_AGENT_EXTENSION_PATH` or workspace package path | package/update channel | must match daemon tool contracts |
| Public installer | configured installer release channel | only on installer release | bootstrap surface |

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

Current installer code treats registry `status=dev_mode` as a test fixture. Spec 128 changes the product model:

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
focusa update plan --latest-version 0.9.80-dev --json
focusa update apply --latest-version 0.9.80-dev --json
focusa update history --json
focusa update rollback --part all --json
focusa update admin --pause --force-check --json
focusa update scheduler --json
focusa update notifications --latest-version 0.9.80-dev --json
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
- `GET /v1/update/history`
- `POST /v1/update/admin`
- `GET /v1/update/scheduler`
- `POST /v1/update/notifications`
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

From `v0.9.188`, the signed release metadata requires one deterministic `focusa.distribution_manifest.v1` contract binding full SHA-256 source trees, all four Rust binaries, Pi and agent context, generated clients, documentation, installer paths, and capability surfaces. Apply and rollback reuse the canonical full-install transaction; partial promotion is rejected rather than recreating skew. Older releases remain valid rollback targets but cannot claim distribution parity.

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

#### Pre-sales macOS distribution boundary (durable operator decision, 2026-07-23)

Paid Apple Developer membership MUST NOT be a prerequisite for Focusa demos, pilots, first customers, or proving trusted OTA before revenue exists.

Until Focusa can fund Developer ID distribution, the canonical macOS mode is `beta_ad_hoc`:

- the release is explicitly labeled **pre-license, unnotarized beta**;
- initial bootstrap trust is the official Focusa GitHub release over HTTPS plus explicit user consent;
- the `.app` must carry a valid ad-hoc code signature so macOS can verify bundle integrity;
- every automatic update artifact and `latest.json` entry must be signed by the dedicated Tauri updater key;
- the installer removes quarantine only after the warning/consent boundary, verifies bundle identifier and code integrity, keeps the previous app, and restores it if launch fails;
- automatic updates remain governed by the normal Focusa update policy and Tauri signature verification;
- the product and release workflow must never describe beta artifacts as Apple-notarized.

`production_notarized` remains a later distribution mode. It requires Developer ID and App Store Connect notarization credentials and must fail closed when those credentials are absent. Agents must preserve this two-mode boundary so paid Apple membership never silently becomes a pre-sales blocker again.

### Installer

- Public installer updates require separate release proof and static installer tests.
- Auto-update must not overwrite installer scripts without release approval.

## Release eligibility and channel policy

`latest approved release` must be mechanically defined. The updater must not infer eligibility from tag recency alone.

| Channel | Intended users | Eligible tags | Default policy | Notes |
| --- | --- | --- | --- | --- |
| `stable` | paying customers | non-prerelease SemVer tags with required CI/proof/release/deploy green | prompt or scheduled auto-update when licensed | safest default |
| `preview` | early adopters | prerelease tags marked preview with required gates green | prompt unless policy enables scheduled | customer-visible but not default |
| `dev` | Focusa developers | `*-dev` tags or release-candidate dev bundles with required gates green | automatic for `dev_mode` | operator wants always latest |
| `nightly` | internal only | signed nightly manifest from trusted builder | automatic only for explicit `dev_mode` opt-in | may be disabled until infra exists |

A release is ineligible when any condition is true:

- release or manifest is yanked, revoked, superseded-with-blocker, or missing;
- required CI, release, deploy, smoke, or installer proof failed;
- required assets, manifest, checksum, signature, or provenance are missing;
- manifest schema is unsupported;
- license does not include required update features;
- target platform is not supported;
- compatibility gate says current data/API/tool contracts cannot safely update;
- release is older than the currently installed pinned version unless rollback was explicitly requested.

## Cryptographic trust and supply-chain provenance

Checksums alone are not enough. Every release bundle must have a signed manifest and per-asset digest.

Required trust fields:

- `manifest_schema_version`;
- `tag`, `commit`, `channel`, `published_at`;
- per-asset `name`, `platform`, `size_bytes`, `sha256`, `signature`;
- signing algorithm, currently `ed25519` unless superseded;
- signing key id and trusted public key fingerprint;
- key-rotation metadata: `valid_from`, `valid_until`, `revoked_at`;
- builder identity, GitHub workflow/run URL, artifact digest, and optional SLSA-style attestation;
- yanked/revoked release status.

Trust rules:

- updater ships or fetches trusted Focusa public keys from a pinned trust root;
- revoked keys and yanked releases are rejected even if checksums match;
- unknown signing keys require explicit operator approval and are never accepted silently;
- asset URLs must use HTTPS, except explicit localhost/dev fixture mode;
- archives must reject path traversal, symlinks outside staging, executable surprises, and oversized assets;
- manifests are data only: no shell, eval, or remote commands from manifest content.

## Prompting, consent, and background update UX

Update policy modes:

| Mode | Behavior | Restart behavior |
| --- | --- | --- |
| `notify` | check and show update only | never restart |
| `prompt` | ask before download/apply | ask again before daemon restart |
| `scheduled` | apply inside maintenance window | restart inside window if allowed |
| `automatic` | apply in background when safe | restart only when policy/license allows |
| `manual` | no automatic checks/apply | operator runs commands |

Prompt requirements:

- explain current version, target version, channel, license level, affected parts, daemon restart impact, and rollback availability;
- use beginner-friendly language: "Your data, projects, license, Workpoints, evidence, and .env files will not be overwritten";
- offer `Update now`, `Later`, `Skip this version`, `Disable auto-update`, and `Show details`;
- warn loudly when daemon restart may interrupt active sessions;
- `dev_mode` on the operator's development host defaults to `automatic` and latest eligible dev build;
- evaluation installs default to `notify`;
- paid non-dev installs default to `prompt` or `scheduled` only after license features allow unattended update.

## Locking, atomic install, and interrupted update recovery

The updater must use a single host-level update lock, for example `/usr/local/lib/focusa/updates/update.lock`.

Locking and recovery rules:

- only one CLI, daemon, scheduler, installer, or cron update can run at a time;
- lock includes PID, started_at, target tag, staged path, and current phase;
- stale lock recovery must verify whether a staged or partial update exists;
- interrupted updates resume from a safe verified phase or roll back;
- no half-written binary may remain executable.

Atomic replacement rules:

1. download into staging;
2. verify size, checksum, signature, manifest, compatibility, and license;
3. snapshot existing binary, permissions, owner/group, xattrs, capabilities, and service metadata;
4. write replacement to a temp file in the destination filesystem;
5. fsync file and directory where supported;
6. atomically rename/swap;
7. preserve executable mode, ownership, SELinux context/xattrs where applicable;
8. verify installed binary hash/version;
9. restart/reload only after all affected local parts are consistent;
10. rollback on health, version, permission, or compatibility proof failure.

Rollback retention:

- keep at least three successful rollback snapshots by default;
- allow policy-controlled retention count/age;
- support part-level rollback and full-bundle rollback;
- record rollback proof and reason in update history.

## Compatibility gates and contract alignment

The manifest must declare compatibility boundaries:

```json
{
  "compatibility": {
    "min_installed_version": "0.9.74-dev",
    "max_skip_minor_versions": 3,
    "daemon_api_contract": "focusa.api.v1",
    "pi_tool_contract": "focusa.pi-tools.v1",
    "data_schema": "focusa.data.v1",
    "requires_migration": false,
    "downgrade_supported": false,
    "requires_restart": ["daemon"],
    "incompatible_if_features_missing": ["packaged_installer"]
  }
}
```

Rules:

- incompatible daemon/CLI/Pi tool contracts block automatic apply;
- required migrations must have dry-run, backup, rollback, and proof;
- downgrade is blocked unless the manifest explicitly supports it;
- Pi sessions must be notified when daemon tool contracts changed;
- menubar/TUI must surface "update required" when API contract is too old/new.

## Scheduler, backoff, and offline behavior

Auto-check policy must define:

- check interval;
- jitter to avoid thundering herd;
- exponential backoff after failures;
- run-on-daemon-startup behavior;
- maintenance window;
- metered-network behavior when detectable;
- offline behavior and cached license grace period;
- maximum consecutive failed update attempts before disabling automatic apply and prompting.

Recommended defaults:

- `dev_mode`: check on daemon startup and every 30 minutes with jitter; automatic apply latest eligible dev build;
- `stable` paid: check daily with jitter; prompt or scheduled apply;
- `evaluation`: check daily; notify only;
- offline: use cached license/update metadata for status, never apply newly downloaded assets until signatures and license are verified.

## License, dev override, and privacy boundary

License checks must be explicit and auditable.

Inputs:

- local signed license file;
- cached entitlement state and expiry/grace period;
- optional online license refresh;
- dev override file or environment variable allowed only on trusted developer hosts;
- policy file selected channel/mode/parts.

`dev_mode` rules:

- `dev_mode` is a first-party developer entitlement, not a public tier;
- dev override must be visible in `focusa update status --json`;
- dev override must not leak into customer/eval installs;
- dev override enables automatic latest eligible dev build by default;
- expired or invalid dev override falls back to normal license behavior.

Privacy rules:

- update checks must not send project names, Workpoints, evidence, file paths, prompts, or local secrets;
- allowed outbound metadata: product version, platform/arch, channel, license tier/feature flags, anonymous install id if enabled, and manifest schema support;
- users can view exactly what metadata is sent;
- enterprise can disable telemetry/update pings and use airgap bundles.

## Platform and service-manager matrix

| Platform | Service manager | Binary install | Notes |
| --- | --- | --- | --- |
| Linux | systemd or user service | `/usr/local/bin` or user-local bin | preserve units/drop-ins; SELinux/xattrs when present |
| macOS | launchd LaunchAgent/LaunchDaemon | `/usr/local/bin`, Homebrew prefix, or app bundle | beta: ad-hoc integrity + Tauri signature + explicit consent; production: Developer ID/notarization/Gatekeeper checks |
| Windows | Windows service or user task | `%ProgramFiles%` or user-local app data | Authenticode/signature checks; service restart rules |
| source checkout | none or developer service | repo target dir/local symlink | dev mode can build/install from local release policy |

The updater must detect package managers and service managers but must not assume root/admin access. It should offer user-local mode when privileged install is unavailable.

## Installer first-run and system environment preflight

The installer must feel like a trustworthy product, not a raw script. It must run a preflight before download/install.

Preflight detection:

- OS, distro/version, kernel, architecture, libc, shell, terminal capability, package manager, service manager, CPU, memory, disk, network, TLS/cert store, proxy environment, PATH write targets, privilege level, existing Focusa install, existing daemon health, existing CLI/TUI versions, license/dev override, and update policy;
- platform compatibility: supported, supported-with-warning, unsupported-with-reason;
- install mode recommendation: system service, user service, portable/local, source checkout, or airgap bundle.

Missing dependency handling:

- detect missing dependencies before downloading large assets;
- show exact packages/commands for the detected package manager;
- ask before installing dependencies unless noninteractive flags explicitly allow it;
- support dry-run;
- support `--assume-yes` only with clear summary and safety checks;
- never install unrelated packages;
- if dependency install fails, print copy/paste commands and recovery hints.

Common dependency categories:

- TLS/certificates (`ca-certificates`, `openssl` or platform equivalent);
- download tool (`curl` or bundled downloader fallback);
- archive tool (`tar`, `unzip`, platform equivalent);
- service manager (`systemd`, `launchd`, Windows service support) when installing daemon service;
- shell support (`bash`, `sh`, PowerShell) and terminal capability;
- optional build/dev dependencies only when source/dev channel requires local build.

Intro and terminal UX (as amended by Spec 132):

- `focusa install` owns terminal presentation in Rust through `focusa-terminal-ui`; the shell and PowerShell bootstrappers remain thin handoff surfaces;
- animated modes render to stderr only and JSON remains a single stdout document;
- animation is disabled by `--quiet`, `--json`, non-TTY stderr, CI, `TERM=dumb`, too-small terminals, or `FOCUSA_INSTALL_UI=plain`;
- `--no-animation` selects plain mode;
- `NO_COLOR` / `CLICOLOR=0` select monochrome animation on a suitable TTY instead of disabling all motion;
- `FOCUSA_REDUCE_MOTION=1` selects reduced-motion mode on a suitable TTY;
- `FOCUSA_INSTALL_UI=auto|full|mono|reduced|plain` and `FOCUSA_INSTALL_SEED=<u64>` are supported diagnostics/test controls;
- first screen explains what Focusa will install, where, and what data it will not touch;
- progress phases: initialize environment, detect system, validate license, resolve release, download assets, verify checksums/trust, install binaries, integrate Pi, register service, persist PATH, run health checks, finalize, complete/rollback;
- errors should be plain-language with exact recovery commands and terminal state restored before durable output.

Additional installer best practices:

- `--dry-run` prints full plan with no mutations;
- `--doctor` checks existing install;
- `--repair` fixes missing pieces without overwriting user data;
- `--uninstall` removes binaries/services only after confirmation and preserves data by default;
- `--portable` or user-local install mode for no-root environments;
- proxy/offline/airgap support;
- idempotent re-run: rerunning installer upgrades or repairs, not duplicates;
- shell-profile changes require explicit consent and are reversible;
- installer writes initial update policy: `dev_mode` automatic latest, evaluation notify-only, paid prompt/scheduled according to license;
- installer verifies installed CLI/daemon/TUI/session-runner plus manifest-bound agent context immediately and prints a one-command next step.

## Observability, history, and admin controls

Structured events must be visible in update history, daemon API, CLI, TUI, Pi doctor, and menubar where applicable:

- `update_check_started`;
- `update_available`;
- `update_not_needed`;
- `update_plan_created`;
- `update_download_started`;
- `update_verify_failed`;
- `update_apply_started`;
- `update_part_installed`;
- `daemon_restart_planned`;
- `daemon_restart_completed`;
- `update_applied`;
- `update_failed`;
- `rollback_started`;
- `rollback_succeeded`;
- `rollback_failed`.

Admin controls:

```bash
focusa update policy pin --tag v0.9.80-dev
focusa update policy unpin
focusa update policy skip --tag v0.9.81-dev
focusa update policy pause --reason "customer demo"
focusa update policy resume
focusa update check --force
focusa update apply --force-dev-latest
```

`--force-dev-latest` is allowed only for trusted `dev_mode` installs and must still verify release trust and compatibility.

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
6. Update apply verifies signed release manifest, checksums, trusted signing key, and asset signatures before install.
7. Update apply rejects yanked releases, revoked keys, unsupported manifest schemas, missing provenance, and failed required gates.
8. Update apply snapshots rollback binaries, permissions, owner/group, xattrs/capabilities, and service metadata before replacement.
9. Daemon restart happens only when daemon binary changed and policy allows restart.
10. Failed health/version/contract proof triggers rollback.
11. Update history records old version, new version, assets, checksums, signatures, workflow proof, policy, license posture, and rollback location.
12. No update writes over `.env`, license files, Workpoints, Evidence, project data, or runtime state.
13. All Focusa local server parts can be brought to the latest approved tag without manual binary copying.
14. Stale CLI with current daemon is detected and reported as a first-class stale-surface condition.
15. `dev_mode` default policy is automatic latest eligible dev build, including the operator's development host override.
16. Evaluation license cannot perform unattended binary replacement.
17. Bad checksum, bad signature, revoked key, yanked release, or unsupported platform blocks install before mutation.
18. Update lock prevents concurrent CLI/daemon/scheduler/installer updates.
19. Interrupted update resumes safely or rolls back; no half-written executable remains.
20. Atomic install preserves permissions, ownership, xattrs/capabilities, and service metadata.
21. Compatibility gates block automatic apply when daemon API, Pi tool contracts, data schema, or migration constraints are unsafe.
22. Scheduler obeys check intervals, jitter, backoff, maintenance windows, offline rules, and maximum failed-attempt policy.
23. Update checks do not send project names, Workpoints, evidence, prompts, local file paths, `.env`, or secrets.
24. Installer preflight detects OS/distro, arch, shell, terminal, package manager, service manager, privileges, PATH, existing install, license/dev override, and update policy.
25. Installer offers to install missing system dependencies with exact commands, dry-run, assume-yes guardrails, and recovery hints.
26. Installer animation appears only on suitable interactive stderr terminals; `--no-animation` selects plain mode, `--quiet`/`--json`/CI/non-TTY/`TERM=dumb` suppress alternate-screen animation, and `NO_COLOR` selects monochrome animation rather than disabling motion.
27. Installer supports dry-run, doctor, repair, uninstall-preserve-data, portable/user-local, proxy/offline/airgap, and idempotent rerun flows.
28. Installer writes initial update policy based on license: dev mode automatic latest, evaluation notify-only, paid prompt/scheduled according to entitlement.
29. Structured update events are visible in daemon API, CLI status, TUI/menubar/Pi doctor where applicable, and update history.
30. Admin can pin, unpin, skip, pause, resume, force check, and trusted-dev force latest without bypassing trust verification.
31. Static/runtime tests cover stale CLI detection, dev-mode default auto-update, eval unattended denial, checksum/signature failures, daemon restart only when changed, rollback on health failure, interrupted update recovery, installer preflight/dependency prompt, and privacy boundary.
32. `beta_ad_hoc` installs from the official GitHub release with explicit unnotarized-beta consent, validates the app identifier and ad-hoc code integrity, preserves the previous app, and rolls back when launch proof fails.
33. `beta_ad_hoc` Tauri OTA verifies dedicated updater signatures and does not require paid Apple membership; `production_notarized` remains fail-closed on complete Apple signing credentials.

## Implementation status — 2026-07-23

Release-gate audit found the installed scheduler active while the effective policy remained evaluation/notify with every part disabled; CLI/API also hard-coded `auto_apply_allowed=false`, and Pi had no policy control. The current completion slice adds persisted trusted dev override, all-surface policy, policy-derived automatic authority, scheduler-only authorization enforcement, and `/focusa-settings` OTA controls.

Implemented foundations through `focusa-wefzg.11`:

- release manifest/signing/eligibility primitives in `focusa_core::update`;
- `focusa update status/check` read-only stale-surface inventory;
- license/dev-mode update policy defaults with `auto_apply_allowed=false`;
- `focusa install --preflight` system/dependency/terminal UX report;
- `focusa update plan` compatibility, prompt, lock, staging, atomic, recovery, and no-half-written-executable safety plan;
- `focusa update apply` performs guarded locking, staging/download, signed-SHA256SUMS trust resolution, checksum verification, fsync, atomic promotion, daemon-last ordering, version/health probes, rollback journaling, and reverse-order restoration when every apply gate permits mutation;
- `focusa update rollback` restores SHA-verified backup manifests; scheduler installation and policy/admin controls remain separately gated;
- core staged-asset verification now includes declared size, SHA-256, and Ed25519 signature verification against public-key fingerprint, algorithm, key-id, and revocation state;
- `tests/spec128_update_runtime_test.sh` plus cargo integration `spec128_update_runtime_e2e` cover safe runtime surfaces, while deeper successful/failed promotion and cross-platform fault-injection proof remains required;
- macOS `beta_ad_hoc` is the default pre-revenue release mode, with mandatory Tauri updater signing, ad-hoc bundle/archive integrity checks, explicit in-app unnotarized-beta disclosure, and `scripts/install-focusa-menubar-beta.sh` consent/identity/quarantine/rollback bootstrap;
- `production_notarized` is selected only by `FOCUSA_MACOS_RELEASE_MODE` and fails closed unless all Developer ID/App Store Connect fields are present; production notarizes/staples both app and DMG without re-signing after stapling;
- tagged releases are blocked while any GitHub issue labeled `release-gate:compaction-session` remains open.

Customer safety boundary: updater mutation exists but remains deny-by-default unless explicit consent, eligible release resolution, complete assets, checksums/signatures, compatibility, license, lock, rollback, and health gates all pass. Remaining work is full manifest/provenance integration, per-asset signature use in apply, exhaustive atomic failure proof, scheduler/admin completion, and cross-part/platform parity—not reimplementation of the existing updater.

Proof commands:

```bash
cargo fmt --check
cargo test -p focusa-cli update::
cargo test -p focusa-api update::
bash tests/spec128_update_status_static_test.sh
bash tests/spec128_installer_preflight_static_test.sh
bash tests/spec128_update_runtime_test.sh
bash tests/spec128_menubar_updater_static_test.sh
bash -n scripts/install-focusa-menubar-beta.sh
cargo test -p focusa-cli --test spec128_update_runtime_e2e
cargo test -p focusa-cli --test cross_phase_smoke_e2e
cargo test -p focusa-api trajectory
```

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
