#!/usr/bin/env bash
# Spec 117 §8 — TUI title + Deck Home static guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

APP="$ROOT_DIR/crates/focusa-tui/src/app.rs"
MAIN="$ROOT_DIR/crates/focusa-tui/src/main.rs"
VIEWS="$ROOT_DIR/crates/focusa-tui/src/views/mod.rs"
HOME="$ROOT_DIR/crates/focusa-tui/src/views/deck_home.rs"

[[ -f "$HOME" ]] || fail "Deck Home view missing"

grep -qF 'DeckHome' "$APP" || fail "Tab::DeckHome missing"
grep -qF 'Tab::DeckHome' "$VIEWS" || fail "DeckHome render dispatch missing"
grep -qF 'Focusa Mission Deck' "$VIEWS" || fail "TUI header title not renamed"
grep -qF 'Focusa Mission Deck' "$MAIN" || fail "Headless/help title missing"
grep -qF 'd:DeckHome' "$MAIN" || fail "headless tabs missing d:DeckHome"
grep -qF 'default_tab": "DeckHome"' "$MAIN" || fail "headless default_tab missing"
pass "TUI title/headless metadata expose Focusa Mission Deck + DeckHome"

for needle in \
  'Mission Deck' \
  'Deck Home' \
  'Next safe action' \
  'Beginner Orientation' \
  'Bind project, resume workpoint, capture proof'; do
  grep -qF -- "$needle" "$HOME" || fail "Deck Home view missing: $needle"
done
pass "Deck Home view has mission-oriented landing panels"

echo "focusa-117 tui-title-home static test: PASS"