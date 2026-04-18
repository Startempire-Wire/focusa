#!/bin/bash
# SPEC-79 / Doc-66 slice A: ontology contracts must include affordance object/link/action schemas.
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

if rg -n '"capability"|"tool_surface"|"permission"|"authority_boundary"|"execution_context"|"affordance"' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "object schema includes core affordance ontology object types"
else
  log_fail "object schema missing one or more core affordance ontology object types"
fi

if rg -n '"enabled_by"|"requires_permission"|"bounded_by_authority"|"consumes_resource"|"available_in_context"|"supports_execution_of"' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "link schema includes affordance execution relation types"
else
  log_fail "link schema missing affordance execution relation types"
fi

if rg -n '"detect_affordances"|"verify_permissions"|"verify_preconditions"|"estimate_reliability"|"choose_execution_path"|"mark_unavailable"' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "action schema includes affordance execution action types"
else
  log_fail "action schema missing affordance execution action types"
fi

if rg -n '"capability" => &\["id", "capability_kind", "status"\]' "$ROUTE_FILE" >/dev/null 2>&1 \
  && rg -n '"affordance" => &\["id", "affordance_kind", "status"\]' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "new affordance object types define required property schema"
else
  log_fail "required property schema missing for new affordance object types"
fi

echo "=== ONTOLOGY AFFORDANCE SCHEMA CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
