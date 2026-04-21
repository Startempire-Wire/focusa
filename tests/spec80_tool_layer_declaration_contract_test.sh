#!/bin/bash
# Contract: SPEC80 tool-to-layer declaration schema must define required fields and cover planned tools.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/evidence/SPEC80_TOOL_LAYER_DECLARATION_CONTRACT_2026-04-21.md"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

for field in tool_id ontology_layers layer_semantics operation_kind authority_label authority_citations gate_binding; do
  if rg -n "\`$field\`" "$DOC_FILE" >/dev/null 2>&1; then
    log_pass "contract includes required field: $field"
  else
    log_fail "contract missing required field: $field"
  fi
done

for tool in \
  focusa_tree_head \
  focusa_tree_path \
  focusa_tree_snapshot_state \
  focusa_tree_restore_state \
  focusa_tree_diff_context \
  focusa_metacog_capture \
  focusa_metacog_retrieve \
  focusa_metacog_reflect \
  focusa_metacog_plan_adjust \
  focusa_metacog_evaluate_outcome
  do
  if rg -n "\`$tool\`" "$DOC_FILE" >/dev/null 2>&1; then
    log_pass "declaration matrix covers planned tool: $tool"
  else
    log_fail "declaration matrix missing planned tool: $tool"
  fi
done

if rg -n '1\.\.[ ]*12|outside `1\.\.[ ]*12`|range `1\.\.12`|range `1\.\.12`|range `1\.\.12`' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "validation rules enforce 12-layer numeric boundary"
else
  log_fail "validation rules missing explicit 12-layer boundary"
fi

echo "=== SPEC80 TOOL-LAYER DECLARATION CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
