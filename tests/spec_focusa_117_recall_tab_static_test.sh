#!/usr/bin/env bash
# Spec 117 §15 — lightweight advisory Recall Tab static guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

RECALL="$ROOT_DIR/crates/focusa-tui/src/views/recall.rs"
APP="$ROOT_DIR/crates/focusa-tui/src/app.rs"
MAIN="$ROOT_DIR/crates/focusa-tui/src/main.rs"
MOD="$ROOT_DIR/crates/focusa-tui/src/views/mod.rs"

[[ -f "$RECALL" ]] || fail "recall.rs missing"
for needle in \
  'Mission Recall tab' \
  'RECALL_SEARCH_SOURCES' \
  'Focusa events' \
  'Workpoints' \
  'Evidence refs' \
  'UIAI diagnostics packets' \
  'RECALL_CARD_FIELDS' \
  'memory_status' \
  'scope_status' \
  'proof_status' \
  'allowed_use' \
  'RECALL_AUTHORITY_RULE' \
  'Recall is advisory' \
  'operator approval'; do
  grep -qF -- "$needle" "$RECALL" || fail "recall tab missing: $needle"
done
pass "Recall tab covers advisory sources, card fields, and authority boundary"

grep -qF 'Recall,' "$APP" || fail "Tab::Recall missing"
grep -qF 'Tab::Recall => "Recall"' "$APP" || fail "Recall tab label missing"
grep -qF "KeyCode::Char('/') => app.tab = app::Tab::Recall" "$MAIN" || fail "Recall / hotkey missing"
grep -qF 'recall_tab' "$MAIN" || fail "headless recall metadata missing"
grep -qF 'pub mod recall;' "$MOD" || fail "views mod missing recall export"
grep -qF 'Tab::Recall => recall::render(app, frame, area)' "$MOD" || fail "Recall render dispatch missing"
pass "Recall tab wired into TUI, hotkey, render dispatch, and headless proof"

echo "focusa-117 recall-tab static test: PASS"
