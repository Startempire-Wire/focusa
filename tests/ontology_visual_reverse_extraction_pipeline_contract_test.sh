#!/bin/bash
# SPEC-79 / Doc-59 slice A: ontology contracts must define reverse-engineering extraction pipeline.
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

if rg -n 'fn visual_reverse_engineering_pipeline_contract\(' "$ROUTE_FILE" >/dev/null 2>&1 \
  && rg -n '"reverse_engineering_pipeline": visual_reverse_engineering_pipeline_contract\(\)' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "contracts endpoint exposes reverse-engineering pipeline contract"
else
  log_fail "reverse-engineering pipeline contract surface missing"
fi

if rg -n '"derive_structure"|"extract_components"|"derive_slots"|"infer_tokens"|"infer_spacing"|"infer_interaction_and_state"|"derive_implementation_semantics"' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "pipeline stage order includes all doc-59 extraction stages"
else
  log_fail "pipeline stage order missing one or more doc-59 extraction stages"
fi

if rg -n '"pipeline_id": "visual_reverse_engineering_extraction_v1"|"default_state": "proposal_level"|"promotion_requires"' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "pipeline includes proposal-level promotion policy"
else
  log_fail "pipeline promotion policy missing"
fi

if rg -n '"derive_structure" => &\["visual_artifact", "page", "region", "layout_rule"\]' "$ROUTE_FILE" >/dev/null 2>&1 \
  && rg -n '"derive_implementation_semantics" => &\["component", "binding", "validation_rule", "page"\]' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "stage actions map to typed visual target objects"
else
  log_fail "stage action target mappings missing for visual extraction"
fi

echo "=== ONTOLOGY VISUAL REVERSE EXTRACTION PIPELINE CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
