#!/bin/bash
# Contract: Spec80 E2.1 baseline window computation must implement Appendix C baseline/evaluation semantics deterministically.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/evidence/SPEC80_BASELINE_WINDOW_COMPUTATION_2026-04-21.md"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n 'Baseline window|\[t_enable - 14d, t_enable\)|Evaluation window|rolling 14-day' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "design defines Appendix C baseline and evaluation windows"
else
  log_fail "design missing baseline/evaluation window definitions"
fi

if rg -n '>=200|>=30|>=20' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "design includes Appendix C sample floors"
else
  log_fail "design missing Appendix C sample floors"
fi

if rg -n 'sort ascending|odd N: middle element|even N: arithmetic mean' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "design fixes deterministic median computation method"
else
  log_fail "design missing deterministic median computation method"
fi

if rg -n 'relative_delta = \(eval_value - baseline_median\) / baseline_median|baseline_zero_guard' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "design defines delta formula and zero-baseline guard"
else
  log_fail "design missing delta formula or zero-baseline guard"
fi

if rg -n 'insufficient_sample|baseline_missing|Boundary inclusivity is fixed' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "design defines typed statuses and boundary determinism"
else
  log_fail "design missing typed statuses or boundary determinism"
fi

echo "=== SPEC80 BASELINE WINDOW COMPUTATION RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
