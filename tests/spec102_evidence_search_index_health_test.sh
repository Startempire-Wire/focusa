#!/usr/bin/env bash
set -euo pipefail
BASE="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
TMP="${TMPDIR:-/tmp}/spec102-evidence-search-health"
mkdir -p "$TMP"
KEY="spec102-evidence-$(date +%s)-$$"
TARGET="target:$KEY"
RESULT="result text $KEY"
REF="evidence:$KEY"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

curl -fsS --max-time 20 -H 'Content-Type: application/json' \
  -d "{\"project_root\":\"$ROOT_DIR\",\"continuity_id\":\"spec102-evidence-health\",\"session_id\":\"spec102-evidence-health\",\"mission\":\"Spec102 evidence search health fixture\",\"next_slice\":\"verify evidence search\",\"canonical\":true,\"idempotency_key\":\"wp-$KEY\"}" \
  "$BASE/v1/workpoint/checkpoint" > "$TMP/checkpoint.json" \
  || fail "could not create fixture Workpoint"
WORKPOINT_ID=$(jq -r '.workpoint_id // .record.workpoint_id // empty' "$TMP/checkpoint.json")
[[ -n "$WORKPOINT_ID" ]] || fail "checkpoint response missing workpoint_id"

curl -fsS --max-time 20 -H 'Content-Type: application/json' \
  -d "{\"workpoint_id\":\"$WORKPOINT_ID\",\"target_ref\":\"$TARGET\",\"result\":\"$RESULT\",\"evidence_ref\":\"$REF\"}" \
  "$BASE/v1/workpoint/evidence/link" > "$TMP/link.json" \
  || fail "could not link evidence to fixture Workpoint"

jq -e '.status == "accepted" or .status == "pending"' "$TMP/link.json" >/dev/null || fail "unexpected evidence link response"

for mode in target result ref; do
  case "$mode" in
    target) q="$TARGET" ;;
    result) q="$KEY" ;;
    ref) q="$REF" ;;
  esac
  curl -fsS --max-time 15 -H 'Content-Type: application/json' \
    -d "{\"surface\":\"evidence\",\"selector\":\"search\",\"query\":\"$q\",\"limit\":20}" \
    "$BASE/v1/traverse" > "$TMP/search-$mode.json"
  jq -e --arg ref "$REF" --arg target "$TARGET" '
    .traversal.index_health.status != null
    and .traversal.index_health.source_index != null
    and .traversal.index_health.freshness != null
    and .traversal.index_health.count_semantics != null
    and .traversal.index_health.exact_handle_alternatives != null
    and ([.items[].data? // .items[] | select((.evidence_ref? == $ref) or (.target_ref? == $target) or (.id? == $ref))] | length) > 0
  ' "$TMP/search-$mode.json" >/dev/null || fail "just-linked evidence not discoverable by $mode search and no explicit healthy index metadata"
done
pass "just-linked evidence discoverable by target/result/ref search"

curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d '{"surface":"evidence","selector":"search","query":"definitely-no-spec102-evidence-match","limit":5}' \
  "$BASE/v1/traverse" > "$TMP/search-empty.json"
jq -e '
  .traversal.index_health.status == "healthy"
  and .traversal.index_health.index_lag == false
  and (.summary | contains("index_lag") | not)
  and (.summary | test("lag|scar"; "i") | not)
' "$TMP/search-empty.json" >/dev/null || fail "normal empty evidence search should be healthy with no lag/scar wording"
pass "normal empty evidence search has no lag/scar wording"

echo "SPEC102 evidence search index health test: PASS"
