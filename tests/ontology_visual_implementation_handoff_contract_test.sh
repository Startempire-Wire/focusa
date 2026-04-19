#!/bin/bash
# Runtime contract test: visual→implementation handoff contract must remain behaviorally stable.
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

if echo "$CONTRACTS_JSON" | jq -e '.visual_to_implementation_handoff.pipeline_id == "visual_to_implementation_handoff_v1" and .visual_to_implementation_handoff.source_doc == "docs/64-visual-ui-to-implementation.md"' >/dev/null 2>&1; then
  log_pass "contracts endpoint exposes visual→implementation handoff contract"
else
  log_fail "visual→implementation handoff contract surface missing"
fi

if echo "$CONTRACTS_JSON" | jq -e '.visual_to_implementation_handoff.stage_order == ["derive_component_tree","derive_plumbing_requirements","map_tokens_to_surfaces","map_states_to_views","map_bindings_and_validation","synthesize_completion_checklist"]' >/dev/null 2>&1; then
  log_pass "handoff stage order includes all doc-64 implementation stages"
else
  log_fail "handoff stage order missing one or more doc-64 implementation stages"
fi

if echo "$CONTRACTS_JSON" | jq -e '(.visual_to_implementation_handoff.required_plumbing_classes | length) > 0 and (.visual_to_implementation_handoff.completion_rules | length) > 0' >/dev/null 2>&1; then
  log_pass "handoff contract includes plumbing classes and completion rules"
else
  log_fail "handoff contract metadata missing plumbing/completion fields"
fi

if echo "$CONTRACTS_JSON" | jq -e '.contracts | any(.name=="derive_component_tree" and (.target_types == ["page","region","component","content_slot"])) and any(.name=="synthesize_completion_checklist" and (.target_types == ["verification","acceptance_criterion","task","artifact"]))' >/dev/null 2>&1; then
  log_pass "handoff action mappings target typed implementation objects"
else
  log_fail "handoff action target mappings missing for implementation objects"
fi

echo "=== ONTOLOGY VISUAL IMPLEMENTATION HANDOFF CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
