#!/bin/bash
# Guardrail: canonical mutation routes must not revert to direct shared-state writes.

set -euo pipefail

FAILED=0
PASSED=0

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

log_pass() { echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail() { echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

assert_no_match() {
  local file="$1"
  local pattern="$2"
  local label="$3"
  if rg -n "$pattern" "$file" >/dev/null 2>&1; then
    log_fail "$label present in $file"
  else
    log_pass "$label absent from $file"
  fi
}

assert_has_match() {
  local file="$1"
  local pattern="$2"
  local label="$3"
  if rg -n "$pattern" "$file" >/dev/null 2>&1; then
    log_pass "$label present in $file"
  else
    log_fail "$label missing from $file"
  fi
}

assert_no_match "crates/focusa-api/src/routes/proposals.rs" 'focusa\.write\(\)\.await|save_state\(&next_state\)|\*shared = next_state|write_serial_lock' "proposal route direct canonical write"
assert_has_match "crates/focusa-api/src/routes/proposals.rs" 'Action::EmitEvent' "proposal route daemon dispatch"

assert_no_match "crates/focusa-api/src/routes/constitution.rs" 'focusa\.write\(\)\.await|save_state\(&snapshot\)|write_serial_lock' "constitution route direct canonical write"
assert_has_match "crates/focusa-api/src/routes/constitution.rs" 'Action::EmitEvent' "constitution route daemon dispatch"

assert_no_match "crates/focusa-api/src/routes/threads.rs" 'save_state\(&next_state\)|\*shared = next_state' "thread fork direct canonical write"
assert_has_match "crates/focusa-api/src/routes/threads.rs" 'ThreadForked|Action::EmitEvent' "thread fork reducer event dispatch"

assert_no_match "crates/focusa-api/src/routes/sync_transfer.rs" 'reducer::reduce_with_meta|\*focusa_state = result\.new_state|save_state\(&state_to_save\)' "sync transfer in-route canonical apply"
assert_has_match "crates/focusa-api/src/routes/sync_transfer.rs" 'Action::EmitEvent' "sync transfer daemon dispatch"

assert_no_match "crates/focusa-core/src/runtime/daemon.rs" 'thread\.thesis = thesis|self\.state\.threads\[0\]\.thesis = thesis' "daemon direct thesis mutation"
assert_no_match "crates/focusa-core/src/runtime/daemon.rs" 'crate::memory::semantic::upsert\(\s*&mut self\.state\.memory' "daemon direct semantic memory upsert"
assert_has_match "crates/focusa-core/src/runtime/daemon.rs" 'ThreadThesisUpdated|SemanticMemoryUpserted' "daemon reducer-backed thesis/memory events"

echo "=== CANONICAL WRITER GUARDRAIL RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"

if [ "$FAILED" -ne 0 ]; then
  echo -e "${RED}Canonical writer guardrail failed${NC}"
  exit 1
fi

echo -e "${GREEN}Canonical writer guardrail verified${NC}"
