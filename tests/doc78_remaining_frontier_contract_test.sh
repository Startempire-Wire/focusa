#!/bin/bash
# Contract: Doc-78 remaining frontier must explicitly map blocked implementation slices to prerequisite branches.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/DOC78_REMAINING_IMPLEMENTATION_FRONTIER_2026-04-16.md"
INDEX_FILE="${ROOT_DIR}/docs/DECOMPOSITION_ARTIFACT_INDEX_2026-04-13.md"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if [ -f "$DOC_FILE" ]; then log_pass "doc78 remaining frontier artifact exists"; else log_fail "doc78 remaining frontier artifact missing"; fi

if rg -n 'reuses existing substrate|extends existing substrate|blocked on prerequisite branch|new implementation surface' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "doc defines required reuse/extend/blocked/new mapping contract"
else
  log_fail "doc missing required reuse/extend/blocked/new mapping contract"
fi

if rg -n 'Branch A|routing|operator-priority|current-ask' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "doc ties frontier to routing/operator-priority prerequisites"
else
  log_fail "doc missing routing/operator-priority prerequisite mapping"
fi

if rg -n 'Branch B|trace/eval|quality proof|truthfulness' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "doc ties frontier to trace/eval truthfulness prerequisites"
else
  log_fail "doc missing trace/eval prerequisite mapping"
fi

if rg -n 'Branch C|shared-substrate|70-77|lifecycle/status/retention/governance' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "doc ties frontier to shared-substrate prerequisites"
else
  log_fail "doc missing shared-substrate prerequisite mapping"
fi

if rg -n 'verified BD transition' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "doc closure standard includes verified BD transition evidence"
else
  log_fail "doc closure standard missing verified BD transition requirement"
fi

if rg -n 'DOC78_PRODUCTION_RUNTIME_EVIDENCE_2026-04-17\.md|DOC78_PRODUCTION_RUNTIME_SERIES_EVIDENCE_2026-04-18\.md|FOCUSA_DOC78_PROD_ARTIFACT_DIR|FOCUSA_DOC78_PROD_SERIES_DIR|tests/doc78_production_runtime_series_smoke\.sh' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "doc anchors baseline+sustained production evidence and artifact capture surfaces"
else
  log_fail "doc missing baseline/sustained production evidence linkage"
fi

if rg -n 'DOC78_REMAINING_IMPLEMENTATION_FRONTIER_2026-04-16\.md|DOC78_F1_F5_CLOSURE_SCORECARD_2026-04-17\.md|DOC78_PRODUCTION_RUNTIME_EVIDENCE_2026-04-17\.md|DOC78_PRODUCTION_RUNTIME_SERIES_EVIDENCE_2026-04-18\.md' "$INDEX_FILE" >/dev/null 2>&1; then
  log_pass "artifact index references doc78 frontier + scorecard + baseline/sustained evidence artifacts"
else
  log_fail "artifact index missing one or more doc78 frontier evidence references"
fi

echo "=== DOC78 REMAINING FRONTIER CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
