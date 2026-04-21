#!/bin/bash
# Contract: Spec80-Impl I3 metacognition API rollout must expose capture/retrieve/reflect/adjust/evaluate routes with typed envelopes.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ROUTE_FILE="${ROOT_DIR}/crates/focusa-api/src/routes/metacognition.rs"
SERVER_FILE="${ROOT_DIR}/crates/focusa-api/src/server.rs"
MOD_FILE="${ROOT_DIR}/crates/focusa-api/src/routes/mod.rs"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n '/v1/metacognition/capture|/v1/metacognition/retrieve|/v1/metacognition/reflect|/v1/metacognition/adjust|/v1/metacognition/evaluate' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "metacognition route module exposes all five required endpoints"
else
  log_fail "metacognition route module missing one or more required endpoints"
fi

if rg -n 'CAPTURE_SCHEMA_INVALID|RETRIEVE_UNAVAILABLE|REFLECT_INPUT_INVALID|ADJUST_POLICY_CONFLICT|EVAL_INPUT_INVALID' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "handlers define typed error codes for all domain actions"
else
  log_fail "typed error codes are incomplete for metacognition domain"
fi

if rg -n '"capture_id"|"candidates"|"reflection_id"|"adjustment_id"|"evaluation_id"|"promote_learning"' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "handlers provide contract-critical success envelope fields"
else
  log_fail "success envelope fields are incomplete"
fi

if rg -n 'pub mod metacognition;' "$MOD_FILE" >/dev/null 2>&1 && rg -n 'merge\(routes::metacognition::router\(\)\)' "$SERVER_FILE" >/dev/null 2>&1; then
  log_pass "metacognition router is wired into API server"
else
  log_fail "metacognition router is not wired into API server"
fi

echo "=== SPEC80 IMPL METACOGNITION API CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
