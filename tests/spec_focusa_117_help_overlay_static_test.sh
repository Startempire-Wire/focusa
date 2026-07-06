#!/usr/bin/env bash
# Spec 117 §9 — Help Overlay static guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

HELP="$ROOT_DIR/crates/focusa-tui/src/views/help_overlay.rs"
APP="$ROOT_DIR/crates/focusa-tui/src/app.rs"
MAIN="$ROOT_DIR/crates/focusa-tui/src/main.rs"
VIEWS="$ROOT_DIR/crates/focusa-tui/src/views/mod.rs"

[[ -f "$HELP" ]] || fail "help_overlay.rs missing"

for needle in \
  'HELP_TOPICS' \
  'Workpoint' \
  'Evidence' \
  'Recall' \
  'Mission Ladder' \
  'Authority badges' \
  'press h or ? to close' \
  'Beginner Mode shows one primary next safe action'; do
  grep -qF -- "$needle" "$HELP" || fail "help overlay missing: $needle"
done
pass "help overlay explains required concepts in plain language"

grep -qF 'pub show_help: bool' "$APP" || fail "App missing show_help state"
grep -qF 'pub fn toggle_help' "$APP" || fail "App missing toggle_help"
grep -qF "KeyCode::Char('h') | KeyCode::Char('?')" "$MAIN" || fail "main.rs missing help toggle keybinding"
grep -qF 'help_overlay' "$MAIN" || fail "headless proof missing help overlay metadata"
grep -qF 'help_overlay::render(frame, area)' "$VIEWS" || fail "root render does not draw help overlay"
pass "help overlay state, keybinding, render, and headless metadata wired"

echo "focusa-117 help-overlay static test: PASS"