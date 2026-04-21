#!/bin/bash
# Contract: SPEC80 normalized error+permission model must cover all tools and error catalogs.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/evidence/SPEC80_ERROR_PERMISSION_MODEL_2026-04-21.md"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

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
    log_pass "permission map includes tool: $tool"
  else
    log_fail "permission map missing tool: $tool"
  fi
done

for perm in lineage:read state:write metacognition:read metacognition:write; do
  if rg -n "\`$perm\`" "$DOC_FILE" >/dev/null 2>&1; then
    log_pass "model includes permission scope: $perm"
  else
    log_fail "model missing permission scope: $perm"
  fi
done

for err in \
  TREE_HEAD_UNAVAILABLE \
  SESSION_NOT_FOUND \
  CLT_NODE_NOT_FOUND \
  SNAPSHOT_WRITE_DENIED \
  SNAPSHOT_CONFLICT \
  SNAPSHOT_NOT_FOUND \
  RESTORE_CONFLICT \
  AUTHORITY_DENIED \
  DIFF_INPUT_INVALID \
  CAPTURE_SCHEMA_INVALID \
  RETRIEVE_UNAVAILABLE \
  RETRIEVE_BUDGET_EXCEEDED \
  REFLECT_INPUT_INVALID \
  ADJUST_POLICY_CONFLICT \
  EVAL_INPUT_INVALID
  do
  if rg -n "$err" "$DOC_FILE" >/dev/null 2>&1; then
    log_pass "model includes error code: $err"
  else
    log_fail "model missing error code: $err"
  fi
done

if rg -n '/v1/reflect/\*.*non-equivalent|MUST NOT be treated as contract closure' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "model captures reflect/metacognition non-equivalence guard"
else
  log_fail "model missing reflect/metacognition non-equivalence guard"
fi

echo "=== SPEC80 ERROR/PERMISSION MODEL RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
