#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ROUTE="$ROOT_DIR/crates/focusa-api/src/routes/call_stack.rs"
TOOLS="$ROOT_DIR/apps/pi-extension/src/tools.ts"
CONTRACTS="$ROOT_DIR/apps/pi-extension/src/tool-contracts.ts"
REGISTRY="$ROOT_DIR/docs/current/focusa-tool-contracts.json"
CHOREO="$ROOT_DIR/docs/current/focusa-tool-choreography.json"
DOC="$ROOT_DIR/docs/focusa-tools/tools/focusa_call_stack_verify.md"
SPEC="$ROOT_DIR/docs/106-focusa-vision-tightening-spec.md"

fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for needle in \
  '/v1/call-stack/verify' \
  'CallStackVerifyRequest' \
  'read_call_stack_designs' \
  'drift_status' \
  'entry_surface_exists' \
  'tool_result_v1' \
  'call_stack_design_not_found'; do
  rg -n -F "$needle" "$ROUTE" >/dev/null || fail "call_stack.rs missing $needle"
done
pass "API call-stack verify route and drift checks present"

for needle in \
  'name: "focusa_call_stack_verify"' \
  '/call-stack/verify' \
  'call stack verify →' \
  'drift_status'; do
  rg -n -F "$needle" "$TOOLS" >/dev/null || fail "Pi tools missing $needle"
done
pass "Pi tool focusa_call_stack_verify registered"

for file in "$CONTRACTS" "$REGISTRY" "$CHOREO" "$DOC"; do
  rg -n -F 'focusa_call_stack_verify' "$file" >/dev/null || fail "$file missing focusa_call_stack_verify"
done
pass "contracts, choreography, and docs include focusa_call_stack_verify"

for needle in \
  'entry surface exists' \
  'handler exists' \
  'service/adapters exist or are marked planned' \
  'output envelope matches `tool_result_v1` expectations' \
  'design still aligns with active STG/Workpoint'; do
  rg -n -F "$needle" "$SPEC" >/dev/null || fail "Spec106 missing verify requirement $needle"
done
pass "Spec106 verify requirements preserved"

echo "call stack verify static test: PASS"
