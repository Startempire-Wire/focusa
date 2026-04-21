#!/bin/bash
# Contract: Spec80 F3.1 bead metadata lint policy must enforce §19.3 labels and §20.4 closure citation requirements.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/evidence/SPEC80_F3_1_BEAD_METADATA_LINT_POLICY_SPEC_2026-04-21.md"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n 'implemented-now|documented-authority|planned-extension' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "policy requires one primary anti-false-weaving label"
else
  log_fail "policy missing required label taxonomy"
fi

if rg -n 'Closure reason must include|code: file:line|spec section|Evidence citations' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "policy requires closure citation tuple"
else
  log_fail "policy missing closure citation tuple"
fi

if rg -n 'implemented-now.*require code citation|Label/claim mismatch is lint failure' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "policy enforces implemented-now citation and mismatch failures"
else
  log_fail "policy missing implemented-now citation or mismatch checks"
fi

if rg -n '"bead_id"|"pass"|"violations"|LABEL_MISSING' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "policy defines lint output schema"
else
  log_fail "policy missing lint output schema"
fi

if rg -n '§19\.3|§20\.4|SPEC80_LABEL_TAXONOMY_ENFORCEMENT' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "policy includes authoritative evidence references"
else
  log_fail "policy missing authority/evidence references"
fi

echo "=== SPEC80 BEAD METADATA LINT POLICY RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
