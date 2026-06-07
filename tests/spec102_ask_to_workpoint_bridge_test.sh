#!/usr/bin/env bash
set -euo pipefail
BASE="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for file in crates/focusa-api/src/routes/project.rs apps/pi-extension/src/tools.ts; do
  for term in ask_to_workpoint_bridge ask_differs_from_active_workpoint recommended_bridge_action exact_next_action checkpoint_payload_hint safe_after_identity_verification; do
    rg -F "$term" "$file" >/dev/null || fail "$file missing bridge term $term"
  done
  pass "$file declares Ask-to-Workpoint bridge terms"
done

if curl -fsS --max-time 10 "$BASE/v1/health" >/dev/null 2>&1; then
  KEY="spec102-ask-bridge-$$"
  curl -fsS --max-time 15 "$BASE/v1/project/card?project_root=$ROOT_DIR&current_ask=Write%20a%20short%20UX%20report%20for%20Spec102%20$KEY" >/tmp/spec102-ask-bridge-card.json
  jq -e '
    .ask_to_workpoint_bridge.safe_after_identity_verification == true
    and (.ask_to_workpoint_bridge.recommended_bridge_action | type == "string")
    and (.ask_to_workpoint_bridge.exact_next_action | type == "string")
    and (.ask_to_workpoint_bridge.checkpoint_payload_hint.mission | type == "string")
    and (.ask_to_workpoint_bridge.checkpoint_payload_hint.next_action | type == "string")
  ' /tmp/spec102-ask-bridge-card.json >/dev/null || fail "project card missing actionable ask-to-workpoint bridge"
  pass "project card exposes actionable ask-to-workpoint bridge"
fi

echo "SPEC102 ask-to-workpoint bridge test: PASS"
