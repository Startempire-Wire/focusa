#!/bin/bash
# SPEC-79 / Doc-58 slice A: ontology primitives must define core visual/UI object frontier.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ROUTE_FILE="${ROOT_DIR}/crates/focusa-api/src/routes/ontology.rs"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n '"page"|"region"|"component"|"variant"|"content_slot"|"token"|"layout_rule"|"interaction"|"ui_state"|"binding"|"validation_rule"|"visual_artifact"' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "visual/UI core object frontier types are declared"
else
  log_fail "visual/UI core object frontier types are missing"
fi

if rg -n '"page" => &\["id", "name", "page_kind", "primary_goal", "status"\]' "$ROUTE_FILE" >/dev/null 2>&1 \
  && rg -n '"component" => &\["id", "name", "component_kind", "status"\]' "$ROUTE_FILE" >/dev/null 2>&1 \
  && rg -n '"token" => &\["id", "token_kind", "value", "status"\]' "$ROUTE_FILE" >/dev/null 2>&1 \
  && rg -n '"visual_artifact" => &\["id", "artifact_kind", "status"\]' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "visual/UI frontier objects define required property schemas"
else
  log_fail "required property schemas missing for one or more visual/UI frontier objects"
fi

echo "=== ONTOLOGY VISUAL OBJECT FRONTIER CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
