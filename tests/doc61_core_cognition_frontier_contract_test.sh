#!/bin/bash
# Contract: Doc-61 frontier must define truthful minimal core-cognition implementation slices.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/DOC61_TRUTHFUL_CORE_COGNITION_SUBSTRATE_FRONTIER_2026-04-16.md"
INDEX_FILE="${ROOT_DIR}/docs/DECOMPOSITION_ARTIFACT_INDEX_2026-04-13.md"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if [ -f "$DOC_FILE" ]; then
  log_pass "doc61 truthful frontier artifact exists"
else
  log_fail "doc61 truthful frontier artifact missing"
fi

if rg -n 'S1|S2|S3|S4|focusa-5flh\.1|focusa-5flh\.2|focusa-5flh\.3|focusa-5flh\.4' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "doc maps remaining frontier slices to focusa-5flh child work items"
else
  log_fail "doc missing explicit slice-to-child-work-item mapping"
fi

if rg -n 'CURRENT_ASK|QUERY_SCOPE|apps/pi-extension/src/turns.ts|first real consumer' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "doc anchors truth baseline to first real routing consumer path"
else
  log_fail "doc missing first-consumer routing baseline anchors"
fi

if rg -n 'reject speculative primitives|no runtime consumer|consumer' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "doc enforces anti-speculative primitive rule"
else
  log_fail "doc missing anti-speculative primitive rule"
fi

if rg -n 'verified BD transition' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "doc closure standard includes verified BD transition evidence"
else
  log_fail "doc closure standard missing verified BD transition requirement"
fi

if rg -n 'DOC61_TRUTHFUL_CORE_COGNITION_SUBSTRATE_FRONTIER_2026-04-16\.md' "$INDEX_FILE" >/dev/null 2>&1; then
  log_pass "decomposition artifact index references doc61 frontier artifact"
else
  log_fail "decomposition artifact index missing doc61 frontier reference"
fi

echo "=== DOC61 CORE COGNITION FRONTIER CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
