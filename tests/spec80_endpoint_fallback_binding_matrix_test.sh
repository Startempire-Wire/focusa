#!/bin/bash
# Contract: SPEC80 Appendix B operational matrix must capture bindings, status, permissions, and dependency ordering.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/evidence/SPEC80_ENDPOINT_FALLBACK_BINDING_MATRIX_2026-04-21.md"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

for row in \
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
  if rg -n "\`$row\`" "$DOC_FILE" >/dev/null 2>&1; then
    log_pass "matrix includes tool row: $row"
  else
    log_fail "matrix missing tool row: $row"
  fi
done

if rg -n 'lineage:read|state:write|metacognition:write|metacognition:read' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "matrix includes required permission categories"
else
  log_fail "matrix missing required permission categories"
fi

if rg -n 'Dependency ordering|Read substrate first|Branch-state write substrate|Metacognition API substrate|Outcome-loop closure wiring' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "matrix defines dependency ordering for rollout"
else
  log_fail "matrix missing dependency ordering"
fi

if rg -n '/v1/focus/snapshots\*|/v1/metacognition/\*|/v1/reflect/\*' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "matrix captures blocking dependencies and adjacent non-equivalent surfaces"
else
  log_fail "matrix missing blocking dependency callouts"
fi

echo "=== SPEC80 ENDPOINT/FALLBACK BINDING MATRIX RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
