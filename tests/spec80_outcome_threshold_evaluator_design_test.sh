#!/bin/bash
# Contract: Spec80 E1.2 threshold evaluator design must encode six-contract thresholds and Gate D decision logic.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/evidence/SPEC80_OUTCOME_THRESHOLD_EVALUATOR_DESIGN_2026-04-21.md"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

for metric in \
  strategy_adjusted_turn_rate \
  failed_turn_ratio \
  rework_loop_rate \
  novel_context_strategy_reuse_rate \
  setback_recovery_rate \
  perspective_constraint_density \
  steering_uptake_rate \
  forced_pause_rate_after_steering
  do
  if rg -n "\`$metric\`|$metric" "$DOC_FILE" >/dev/null 2>&1; then
    log_pass "evaluator includes metric threshold: $metric"
  else
    log_fail "evaluator missing metric threshold: $metric"
  fi
done

if rg -n 'pass_count >= 4|Require `pass_count >= 4`' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "evaluator encodes Gate D >=4/6 pass rule"
else
  log_fail "evaluator missing Gate D >=4/6 pass rule"
fi

if rg -n 'failed_turn_ratio worsens by `>5%`|critical regression guard' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "evaluator encodes critical regression override"
else
  log_fail "evaluator missing critical regression override"
fi

if rg -n 'insufficient_sample|cannot be counted as pass' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "evaluator handles insufficient sample semantics"
else
  log_fail "evaluator missing insufficient sample semantics"
fi

echo "=== SPEC80 OUTCOME THRESHOLD EVALUATOR DESIGN RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
