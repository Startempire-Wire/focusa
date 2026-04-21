#!/bin/bash
# Contract: Spec80 E4.1 Gate D report generator must encode >=4/6 rule, regression guard, sample floors, form-volume floors, and deterministic output envelope.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/evidence/SPEC80_COMPOUNDING_GATE_REPORT_GENERATOR_2026-04-21.md"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n 'pass_count >= 4|>=4|six contracts' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "report contract encodes Gate D >=4/6 pass threshold"
else
  log_fail "report contract missing Gate D >=4/6 threshold"
fi

if rg -n 'failed_turn_ratio.*>5%|critical regression guard' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "report contract encodes critical regression override"
else
  log_fail "report contract missing critical regression override"
fi

if rg -n '>=200|>=30|>=20' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "report contract includes Appendix C sample floors"
else
  log_fail "report contract missing Appendix C sample floors"
fi

if rg -n '>=50 valid forms|>=20 novel-context forms|form-volume floors' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "report contract includes Appendix E form-volume floors"
else
  log_fail "report contract missing Appendix E form-volume floors"
fi

if rg -n 'report_id|contracts\[\]|final_decision|insufficient_data|payload hash' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "report contract defines deterministic output envelope and insufficiency semantics"
else
  log_fail "report contract missing deterministic output/insufficiency semantics"
fi

echo "=== SPEC80 COMPOUNDING GATE REPORT GENERATOR RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
