#!/bin/bash
# SPEC-54: forbidden visible-output leak classes must be explicitly defined and consumed.
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

if rg -n 'FORBIDDEN_VISIBLE_OUTPUT_LEAK_CLASSES' "$STATE_FILE" >/dev/null 2>&1; then
  log_pass "forbidden visible-output leak class catalog is defined"
else
  log_fail "missing forbidden visible-output leak class catalog"
fi

if rg -n 'raw_focus_state_serialization|internal_routing_reasons|metacognitive_prose|hidden_trace_dimensions|reducer_internal_state' "$STATE_FILE" >/dev/null 2>&1; then
  log_pass "doc-54 leak classes are explicitly enumerated"
else
  log_fail "doc-54 leak classes are not fully enumerated"
fi

if rg -n 'detectForbiddenVisibleOutputLeakClasses\(' "$STATE_FILE" >/dev/null 2>&1; then
  log_pass "leak-class detector helper is present"
else
  log_fail "leak-class detector helper missing"
fi

if rg -n 'detectForbiddenVisibleOutputLeakClasses\(assistantOutput\)' "$TURNS_FILE" >/dev/null 2>&1; then
  log_pass "turn_end consumes leak-class detector on visible assistant output"
else
  log_fail "visible assistant output is not checked for forbidden leak classes"
fi

if rg -n 'signal_type: "visible_output_leak"|event_type: "visible_output_leak_detected"' "$TURNS_FILE" >/dev/null 2>&1; then
  log_pass "forbidden leak detection is surfaced to gate + trace channels"
else
  log_fail "forbidden leak detection not surfaced to observability channels"
fi

echo "=== VISIBLE OUTPUT LEAK CLASS CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
