#!/bin/bash
# Contract: SPEC80 anti-false-weaving label taxonomy must be explicit and enforceable.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/evidence/SPEC80_LABEL_TAXONOMY_ENFORCEMENT_2026-04-21.md"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

for label in implemented-now documented-authority planned-extension; do
  if rg -n "\`$label\`" "$DOC_FILE" >/dev/null 2>&1; then
    log_pass "taxonomy includes required label: $label"
  else
    log_fail "taxonomy missing required label: $label"
  fi
done

if rg -n 'exactly one primary label|split into separate BDs by label' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "taxonomy defines decomposition enforcement for mixed-state tasks"
else
  log_fail "taxonomy missing decomposition enforcement policy"
fi

if rg -n 'closure reason claiming implementation.*implemented-now.*code citation' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "taxonomy requires implemented-now + code citation for implementation claims"
else
  log_fail "taxonomy missing implementation-claim citation rule"
fi

if rg -n 'docs/80-pi-tree-li-metacognition-tooling-spec.md' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "taxonomy cites SPEC80 authority source"
else
  log_fail "taxonomy missing SPEC80 authority citation"
fi

echo "=== SPEC80 LABEL TAXONOMY ENFORCEMENT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
