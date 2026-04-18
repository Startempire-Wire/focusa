#!/bin/bash
# Contract: decomposition artifact index centralizes audit/matrix/plan/review docs.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/DECOMPOSITION_ARTIFACT_INDEX_2026-04-13.md"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n '^## Audit artifacts$|^## Matrix artifacts$|^## Plan artifacts$|^## Review artifacts$' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "index defines central audit/matrix/plan/review sections"
else
  log_fail "index missing one or more central artifact sections"
fi

if rg -n 'IMPLEMENTATION_CUTOFF_AUDIT_2026-04-13\.md|DECOMPOSITION_COMPLETENESS_CHECKPOINT_2026-04-13\.md' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "audit section anchors core audit artifacts"
else
  log_fail "audit section missing core audit artifacts"
fi

if rg -n 'IMPLEMENTATION_STATUS_MATRIX_2026-04-13\.md|POST_CUTOFF_DOC_TO_BEAD_MAP_2026-04-13\.md|PREREQUISITE_AND_BLOCKING_MAP_2026-04-13\.md' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "matrix section anchors key mapping/matrix artifacts"
else
  log_fail "matrix section missing key mapping/matrix artifacts"
fi

if rg -n 'POST_CUTOFF_DECOMPOSITION_PLAN_2026-04-13\.md|FIRST_CONSUMER_CANDIDATES_2026-04-13\.md' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "plan section anchors decomposition planning artifacts"
else
  log_fail "plan section missing decomposition planning artifacts"
fi

if rg -n 'DECOMPOSITION_OVERLAP_AND_GAP_REVIEW_2026-04-13\.md|BRANCH_ACCEPTANCE_CRITERIA_2026-04-13\.md|PROOF_SURFACE_REQUIREMENTS_2026-04-13\.md' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "review section anchors overlap/acceptance/proof artifacts"
else
  log_fail "review section missing overlap/acceptance/proof artifacts"
fi

if rg -n 'DOC78_F1_F5_CLOSURE_SCORECARD_2026-04-17\.md|DOC78_PRODUCTION_RUNTIME_EVIDENCE_2026-04-17\.md|DOC78_PRODUCTION_RUNTIME_SERIES_EVIDENCE_2026-04-18\.md' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "index includes doc78 scorecard plus baseline/sustained production evidence artifacts"
else
  log_fail "index missing doc78 scorecard or production evidence artifact links"
fi

echo "=== DECOMPOSITION ARTIFACT INDEX CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
