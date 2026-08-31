#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
UPGRADE="$ROOT_DIR/crates/focusa-cli/src/commands/upgrade.rs"
MAIN="$ROOT_DIR/crates/focusa-cli/src/main.rs"
MOD="$ROOT_DIR/crates/focusa-cli/src/commands/mod.rs"
DOC="$ROOT_DIR/docs/current/UPGRADE_COMMAND.md"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

[ -f "$UPGRADE" ] || fail "upgrade.rs missing"
for needle in \
  'pub struct UpgradeArgs' \
  'dry_run' \
  'current_version' \
  'latest_version' \
  'FOCUSA_RELEASE_TAG' \
  'github_releases_latest_api' \
  'resolved_release_tag' \
  'release_tag_override' \
  'system_install' \
  'delegates_to_focusa_install_atomic_stash_and_rollback' \
  'license_preserved' \
  'recovery_hint' \
  'crate::commands::install::run' \
  'crates/focusa-cli/src/commands/upgrade.rs'; do
  rg -n -F "$needle" "$UPGRADE" >/dev/null || fail "upgrade.rs missing marker: $needle"
done
pass "upgrade command has dry-run current/latest, installer delegation, license, and recovery markers"

rg -n -F 'pub mod upgrade;' "$MOD" >/dev/null || fail "commands/mod.rs missing upgrade module"
rg -n -F 'Upgrade(commands::upgrade::UpgradeArgs)' "$MAIN" >/dev/null || fail "main.rs missing Upgrade command variant"
rg -n -F 'commands::upgrade::run(cli.json, args).await' "$MAIN" >/dev/null || fail "main.rs missing Upgrade dispatch"
pass "upgrade command is wired into CLI"

[ -f "$DOC" ] || fail "UPGRADE_COMMAND.md missing"
for needle in \
  'focusa upgrade --dry-run' \
  'same exact immutable release tag' \
  'canonical Releases API' \
  'focusa install' \
  'atomic stash and rollback' \
  'license-preserved' \
  'recovery_hint' \
  'authoritative `/usr/local/bin` surface'; do
  rg -n -F "$needle" "$DOC" >/dev/null || fail "upgrade doc missing marker: $needle"
done
pass "upgrade docs describe evaluator acceptance and usage"

echo "upgrade command static test: PASS"
