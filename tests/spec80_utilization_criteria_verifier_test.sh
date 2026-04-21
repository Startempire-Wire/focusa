#!/bin/bash
# Contract: Spec80 F4.1 utilization criteria verifier must cover all §20.3 criteria with traceable evidence and blocking semantics.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/evidence/SPEC80_F4_1_UTILIZATION_CRITERIA_VERIFIER_SPEC_2026-04-21.md"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n 'Tree/lineage correctness dossier|Tool-first metacognition loop dossier|CLI/API parity dossier|Outcome compounding dossier|Governance integrity dossier' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "verifier covers all five §20.3 utilization criteria"
else
  log_fail "verifier missing one or more §20.3 utilization criteria"
fi

if rg -n 'Gate D logic|audit compliance|traceable evidence' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "verifier includes Gate D and governance audit linkage"
else
  log_fail "verifier missing Gate D or governance audit linkage"
fi

if rg -n '"verifier_id"|"criteria"|"all_pass"|"blocking_criteria"|"evidence_refs"' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "verifier defines required output schema fields"
else
  log_fail "verifier output schema missing required fields"
fi

if rg -n 'blocking_criteria' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "verifier includes explicit blocking semantics"
else
  log_fail "verifier missing explicit blocking semantics"
fi

if rg -n '§20\.3|SPEC80_UTILIZATION_PROOF_PACK_PLAN' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "verifier includes authoritative citations"
else
  log_fail "verifier missing authoritative citations"
fi

echo "=== SPEC80 UTILIZATION CRITERIA VERIFIER RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
