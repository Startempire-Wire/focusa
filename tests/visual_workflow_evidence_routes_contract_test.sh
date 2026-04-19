#!/bin/bash
# Runtime contract: visual evidence workflow routes persist/retrieve artifacts through public surfaces.
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

STORE_JSON="$(curl -sS -X POST "${BASE_URL}/v1/visual-workflow/evidence/store" \
  -H "Content-Type: application/json" \
  -d "{\"run_id\":\"${RUN_ID}\",\"phase\":\"${PHASE}\",\"evidence_kind\":\"${EVIDENCE_KIND}\",\"label\":\"${LABEL}\",\"kind\":\"text\",\"content\":\"visual route contract evidence\"}")"

if echo "$STORE_JSON" | jq -e '.status == "accepted" and .run_id == $rid and .phase == $ph and .evidence_kind == $ek' --arg rid "$RUN_ID" --arg ph "$PHASE" --arg ek "$EVIDENCE_KIND" >/dev/null 2>&1; then
  log_pass "visual evidence store route accepts and returns persisted metadata"
else
  log_fail "visual evidence store route did not accept payload :: ${STORE_JSON}"
fi

LIST_JSON="$(curl -sS "${BASE_URL}/v1/visual-workflow/evidence?run_id=${RUN_ID}&phase=${PHASE}&evidence_kind=${EVIDENCE_KIND}")"
if echo "$LIST_JSON" | jq -e '.count >= 1 and (.evidence | any(.run_id == $rid and .phase == $ph and .evidence_kind == $ek and .label == $lb))' --arg rid "$RUN_ID" --arg ph "$PHASE" --arg ek "$EVIDENCE_KIND" --arg lb "$LABEL" >/dev/null 2>&1; then
  log_pass "visual evidence list route retrieves stored artifacts by filter"
else
  log_fail "visual evidence list route missing stored artifact :: ${LIST_JSON}"
fi

echo "=== VISUAL WORKFLOW EVIDENCE ROUTES CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
