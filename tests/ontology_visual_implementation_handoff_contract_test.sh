#!/bin/bash
# SPEC-79 / Doc-64 slice A: ontology contracts must define visual→implementation handoff contracts.
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

if rg -n 'fn visual_to_implementation_handoff_contract\(' "$ROUTE_FILE" >/dev/null 2>&1 \
  && rg -n '"visual_to_implementation_handoff": visual_to_implementation_handoff_contract\(\)' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "contracts endpoint exposes visual→implementation handoff contract"
else
  log_fail "visual→implementation handoff contract surface missing"
fi

if rg -n '"derive_component_tree"|"derive_plumbing_requirements"|"map_tokens_to_surfaces"|"map_states_to_views"|"map_bindings_and_validation"|"synthesize_completion_checklist"' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "handoff stage order includes all doc-64 implementation stages"
else
  log_fail "handoff stage order missing one or more doc-64 implementation stages"
fi

if rg -n '"pipeline_id": "visual_to_implementation_handoff_v1"|"source_doc": "docs/64-visual-ui-to-implementation.md"|"required_plumbing_classes"|"completion_rules"' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "handoff contract includes source document, plumbing classes, and completion rules"
else
  log_fail "handoff contract metadata missing source/plumbing/completion fields"
fi

if rg -n '"derive_component_tree" => &\["page", "region", "component", "content_slot"\]' "$ROUTE_FILE" >/dev/null 2>&1 \
  && rg -n '"synthesize_completion_checklist" => &\["verification", "acceptance_criterion", "task", "artifact"\]' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "handoff action mappings target typed implementation objects"
else
  log_fail "handoff action target mappings missing for implementation objects"
fi

echo "=== ONTOLOGY VISUAL IMPLEMENTATION HANDOFF CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
