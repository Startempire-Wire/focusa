#!/bin/bash
# Consumer-path contract: doc 61 must name a real first consumer path grounded in runtime loci.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/FIRST_CONSUMER_CANDIDATES_2026-04-13.md"
TURNS_FILE="${ROOT_DIR}/apps/pi-extension/src/turns.ts"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n '### Doc 61 — domain-general cognition primitives' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "doc 61 consumer section exists"
else
  log_fail "doc 61 consumer section missing"
fi

if rg -n 'Selected first real consumer|apps/pi-extension/src/turns.ts' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "doc 61 section names an explicit first consumer path"
else
  log_fail "doc 61 section does not name explicit first consumer path"
fi

if rg -n 'CURRENT_ASK|QUERY_SCOPE|relevant_context_selected|irrelevant_context_excluded' "$TURNS_FILE" >/dev/null 2>&1; then
  log_pass "named consumer anchors exist in turns.ts runtime path"
else
  log_fail "named consumer anchors missing in turns.ts runtime path"
fi

echo "=== DOC 61 FIRST CONSUMER PATH RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
