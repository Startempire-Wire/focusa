#!/bin/bash
# Contract: Spec80 C4.2 compatibility policy must define versioning classes, hard rules, and deprecation gates.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/evidence/SPEC80_CLI_JSON_COMPATIBILITY_POLICY_2026-04-21.md"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n 'Patch-compatible|Minor-compatible|Major-breaking' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "policy defines patch/minor/major compatibility classes"
else
  log_fail "policy missing compatibility classes"
fi

if rg -n 'Required fields.*immutable|optional by default|type changes require major bump|Enum narrowing.*major bump' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "policy defines hard field-evolution rules"
else
  log_fail "policy missing hard field-evolution rules"
fi

if rg -n 'status="not_implemented"|planned-extension envelope|planned_api_path|label' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "policy governs planned-extension envelope stability"
else
  log_fail "policy missing planned-extension envelope governance"
fi

if rg -n 'Minimum deprecation window|Removal can occur only in next major schema id' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "policy defines deprecation window and removal gate"
else
  log_fail "policy missing deprecation/removal policy"
fi

if rg -n 'FAIL if.*missing required field|FAIL if.*type without schema-major bump|FAIL if.*envelope field set changes' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "policy defines explicit validation fail gates"
else
  log_fail "policy missing validation fail gates"
fi

echo "=== SPEC80 CLI JSON COMPATIBILITY POLICY RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
