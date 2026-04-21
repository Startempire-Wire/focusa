#!/bin/bash
# Contract: Spec80 C3.1 metacognition CLI command surface design must define required subcommands and planned API parity bindings.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/evidence/SPEC80_CLI_METACOG_COMMAND_SURFACE_2026-04-21.md"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

for sub in capture retrieve reflect adjust evaluate; do
  if rg -n "\`$sub\`" "$DOC_FILE" >/dev/null 2>&1; then
    log_pass "design includes metacognition subcommand: $sub"
  else
    log_fail "design missing metacognition subcommand: $sub"
  fi
done

if rg -n 'focusa metacognition <subcommand>' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "design defines first-class metacognition command domain"
else
  log_fail "design missing first-class command domain definition"
fi

if rg -n '/v1/metacognition/capture|/v1/metacognition/retrieve|/v1/metacognition/reflect|/v1/metacognition/adjust|/v1/metacognition/evaluate' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "design includes full planned API parity bindings"
else
  log_fail "design missing one or more planned API parity bindings"
fi

if rg -n 'status: "not_implemented"|planned-extension envelope' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "design defines deterministic planned-extension JSON envelope"
else
  log_fail "design missing deterministic planned-extension JSON envelope"
fi

if rg -n 'adjacent but not equivalent|not equivalent to the metacognition domain contract' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "design enforces reflect/metacognition non-equivalence guard"
else
  log_fail "design missing non-equivalence guard"
fi

echo "=== SPEC80 CLI METACOG COMMAND SURFACE DESIGN RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
