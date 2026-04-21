#!/bin/bash
# Contract: Spec80 D1.2 tree navigation restore scenario spec must encode Appendix D §17.2 checksum invariants.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/evidence/SPEC80_TREE_NAVIGATION_RESTORE_SCENARIO_SPEC_2026-04-21.md"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n 'Appendix D §17\.2|Tree Navigation Restore Scenario Spec|tree navigation restore' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec references Appendix D §17.2 authority"
else
  log_fail "spec missing §17.2 authority reference"
fi

if rg -n 'B1 -> B2 -> B1|N cycles|baseline snapshot' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "scenario narrative includes repeated navigation loop"
else
  log_fail "scenario narrative missing repeated navigation loop"
fi

if rg -n 'Per-branch checksum stability|Cycle invariance|No cross-branch bleed' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec defines required checksum invariants"
else
  log_fail "spec missing required checksum invariants"
fi

if rg -n 'per-cycle checksum table|mismatch count|must be 0|replay transcript id' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec defines measurable evidence outputs"
else
  log_fail "spec missing measurable evidence outputs"
fi

if rg -n 'Gate C|stable checksums|zero silent mutation events' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec links scenario to Gate C closure criteria"
else
  log_fail "spec missing Gate C linkage"
fi

echo "=== SPEC80 TREE NAVIGATION RESTORE SCENARIO SPEC RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
