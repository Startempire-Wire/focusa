#!/bin/bash
# Contract: SPEC80 metacognitive contract pack must include required tools, errors, layers, and planned bindings.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/evidence/SPEC80_METACOG_TOOL_CONTRACTS_2026-04-21.md"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

for tool in \
  focusa_metacog_capture \
  focusa_metacog_retrieve \
  focusa_metacog_reflect \
  focusa_metacog_plan_adjust \
  focusa_metacog_evaluate_outcome
  do
  if rg -n "\`$tool\`" "$DOC_FILE" >/dev/null 2>&1; then
    log_pass "contract pack includes tool: $tool"
  else
    log_fail "contract pack missing tool: $tool"
  fi
done

for code in \
  CAPTURE_SCHEMA_INVALID \
  RETRIEVE_UNAVAILABLE \
  RETRIEVE_BUDGET_EXCEEDED \
  REFLECT_INPUT_INVALID \
  ADJUST_POLICY_CONFLICT \
  EVAL_INPUT_INVALID
  do
  if rg -n "$code" "$DOC_FILE" >/dev/null 2>&1; then
    log_pass "contract pack includes error code: $code"
  else
    log_fail "contract pack missing error code: $code"
  fi
done

if rg -n '/v1/metacognition/capture|/v1/metacognition/retrieve|/v1/metacognition/reflect|/v1/metacognition/adjust|/v1/metacognition/evaluate' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "contract pack includes full planned metacognition API bindings"
else
  log_fail "contract pack missing one or more planned metacognition API bindings"
fi

if rg -n 'focusa metacognition capture --json|focusa metacognition retrieve --json|focusa metacognition reflect --json|focusa metacognition adjust --json|focusa metacognition evaluate --json' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "contract pack includes full planned metacognition CLI fallback bindings"
else
  log_fail "contract pack missing one or more planned metacognition CLI fallback bindings"
fi

if rg -n '/v1/reflect/\* exists|not contract-equivalent surface' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "contract pack captures reflect-surface non-equivalence checkpoint"
else
  log_fail "contract pack missing reflect-surface checkpoint"
fi

echo "=== SPEC80 METACOG CONTRACT PACK RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
