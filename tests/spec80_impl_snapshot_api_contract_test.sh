#!/bin/bash
# Contract: Spec80-Impl I2 snapshot API rollout must expose snapshot/create/restore/diff endpoints with typed envelopes.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ROUTE_FILE="${ROOT_DIR}/crates/focusa-api/src/routes/snapshots.rs"
SERVER_FILE="${ROOT_DIR}/crates/focusa-api/src/server.rs"
MOD_FILE="${ROOT_DIR}/crates/focusa-api/src/routes/mod.rs"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n 'route\("/v1/focus/snapshots", post\(create_snapshot\)\)|route\("/v1/focus/snapshots/restore", post\(restore_snapshot\)\)|route\("/v1/focus/snapshots/diff", post\(diff_snapshots\)\)' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "snapshot route module exposes create/restore/diff endpoints"
else
  log_fail "snapshot route module missing one or more endpoint bindings"
fi

if rg -n '"code": "SNAPSHOT_NOT_FOUND"|"code": "DIFF_INPUT_INVALID"|"status": "ok"' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "snapshot handlers provide typed success/error envelopes"
else
  log_fail "snapshot handlers missing typed envelope fields"
fi

if rg -n '"conflicts"|"restore_mode"|"checksum"|"version_delta"' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "restore/diff outputs include contract-critical fields"
else
  log_fail "restore/diff outputs missing contract-critical fields"
fi

if rg -n 'pub mod snapshots;' "$MOD_FILE" >/dev/null 2>&1 && rg -n 'merge\(routes::snapshots::router\(\)\)' "$SERVER_FILE" >/dev/null 2>&1; then
  log_pass "snapshot router is wired into API server"
else
  log_fail "snapshot router is not wired into API server"
fi

echo "=== SPEC80 IMPL SNAPSHOT API CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
