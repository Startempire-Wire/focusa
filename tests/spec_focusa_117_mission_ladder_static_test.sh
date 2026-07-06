#!/usr/bin/env bash
# Spec 117 §6.4 / §14.1 — Mission Ladder panel static guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

LADDER="$ROOT_DIR/crates/focusa-tui/src/views/mission_ladder.rs"
APP="$ROOT_DIR/crates/focusa-tui/src/app.rs"
MAIN="$ROOT_DIR/crates/focusa-tui/src/main.rs"
HOME="$ROOT_DIR/crates/focusa-tui/src/views/deck_home.rs"
MOD="$ROOT_DIR/crates/focusa-tui/src/views/mod.rs"

[[ -f "$LADDER" ]] || fail "mission_ladder.rs missing"

for needle in \
  'Mission Ladder' \
  'LADDER_LEVELS' \
  '"HLT", "MLG", "STG", "Workpoint", "Evidence"' \
  'pub fn render' \
  'pub fn ladder_lines' \
  'unavailable — run focusa trajectory view' \
  'missing — no proof visible yet'; do
  grep -qF -- "$needle" "$LADDER" || fail "mission ladder missing: $needle"
done
pass "mission ladder panel has required visual levels and unavailable/recovery states"

grep -qF 'pub mod mission_ladder;' "$MOD" || fail "views mod missing mission_ladder export"
grep -qF 'mission_ladder::render(app, frame, chunks[2])' "$HOME" || fail "Deck Home does not render Mission Ladder"
grep -qF '("trajectory_view", "/v1/trajectory/view")' "$APP" || fail "TUI app does not fetch trajectory_view"
grep -qF 'mission_ladder_levels' "$MAIN" || fail "headless proof missing mission_ladder_levels"
pass "mission ladder wired into Deck Home, data fetch, and headless proof"

echo "focusa-117 mission-ladder static test: PASS"
