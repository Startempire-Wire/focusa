#!/usr/bin/env bash
set -euo pipefail
BASE="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for term in empty_state empty_because none_exist wrong_selector wrong_scope index_unavailable permission_blocked cold_path_disabled not_checked repair_or_retry; do
  rg -F "$term" crates/focusa-api/src/routes/traverse.rs >/dev/null || fail "traverse empty state missing $term"
done
pass "traverse declares agent-safe empty-state vocabulary"

curl -fsS --max-time 10 "$BASE/v1/health" >/dev/null || { echo "daemon unavailable; static checks only"; exit 0; }
MISS="spec102-empty-miss-$$"
curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d "{\"surface\":\"evidence\",\"selector\":\"workpoint\",\"query\":\"$MISS\",\"limit\":5}" \
  "$BASE/v1/traverse" >/tmp/spec102-empty-evidence.json
jq -e --arg q "$MISS" '
  .items == []
  and .traversal.returned == 0
  and .traversal.empty_state.empty_because == "none_exist"
  and .traversal.empty_state.scope.surface == "evidence"
  and .traversal.empty_state.selector == "workpoint"
  and (.traversal.empty_state.repair_or_retry | test("query|selector|scope|capture|link"; "i"))
  and (.traversal.empty_state.next_selector | type == "string")
  and (.empty_state.empty_because == "none_exist")
' /tmp/spec102-empty-evidence.json >/dev/null || fail "zero evidence traversal missing calm none_exist empty_state"
pass "true empty evidence result is calm and classified"

curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d '{"surface":"unknown_empty_surface","selector":"window","limit":5}' \
  "$BASE/v1/traverse" >/tmp/spec102-empty-blocked.json
jq -e '
  .status == "validation_rejected"
  and .traversal.empty_state.empty_because == "wrong_selector"
  and (.traversal.empty_state.repair_or_retry | test("supported|selector|surface"; "i"))
  and .empty_state.empty_because == "wrong_selector"
' /tmp/spec102-empty-blocked.json >/dev/null || fail "unsupported surface missing wrong_selector empty_state"
pass "unsupported/ambiguous empty identifies wrong_selector repair path"

echo "SPEC102 agent-safe empty states test: PASS"
