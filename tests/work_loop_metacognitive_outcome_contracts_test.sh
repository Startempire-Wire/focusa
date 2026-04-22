#!/bin/bash
# Contract: metacognitive outcome contracts must be machine-checkable and surfaced in work-loop objective profile.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/79-focusa-governed-continuous-work-loop.md"
ROUTE_FILE="${ROOT_DIR}/crates/focusa-api/src/routes/work_loop.rs"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n 'Metacognitive Outcome Contracts \(Machine-Checkable\)' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "doc79 defines machine-checkable metacognitive outcome contracts section"
else
  log_fail "doc79 missing metacognitive outcome contracts section"
fi

for id in \
  self_monitoring_signal \
  strategy_selection_signal \
  progress_regulation_signal \
  transfer_to_new_context_signal \
  motivation_ownership_signal \
  social_emotional_perspective_signal \
  teaching_regulation_signal
  do
  if rg -n "$id" "$DOC_FILE" >/dev/null 2>&1; then
    log_pass "doc79 includes contract id: $id"
  else
    log_fail "doc79 missing contract id: $id"
  fi
done

if rg -n 'fn metacognitive_outcome_contracts\(' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "work_loop route defines metacognitive outcome contract payload"
else
  log_fail "work_loop route missing metacognitive_outcome_contracts function"
fi

if rg -n '"metacognitive_outcome_contracts"\s*:\s*metacognitive_outcome_contracts\(\)' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "objective profile surfaces metacognitive outcome contracts"
else
  log_fail "objective profile missing metacognitive_outcome_contracts surface"
fi

echo "=== WORK LOOP METACOGNITIVE OUTCOME CONTRACTS RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
