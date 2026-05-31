#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKPOINT_RS="$ROOT_DIR/crates/focusa-api/src/routes/workpoint.rs"
STATE_TS="$ROOT_DIR/apps/pi-extension/src/state.ts"
TURNS_TS="$ROOT_DIR/apps/pi-extension/src/turns.ts"
COMPACTION_TS="$ROOT_DIR/apps/pi-extension/src/compaction.ts"
SCOPE_DOC="$ROOT_DIR/docs/current/WORKPOINT_SESSION_SCOPE_GUARD.md"

fail() { echo "✗ FAIL: $1" >&2; exit 1; }
pass() { echo "✓ PASS: $1"; }

rg -n 'canonical_for_saved_scope|matches_current_ask_scope|action_authority_for_current_ask|scope_conflict_reason|current_ask_scope' "$WORKPOINT_RS" >/dev/null \
  || fail "Workpoint resume packet does not expose saved-scope/current-action authority fields"
pass "Workpoint resume exposes saved-scope/current-action authority fields"

rg -n 'current_ask:|current_ask_scope|currentAskProjectConflictReason|action_authority_for_current_ask' "$WORKPOINT_RS" "$STATE_TS" "$COMPACTION_TS" >/dev/null \
  || fail "Current operator ask is not included in action-authority arbitration"
pass "Current ask participates in action-authority arbitration"

rg -n 'interface PiCurrentAskScopeVerdict|buildCurrentAskScopeVerdict|formatCurrentAskScopeVerdictLines|CURRENT_ASK_SCOPE_VERDICT|override_candidate' "$STATE_TS" "$TURNS_TS" "$COMPACTION_TS" >/dev/null \
  || fail "Current-ask project override detector/verdict is missing"
pass "Current-ask project override detector/verdict is wired"

rg -n 'wrong place|not this repo|remote project|planmarr|plan-the-marriage|PTM' "$WORKPOINT_RS" "$STATE_TS" >/dev/null \
  || fail "Semantic project correction phrases are not detected before action authority is granted"
pass "Semantic project corrections can suppress action authority"

rg -n 'action_authority_suppressed|saved_scope_as_current_action_authority|verify/rebind the current operator-indicated project' "$ROOT_DIR/apps/pi-extension/src/tools.ts" >/dev/null \
  || fail "Pi workpoint resume tool does not surface action-authority suppression"
pass "Pi tool surfaces action-authority suppression and rebind route"

python3 - "$STATE_TS" <<'PY'
from pathlib import Path
import sys
text = Path(sys.argv[1]).read_text()
anchor = text.find('MEMORY_ANCHOR:')
verdict = text.find('ATTENTION_RECALL_VERDICT')
authority = text.find('action_authority_for_current_ask')
if min(anchor, verdict, authority) < 0:
    raise SystemExit('memory anchor/verdict/action authority markers missing')
if not (anchor < verdict):
    raise SystemExit('attention verdict does not follow protected memory anchor')
PY
pass "Attention/scope verdict remains in protected Focus Slice prefix"

rg -n 'canonical for its saved|wrong action anchor|Operator declares a different project|current-action authority is suppressed' "$SCOPE_DOC" >/dev/null \
  || fail "Workpoint scope guard docs do not explain saved canonicality vs action authority"
pass "Scope guard docs explain saved canonicality vs action authority"

echo "Project scope override static test: PASS"
