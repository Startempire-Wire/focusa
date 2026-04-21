#!/bin/bash
# Contract: Spec80 F1.2 phase 3-4 evidence checks must map §11 phase goals to Gate D/E closure evidence rules.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/evidence/SPEC80_F1_2_PHASE_3_4_EVIDENCE_CHECKS_SPEC_2026-04-21.md"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n 'Phase 3 — Metacognition compounding evidence checks|Phase 4 — Evaluation/hardening evidence checks' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec defines readiness sections for phases 3 and 4"
else
  log_fail "spec missing phase 3/4 readiness sections"
fi

if rg -n 'capture/retrieve/reflect/plan_adjust/evaluate_outcome|Practice\+observation form contract|sample floors|form-volume floors' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "phase 3 includes compounding chain and data preconditions"
else
  log_fail "phase 3 readiness requirements incomplete"
fi

if rg -n 'pass\|fail\|insufficient_data|failed_turn_ratio >5%|zero silent mutation sentinel|auditable mutation path' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "phase 4 includes decision envelope, regression guard, and governance integrity checks"
else
  log_fail "phase 4 readiness requirements incomplete"
fi

if rg -n 'Program closure eligibility|Phase 3 and Phase 4 both `ready=true`|Gate D|Gate E' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "spec defines closure eligibility and gate bindings"
else
  log_fail "spec missing closure eligibility and gate bindings"
fi

if rg -n '"phase": "3\|4"|"required_checks"|"gate_bindings"|"blocking_items"' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "readiness report schema includes required fields"
else
  log_fail "readiness report schema missing required fields"
fi

echo "=== SPEC80 PHASE 3-4 EVIDENCE CHECKS RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
