#!/usr/bin/env bash
# Spec 117 §12 Walkthrough engine static guard.
# Verifies schema constants, catalog, and module wiring.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

WT="$ROOT_DIR/crates/focusa-cli/src/commands/walkthrough.rs"
MOD="$ROOT_DIR/crates/focusa-cli/src/commands/mod.rs"
MAIN="$ROOT_DIR/crates/focusa-cli/src/main.rs"

[[ -f "$WT" ]] || fail "walkthrough.rs missing"

for needle in \
  'focusa.walkthrough.v1' \
  'SCHEMA_VERSION' \
  'WalkthroughEvent' \
  'EventType' \
  'AuthorityPosture' \
  'first_mission' \
  'WalkthroughArgs' \
  '~/.focusa/deck/walkthroughs' \
  'Started' \
  'Advanced' \
  'Completed' \
  'Reset' \
  'Blocked' ; do
  grep -qF -- "$needle" "$WT" || fail "walkthrough module missing: $needle"
done
pass "walkthrough module has schema, events, audiences, and first-mission catalog"

grep -qF "pub mod walkthrough;" "$MOD" || fail "commands mod missing walkthrough export"
grep -qF "Walkthrough(commands::walkthrough::WalkthroughArgs)" "$MAIN" || fail "main.rs missing Walkthrough command"
pass "walkthrough CLI command wired"

cd "$ROOT_DIR" && python3 - <<'PY'
from pathlib import Path
import re
text = Path('crates/focusa-cli/src/commands/walkthrough.rs').read_text()
assert 'SCHEMA_VERSION: &str = "focusa.walkthrough.v1"' in text, text
assert 'first_mission' in text, text
assert 'write_event' in text, text
assert 'progress' in text, text
assert 'list_catalog' in text, text
# spec 117 §12.2 storage path
assert '.focusa/deck/walkthroughs' in text, text
PY
pass "walkthrough module structural invariants hold"

echo "focusa-117 walkthrough-schema static test: PASS"