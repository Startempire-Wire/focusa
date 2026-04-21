#!/bin/bash
# Contract: Spec80 D1.1 fork integrity scenario spec must encode Appendix D §17.1 replay invariants.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/evidence/SPEC80_FORK_INTEGRITY_SCENARIO_SPEC_2026-04-21.md"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n 'Appendix D §17\.1|fork snapshot integrity|Fork Integrity Scenario Spec' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec references Appendix D §17.1 fork integrity authority"
else
  log_fail "spec missing §17.1 authority reference"
fi

if rg -n 'decision `D1`|decision `D2`|Restore branch A|Assert `D2` is absent' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "scenario narrative includes D1/D2 branch-isolation sequence"
else
  log_fail "scenario narrative missing D1/D2 sequence"
fi

if rg -n 'Branch isolation|Snapshot identity|No silent merge' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec defines required fork integrity invariants"
else
  log_fail "spec missing required invariants"
fi

if rg -n 'checksum|missing fork/snapshot/restore event|contains D1|not contains D2' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec defines measurable failure/pass evidence artifacts"
else
  log_fail "spec missing measurable evidence artifacts"
fi

if rg -n 'Gate C|stable checksums|zero silent mutation events' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec links scenario to Gate C closure criteria"
else
  log_fail "spec missing Gate C linkage"
fi

echo "=== SPEC80 FORK INTEGRITY SCENARIO SPEC RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
