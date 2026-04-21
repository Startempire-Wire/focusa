#!/bin/bash
# Contract: Spec80 D4.2 silent mutation sentinel spec must enforce §9 mutation-path invariants and Gate C zero-silent-mutation rule.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/evidence/SPEC80_SILENT_MUTATION_SENTINEL_CHECKS_2026-04-21.md"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n 'every mutation maps to explicit command/tool/event path|no hidden mutation|silent mutation event count remains zero' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec captures §9 mutation-path invariants"
else
  log_fail "spec missing §9 mutation-path invariants"
fi

if rg -n 'Undeclared apply rule|Path mismatch rule|Policy-layer rule|Replay parity rule' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec defines deterministic silent-mutation detection rules"
else
  log_fail "spec missing silent-mutation detection rules"
fi

if rg -n 'silent_mutation_count|violations\[\]|decision.*pass\|fail|silent_mutation_count == 0' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec defines sentinel output contract and pass condition"
else
  log_fail "spec missing sentinel output contract or pass condition"
fi

if rg -n 'fork integrity|tree navigation restore|merge conflict visibility|compaction survival' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec requires sentinel coverage across all Appendix D scenarios"
else
  log_fail "spec missing full Appendix D scenario coverage"
fi

if rg -n 'Gate C|zero silent mutation events|§9 invariant' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec links sentinel checks to Gate C and §9"
else
  log_fail "spec missing Gate C/§9 linkage"
fi

echo "=== SPEC80 SILENT MUTATION SENTINEL CHECKS RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
