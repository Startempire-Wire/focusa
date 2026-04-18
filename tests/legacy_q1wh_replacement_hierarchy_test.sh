#!/bin/bash
# Legacy reconciliation contract: focusa-q1wh must map explicitly onto docs 70-77 replacement hierarchy.
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

if rg -n '### `focusa-q1wh`' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "focusa-q1wh reconciliation section exists"
else
  log_fail "focusa-q1wh reconciliation section missing"
fi

if rg -n 'focusa-jz89|focusa-3zav|focusa-ru3s|focusa-2m6e|focusa-eczn|focusa-v2n5|focusa-suwi|focusa-e8wn|focusa-16us|focusa-eg8i' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "replacement bead hierarchy for docs 70-77 is explicitly listed"
else
  log_fail "replacement bead hierarchy list missing required beads"
fi

if rg -n 'doc 70 → `focusa-3zav`|doc 71 → `focusa-suwi`|doc 72 → `focusa-e8wn`|doc 73 → `focusa-16us`|doc 74 → `focusa-eg8i`|doc 75 → `focusa-2m6e`|doc 76 → `focusa-eczn`|doc 77 → `focusa-v2n5`' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "doc-to-bead mapping for docs 70-77 is explicit"
else
  log_fail "doc-to-bead mapping for docs 70-77 is incomplete"
fi

echo "=== LEGACY Q1WH REPLACEMENT HIERARCHY RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
