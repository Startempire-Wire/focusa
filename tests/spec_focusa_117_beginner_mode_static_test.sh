#!/usr/bin/env bash
# Spec 117 §10 — Beginner Mode state machine static guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

BM="$ROOT_DIR/crates/focusa-tui/src/beginner_mode.rs"
APP="$ROOT_DIR/crates/focusa-tui/src/app.rs"
MAIN="$ROOT_DIR/crates/focusa-tui/src/main.rs"
HOME="$ROOT_DIR/crates/focusa-tui/src/views/deck_home.rs"

[[ -f "$BM" ]] || fail "beginner_mode.rs missing"

for needle in \
  'BeginnerModeState' \
  'Disconnected' \
  'Unbound' \
  'NoWorkpoint' \
  'NoEvidence' \
  'Resumable' \
  'pub fn assess' \
  'DECISION_TREE' \
  'focusa start' \
  'focusa init --quickstart' \
  'focusa workpoint checkpoint' \
  'focusa workpoint resume'; do
  grep -qF -- "$needle" "$BM" || fail "beginner mode module missing: $needle"
done
pass "beginner mode state machine covers required states and actions"

grep -qF 'mod beginner_mode;' "$MAIN" || fail "main.rs missing beginner_mode module"
grep -qF 'beginner_mode_decision_tree' "$MAIN" || fail "headless proof missing beginner mode decision tree"
grep -qF '"workpoint_resume", "/v1/workpoint/resume"' "$APP" || fail "TUI app does not fetch workpoint resume state"
grep -qF 'beginner_state:' "$HOME" || fail "Deck Home does not render beginner state"
grep -qF 'primary_action:' "$HOME" || fail "Deck Home does not render one primary action"
pass "beginner mode state exposed in Deck Home and headless proof"

echo "focusa-117 beginner-mode static test: PASS"