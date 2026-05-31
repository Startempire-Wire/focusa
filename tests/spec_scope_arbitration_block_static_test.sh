#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STATE_TS="$ROOT_DIR/apps/pi-extension/src/state.ts"
TURNS_TS="$ROOT_DIR/apps/pi-extension/src/turns.ts"
COMPACTION_TS="$ROOT_DIR/apps/pi-extension/src/compaction.ts"

fail() { echo "✗ FAIL: $1" >&2; exit 1; }
pass() { echo "✓ PASS: $1"; }

rg -n 'CURRENT_ASK_SCOPE_VERDICT|formatCurrentAskScopeVerdictLines|buildCurrentAskScopeVerdict|action_authority_for_current_ask' "$STATE_TS" "$TURNS_TS" "$COMPACTION_TS" >/dev/null \
  || fail "Scope arbitration verdict block is not wired"
pass "Scope arbitration verdict block is wired"

python3 - "$TURNS_TS" <<'PY'
from pathlib import Path
import sys
text = Path(sys.argv[1]).read_text()
checks = [
    ('formatAttentionRecallFocusSliceLines', 'formatCurrentAskScopeVerdictLines'),
    ('current_ask_scope_verdict', 'buildSliceSection("workpoint"'),
]
for left, right in checks:
    li, ri = text.find(left), text.find(right)
    if li < 0 or ri < 0 or li > ri:
        raise SystemExit(f'ordering failed: {left} before {right}')
PY
pass "Focus Slice places attention/scope verdict before Workpoint"

python3 - "$COMPACTION_TS" <<'PY'
from pathlib import Path
import sys
text = Path(sys.argv[1]).read_text()
li = text.find('formatCurrentAskScopeVerdictLines')
ri = text.find('# Workpoint Resume Packet')
if li < 0 or ri < 0 or li > ri:
    raise SystemExit('compaction scope verdict does not precede Workpoint packet')
PY
pass "Compaction places scope verdict before Workpoint packet"

rg -n 'focusa_project_verify.*focusa_project_identity.*focusa_workpoint_checkpoint|recap_scope_conflict' "$STATE_TS" >/dev/null \
  || fail "Scope conflict does not route to verify/rebind before action"
pass "Scope conflict routes to verify/rebind"

echo "Scope arbitration block static test: PASS"
