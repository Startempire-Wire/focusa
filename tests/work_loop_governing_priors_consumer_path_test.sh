#!/bin/bash
# SPEC-79 / Doc-71 slice B: governing priors must influence at least one live ranking consumer path.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
STATE_FILE="${ROOT_DIR}/apps/pi-extension/src/state.ts"
TURNS_FILE="${ROOT_DIR}/apps/pi-extension/src/turns.ts"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n 'type PiGoverningPriorKind' "$STATE_FILE" >/dev/null 2>&1; then
  log_pass "governing-prior categories are explicitly modeled"
else
  log_fail "governing-prior categories missing"
fi

if rg -n 'governingPriors\?: PiGoverningPriorKind\[\]' "$STATE_FILE" >/dev/null 2>&1; then
  log_pass "ranked selection API accepts governing priors"
else
  log_fail "ranked selection API does not accept governing priors"
fi

if rg -n 'priorBoost|governingPriorContribution|GOVERNING_PRIOR_BAND_BOOST' "$STATE_FILE" >/dev/null 2>&1; then
  log_pass "governing priors contribute to ranking score"
else
  log_fail "no score contribution from governing priors"
fi

if rg -n 'governingPriors: activeGoverningPriors' "$TURNS_FILE" >/dev/null 2>&1; then
  log_pass "context builder wires governing priors into live ranking consumers"
else
  log_fail "turn context ranking consumer is not wired to governing priors"
fi

if rg -n 'event_type: "governing_priors_applied"' "$TURNS_FILE" >/dev/null 2>&1; then
  log_pass "trace output records governing-prior application"
else
  log_fail "missing governing-prior trace output"
fi

echo "=== WORK-LOOP GOVERNING PRIORS CONSUMER PATH RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
