#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RECOVER="$ROOT_DIR/crates/focusa-cli/src/commands/recover.rs"
MAIN="$ROOT_DIR/crates/focusa-cli/src/main.rs"
MOD="$ROOT_DIR/crates/focusa-cli/src/commands/mod.rs"
DOC="$ROOT_DIR/docs/current/RECOVER_COMMAND.md"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

[ -f "$RECOVER" ] || fail "recover.rs missing"
for needle in \
  'pub struct RecoverArgs' \
  'dry_run' \
  'project_root' \
  'continuity_id' \
  'no_start_daemon' \
  'daemon_unavailable_or_crashed' \
  'proposed_recovery' \
  'workpoint_resume' \
  'recovery_hint' \
  'crates/focusa-cli/src/commands/recover.rs'; do
  rg -n -F "$needle" "$RECOVER" >/dev/null || fail "recover.rs missing marker: $needle"
done
pass "recover command has dry-run, scope, daemon, workpoint, and recovery envelope markers"

rg -n -F 'pub mod recover;' "$MOD" >/dev/null || fail "commands/mod.rs missing recover module"
rg -n -F 'Recover(commands::recover::RecoverArgs)' "$MAIN" >/dev/null || fail "main.rs missing Recover command variant"
rg -n -F 'commands::recover::run(cli.json, args).await' "$MAIN" >/dev/null || fail "main.rs missing Recover dispatch"
pass "recover command is wired into CLI"

[ -f "$DOC" ] || fail "RECOVER_COMMAND.md missing"
for needle in \
  'focusa recover --dry-run' \
  'focusa recover --project-root' \
  'daemon_unavailable_or_crashed' \
  'last canonical Workpoint' \
  'recovery_hint' \
  'focusa-recover-cmd'; do
  rg -n -F "$needle" "$DOC" >/dev/null || fail "recover doc missing marker: $needle"
done
pass "recover docs describe evaluator acceptance and usage"

echo "recover command static test: PASS"
