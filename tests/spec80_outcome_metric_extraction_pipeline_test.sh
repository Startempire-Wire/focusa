#!/bin/bash
# Contract: Spec80 E1.1 outcome metric extraction pipeline must map all six contracts and emit deterministic score inputs.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/evidence/SPEC80_OUTCOME_METRIC_EXTRACTION_PIPELINE_2026-04-21.md"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

for contract in \
  Self-regulation \
  Outcome\ quality \
  Transfer \
  Motivation/ownership \
  Social/perspective\ quality \
  Instructor/operator\ regulation
  do
  if rg -n "$contract" "$DOC_FILE" >/dev/null 2>&1; then
    log_pass "pipeline maps outcome contract: $contract"
  else
    log_fail "pipeline missing outcome contract: $contract"
  fi
done

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
    log_pass "pipeline includes metric: $metric"
  else
    log_fail "pipeline missing metric: $metric"
  fi
done

if rg -n 'rolling 14 days|prior 14-day median|>=200|>=30|>=20' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "pipeline defines baseline window and sample floor policy"
else
  log_fail "pipeline missing baseline/sample-floor policy"
fi

if rg -n 'metric_id|numerator|denominator|baseline_value|relative_delta|sample_size_ok' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "pipeline defines extraction output contract fields"
else
  log_fail "pipeline missing extraction output contract fields"
fi

if rg -n 'Same input telemetry slice must produce identical metric rows|no implicit zero division' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "pipeline defines determinism constraints"
else
  log_fail "pipeline missing determinism constraints"
fi

echo "=== SPEC80 OUTCOME METRIC EXTRACTION PIPELINE RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
