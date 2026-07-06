#!/usr/bin/env bash
# Spec 117 §19 — focusa deck CLI alias static + functional guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

DECK="$ROOT_DIR/crates/focusa-cli/src/commands/deck.rs"
MAIN="$ROOT_DIR/crates/focusa-cli/src/main.rs"
MOD="$ROOT_DIR/crates/focusa-cli/src/commands/mod.rs"

[[ -f "$DECK" ]] || fail "deck command module missing"

for needle in \
  'focusa deck' \
  'Mission Deck CLI alias' \
  'locate_tui_binary' \
  'headless_self_test' \
  'pub async fn run' \
  'PWA /deck planned' ; do
  grep -qE -- "$needle" "$DECK" || fail "deck module missing: $needle"
done
pass "deck module has the canonical focusa deck wiring"

grep -qF "pub mod deck;" "$MOD" || fail "commands mod missing deck export"
grep -qF "Deck(commands::deck::DeckArgs)" "$MAIN" || fail "main.rs missing Deck command"
pass "focusa deck CLI command wired"

cd "$ROOT_DIR" && python3 - <<'PY'
from pathlib import Path
text = Path('crates/focusa-cli/src/commands/deck.rs').read_text()
assert 'fn locate_tui_binary' in text
assert 'FOCUSA_TUI_BIN' in text
assert 'recovery_hint' in text
PY
pass "deck module structural invariants hold"

echo "focusa-117 deck-cli-alias static test: PASS"