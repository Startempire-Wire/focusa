#!/bin/bash
# Contract: Spec80 D2.1 merge conflict visibility scenario spec must enforce explicit conflicts[] and no silent overwrite semantics.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/evidence/SPEC80_MERGE_CONFLICT_VISIBILITY_SCENARIO_SPEC_2026-04-21.md"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n 'Appendix D §17\.3|Merge Conflict Visibility Scenario Spec' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec references Appendix D §17.3 authority"
else
  log_fail "spec missing §17.3 authority reference"
fi

if rg -n 'restore_mode=merge|explicit `conflicts\[\]`|no conflicting field is silently overwritten' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "scenario narrative includes merge-mode conflict visibility requirements"
else
  log_fail "scenario narrative missing merge-mode conflict visibility requirements"
fi

if rg -n 'Conflict visibility|No silent overwrite|Mode fidelity' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec defines required merge conflict invariants"
else
  log_fail "spec missing required merge conflict invariants"
fi

if rg -n 'reported_conflicts|silent-overwrite count|must be 0|replay transcript id' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec defines measurable merge conflict evidence outputs"
else
  log_fail "spec missing measurable merge conflict evidence outputs"
fi

if rg -n 'Gate C|zero silent mutation events' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec links merge conflict scenario to Gate C closure"
else
  log_fail "spec missing Gate C linkage"
fi

echo "=== SPEC80 MERGE CONFLICT VISIBILITY SCENARIO SPEC RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
