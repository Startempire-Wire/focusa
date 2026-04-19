#!/bin/bash
# Runtime contract test: reverse-engineering extraction pipeline must be projected and typed.
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

if echo "$CONTRACTS_JSON" | jq -e '.reverse_engineering_pipeline.pipeline_id == "visual_reverse_engineering_extraction_v1"' >/dev/null 2>&1; then
  log_pass "contracts endpoint exposes reverse-engineering pipeline contract"
else
  log_fail "reverse-engineering pipeline contract surface missing"
fi

if echo "$CONTRACTS_JSON" | jq -e '.reverse_engineering_pipeline.stage_order == ["derive_structure","extract_components","derive_slots","infer_tokens","infer_spacing","infer_interaction_and_state","derive_implementation_semantics"]' >/dev/null 2>&1; then
  log_pass "pipeline stage order includes all doc-59 extraction stages"
else
  log_fail "pipeline stage order missing one or more doc-59 extraction stages"
fi

if echo "$CONTRACTS_JSON" | jq -e '.reverse_engineering_pipeline.promotion_policy.default_state == "proposal_level" and (.reverse_engineering_pipeline.promotion_policy.promotion_requires | length) > 0' >/dev/null 2>&1; then
  log_pass "pipeline includes proposal-level promotion policy"
else
  log_fail "pipeline promotion policy missing"
fi

if echo "$CONTRACTS_JSON" | jq -e '.contracts | any(.name=="derive_structure" and (.target_types == ["visual_artifact","page","region","layout_rule"])) and any(.name=="derive_implementation_semantics" and (.target_types == ["component","binding","validation_rule","page"]))' >/dev/null 2>&1; then
  log_pass "stage actions map to typed visual target objects"
else
  log_fail "stage action target mappings missing for visual extraction"
fi

echo "=== ONTOLOGY VISUAL REVERSE EXTRACTION PIPELINE CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
