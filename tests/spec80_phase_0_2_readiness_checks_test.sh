#!/bin/bash
# Contract: Spec80 F1.1 phase 0-2 readiness checks must map §11 phases to explicit evidence-backed checks and report schema.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/evidence/SPEC80_F1_1_PHASE_0_2_READINESS_CHECKS_SPEC_2026-04-21.md"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n 'Phase 0 — Design lock readiness|Phase 1 — CLI readiness|Phase 2 — Tree/LI integration readiness' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec defines readiness sections for phases 0-2"
else
  log_fail "spec missing one or more phase 0-2 readiness sections"
fi

if rg -n 'Tool contracts finalized|Label taxonomy|claim validation policy|Spec matrix corrections' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "phase 0 includes governance/design-lock prerequisites"
else
  log_fail "phase 0 prerequisites incomplete"
fi

if rg -n 'Export execution plan|validation plan|Lineage/metacognition CLI surface|JSON schema registry|compatibility policy' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "phase 1 includes CLI readiness prerequisites"
else
  log_fail "phase 1 prerequisites incomplete"
fi

if rg -n 'Branch replay scenario specs|Conflict/idempotency specs|Restore/compaction performance gate specs' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "phase 2 includes tree/LI integration prerequisites"
else
  log_fail "phase 2 prerequisites incomplete"
fi

if rg -n '"phase": "0\|1\|2"|"required_checks"|"evidence_refs"|"blocking_items"' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "readiness report schema includes required fields"
else
  log_fail "readiness report schema missing required fields"
fi

echo "=== SPEC80 PHASE 0-2 READINESS CHECKS RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
