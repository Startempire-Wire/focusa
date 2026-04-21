#!/bin/bash
# Contract: Spec80 C1.2 export validation/test plan must cover dataset families, dry-run/execution split, and typed error envelopes.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/evidence/SPEC80_EXPORT_VALIDATION_TEST_PLAN_2026-04-21.md"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

for ds in sft preference contrastive long-horizon; do
  if rg -n "\`$ds\`|$ds" "$DOC_FILE" >/dev/null 2>&1; then
    log_pass "plan covers dataset family: $ds"
  else
    log_fail "plan missing dataset family: $ds"
  fi
done

if rg -n 'Dry-run parity|Execution mode|dry-run.*never mutates|non-dry-run' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "plan distinguishes dry-run vs execution mode semantics"
else
  log_fail "plan missing dry-run vs execution mode distinction"
fi

if rg -n 'status:\"error\", code, reason, dataset_type|typed error envelope' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "plan defines typed error envelope requirements"
else
  log_fail "plan missing typed error envelope requirements"
fi

if rg -n 'Gate C1-A|Gate C1-B|Gate C1-C|Gate C1-D' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "plan defines explicit acceptance gates for C1 closure"
else
  log_fail "plan missing explicit C1 acceptance gates"
fi

echo "=== SPEC80 EXPORT VALIDATION PLAN RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
