#!/bin/bash
# Contract: Spec80 D4.1 compaction survival scenario spec must encode §17.4 restore equivalence invariants.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/evidence/SPEC80_COMPACTION_SURVIVAL_SCENARIO_SPEC_2026-04-21.md"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n 'Appendix D §17\.4|Compaction Survival Scenario Spec' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec references Appendix D §17.4 authority"
else
  log_fail "spec missing §17.4 authority reference"
fi

if rg -n 'snapshot `S0`|Execute compaction|Restore branch state from snapshot `S0`' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "scenario narrative includes snapshot-compaction-restore sequence"
else
  log_fail "scenario narrative missing snapshot-compaction-restore sequence"
fi

if rg -n 'Decision-set equivalence|Constraint-set equivalence|No compaction loss' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec defines required compaction survival invariants"
else
  log_fail "spec missing required compaction survival invariants"
fi

if rg -n 'artifact-count pre/post compaction summary|mismatch count.*must be 0|replay transcript id' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec defines measurable compaction survival evidence outputs"
else
  log_fail "spec missing measurable compaction survival evidence outputs"
fi

if rg -n 'Gate C|stable checksums|zero silent mutation events' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec links scenario to Gate C closure criteria"
else
  log_fail "spec missing Gate C linkage"
fi

echo "=== SPEC80 COMPACTION SURVIVAL SCENARIO SPEC RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
