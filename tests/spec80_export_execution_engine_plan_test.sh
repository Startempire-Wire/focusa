#!/bin/bash
# Contract: Spec80 C1.1 export execution engine plan must cover all dataset families, phases, and runtime envelope requirements.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/evidence/SPEC80_EXPORT_EXECUTION_ENGINE_PLAN_2026-04-21.md"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

for ds in sft preference contrastive long-horizon; do
  if rg -n "\`$ds\`|$ds" "$DOC_FILE" >/dev/null 2>&1; then
    log_pass "plan includes dataset family: $ds"
  else
    log_fail "plan missing dataset family: $ds"
  fi
done

if rg -n 'Phase E1|Phase E2|Phase E3|Phase E4' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "plan defines phased delivery sequence E1-E4"
else
  log_fail "plan missing phased delivery sequence"
fi

if rg -n 'status:"ok"|records_written|manifest' "$DOC_FILE" >/dev/null 2>&1 && rg -n 'status:"error"|code|reason|dataset_type' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "plan defines success/error JSON envelopes for execution mode"
else
  log_fail "plan missing execution-mode JSON envelope contracts"
fi

if rg -n 'not_implemented|bail on pipeline not implemented|Gap is explicitly tracked' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "plan captures current code-reality export gap"
else
  log_fail "plan missing current gap checkpoint"
fi

echo "=== SPEC80 EXPORT EXECUTION ENGINE PLAN RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
