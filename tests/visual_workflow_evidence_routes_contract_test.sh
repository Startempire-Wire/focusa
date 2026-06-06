#!/bin/bash
# Runtime contract: visual evidence workflow routes persist/retrieve exact scoped artifacts through public surfaces.
set -euo pipefail
BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

RUN_ID="vw-$(date +%s%N)"
PHASE="critique"
EVIDENCE_KIND="comparison"
LABEL="runtime-contract"
PROJECT_ROOT="/home/wirebot/focusa"
CONTINUITY_ID="focusa-cont-visual-runtime"
WORKPOINT_ID="visual-workpoint-runtime"

store_payload() {
  local content="$1"
  jq -nc \
    --arg run_id "$RUN_ID" \
    --arg phase "$PHASE" \
    --arg evidence_kind "$EVIDENCE_KIND" \
    --arg lb "$LABEL" \
    --arg content "$content" \
    --arg project_root "$PROJECT_ROOT" \
    --arg continuity_id "$CONTINUITY_ID" \
    --arg workpoint_id "$WORKPOINT_ID" \
    '{run_id:$run_id,phase:$phase,evidence_kind:$evidence_kind,label:$lb,kind:"text",content:$content,project_root:$project_root,continuity_id:$continuity_id,workpoint_id:$workpoint_id}'
}

STORE_JSON="$(curl -sS -X POST "${BASE_URL}/v1/visual-workflow/evidence/store" \
  -H "Content-Type: application/json" \
  -d "$(store_payload 'visual route contract evidence A')")"

if echo "$STORE_JSON" | jq -e '.status == "accepted" and .run_id == $rid and .phase == $ph and .evidence_kind == $ek and (.id == .handle.id) and .handle.project_root == $root and .handle.continuity_id == $cont and .scope.workpoint_id == $wp and (.tool_result_v1.evidence_refs[0] | startswith("focusa-handle:"))' --arg rid "$RUN_ID" --arg ph "$PHASE" --arg ek "$EVIDENCE_KIND" --arg root "$PROJECT_ROOT" --arg cont "$CONTINUITY_ID" --arg wp "$WORKPOINT_ID" >/dev/null 2>&1; then
  log_pass "visual evidence store route returns exact scoped handle metadata"
else
  log_fail "visual evidence store route did not return exact scoped handle :: ${STORE_JSON}"
fi

FIRST_ID="$(echo "$STORE_JSON" | jq -r '.id // empty')"
STORE_JSON_2="$(curl -sS -X POST "${BASE_URL}/v1/visual-workflow/evidence/store" \
  -H "Content-Type: application/json" \
  -d "$(store_payload 'visual route contract evidence B duplicate label')")"
SECOND_ID="$(echo "$STORE_JSON_2" | jq -r '.id // empty')"

if [ -n "$FIRST_ID" ] && [ -n "$SECOND_ID" ] && [ "$FIRST_ID" != "$SECOND_ID" ] && echo "$STORE_JSON_2" | jq -e '.id == .handle.id and .handle.label == ("visual:" + $rid + ":" + $ph + ":" + $ek + ":" + $lb)' --arg rid "$RUN_ID" --arg ph "$PHASE" --arg ek "$EVIDENCE_KIND" --arg lb "$LABEL" >/dev/null 2>&1; then
  log_pass "duplicate visual labels return distinct exact handles"
else
  log_fail "duplicate visual labels were ambiguous :: first=${FIRST_ID} second=${SECOND_ID} json=${STORE_JSON_2}"
fi

LIST_JSON="$(curl -sS "${BASE_URL}/v1/visual-workflow/evidence?run_id=${RUN_ID}&phase=${PHASE}&evidence_kind=${EVIDENCE_KIND}")"
if echo "$LIST_JSON" | jq -e '.count >= 2 and (.evidence | map(select(.run_id == $rid and .phase == $ph and .evidence_kind == $ek and .label == $lb and .handle.project_root == $root and .handle.continuity_id == $cont)) | length >= 2)' --arg rid "$RUN_ID" --arg ph "$PHASE" --arg ek "$EVIDENCE_KIND" --arg lb "$LABEL" --arg root "$PROJECT_ROOT" --arg cont "$CONTINUITY_ID" >/dev/null 2>&1; then
  log_pass "visual evidence list route retrieves duplicate labels by exact scoped handles"
else
  log_fail "visual evidence list route missing scoped duplicate artifacts :: ${LIST_JSON}"
fi

echo "=== VISUAL WORKFLOW EVIDENCE ROUTES CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
