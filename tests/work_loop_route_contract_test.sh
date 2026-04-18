#!/bin/bash
# SPEC-79 route contract aliases for status/checkpoints.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ROUTE_FILE="${ROOT_DIR}/crates/focusa-api/src/routes/work_loop.rs"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }
if rg -n 'route\("/v1/work-loop/status", get\(status\)\)' "$ROUTE_FILE" >/dev/null 2>&1; then log_pass "GET /v1/work-loop/status route exists"; else log_fail "GET /v1/work-loop/status route missing"; fi
if rg -n 'async fn closure_replay_evidence' "$ROUTE_FILE" >/dev/null 2>&1 && rg -n '"/v1/work-loop/replay/closure-evidence"' "$ROUTE_FILE" >/dev/null 2>&1 && rg -n 'get\(closure_replay_evidence\)' "$ROUTE_FILE" >/dev/null 2>&1; then log_pass "GET /v1/work-loop/replay/closure-evidence route exists"; else log_fail "GET /v1/work-loop/replay/closure-evidence route missing"; fi
if rg -n 'async fn checkpoints' "$ROUTE_FILE" >/dev/null 2>&1 && rg -n 'route\("/v1/work-loop/checkpoints", get\(checkpoints\)\)' "$ROUTE_FILE" >/dev/null 2>&1; then log_pass "GET /v1/work-loop/checkpoints route exists"; else log_fail "GET /v1/work-loop/checkpoints route missing"; fi
if rg -n 'async fn closure_replay_bundle' "$ROUTE_FILE" >/dev/null 2>&1 && rg -n '"/v1/work-loop/replay/closure-bundle"' "$ROUTE_FILE" >/dev/null 2>&1 && rg -n 'get\(closure_replay_bundle\)' "$ROUTE_FILE" >/dev/null 2>&1; then log_pass "GET /v1/work-loop/replay/closure-bundle route exists"; else log_fail "GET /v1/work-loop/replay/closure-bundle route missing"; fi
echo "=== WORK-LOOP ROUTE CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
