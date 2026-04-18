#!/bin/bash
# Legacy reconciliation contract: focusa-hd8j must map explicitly onto docs 51-57 replacement hierarchy.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/LEGACY_BEAD_RECONCILIATION_2026-04-13.md"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n '### `focusa-hd8j`' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "focusa-hd8j reconciliation section exists"
else
  log_fail "focusa-hd8j reconciliation section missing"
fi

if rg -n 'focusa-7u1f|focusa-e3id|focusa-jtrl|focusa-93sn|focusa-n4fo' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "replacement bead hierarchy is explicitly listed"
else
  log_fail "replacement bead hierarchy list missing required beads"
fi

if rg -n 'doc 51 → `focusa-7u1f`|doc 52 → `focusa-e3id`|doc 53 → `focusa-e3id`|doc 54 → `focusa-jtrl`|docs 55 / 55-impl → `focusa-93sn`|doc 56 → `focusa-e3id`|doc 57 → `focusa-n4fo`' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "doc-to-bead mapping for docs 51-57 is explicit"
else
  log_fail "doc-to-bead mapping for docs 51-57 is incomplete"
fi

echo "=== LEGACY HD8J REPLACEMENT HIERARCHY RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
