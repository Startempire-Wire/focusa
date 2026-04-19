#!/bin/bash
# Runtime contract test: ontology route metadata must expose executable contract semantics.
set -euo pipefail
BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

CONTRACTS_JSON="$(curl -sS "${BASE_URL}/v1/ontology/contracts")"
WORLD_JSON="$(curl -sS "${BASE_URL}/v1/ontology/world")"

if echo "$CONTRACTS_JSON" | jq -e '.route_behavior.surface == "GET /v1/ontology/contracts" and .route_behavior.read_only == true and .route_behavior.mutates_canonical_state == false' >/dev/null 2>&1; then
  log_pass "contracts route publishes read-only behavior metadata"
else
  log_fail "contracts route behavior metadata missing or wrong"
fi

if echo "$CONTRACTS_JSON" | jq -e '.contracts | all(has("tool_action_metadata") and has("trace_metadata") and has("eval_metadata") and has("projection_metadata") and has("governance_metadata"))' >/dev/null 2>&1; then
  log_pass "all action contracts expose tool/trace/eval/projection/governance metadata"
else
  log_fail "one or more contracts missing required metadata surfaces"
fi

if echo "$CONTRACTS_JSON" | jq -e '.contracts | any(.name=="build_query_scope" and (.tool_mappings | any(.path=="/v1/work-loop/context"))) and any(.name=="resolve_identity" and (.tool_mappings | any(.path=="/v1/references/search"))) and any(.name=="execute_migration" and (.tool_mappings | any(.path=="/v1/events/recent")))' >/dev/null 2>&1; then
  log_pass "post-doc67/74/77 action families expose concrete runtime route mappings"
else
  log_fail "expected runtime route mappings for new action families missing"
fi

if echo "$WORLD_JSON" | jq -e '.action_catalog | all(has("constraint_checked") and has("reducer_visible") and has("runtime_execution_supported")) and any(.name=="build_query_scope" and .runtime_execution_supported == true)' >/dev/null 2>&1; then
  log_pass "world action catalog surfaces computed reducer/runtime metadata flags"
else
  log_fail "world action catalog metadata flags missing or inconsistent"
fi

echo "=== ONTOLOGY ROUTE METADATA CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
