# Pi Package Activation Transaction and OTA Rollback (#309)

**Issue:** Startempire-Wire/focusa#309 (P0, closed 2026-08-15)
**Commit:** `32b6440f` on `local/work-loop-completion`
**Module:** `crates/focusa-cli/src/commands/pi_package.rs`
**Tests:** `crates/focusa-cli/src/commands/install_pi_package_transaction_tests.rs`

## Problem

`integrate_pi_extension()` combined archive inspection, extraction, npm
setup, stale-package cleanup, backup, activation, rollback, and cleanup into
one opaque function that:

1. created `.focusa-backup-*` **inside** the live Pi auto-discovery root
   (`~/.pi/agent/extensions/`) — Pi loads those as a second Focusa package
   (production incident: `focusa-runtime.legacy-0.9.143` duplicate
   registration forced `pi -ne`);
2. deleted the backup immediately after activation, so any later phase
   failure could not restore the prior Pi package;
3. returned only a display path — OTA kept no typed Pi activation receipt
   in its transaction state, and the rollback ledger (`package_promoted`)
   had no Pi entry.

## Design

### Shared activation transaction (`pi_package.rs`)

| Boundary | Behavior |
| --- | --- |
| `retire_focusa_packages(root, retired_root)` | Moves **only verified Focusa-owned** entries out of the discovery root into the sibling non-discovery root `~/.pi/agent/retired-extensions/`. Two gates must both pass: entry name matches a known Focusa legacy/backup/old pattern AND `package.json` identity is `focusa-pi-bridge`. Unrelated extensions are never touched. Compatibility symlinks resolving to the canonical target are preserved; stale Focusa aliases are removed. |
| `prepare_pi_package(asset, install_root, npm)` | Archive inspection (path-safety + required `pi-extension/package.json`), extraction under a unique staging root, `npm install --omit=dev --ignore-scripts`. Nothing under discovery is touched. |
| `activate_pi_package(staged, root, version)` | Retires legacy entries → backs up the prior `focusa` dir into `retired-extensions/backups/` (outside discovery) → promotes the staged package via atomic rename (cross-device-safe) → restores the backup if promotion fails. Returns a typed `PiActivationReceipt {schema, destination, version, prior{backup,sha256}, retired, activated_at}`. |
| `commit_pi_activation(receipt)` | Removes the prior-package backup. Called **only** after the caller's wider transaction settles. |
| `rollback_pi_activation(receipt)` | Sets the promoted package aside, restores the exact prior package, removes the set-aside copy. Without a prior, removes the promoted package. |

`integrate_pi_extension()` is now a thin wrapper (prepare → activate →
commit) so the installer settles immediately — install is a single-package
transaction — while OTA drives the boundaries itself.

### OTA apply (`update.rs`)

- `phase_pi_package_apply(state, stage, destination_root, url, sha256,
  version)` — download, checksum, prepare, activate, persist the receipt to
  `state/pi-extension-activation.json`, write the restart-required marker,
  then honor the fault-injection point.
- The apply loop retains the receipt in transaction state. **Any failure
  after Pi activation** rolls the Pi package back together with the
  promoted binary parts; the rollback journal records
  `pi_extension_restored`.
- Success path: the rollback manifest gains the `pi_extension` entry
  (target, backup, prior sha256), the restart-required receipt and all
  package phases complete, and only then is the prior backup committed
  (removed). The completed journal records
  `pi_extension: {activated, commit}`.
- Fault injection: `FOCUSA_UPDATE_FAULT_AFTER_PI_ACTIVATION=1` fails the
  update immediately after Pi activation.

### Enforcement

- `tests/installer_update_policy_static_test.sh` checks the policy doc,
  AGENTS.md one-canonical rule, and the transaction code markers.
- The static gate runs in `scripts/git-hooks/pre-push` and as a named CI
  step; focused `pi_package`/`update` tests + `--all-targets` clippy run in
  the CI rust job.

### Documentation

- `docs/current/INSTALLER_UPDATE_POLICY.md` — Pi extension package
  transaction policy section (one canonical package, retired-extensions,
  typed receipt, rollback/commit rules, fault injection, `-ne` prohibition).
- `AGENTS.md` — mandatory one-canonical-Pi-package rule.

## Verification

- `cargo test -p focusa-cli pi_package` → 8/8 (activation/commit cycle,
  exact-prior rollback, retirement identity gating + alias preservation,
  activation-failure restore, OTA fault-after-activation rollback, receipt
  JSON round-trip, name-pattern classification).
- `cargo test -p focusa-cli update` → 12/12 (serial threads; includes the
  git-managed-source gate).
- `cargo clippy -p focusa-cli --all-targets -- -D warnings` → clean.
- Static policy gate + commit-message policy test → PASS.
- Live discovery hygiene verified: exactly one canonical `focusa` package
  plus the `focusa-runtime -> focusa` compatibility symlink; no stage,
  backup, legacy, rollback, disabled, or quarantine entries under
  discovery.

## Operational notes

- The two installed extension directories on the anchor server are the
  same target (symlink) — patching one patches both.
- OTA activation can overwrite the installed extension; crash-safe OTA
  commit/rollback landed in this issue (#309). Keep the release pipeline
  canonical (AGENTS.md) — no local release artifacts.
- Pre-fix backups for the live stale-ctx hotfix:
  `/root/.pi/focusa-ext-fix-backup-20260815/`.
