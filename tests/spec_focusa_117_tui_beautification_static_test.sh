#!/usr/bin/env bash
# Spec 117 launch blocker — final TUI beautification static guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

HOME="$ROOT_DIR/crates/focusa-tui/src/views/deck_home.rs"
MAIN="$ROOT_DIR/crates/focusa-tui/src/main.rs"
MOD="$ROOT_DIR/crates/focusa-tui/src/views/mod.rs"

for needle in \
  'BEAUTIFICATION_CHECKLIST' \
  'clear_mission_headline' \
  'visible_scope_badge' \
  'visible_proof_meter' \
  'one_primary_next_action' \
  'plain_language_why' \
  'discoverable_hotkeys' \
  'keep the mission, prove the handoff' \
  'Scope badge' \
  'Proof meter' \
  'Mission Control · Do This Next' \
  'Keys          d Deck · n next · / recall · h help · r refresh · q quit'; do
  grep -qF -- "$needle" "$HOME" || fail "Deck Home beautification missing: $needle"
done
pass "Deck Home has polished headline, labels, badges, and key hints"

grep -qF 'deck_home_beautification_checklist' "$MAIN" || fail "headless proof missing beautification checklist"
grep -qF 'pub mod deck_home;' "$MOD" || fail "deck_home not exported for headless proof"
pass "beautification checklist exposed in headless proof"

echo "focusa-117 tui-beautification static test: PASS"
