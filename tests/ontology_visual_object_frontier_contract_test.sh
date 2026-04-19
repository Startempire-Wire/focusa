#!/bin/bash
# Runtime contract test: visual/UI object frontier must be projected with typed schemas.
set -euo pipefail
BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

PRIMES_JSON="$(curl -sS "${BASE_URL}/v1/ontology/primitives")"

if echo "$PRIMES_JSON" | jq -e '.object_types | any(.type_name=="page") and any(.type_name=="region") and any(.type_name=="component") and any(.type_name=="variant") and any(.type_name=="content_slot") and any(.type_name=="token") and any(.type_name=="layout_rule") and any(.type_name=="interaction") and any(.type_name=="ui_state") and any(.type_name=="binding") and any(.type_name=="validation_rule") and any(.type_name=="visual_artifact")' >/dev/null 2>&1; then
  log_pass "visual/UI core object frontier types are projected"
else
  log_fail "visual/UI core object frontier types are missing"
fi

if echo "$PRIMES_JSON" | jq -e '.object_types | any(.type_name=="page" and (.required_properties == ["id","name","page_kind","primary_goal","status"])) and any(.type_name=="component" and (.required_properties == ["id","name","component_kind","status"])) and any(.type_name=="token" and (.required_properties == ["id","token_kind","value","status"])) and any(.type_name=="visual_artifact" and (.required_properties == ["id","artifact_kind","status"]))' >/dev/null 2>&1; then
  log_pass "visual/UI frontier objects expose required property schemas"
else
  log_fail "required property schemas missing for one or more visual/UI frontier objects"
fi

echo "=== ONTOLOGY VISUAL OBJECT FRONTIER CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
