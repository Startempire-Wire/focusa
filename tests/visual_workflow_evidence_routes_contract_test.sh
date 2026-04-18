#!/bin/bash
# SPEC-79 / Doc-62 slice B: visual evidence workflow must expose runtime persistence/retrieval routes via reducer-owned actions.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ROUTE_FILE="${ROOT_DIR}/crates/focusa-api/src/routes/visual_workflow.rs"
SERVER_FILE="${ROOT_DIR}/crates/focusa-api/src/server.rs"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n 'route\("/v1/visual-workflow/evidence/store", post\(store_visual_evidence\)\)|route\("/v1/visual-workflow/evidence", get\(list_visual_evidence\)\)' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "visual workflow evidence store/list routes are exposed"
else
  log_fail "visual workflow evidence routes missing"
fi

if rg -n 'send\(Action::StoreArtifact' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "evidence persistence flows through reducer action channel"
else
  log_fail "evidence persistence does not use reducer action channel"
fi

if rg -n 'label\.starts_with\("visual:"\)|splitn\(5, '\''\:'\''\)' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "evidence retrieval route parses visual workflow labels"
else
  log_fail "evidence retrieval route missing visual label parsing"
fi

if rg -n 'merge\(routes::visual_workflow::router\(\)\)' "$SERVER_FILE" >/dev/null 2>&1; then
  log_pass "visual workflow routes are mounted in server"
else
  log_fail "visual workflow router not mounted in server"
fi

echo "=== VISUAL WORKFLOW EVIDENCE ROUTES CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
